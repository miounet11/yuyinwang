// 权限管理命令模块

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn IOHIDCheckAccess(requestType: i32) -> i32;
}

unsafe extern "C" {
    pub fn AXIsProcessTrusted() -> bool;
}

#[cfg(target_os = "macos")]
const kIOHIDRequestTypeListenEvent: i32 = 1;

// 简化实现，不使用复杂的 objc_foundation

/// 检查权限状态的命令
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn check_permission(permission_type: String) -> Result<bool, String> {
    // Input validation - only allow specific permission types
    let allowed_permissions = [
        "accessibility",
        "microphone", 
        "file-system",
        "notifications",
        "screen-recording",
        "automation",
        "input-monitoring"
    ];
    
    if !allowed_permissions.contains(&permission_type.as_str()) {
        return Err(format!("Invalid permission type: {}", permission_type));
    }
    
    match permission_type.as_str() {
        "accessibility" => {
            // 使用 AXIsProcessTrusted 检查应用是否被信任
            let trusted = unsafe { AXIsProcessTrusted() };
            Ok(trusted)
        },
        "microphone" => {
            // 检查麦克风权限状态
            match std::process::Command::new("osascript")
                .args(&["-e", r#"
                    on run
                        try
                            set microphonePermission to (do shell script "sqlite3 ~/Library/Application\\ Support/com.apple.TCC/TCC.db \"SELECT allowed FROM access WHERE service='kTCCServiceMicrophone' AND (client LIKE '%Recording%' OR client LIKE '%recordingking%' OR client LIKE '%com.recordingking%') LIMIT 1;\"" with administrator privileges)
                            if microphonePermission is "1" then
                                return true
                            else
                                return false
                            end if
                        on error
                            -- 如果无法查询数据库，尝试实际测试麦克风
                            try
                                do shell script "rec -q -t raw -r 44100 -b 16 -c 1 -e signed-integer /dev/null trim 0 0.1 2>/dev/null"
                                return true
                            on error
                                return false
                            end try
                        end try
                    end run
                "#])
                .output() {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let has_permission = output_str.trim() == "true";
                    println!("🎤 麦克风权限检查结果: {}", has_permission);
                    Ok(has_permission)
                },
                Err(e) => {
                    println!("❌ 麦克风权限检查失败: {}", e);
                    Ok(false)
                }
            }
        },
        "file-system" => {
            // 文件系统权限通常是自动授予的
            Ok(true)
        },
        "notifications" => {
            // 通知权限检查
            match std::process::Command::new("osascript")
                .args(&["-e", "display notification \"权限测试\" with title \"Recording King\""])
                .output() {
                Ok(output) => {
                    Ok(output.status.success())
                },
                Err(_) => Ok(false)
            }
        },
        "screen-recording" => {
            // 屏幕录制权限检查
            match std::process::Command::new("osascript")
                .args(&["-e", "tell application \"System Events\" to get name of first process"])
                .output() {
                Ok(output) => {
                    Ok(output.status.success())
                },
                Err(_) => Ok(false)
            }
        },
        "automation" => {
            // 自动化权限检查
            match std::process::Command::new("osascript")
                .args(&["-e", "tell application \"System Events\" to return 1"])
                .output() {
                Ok(output) => {
                    Ok(output.status.success())
                },
                Err(_) => Ok(false)
            }
        },
        "input-monitoring" => {
            // 输入监控权限检查
            unsafe {
                let status = IOHIDCheckAccess(kIOHIDRequestTypeListenEvent);
                let has_permission = status == 1; // kIOHIDAccessTypeGranted = 1
                println!("⌨️ 输入监控权限状态: {} (raw: {})", has_permission, status);
                Ok(has_permission)
            }
        },
        _ => Ok(false)
    }
}

/// 非 macOS 系统的权限检查
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn check_permission(permission_type: String) -> Result<bool, String> {
    // Input validation - only allow specific permission types
    let allowed_permissions = [
        "accessibility",
        "microphone", 
        "file-system",
        "notifications",
        "screen-recording",
        "automation",
        "input-monitoring"
    ];
    
    if !allowed_permissions.contains(&permission_type.as_str()) {
        return Err(format!("Invalid permission type: {}", permission_type));
    }
    
    // 其他操作系统默认授予权限
    match permission_type.as_str() {
        "microphone" | "file-system" | "notifications" => Ok(true),
        _ => Ok(false)
    }
}

/// 请求权限的命令
#[tauri::command]
pub async fn request_permission(permission_type: String) -> Result<bool, String> {
    // Input validation - only allow specific permission types
    let allowed_permissions = [
        "accessibility",
        "microphone",
        "input-monitoring"
    ];
    
    if !allowed_permissions.contains(&permission_type.as_str()) {
        return Err(format!("Invalid permission type: {}", permission_type));
    }
    
    match permission_type.as_str() {
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
        // Input validation - only allow specific preference panes
        let allowed_panes = [
            "accessibility",
            "microphone",
            "input-monitoring",
            "screen-recording",
            "automation",
            "file-system",
            "notifications"
        ];
        
        if !allowed_panes.contains(&preference_pane.as_str()) {
            return Err(format!("Invalid preference pane: {}", preference_pane));
        }
        
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
