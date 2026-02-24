use crate::core::error::Result;
use crate::services::quick_input::QuickInputService;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn quick_input_is_active(service: State<'_, QuickInputService>) -> Result<bool> {
    Ok(service.is_active().await)
}

/// 注册按住说话快捷键（用 rdev 全局监听）
#[tauri::command]
pub async fn register_global_shortcut(app: AppHandle, key: String) -> Result<()> {
    let service = app.state::<QuickInputService>();
    service.register_shortcut(&key, app.clone())?;

    // 保存到设置
    let state = app.state::<crate::services::state::AppState>();
    {
        let mut settings = state.settings.lock();
        settings.shortcut_key = Some(key);
    }
    let settings = state.settings.lock().clone();
    let _ = state.database.save_settings(&settings);

    Ok(())
}

/// 取消快捷键
#[tauri::command]
pub async fn unregister_global_shortcut(app: AppHandle, key: String) -> Result<()> {
    let service = app.state::<QuickInputService>();
    service.unregister_shortcut();

    // 清除设置
    let state = app.state::<crate::services::state::AppState>();
    {
        let mut settings = state.settings.lock();
        settings.shortcut_key = None;
    }
    let settings = state.settings.lock().clone();
    let _ = state.database.save_settings(&settings);

    println!("⌨️ 快捷键已取消: {}", key);
    Ok(())
}
