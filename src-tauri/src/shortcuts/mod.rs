use tauri::{AppHandle, GlobalShortcutManager, Manager};
use std::sync::{Arc, Mutex};
use crate::errors::AppResult;

#[cfg(target_os = "macos")]
use objc::{sel, sel_impl};

pub struct ShortcutManager {
    app_handle: AppHandle,
    registered_shortcuts: Arc<Mutex<Vec<String>>>,
}

impl ShortcutManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            registered_shortcuts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 注册快速语音输入快捷键
    pub fn register_voice_input_shortcut(&self, shortcut: &str) -> AppResult<()> {
        let app_handle = self.app_handle.clone();
        let shortcut_str = shortcut.to_string();
        let shortcut_clone = shortcut_str.clone();
        
        // 注册全局快捷键
        self.app_handle
            .global_shortcut_manager()
            .register(&shortcut_str, move || {
                println!("🎤 快速语音输入快捷键触发: {}", shortcut_clone);
                
                // 发送事件到前端
                app_handle
                    .emit_all("quick_voice_input_triggered", ())
                    .unwrap_or_else(|e| {
                        eprintln!("发送快速语音输入事件失败: {}", e);
                    });
                
                // 创建或显示快速输入窗口
                let _ = create_quick_input_window(&app_handle);
            })
            .map_err(|e| {
                crate::errors::AppError::ShortcutError(format!(
                    "注册快捷键 {} 失败: {}",
                    shortcut, e
                ))
            })?;

        // 记录已注册的快捷键
        self.registered_shortcuts.lock().unwrap().push(shortcut.to_string());
        
        println!("✅ 已注册快速语音输入快捷键: {}", shortcut);
        Ok(())
    }

    /// 注销所有快捷键
    pub fn unregister_all(&self) -> AppResult<()> {
        let shortcuts = self.registered_shortcuts.lock().unwrap();
        for shortcut in shortcuts.iter() {
            self.app_handle
                .global_shortcut_manager()
                .unregister(shortcut)
                .map_err(|e| {
                    crate::errors::AppError::ShortcutError(format!(
                        "注销快捷键 {} 失败: {}",
                        shortcut, e
                    ))
                })?;
        }
        Ok(())
    }
}

/// 创建快速输入窗口
fn create_quick_input_window(app_handle: &AppHandle) -> AppResult<()> {
    use tauri::{WindowBuilder, WindowUrl};
    
    // 检查窗口是否已存在
    if let Some(window) = app_handle.get_window("quick-voice-input") {
        // 如果窗口已存在，显示并聚焦
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    
    // 创建新的快速输入窗口
    let window = WindowBuilder::new(
        app_handle,
        "quick-voice-input",
        WindowUrl::App("quick-voice-input".into()),
    )
    .title("")
    .decorations(false)
    .always_on_top(true)
    .resizable(false)
    .skip_taskbar(true)
    .inner_size(400.0, 120.0)
    .build()
    .map_err(|e| {
        crate::errors::AppError::WindowError(format!("创建快速输入窗口失败: {}", e))
    })?;
    
    // 设置窗口位置（跟随鼠标）
    if let Ok(position) = get_cursor_position() {
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: position.0 as i32,
            y: (position.1 - 60.0) as i32,
        }));
    }
    
    Ok(())
}

/// 获取鼠标光标位置
#[cfg(target_os = "macos")]
fn get_cursor_position() -> AppResult<(f64, f64)> {
    use cocoa::foundation::NSPoint;
    use objc::runtime::Object;
    
    unsafe {
        let ns_event_class = objc::class!(NSEvent);
        let mouse_location: NSPoint = objc::msg_send![ns_event_class, mouseLocation];
        
        // macOS 坐标系原点在左下角，需要转换
        let screens: *mut Object = objc::msg_send![objc::class!(NSScreen), screens];
        let main_screen: *mut Object = objc::msg_send![screens, objectAtIndex:0];
        let frame: cocoa::foundation::NSRect = objc::msg_send![main_screen, frame];
        
        Ok((mouse_location.x, frame.size.height - mouse_location.y))
    }
}

#[cfg(not(target_os = "macos"))]
fn get_cursor_position() -> AppResult<(f64, f64)> {
    // 其他平台的实现
    Ok((100.0, 100.0))
}

/// 插入文本到当前应用
#[cfg(target_os = "macos")]
pub fn insert_text_to_active_app(text: &str) -> AppResult<()> {
    use std::process::Command;
    
    // 使用 AppleScript 插入文本
    let script = format!(
        r#"
        tell application "System Events"
            keystroke "{}"
        end tell
        "#,
        text.replace("\"", "\\\"")
    );
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| {
            crate::errors::AppError::SystemError(format!("执行 AppleScript 失败: {}", e))
        })?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(crate::errors::AppError::SystemError(format!(
            "插入文本失败: {}",
            error
        )));
    }
    
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn insert_text_to_active_app(text: &str) -> AppResult<()> {
    // 其他平台的实现
    println!("插入文本: {}", text);
    Ok(())
}