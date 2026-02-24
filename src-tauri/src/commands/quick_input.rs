use crate::core::error::Result;
use crate::services::quick_input::QuickInputService;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn quick_input_is_active(service: State<'_, QuickInputService>) -> Result<bool> {
    Ok(service.is_active().await)
}

/// 注册按住说话快捷键（用 rdev 全局监听）
#[tauri::command]
pub async fn register_global_shortcut(
    app: AppHandle,
    key: String,
    activation_mode: Option<String>,
) -> Result<()> {
    let service = app.state::<QuickInputService>();
    let mode = activation_mode.as_deref().unwrap_or("hold-or-toggle");
    service.register_shortcut(&key, mode, app.clone())?;

    // 保存到设置
    let state = app.state::<crate::services::state::AppState>();
    {
        let mut settings = state.settings.lock();
        settings.shortcut_key = Some(key);
        settings.activation_mode = mode.to_string();
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

/// 更新激活模式（不重新注册快捷键）
#[tauri::command]
pub async fn update_activation_mode(app: AppHandle, mode: String) -> Result<()> {
    let service = app.state::<QuickInputService>();
    service.set_activation_mode(&mode);

    // 保存到设置
    let state = app.state::<crate::services::state::AppState>();
    {
        let mut settings = state.settings.lock();
        settings.activation_mode = mode.clone();
    }
    let settings = state.settings.lock().clone();
    let _ = state.database.save_settings(&settings);

    println!("⌨️ 激活模式已更新: {}", mode);
    Ok(())
}
