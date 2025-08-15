// 权限管理命令模块

#[cfg(target_os = "macos")]
extern "C" {
    fn IOHIDCheckAccess(requestType: i32) -> i32;
}

#[cfg(target_os = "macos")]
const kIOHIDRequestTypeListenEvent: i32 = 1;

// 简化实现，不使用复杂的 objc_foundation

/// 检查权限状态的命令
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn check_permission(permission: String) -> Result<String, String> {
    match permission.as_str() {
        "accessibility" => {
            // 检查辅助功能权限
            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg("tell application \"System Events\" to get UI elements enabled")
                .output()
                .map_err(|e| e.to_string())?;
            
            let result = String::from_utf8_lossy(&output.stdout);
            if result.trim() == "true" {
                Ok("granted".to_string())
            } else {
                Ok("denied".to_string())
            }
        },
        "microphone" => {
            // 使用简化的权限检查方法
            match std::process::Command::new("osascript")
                .args(&["-e", "tell application \"System Preferences\" to return 1"])
                .output() {
                Ok(output) => {
                    if output.status.success() {
                        Ok("granted".to_string())
                    } else {
                        Ok("not-determined".to_string())
                    }
                },
                Err(_) => Ok("not-determined".to_string())
            }
        },
        "file-system" => {
            // 文件系统权限通常是自动授予的
            Ok("granted".to_string())
        },
        "notifications" => {
            // 通知权限检查
            match std::process::Command::new("osascript")
                .args(&["-e", "display notification \"权限测试\" with title \"Recording King\""])
                .output() {
                Ok(output) => {
                    if output.status.success() {
                        Ok("granted".to_string())
                    } else {
                        Ok("not-determined".to_string())
                    }
                },
                Err(_) => Ok("not-determined".to_string())
            }
        },
        "screen-recording" => {
            // 屏幕录制权限检查
            match std::process::Command::new("osascript")
                .args(&["-e", "tell application \"System Events\" to get name of first process"])
                .output() {
                Ok(output) => {
                    if output.status.success() {
                        Ok("granted".to_string())
                    } else {
                        Ok("denied".to_string())
                    }
                },
                Err(_) => Ok("denied".to_string())
            }
        },
        "automation" => {
            // 自动化权限检查
            match std::process::Command::new("osascript")
                .args(&["-e", "tell application \"System Events\" to return 1"])
                .output() {
                Ok(output) => {
                    if output.status.success() {
                        Ok("granted".to_string())
                    } else {
                        Ok("denied".to_string())
                    }
                },
                Err(_) => Ok("denied".to_string())
            }
        },
        "input-monitoring" => {
            // 输入监控权限检查
            unsafe {
                let status = IOHIDCheckAccess(kIOHIDRequestTypeListenEvent);
                let status_str = match status {
                    1 => "granted".to_string(), // kIOHIDAccessTypeGranted
                    0 => "denied".to_string(),  // kIOHIDAccessTypeDenied
                    _ => "not-determined".to_string(),
                };
                println!("⌨️ 输入监控权限状态: {} (raw: {})", status_str, status);
                Ok(status_str)
            }
        },
        _ => Ok("not-determined".to_string())
    }
}

/// 非 macOS 系统的权限检查
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn check_permission(permission: String) -> Result<String, String> {
    // 其他操作系统默认授予权限
    match permission.as_str() {
        "microphone" | "file-system" | "notifications" => Ok("granted".to_string()),
        _ => Ok("not-determined".to_string())
    }
}

/// 请求权限的命令
#[tauri::command]
pub async fn request_permission(permission: String) -> Result<bool, String> {
    match permission.as_str() {
        "accessibility" => request_accessibility_permission().await,
        "microphone" => request_microphone_permission().await,
        "input-monitoring" => request_input_monitoring_permission().await,
        _ => Ok(false)
    }
}

#[cfg(target_os = "macos")]
async fn request_accessibility_permission() -> Result<bool, String> {
    let script = r#"
        tell application "System Preferences"
            activate
            set current pane to pane "com.apple.preference.security"
            delay 1
            tell application "System Events"
                tell process "System Preferences"
                    click tab "Privacy" of tab group 1 of window "Security & Privacy"
                    delay 0.5
                    select (row 1 of table 1 of scroll area 1 of tab group 1 of window "Security & Privacy" where value of static text 1 is "Accessibility")
                end tell
            end tell
        end tell
    "#;
    
    match std::process::Command::new("osascript")
        .args(&["-e", script])
        .output() {
        Ok(_) => Ok(true),
        Err(e) => {
            eprintln!("打开系统偏好设置失败: {}", e);
            // 回退到简单的打开方法
            match std::process::Command::new("open")
                .args(&["x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"])
                .status() {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("无法打开系统偏好设置: {}", e))
            }
        }
    }
}

#[cfg(target_os = "macos")]
async fn request_microphone_permission() -> Result<bool, String> {
    match std::process::Command::new("open")
        .args(&["x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"])
        .status() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("无法打开系统偏好设置: {}", e))
    }
}

#[cfg(target_os = "macos")]
async fn request_input_monitoring_permission() -> Result<bool, String> {
    match std::process::Command::new("open")
        .args(&["x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"])
        .status() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("无法打开系统偏好设置: {}", e))
    }
}

#[cfg(not(target_os = "macos"))]
async fn request_accessibility_permission() -> Result<bool, String> {
    Ok(true) // 非 macOS 系统默认已授权
}

#[cfg(not(target_os = "macos"))]
async fn request_microphone_permission() -> Result<bool, String> {
    Ok(true) // 非 macOS 系统默认已授权
}

#[cfg(not(target_os = "macos"))]
async fn request_input_monitoring_permission() -> Result<bool, String> {
    Ok(true) // 非 macOS 系统默认已授权
}

/// 打开系统偏好设置的通用命令
#[tauri::command]
pub async fn open_system_preferences(preference_pane: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let url = match preference_pane.as_str() {
            "accessibility" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            "microphone" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone", 
            "input-monitoring" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
            "screen-recording" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
            "automation" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Automation",
            "file-system" => "x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders",
            "notifications" => "x-apple.systempreferences:com.apple.preference.notifications",
            _ => return Err("未知的偏好设置面板".to_string())
        };
        
        match std::process::Command::new("open").arg(url).status() {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("无法打开系统偏好设置: {}", e))
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // 其他平台的实现可以在这里添加
        Ok(())
    }
}