use tauri::{State, Manager, GlobalShortcutManager};
use std::sync::Arc;
use crate::shortcuts::{ShortcutManager, insert_text_to_active_app};

#[cfg(target_os = "macos")]
use objc::{sel, sel_impl};

/// 注册快速语音输入快捷键
#[tauri::command]
pub async fn register_voice_shortcut(
    shortcut_manager: State<'_, Arc<ShortcutManager>>,
    shortcut: String,
    trigger_mode: Option<String>,
) -> Result<(), String> {
    let mode = trigger_mode.as_deref().unwrap_or("press");
    shortcut_manager
        .register_voice_input_shortcut(&shortcut, mode)
        .map_err(|e| e.to_string())
}

/// 注销所有语音快捷键
#[tauri::command]
pub async fn unregister_all_voice_shortcuts(
    shortcut_manager: State<'_, Arc<ShortcutManager>>,
) -> Result<(), String> {
    shortcut_manager
        .unregister_all()
        .map_err(|e| e.to_string())
}

/// 获取鼠标光标位置
#[tauri::command]
pub async fn get_cursor_position() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::foundation::NSPoint;
        use objc::runtime::Object;
        
        unsafe {
            let ns_event_class = objc::class!(NSEvent);
            let mouse_location: NSPoint = objc::msg_send![ns_event_class, mouseLocation];
            
            // macOS 坐标系原点在左下角，需要转换
            let screens: *mut Object = objc::msg_send![objc::class!(NSScreen), screens];
            let main_screen: *mut Object = objc::msg_send![screens, objectAtIndex:0];
            let frame: cocoa::foundation::NSRect = objc::msg_send![main_screen, frame];
            
            Ok(serde_json::json!({
                "x": mouse_location.x,
                "y": frame.size.height - mouse_location.y
            }))
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Ok(serde_json::json!({
            "x": 100,
            "y": 100
        }))
    }
}

/// 插入文本到当前活动应用
#[tauri::command]
pub async fn insert_text_to_app(text: String) -> Result<(), String> {
    insert_text_to_active_app(&text)
        .map_err(|e| e.to_string())
}

/// 配置快捷键设置
#[tauri::command]
pub async fn configure_voice_shortcuts(
    shortcut_manager: State<'_, Arc<ShortcutManager>>,
    config: VoiceShortcutConfig,
) -> Result<(), String> {
    // 先注销所有现有快捷键
    let _ = shortcut_manager.unregister_all();
    
    // 注册新的快捷键
    if config.enabled {
        shortcut_manager
            .register_voice_input_shortcut(&config.shortcut, &config.trigger_mode)
            .map_err(|e| e.to_string())?;
    }
    
    // 保存配置到本地存储
    save_shortcut_config(config).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 加载快捷键配置
#[tauri::command]
pub async fn load_voice_shortcut_config() -> Result<VoiceShortcutConfig, String> {
    load_shortcut_config().map_err(|e| e.to_string())
}

/// 触发语音输入测试
#[tauri::command]
pub async fn trigger_voice_input_test(
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 发送事件到前端触发测试
    app_handle
        .emit_all("quick_voice_input_triggered", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 调试快捷键状态
#[tauri::command]
pub async fn debug_shortcut_status(
    shortcut_manager: State<'_, Arc<ShortcutManager>>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    println!("🔍 调试快捷键状态...");
    
    // 检查注册的快捷键
    let registered = shortcut_manager.registered_shortcuts.lock().unwrap();
    println!("📋 已注册的快捷键: {:?}", *registered);
    
    // 检查全局快捷键管理器状态
    let is_registered = if !registered.is_empty() {
        // 由于API限制，我们只能检查是否有注册的快捷键记录
        true
    } else {
        false
    };
    
    // 加载配置文件
    let config = load_shortcut_config().map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "registered_shortcuts": *registered,
        "is_registered": is_registered,
        "config": config,
        "manager_available": true
    }))
}

/// 显示悬浮输入窗口
#[tauri::command]
pub async fn show_floating_input(
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 显示悬浮输入窗口
    if let Some(window) = app_handle.get_window("floating-input") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        // 发送事件通知窗口已被触发 - 使用新的事件名称
        window.emit("voice_input_triggered", ()).map_err(|e| e.to_string())?;
    } else {
        return Err("悬浮输入窗口未找到".to_string());
    }
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VoiceShortcutConfig {
    pub enabled: bool,
    pub shortcut: String,
    pub auto_insert: bool,
    pub use_floating_window: bool,
    pub preferred_model: String,
    #[serde(default = "default_trigger_mode")]
    pub trigger_mode: String,
    #[serde(default = "default_hold_duration")]
    pub hold_duration: u32,
}

fn default_trigger_mode() -> String {
    "press".to_string()
}

fn default_hold_duration() -> u32 {
    300
}

impl Default for VoiceShortcutConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shortcut: "CmdOrCtrl+Shift+Y".to_string(),
            auto_insert: true,
            use_floating_window: true,
            preferred_model: "luyingwang-online".to_string(),
            trigger_mode: default_trigger_mode(),
            hold_duration: default_hold_duration(),
        }
    }
}

fn save_shortcut_config(config: VoiceShortcutConfig) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    
    let config_dir = directories::BaseDirs::new()
        .ok_or("无法获取用户目录")?
        .config_dir()
        .join("recording-king");
    
    fs::create_dir_all(&config_dir)?;
    
    let config_path = config_dir.join("voice_shortcuts.json");
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, json)?;
    
    Ok(())
}

pub fn load_shortcut_config() -> Result<VoiceShortcutConfig, Box<dyn std::error::Error>> {
    use std::fs;
    
    let config_path = directories::BaseDirs::new()
        .ok_or("无法获取用户目录")?
        .config_dir()
        .join("recording-king")
        .join("voice_shortcuts.json");
    
    if !config_path.exists() {
        return Ok(VoiceShortcutConfig::default());
    }
    
    let json = fs::read_to_string(config_path)?;
    let config = serde_json::from_str(&json)?;
    Ok(config)
}

/// 启动长按快捷键监听 (简化版)
#[tauri::command]
pub async fn start_long_press_monitoring(app: tauri::AppHandle) -> Result<String, String> {
    println!("🔄 启动长按快捷键监听 (使用Option+L模拟)");
    
    let shortcut = "Option+L";
    let app_clone = app.clone();
    
    match app.global_shortcut_manager().register(shortcut, move || {
        println!("🎙️ 长按快捷键触发 (Option+L)");
        
        if let Some(window) = app_clone.get_window("floating-input") {
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.emit("voice_input_triggered", serde_json::json!({
                "trigger": "long_press_simulation",
                "key_combo": "option+l",
                "timestamp": chrono::Utc::now().timestamp_millis()
            }));
            println!("✅ 语音输入窗口已触发 (模拟长按)");
        } else {
            println!("❌ 悬浮输入窗口未找到");
        }
    }) {
        Ok(_) => {
            println!("✅ 长按快捷键监听已启动 (使用 Option+L 模拟)");
            Ok("长按快捷键监听已启动 (使用 Option+L 模拟)".to_string())
        },
        Err(e) => {
            println!("❌ 启动长按快捷键监听失败: {}", e);
            Err(format!("启动失败: {}", e))
        }
    }
}

/// 测试长按触发
#[tauri::command] 
pub async fn test_long_press_trigger(app: tauri::AppHandle) -> Result<String, String> {
    println!("🧪 测试长按触发功能");
    
    if let Some(window) = app.get_window("floating-input") {
        match window.show() {
            Ok(_) => {
                let _ = window.set_focus();
                let _ = window.emit("voice_input_triggered", serde_json::json!({
                    "trigger": "test",
                    "key_combo": "test",
                    "timestamp": chrono::Utc::now().timestamp_millis()
                }));
                Ok("长按触发测试完成 - 悬浮窗口已显示".to_string())
            },
            Err(e) => {
                Err(format!("显示悬浮窗口失败: {}", e))
            }
        }
    } else {
        Err("悬浮输入窗口未找到".to_string())
    }
}

/// 获取长按状态
#[tauri::command]
pub async fn get_long_press_status() -> Result<String, String> {
    Ok(serde_json::json!({
        "enabled": true,
        "threshold_ms": 500,
        "monitored_keys": ["option+l (模拟长按)"],
        "description": "使用 Option+L 模拟长按 Option+Space 触发语音输入",
        "note": "这是简化版实现，使用普通快捷键模拟长按效果"
    }).to_string())
}