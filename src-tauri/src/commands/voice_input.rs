use serde::{Deserialize, Serialize};
use tauri::command;
use rand::Rng;
use uuid::Uuid;

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
                
                // 获取音频电平并发送到前端
                let audio_level = {
                    let recorder = recorder_clone.lock();
                    // 这里应该从录音器获取实际的音频电平
                    // 暂时使用模拟值
                    if rand::random::<f32>() > 0.3 {
                        0.1 + rand::random::<f32>() * 0.5
                    } else {
                        0.01
                    }
                };
                
                // 发送音频电平事件
                if let Err(e) = app_handle.emit_all("audio_level", audio_level) {
                    eprintln!("发送音频电平事件失败: {}", e);
                }
                
                // 每2秒发送一次实时转录（模拟）
                if last_transcription_time.elapsed() > Duration::from_secs(2) {
                    // 模拟实时转录文本
                    let transcribed_text = match rand::random::<u8>() % 3 {
                        0 => "你好，请问有什么可以帮助你的",
                        1 => "今天天气真不错",
                        _ => "我正在录音并实时转录",
                    };
                    
                    // 发送实时转录事件
                    if let Err(e) = app_handle.emit_all("realtime_transcription", transcribed_text) {
                        eprintln!("发送实时转录事件失败: {}", e);
                    }
                    
                    last_transcription_time = std::time::Instant::now();
                }
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
    
    // 停止录音并获取音频数据
    let audio_data = {
        let mut recorder = state.audio_recorder.lock();
        recorder.stop_recording()
            .map_err(|e| format!("停止录音失败: {}", e))?
    };
    
    if audio_data.is_empty() {
        return Ok(String::new());
    }
    
    println!("📊 录音已停止，音频样本数: {}", audio_data.len());
    
    // 创建临时WAV文件
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("voice_input_{}.wav", uuid::Uuid::new_v4()));
    
    // 写入WAV文件
    crate::commands::create_wav_file(&temp_file, &audio_data, 48000, 1)
        .map_err(|e| format!("创建WAV文件失败: {}", e))?;
    
    // 使用默认模型进行转录
    let config = TranscriptionConfig {
        model_name: "whisper-tiny".to_string(),
        language: Some("zh".to_string()),
        temperature: Some(0.0),
        is_local: true,
        api_endpoint: None,
    };
    
    // 进行转录
    let result = state.transcription_service
        .transcribe_audio(temp_file.to_str().unwrap(), &config)
        .await
        .map_err(|e| format!("转录失败: {}", e))?;
    
    // 清理临时文件
    if let Err(e) = std::fs::remove_file(&temp_file) {
        eprintln!("清理临时文件失败: {}", e);
    }
    
    println!("✅ 语音转录完成: {}", result.text);
    
    Ok(result.text)
}

/// 将文本注入到当前活动的应用
#[command]
pub async fn inject_text_to_active_app(text: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSAutoreleasePool, NSString};
        use objc::{msg_send, sel, sel_impl};
        
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            
            // 使用 AppleScript 注入文本
            let script = format!(
                r#"tell application "System Events" to keystroke "{}""#,
                text.replace("\"", "\\\"")
            );
            
            let ns_script_class = objc::class!(NSAppleScript);
            let ns_script: id = msg_send![ns_script_class, alloc];
            let script_string = NSString::alloc(nil).init_str(&script);
            let ns_script: id = msg_send![ns_script, initWithSource:script_string];
            
            if ns_script != nil {
                let _: id = msg_send![ns_script, executeAndReturnError:nil];
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