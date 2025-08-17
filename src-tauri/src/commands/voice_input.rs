use serde::{Deserialize, Serialize};
use tauri::command;
use crate::types::TranscriptionConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAppInfo {
    pub name: String,
    pub icon: Option<String>,
    pub bundle_id: Option<String>,
}

/// 获取当前活动应用的信息（语音输入用）
#[command]
pub async fn get_active_app_info_for_voice() -> Result<ActiveAppInfo, String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSString, NSAutoreleasePool};
        use objc::{msg_send, sel, sel_impl};
        
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            
            // 获取 NSWorkspace
            let workspace_class = objc::class!(NSWorkspace);
            let workspace: id = msg_send![workspace_class, sharedWorkspace];
            
            // 获取当前活动应用
            let active_app: id = msg_send![workspace, frontmostApplication];
            
            if active_app != nil {
                // 获取应用名称
                let localized_name: id = msg_send![active_app, localizedName];
                let name = if localized_name != nil {
                    let name_str = NSString::UTF8String(localized_name);
                    if !name_str.is_null() {
                        std::ffi::CStr::from_ptr(name_str)
                            .to_string_lossy()
                            .to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };
                
                // 获取 bundle identifier
                let bundle_id_ns: id = msg_send![active_app, bundleIdentifier];
                let bundle_id = if bundle_id_ns != nil {
                    let bundle_str = NSString::UTF8String(bundle_id_ns);
                    if !bundle_str.is_null() {
                        Some(std::ffi::CStr::from_ptr(bundle_str)
                            .to_string_lossy()
                            .to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                // 获取应用图标（可选，较复杂）
                // 这里简化处理，返回None
                let icon = None;
                
                pool.drain();
                
                return Ok(ActiveAppInfo {
                    name,
                    icon,
                    bundle_id,
                });
            }
            
            pool.drain();
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // 其他平台的实现
        return Ok(ActiveAppInfo {
            name: "Current Application".to_string(),
            icon: None,
            bundle_id: None,
        });
    }
    
    Err("无法获取活动应用信息".to_string())
}

/// 开始语音录音（支持实时转录和VAD）
#[command]
pub async fn start_voice_recording(
    _device_id: String,
    realtime: bool,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use crate::AppState;
    use tauri::Manager;
    use std::sync::Arc;
    use std::time::Duration;
    
    let state = app.state::<AppState>();
    
    // 检查是否已在录音
    {
        let is_recording = state.is_recording.lock();
        if *is_recording {
            println!("⚠️ 已在录音中，跳过重复初始化");
            return Ok("录音已在进行中".to_string());
        }
    }
    
    // 获取录音器并启动录音
    {
        let mut recorder = state.audio_recorder.lock();
        
        // 重置静音检测
        recorder.reset_silence_detection();
        
        // 开始录音
        recorder.start_recording()
            .map_err(|e| format!("启动录音失败: {}", e))?;
    }
    
    // 设置录音状态
    {
        let mut is_recording = state.is_recording.lock();
        *is_recording = true;
    }
    
    println!("🎙️ 语音录音已启动（VAD模式）");
    
    // 启动VAD监测和自动停止
    if realtime {
        let app_handle = app.clone();
        let recorder_clone = Arc::clone(&state.audio_recorder);
        let is_recording_clone = Arc::clone(&state.is_recording);
        
        // 启动后台任务监测音频电平和静音
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            const MAX_SILENCE_DURATION: Duration = Duration::from_secs(2); // 2秒静音后自动停止
            const MIN_RECORDING_DURATION: Duration = Duration::from_millis(500); // 最少录音0.5秒
            let start_time = std::time::Instant::now();
            let mut has_sound = false;
            
            loop {
                interval.tick().await;
                
                // 检查是否还在录音
                let (is_recording, audio_level, silence_duration) = {
                    let recorder = recorder_clone.lock();
                    (
                        recorder.is_recording(),
                        recorder.get_current_audio_level().unwrap_or(0.0),
                        recorder.get_silence_duration(),
                    )
                };
                
                if !is_recording {
                    println!("🛑 录音已停止（外部触发）");
                    break;
                }
                
                // 检测到声音
                if audio_level > 0.01 {
                    has_sound = true;
                }
                
                // 发送音频电平事件到前端
                if let Err(e) = app_handle.emit_all("audio_level", audio_level) {
                    eprintln!("发送音频电平事件失败: {}", e);
                }
                
                // 检查是否应该自动停止录音
                let recording_duration = std::time::Instant::now().duration_since(start_time);
                
                // 条件：录音超过最小时长 + 检测到过声音 + 静音超过阈值
                if recording_duration > MIN_RECORDING_DURATION 
                    && has_sound 
                    && silence_duration > MAX_SILENCE_DURATION {
                    
                    println!("🔇 检测到静音超过{}秒，自动停止录音", MAX_SILENCE_DURATION.as_secs());
                    
                    // 触发停止录音
                    if let Err(e) = app_handle.emit_all("auto_stop_recording", true) {
                        eprintln!("发送自动停止事件失败: {}", e);
                    }
                    
                    // 直接调用停止函数
                    match crate::commands::stop_voice_recording(app_handle.clone()).await {
                        Ok(text) => {
                            println!("✅ 语音输入完成: {}", text);
                        }
                        Err(e) => {
                            eprintln!("❌ 停止录音失败: {}", e);
                        }
                    }
                    
                    break;
                }
                
                // 发送VAD状态到前端
                let vad_status = serde_json::json!({
                    "is_speaking": audio_level > 0.01,
                    "audio_level": audio_level,
                    "silence_duration": silence_duration.as_millis(),
                    "recording_duration": recording_duration.as_millis(),
                });
                
                if let Err(e) = app_handle.emit_all("vad_status", vad_status) {
                    eprintln!("发送VAD状态失败: {}", e);
                }
            }
            
            // 确保状态正确重置
            {
                let mut is_recording = is_recording_clone.lock();
                *is_recording = false;
            }
        });
        
        println!("✅ 启动VAD（语音活动检测）模式");
    }
    
    Ok("录音已开始".to_string())
}

/// 停止语音录音并返回最终转录结果
#[command]
pub async fn stop_voice_recording(app: tauri::AppHandle) -> Result<String, String> {
    use crate::{AppState, types::TranscriptionConfig};
    use tauri::Manager;
    use std::path::PathBuf;
    
    let state = app.state::<AppState>();
    
    // 获取用户选择的模型设置，如果用户设置的是旧模型则回退到LuYinWang
    let user_selected_model = {
        let settings = state.settings.lock();
        let configured_model = settings.transcription.default_model.clone();
        
        // 如果用户配置的是旧的whisper模型，自动回退到LuYinWang在线服务
        if configured_model == "whisper-1" || configured_model.starts_with("whisper-") {
            println!("⚠️ 检测到旧的模型配置 '{}', 自动使用LuYinWang在线服务", configured_model);
            "luyingwang-online".to_string()
        } else {
            configured_model
        }
    };
    
    // 检查是否在录音
    {
        let is_recording = state.is_recording.lock();
        if !*is_recording {
            println!("⚠️ 当前没有在录音");
            return Ok(String::new());
        }
    }
    
    // 停止录音并获取音频数据和采样率
    let (audio_data, actual_sample_rate) = {
        let mut recorder = state.audio_recorder.lock();
        
        // 获取实际采样率
        let sample_rate = recorder.get_sample_rate();
        
        println!("🛑 停止录音");
        let audio = recorder.stop_recording()
            .map_err(|e| format!("停止录音失败: {}", e))?;
        
        (audio, sample_rate)
    };
    
    // 重置录音状态
    {
        let mut is_recording = state.is_recording.lock();
        *is_recording = false;
    }
    
    if audio_data.is_empty() {
        return Ok(String::new());
    }
    
    println!("📊 录音已停止，音频样本数: {}", audio_data.len());
    println!("🎤 音频时长: {:.2}秒", audio_data.len() as f32 / actual_sample_rate as f32);
    println!("📊 实际采样率: {} Hz", actual_sample_rate);
    println!("🔊 音频数据前10个样本: {:?}", &audio_data[..10.min(audio_data.len())]);
    
    // 如果音频数据太短，返回空字符串（基于实际采样率判断）
    if audio_data.len() < actual_sample_rate as usize { // 小于1秒的音频
        println!("⚠️ 音频太短（小于1秒），跳过转录");
        return Ok(String::new());
    }
    
    // 创建临时WAV文件
    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let temp_file = temp_dir.join(format!("voice_input_{}.wav", timestamp));
    
    // 如果采样率不是16kHz，进行重采样以兼容转录服务
    let (audio_for_transcription, transcription_sample_rate) = if actual_sample_rate != 16000 {
        println!("🔄 重采样音频从 {} Hz 到 16000 Hz 以兼容转录服务", actual_sample_rate);
        let resampled = crate::commands::resample_audio(&audio_data, actual_sample_rate, 16000);
        (resampled, 16000)
    } else {
        (audio_data.clone(), actual_sample_rate)
    };
    
    // 写入WAV文件 - 使用16kHz采样率以兼容转录服务
    println!("💾 准备保存WAV文件到: {:?}", temp_file);
    crate::commands::create_wav_file(&temp_file, &audio_for_transcription, transcription_sample_rate, 1)
        .map_err(|e| {
            eprintln!("❌ 创建WAV文件失败: {}", e);
            format!("创建WAV文件失败: {}", e)
        })?;
    
    // 验证文件是否创建成功
    if temp_file.exists() {
        let file_size = std::fs::metadata(&temp_file).unwrap().len();
        println!("✅ WAV文件创建成功，大小: {} 字节", file_size);
    } else {
        eprintln!("❌ WAV文件未创建！");
    }
    
    // 根据用户选择的模型创建转录配置
    let config = create_transcription_config(&user_selected_model);
    
    println!("🎯 使用用户选择的模型: {}", user_selected_model);
    println!("🔧 转录配置 - 模型名: {}, 是否本地: {}", config.model_name, config.is_local);
    
    // 进行转录
    println!("🎯 开始转录，模型: {}, 语言: {:?}", config.model_name, config.language);
    println!("📂 WAV文件: {:?}, 大小: {} 样本", temp_file, audio_data.len());
    
    let result = state.transcription_service
        .transcribe_audio(temp_file.to_str().unwrap(), &config)
        .await
        .map_err(|e| {
            eprintln!("❌ 转录服务错误: {}", e);
            // 如果是API错误，不要抛出错误，而是返回空字符串让前端重试
            println!("⚠️ 转录失败，将返回空字符串以便前端重试");
            format!("转录失败: {}", e)
        })?;
    
    let final_text = result.text.trim().to_string();
    
    // 备份机制：无论成功与否都先备份，方便调试
    let backup_dir = directories::UserDirs::new()
        .and_then(|dirs| Some(dirs.document_dir()?.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("RecordingKing")
        .join(if final_text.is_empty() { "failed_transcriptions" } else { "successful_transcriptions" });
    
    if !backup_dir.exists() {
        std::fs::create_dir_all(&backup_dir).ok();
    }
    
    let backup_file = backup_dir.join(format!("voice_input_{}.wav", timestamp));
    if let Err(e) = std::fs::copy(&temp_file, &backup_file) {
        eprintln!("❌ 备份音频文件失败: {}", e);
    } else {
        println!("💾 音频已备份到: {:?}", backup_file);
        
        // 同时保存转录结果到文本文件
        let result_file = backup_dir.join(format!("voice_input_{}_result.txt", timestamp));
        let result_content = if final_text.is_empty() {
            format!("转录失败\n时间: {}\n模型: {}\n音频大小: {} bytes", 
                    timestamp, user_selected_model, audio_data.len() * 2)
        } else {
            format!("转录成功\n时间: {}\n模型: {}\n音频大小: {} bytes\n结果: {}", 
                    timestamp, user_selected_model, audio_data.len() * 2, final_text)
        };
        
        if let Err(e) = std::fs::write(&result_file, result_content) {
            eprintln!("❌ 保存结果文件失败: {}", e);
        } else {
            println!("📝 结果已保存到: {:?}", result_file);
        }
    }
    
    // 只有在转录成功后才删除临时文件
    if !final_text.is_empty() {
        if let Err(e) = std::fs::remove_file(&temp_file) {
            eprintln!("清理临时文件失败: {}", e);
        } else {
            println!("🗑️ 已删除临时文件");
        }
    } else {
        println!("💾 保留临时文件以便重试: {:?}", temp_file);
    }
    
    if final_text.is_empty() {
        println!("⚠️ 转录结果为空，可能是API问题、静音或识别失败");
        println!("🔍 音频文件大小: {} 字节", audio_data.len() * 2);  // 每个样本2字节
    } else {
        println!("✅ 语音转录成功: '{}'", final_text);
        
        // 发送转录结果事件到前端，以便添加到历史记录
        // 注意：不设置 audio_file_path，这样会被分类为 LIVE（实时听写）
        let transcription_entry = crate::types::TranscriptionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            text: final_text.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            duration: audio_data.len() as f64 / actual_sample_rate as f64,
            model: user_selected_model.clone(),
            confidence: 0.95,
            audio_file_path: None,  // 重要：设置为 None 以标记为 LIVE 类型
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            tags: None,
            metadata: None,
        };
        
        // 保存到数据库
        {
            let db_manager = state.database.clone();
            if let Err(e) = db_manager.insert_transcription(&transcription_entry) {
                eprintln!("❌ 保存语音输入历史记录失败: {}", e);
            } else {
                println!("✅ 语音输入历史记录已保存");
            }
        }
        
        // 发送事件到前端
        if let Err(e) = app.emit_all("transcription_result", &transcription_entry) {
            eprintln!("❌ 发送语音输入转录结果事件失败: {}", e);
        } else {
            println!("✅ 语音输入转录结果事件已发送到前端");
        }
    }
    
    Ok(final_text)
}

/// 获取当前使用的模型信息
#[command]
pub fn get_current_model_info(app: tauri::AppHandle) -> Result<String, String> {
    use crate::AppState;
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    let settings = state.settings.lock();
    let model = settings.transcription.default_model.clone();
    
    // 如果用户配置的是旧的whisper模型，自动回退到LuYinWang在线服务
    let final_model = if model == "whisper-1" || model.starts_with("whisper-") {
        "luyingwang-online".to_string()
    } else {
        model
    };
    
    Ok(final_model)
}

/// 将文本注入到当前活动的应用
#[command]
pub async fn inject_text_to_active_app(text: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSAutoreleasePool, NSString};
        use objc::{msg_send, sel, sel_impl};
        
        // 安全检查：获取当前活动应用，确保不是向自己注入
        let current_app = get_active_app_info_for_voice().await.ok();
        if let Some(app_info) = current_app {
            if app_info.name.contains("Recording King") || 
               app_info.bundle_id.as_ref().map_or(false, |id| id.contains("recordingking")) {
                eprintln!("⚠️ 警告：尝试向自己注入文本，跳过操作以防止崩溃");
                return Err("无法向 Recording King 自身注入文本".to_string());
            }
        }
        
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            
            // 使用更可靠的粘贴板方法注入文本
            // 1. 先备份当前剪贴板内容
            let pasteboard_class = objc::class!(NSPasteboard);
            let general_pasteboard: id = msg_send![pasteboard_class, generalPasteboard];
            
            // NSPasteboardTypeString 常量
            let string_type = NSString::alloc(nil).init_str("public.utf8-plain-text");
            let old_contents: id = msg_send![general_pasteboard, stringForType:string_type];
            
            // 2. 将文本写入剪贴板
            let text_string = NSString::alloc(nil).init_str(&text);
            let _: () = msg_send![general_pasteboard, clearContents];
            let _: bool = msg_send![general_pasteboard, setString:text_string forType:string_type];
            
            // 3. 使用Cmd+V粘贴 - 比keystroke更可靠
            let script = r#"tell application "System Events" to key code 9 using command down"#;
            
            let ns_script_class = objc::class!(NSAppleScript);
            let ns_script: id = msg_send![ns_script_class, alloc];
            let script_string = NSString::alloc(nil).init_str(script);
            let ns_script: id = msg_send![ns_script, initWithSource:script_string];
            
            if ns_script != nil {
                let _: id = msg_send![ns_script, executeAndReturnError:nil];
                
                // 4. 延迟一点后恢复剪贴板内容（如果之前有内容）
                std::thread::sleep(std::time::Duration::from_millis(100));
                if old_contents != nil {
                    let _: () = msg_send![general_pasteboard, clearContents];
                    let _: bool = msg_send![general_pasteboard, setString:old_contents forType:string_type];
                }
            }
            
            pool.drain();
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // 其他平台的实现
        Err("当前平台不支持文本注入".to_string())
    }
}

/// 根据用户选择的模型创建转录配置
fn create_transcription_config(model_name: &str) -> TranscriptionConfig {
    match model_name {
        "luyingwang-online" => TranscriptionConfig {
            model_name: "luyin-api".to_string(),
            language: Some("auto".to_string()),
            temperature: Some(0.0),
            is_local: false,
            api_endpoint: None,
        },
        "gpt-4o-mini" => TranscriptionConfig {
            model_name: "gpt-4o-mini".to_string(),
            language: Some("auto".to_string()),
            temperature: Some(0.0),
            is_local: false,
            api_endpoint: None,
        },
        model_name if model_name.starts_with("whisper-") => TranscriptionConfig {
            model_name: model_name.to_string(),
            language: Some("zh".to_string()),
            temperature: Some(0.0),
            is_local: true,
            api_endpoint: None,
        },
        _ => {
            // 默认使用LuYinWang在线转录服务
            println!("⚠️ 未知模型 '{}', 使用默认的LuYinWang在线服务", model_name);
            TranscriptionConfig {
                model_name: "luyin-api".to_string(),
                language: Some("auto".to_string()),
                temperature: Some(0.0),
                is_local: false,
                api_endpoint: None,
            }
        }
    }
}