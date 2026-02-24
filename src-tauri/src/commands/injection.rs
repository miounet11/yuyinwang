use crate::core::{error::Result, injection};

#[tauri::command]
pub fn inject_text(text: String, delay_ms: Option<u64>) -> Result<()> {
    injection::inject_text(&text, delay_ms.unwrap_or(100))
}

#[tauri::command]
pub fn check_injection_permission() -> Result<bool> {
    Ok(injection::check_accessibility_permission())
}

/// 请求辅助功能权限（弹出系统引导对话框）
#[tauri::command]
pub fn request_injection_permission() -> Result<bool> {
    Ok(injection::request_accessibility_permission())
}
