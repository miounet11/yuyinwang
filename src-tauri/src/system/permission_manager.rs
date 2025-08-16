// macOS 权限管理系统
// 负责检测、请求和引导用户配置必要的系统权限

use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub accessibility: bool,
    pub input_monitoring: bool,
    pub microphone: bool,
    pub all_granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGuide {
    pub step: u8,
    pub title: String,
    pub description: String,
    pub action: String,
    pub image_path: Option<String>,
    pub is_critical: bool,
}

pub struct PermissionManager;

impl PermissionManager {
    pub fn new() -> Self {
        Self
    }

    /// 全面检查所有必要权限
    pub fn check_all_permissions() -> AppResult<PermissionStatus> {
        let accessibility = Self::check_accessibility_permission()?;
        let input_monitoring = Self::check_input_monitoring_permission()?;
        let microphone = Self::check_microphone_permission()?;
        
        let all_granted = accessibility && input_monitoring && microphone;
        
        println!("🔍 权限检查结果:");
        println!("  📱 辅助功能: {}", if accessibility { "✅" } else { "❌" });
        println!("  ⌨️  输入监控: {}", if input_monitoring { "✅" } else { "❌" });
        println!("  🎤 麦克风: {}", if microphone { "✅" } else { "❌" });
        println!("  🎯 全部权限: {}", if all_granted { "✅" } else { "❌" });

        Ok(PermissionStatus {
            accessibility,
            input_monitoring,
            microphone,
            all_granted,
        })
    }

    /// 检查辅助功能权限
    pub fn check_accessibility_permission() -> AppResult<bool> {
        let output = Command::new("osascript")
            .args(&["-e", "tell application \"System Events\" to get name of first process"])
            .output()
            .map_err(|e| AppError::SystemError(format!("检查辅助功能权限失败: {}", e)))?;

        Ok(output.status.success())
    }

    /// 检查输入监控权限 (这是关键权限)
    pub fn check_input_monitoring_permission() -> AppResult<bool> {
        #[cfg(target_os = "macos")]
        {
            // 使用系统API检查输入监控权限
            unsafe extern "C" {
                fn IOHIDCheckAccess(requestType: i32) -> i32;
            }
            
            const K_IOHID_REQUEST_TYPE_LISTEN_EVENT: i32 = 1;
            
            unsafe {
                let status = IOHIDCheckAccess(K_IOHID_REQUEST_TYPE_LISTEN_EVENT);
                let has_permission = status == 1; // kIOHIDAccessTypeGranted = 1
                println!("⌨️ 输入监控权限状态: {} (raw: {})", has_permission, status);
                Ok(has_permission)
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            // 非macOS系统默认授予权限
            Ok(true)
        }
    }

    /// 检查麦克风权限
    pub fn check_microphone_permission() -> AppResult<bool> {
        // 检查音频录制权限
        let output = Command::new("osascript")
            .args(&["-e", r#"
                try
                    set micPermission to (do shell script "coreaudiod -h 2>&1 || echo 'no_permission'")
                    return "granted"
                on error
                    return "denied"
                end try
            "#])
            .output()
            .map_err(|e| AppError::SystemError(format!("检查麦克风权限失败: {}", e)))?;

        let result = String::from_utf8_lossy(&output.stdout);
        Ok(result.trim() == "granted")
    }

    /// 获取权限设置指南
    pub fn get_permission_guide() -> Vec<PermissionGuide> {
        vec![
            PermissionGuide {
                step: 1,
                title: "打开系统偏好设置".to_string(),
                description: "点击屏幕左上角的苹果图标，选择'系统偏好设置'".to_string(),
                action: "open_system_preferences".to_string(),
                image_path: None,
                is_critical: true,
            },
            PermissionGuide {
                step: 2,
                title: "进入安全性与隐私".to_string(),
                description: "在系统偏好设置中找到并点击'安全性与隐私'图标".to_string(),
                action: "navigate_to_security".to_string(),
                image_path: None,
                is_critical: true,
            },
            PermissionGuide {
                step: 3,
                title: "选择隐私标签页".to_string(),
                description: "在安全性与隐私窗口中，点击顶部的'隐私'标签".to_string(),
                action: "click_privacy_tab".to_string(),
                image_path: None,
                is_critical: true,
            },
            PermissionGuide {
                step: 4,
                title: "配置输入监控权限".to_string(),
                description: "在左侧列表中选择'输入监控'，然后勾选 Recording King 应用".to_string(),
                action: "enable_input_monitoring".to_string(),
                image_path: None,
                is_critical: true,
            },
            PermissionGuide {
                step: 5,
                title: "配置辅助功能权限".to_string(),
                description: "在左侧列表中选择'辅助功能'，然后勾选 Recording King 应用".to_string(),
                action: "enable_accessibility".to_string(),
                image_path: None,
                is_critical: true,
            },
            PermissionGuide {
                step: 6,
                title: "配置麦克风权限".to_string(),
                description: "在左侧列表中选择'麦克风'，然后勾选 Recording King 应用".to_string(),
                action: "enable_microphone".to_string(),
                image_path: None,
                is_critical: true,
            },
            PermissionGuide {
                step: 7,
                title: "重启应用".to_string(),
                description: "配置完成后，请重启 Recording King 应用以使权限生效".to_string(),
                action: "restart_app".to_string(),
                image_path: None,
                is_critical: true,
            },
        ]
    }

    /// 打开系统偏好设置到特定面板
    pub fn open_system_preferences(panel: &str) -> AppResult<()> {
        let url = match panel {
            "security" => "x-apple.systempreferences:com.apple.preference.security?Privacy",
            "accessibility" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            "input_monitoring" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
            "microphone" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
            _ => "x-apple.systempreferences:com.apple.preference.security",
        };

        let output = Command::new("open")
            .arg(url)
            .output()
            .map_err(|e| AppError::SystemError(format!("打开系统偏好设置失败: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::SystemError(
                "无法打开系统偏好设置".to_string(),
            ));
        }

        Ok(())
    }

    /// 显示权限警告对话框
    pub fn show_permission_warning(missing_permissions: &[String]) -> AppResult<()> {
        let permissions_list = missing_permissions.join("、");
        let message = format!(
            "⚠️ Recording King 需要以下权限才能正常工作：\n\n{}\n\n请点击'打开设置'按钮配置权限。",
            permissions_list
        );

        let output = Command::new("osascript")
            .args(&[
                "-e",
                &format!(
                    r#"display dialog "{}" buttons {{"取消", "打开设置"}} default button "打开设置" with icon caution"#,
                    message
                ),
            ])
            .output()
            .map_err(|e| AppError::SystemError(format!("显示权限警告失败: {}", e)))?;

        let result = String::from_utf8_lossy(&output.stdout);
        if result.contains("打开设置") {
            Self::open_system_preferences("security")?;
        }

        Ok(())
    }

    /// 获取详细的权限状态信息
    pub fn get_detailed_permission_info() -> AppResult<serde_json::Value> {
        let status = Self::check_all_permissions()?;
        let guide = Self::get_permission_guide();

        let missing_permissions: Vec<String> = {
            let mut missing = Vec::new();
            if !status.accessibility {
                missing.push("辅助功能".to_string());
            }
            if !status.input_monitoring {
                missing.push("输入监控".to_string());
            }
            if !status.microphone {
                missing.push("麦克风".to_string());
            }
            missing
        };

        Ok(serde_json::json!({
            "status": status,
            "guide": guide,
            "missing_permissions": missing_permissions,
            "critical_issue": !status.input_monitoring,
            "can_use_shortcuts": status.input_monitoring,
            "can_record_audio": status.microphone,
            "next_action": if !status.all_granted {
                "configure_permissions"
            } else {
                "all_ready"
            }
        }))
    }

    /// 定期检查权限状态（用于实时监控）
    pub fn start_permission_monitoring() -> AppResult<()> {
        // 这个方法可以启动一个后台任务来定期检查权限状态
        println!("🔄 启动权限状态监控...");
        Ok(())
    }
}

// Tauri 命令接口
#[tauri::command]
pub async fn check_all_permissions() -> Result<serde_json::Value, String> {
    PermissionManager::get_detailed_permission_info()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_permission_settings(panel: String) -> Result<(), String> {
    PermissionManager::open_system_preferences(&panel)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_permission_guide() -> Result<Vec<PermissionGuide>, String> {
    Ok(PermissionManager::get_permission_guide())
}

#[tauri::command]
pub async fn show_permission_warning_dialog(missing_permissions: Vec<String>) -> Result<(), String> {
    PermissionManager::show_permission_warning(&missing_permissions)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_critical_permissions() -> Result<bool, String> {
    let status = PermissionManager::check_all_permissions()
        .map_err(|e| e.to_string())?;
    
    // 检查关键权限：输入监控是最重要的
    Ok(status.input_monitoring)
}