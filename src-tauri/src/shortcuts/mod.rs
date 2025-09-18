use crate::errors::AppResult;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, GlobalShortcutManager, Manager};

#[cfg(target_os = "macos")]
use objc::{sel, sel_impl};

mod fn_key_listener;
pub use fn_key_listener::FnKeyListener;

pub mod global_shortcuts;
pub use global_shortcuts::EnhancedShortcutManager;

pub mod unified_shortcut_manager;
pub use unified_shortcut_manager::{PerformanceReport, ShortcutMetrics, UnifiedShortcutManager};

pub struct ShortcutManager {
    app_handle: AppHandle,
    pub registered_shortcuts: Arc<Mutex<Vec<String>>>,
    fn_key_listener: Option<Arc<FnKeyListener>>,
}

impl ShortcutManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let fn_listener = Arc::new(FnKeyListener::new(app_handle.clone()));

        Self {
            app_handle,
            registered_shortcuts: Arc::new(Mutex::new(Vec::new())),
            fn_key_listener: Some(fn_listener),
        }
    }

    /// 启动 Fn 键监听器
    pub fn start_fn_key_listener(&self) -> AppResult<()> {
        if let Some(listener) = &self.fn_key_listener {
            listener.start()?;
            println!("✅ Fn 键监听器已启动（双击 Fn 键触发悬浮输入）");
        }
        Ok(())
    }

    /// 注册快速语音输入快捷键（支持长按和单击模式）
    pub fn register_voice_input_shortcut(
        &self,
        shortcut: &str,
        trigger_mode: &str,
    ) -> AppResult<()> {
        // 如果要求使用Fn键，则使用Option键作为替代
        let actual_shortcut = if shortcut == "Fn" || shortcut.contains("Fn") {
            "Option+Space".to_string()
        } else {
            shortcut.to_string()
        };

        println!(
            "🔧 开始注册快速语音输入快捷键: {} (模式: {})",
            actual_shortcut, trigger_mode
        );

        // 先检查是否已注册
        let is_registered = self
            .app_handle
            .global_shortcut_manager()
            .is_registered(&actual_shortcut)
            .unwrap_or(false);

        if is_registered {
            println!("⚠️ 快捷键 {} 已注册，先注销", actual_shortcut);
            let _ = self
                .app_handle
                .global_shortcut_manager()
                .unregister(&actual_shortcut);
        }

        let app_handle = self.app_handle.clone();
        let shortcut_str = actual_shortcut.clone();
        let shortcut_clone = shortcut_str.clone();
        let trigger_mode_clone = trigger_mode.to_string();

        // 注册全局快捷键
        let register_result =
            self.app_handle
                .global_shortcut_manager()
                .register(&shortcut_str, move || {
                    println!(
                        "🎤 快速语音输入快捷键触发: {} (模式: {})",
                        shortcut_clone, trigger_mode_clone
                    );
                    eprintln!(
                        "🎤 快速语音输入快捷键触发: {} (模式: {})",
                        shortcut_clone, trigger_mode_clone
                    );

                    // 添加系统日志
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("osascript")
                            .arg("-e")
                            .arg(&format!(
                            "display notification \"快捷键触发: {}\" with title \"Recording King\"",
                            shortcut_clone
                        ))
                            .spawn();
                    }

                    // 显示悬浮输入窗口
                    if let Some(window) = app_handle.get_window("floating-input") {
                        // 显示窗口
                        if let Err(e) = window.show() {
                            eprintln!("❌ 显示窗口失败: {}", e);
                        }
                        if let Err(e) = window.set_focus() {
                            eprintln!("❌ 设置焦点失败: {}", e);
                        }
                        // 发送触发事件到窗口
                        if let Err(e) = window.emit("floating_input_triggered", ()) {
                            eprintln!("❌ 发送事件失败: {}", e);
                        }

                        // 根据触发模式发送不同的事件
                        if trigger_mode_clone == "hold" {
                            window
                                .emit("voice_input_hold_start", ())
                                .unwrap_or_else(|e| {
                                    eprintln!("发送长按开始事件失败: {}", e);
                                });
                        }
                    } else {
                        eprintln!("❌ 悬浮输入窗口未找到，尝试创建快速输入窗口");
                        // 回退：创建或显示快速输入窗口
                        let _ = create_quick_input_window(&app_handle);
                    }
                });

        match register_result {
            Ok(_) => {
                println!("✅ 快捷键注册成功: {}", actual_shortcut);
            }
            Err(e) => {
                eprintln!("❌ 快捷键注册失败: {} - {}", actual_shortcut, e);
                return Err(crate::errors::AppError::ShortcutError(format!(
                    "注册快捷键 {} 失败: {}",
                    actual_shortcut, e
                )));
            }
        }

        // 记录已注册的快捷键
        self.registered_shortcuts
            .lock()
            .unwrap()
            .push(actual_shortcut.clone());

        println!(
            "✅ 已注册快速语音输入快捷键: {} (模式: {})",
            actual_shortcut, trigger_mode
        );
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
    .map_err(|e| crate::errors::AppError::WindowError(format!("创建快速输入窗口失败: {}", e)))?;

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
