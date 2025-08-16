use serde::{Deserialize, Serialize};
use tauri::command;

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
    device_id: String,
    realtime: bool,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use crate::AppState;
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    let mut recorder = state.audio_recorder.lock();
    
    // 设置实时模式
    if realtime {
        // 这里可以设置实时转录的配置
        println!("启动实时语音转录模式");
    }
    
    // 开始录音
    recorder.start_recording()
        .map_err(|e| format!("启动录音失败: {}", e))?;
    
    Ok("录音已开始".to_string())
}

/// 停止语音录音并返回最终转录结果
#[command]
pub async fn stop_voice_recording(app: tauri::AppHandle) -> Result<String, String> {
    use crate::AppState;
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    let mut recorder = state.audio_recorder.lock();
    
    // 停止录音并获取音频数据
    let audio_data = recorder.stop_recording()
        .map_err(|e| format!("停止录音失败: {}", e))?;
    
    if audio_data.is_empty() {
        return Ok(String::new());
    }
    
    // 这里应该调用转录服务
    // 暂时返回模拟数据
    Ok("你好，这是语音转录的结果".to_string())
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