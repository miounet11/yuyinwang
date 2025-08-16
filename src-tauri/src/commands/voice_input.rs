use serde::{Deserialize, Serialize};
use tauri::command;
use crate::types::TranscriptionConfig;
// 移除未使用的 rand 导入

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

/// 开始语音录音（支持实时转录）
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
    let mut recorder = state.audio_recorder.lock();
    
    // 开始录音
    recorder.start_recording()
        .map_err(|e| format!("启动录音失败: {}", e))?;
    
    println!("🎙️ 语音录音已启动");
    
    // 如果是实时模式，启动音频电平监测和实时转录
    if realtime {
        let app_handle = app.clone();
        let recorder_clone = Arc::clone(&state.audio_recorder);
        
        // 启动后台任务监测音频电平
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            let mut last_transcription_time = std::time::Instant::now();
            
            loop {
                interval.tick().await;
                
                // 获取当前音频电平
                let is_recording = {
                    let recorder = recorder_clone.lock();
                    recorder.is_recording()
                };
                
                if !is_recording {
                    break;
                }
                
                // 获取真实的音频电平并发送到前端
                let audio_level = {
                    let recorder = recorder_clone.lock();
                    // 获取实际的音频电平
                    recorder.get_current_audio_level().unwrap_or(0.0)
                };
                
                // 发送音频电平事件
                if let Err(e) = app_handle.emit_all("audio_level", audio_level) {
                    eprintln!("发送音频电平事件失败: {}", e);
                }
                
                // 实时转录功能 - 暂时禁用模拟数据
                // TODO: 实现真实的实时转录
                // 1. 从录音器获取音频缓冲区片段
                // 2. 发送到转录服务
                // 3. 发送转录结果到前端
                
                // 暂时不发送假的转录数据
                // 只在停止录音时进行完整转录
            }
        });
        
        println!("启动实时语音转录模式");
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
    
    // 停止录音并获取音频数据
    let audio_data = {
        let mut recorder = state.audio_recorder.lock();
        
        // 检查是否正在录音，如果没有录音就直接返回空
        if !recorder.is_recording() {
            println!("⚠️ 当前没有在录音，可能已经停止或未开始");
            return Ok(String::new());
        }
        
        println!("🛑 停止录音");
        recorder.stop_recording()
            .map_err(|e| format!("停止录音失败: {}", e))?
    };
    
    if audio_data.is_empty() {
        return Ok(String::new());
    }
    
    println!("📊 录音已停止，音频样本数: {}", audio_data.len());
    println!("🎤 音频时长: {:.2}秒", audio_data.len() as f32 / 16000.0);
    println!("🔊 音频数据前10个样本: {:?}", &audio_data[..10.min(audio_data.len())]);
    
    // 如果音频数据太短，返回空字符串
    if audio_data.len() < 16000 { // 小于1秒的音频
        println!("⚠️ 音频太短，跳过转录");
        return Ok(String::new());
    }
    
    // 创建临时WAV文件
    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let temp_file = temp_dir.join(format!("voice_input_{}.wav", timestamp));
    
    // 写入WAV文件 - 修复：使用录音器的实际采样率16kHz而不是错误的48kHz
    println!("💾 准备保存WAV文件到: {:?}", temp_file);
    crate::commands::create_wav_file(&temp_file, &audio_data, 16000, 1)
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
    
    // 不要立即删除临时文件，以便重试
    // 只有在转录成功后才删除
    let should_delete = !final_text.is_empty();
    
    if should_delete {
        if let Err(e) = std::fs::remove_file(&temp_file) {
            eprintln!("清理临时文件失败: {}", e);
        }
        println!("🗑️ 已删除临时文件");
    } else {
        println!("💾 保留临时文件以便重试: {:?}", temp_file);
        // 复制到备份目录
        let backup_dir = directories::UserDirs::new()
            .and_then(|dirs| Some(dirs.document_dir()?.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("RecordingKing")
            .join("failed_transcriptions");
        
        if !backup_dir.exists() {
            std::fs::create_dir_all(&backup_dir).ok();
        }
        
        let backup_file = backup_dir.join(format!("voice_input_{}.wav", timestamp));
        if let Err(e) = std::fs::copy(&temp_file, &backup_file) {
            eprintln!("❌ 备份音频文件失败: {}", e);
        } else {
            println!("💾 音频已备份到: {:?}", backup_file);
        }
    }
    
    if final_text.is_empty() {
        println!("⚠️ 转录结果为空，可能是API问题、静音或识别失败");
        println!("🔍 音频文件大小: {} 字节", audio_data.len() * 2);  // 每个样本2字节
    } else {
        println!("✅ 语音转录成功: '{}'", final_text);
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