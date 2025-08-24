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
    // 在显示窗口之前先获取当前活动应用
    let active_app_info = crate::commands::voice_input::get_active_app_info_for_voice()
        .await
        .unwrap_or(crate::commands::voice_input::ActiveAppInfo {
            name: "Unknown".to_string(),
            bundle_id: None,
        });
    
    println!("当前活动应用: {}", active_app_info.name);
    
    // 显示悬浮输入窗口
    if let Some(window) = app_handle.get_window("floating-input") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        // 发送事件通知窗口已被触发，并包含原始活动应用信息
        window.emit("voice_input_triggered", &active_app_info).map_err(|e| e.to_string())?;
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
    #[serde(default = "default_realtime_injection")]
    pub realtime_injection: bool,
    #[serde(default = "default_hold_release_delay_ms")]
    pub hold_release_delay_ms: u32,
}

fn default_trigger_mode() -> String {
    "press".to_string()
}

fn default_hold_duration() -> u32 {
    300
}

fn default_realtime_injection() -> bool {
    true
}

fn default_hold_release_delay_ms() -> u32 {
    150
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
            realtime_injection: default_realtime_injection(),
            hold_release_delay_ms: default_hold_release_delay_ms(),
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

// Week 3: 渐进式触发系统命令

/// 启动渐进式长按触发监听
#[tauri::command]
pub async fn start_progressive_trigger_monitoring(
    app: tauri::AppHandle,
    config: Option<crate::shortcuts::ProgressiveTriggerConfig>,
) -> Result<String, String> {
    use crate::shortcuts::{ProgressiveTriggerManager, ProgressiveTriggerConfig};
    use std::sync::{Arc, Mutex};
    
    let trigger_config = config.unwrap_or_else(ProgressiveTriggerConfig::default);
    println!("🚀 启动渐进式长按触发监听: {:?}", trigger_config.shortcut);
    
    // 创建触发管理器 (这里简化为直接使用，实际应用中可能需要全局状态管理)
    let mut manager = ProgressiveTriggerManager::new(trigger_config);
    
    match manager.initialize(app.clone()) {
        Ok(_) => {
            match manager.start_monitoring().await {
                Ok(message) => {
                    println!("✅ 渐进式触发监听启动成功: {}", message);
                    Ok(message)
                }
                Err(e) => {
                    println!("❌ 启动失败: {}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            println!("❌ 初始化失败: {}", e);
            Err(e)
        }
    }
}

/// 更新渐进式触发配置
#[tauri::command]
pub async fn update_progressive_trigger_config(
    config: crate::shortcuts::ProgressiveTriggerConfig,
) -> Result<String, String> {
    // TODO: 实际实现中需要访问全局管理器实例
    println!("🔧 更新渐进式触发配置: {:?}", config);
    Ok("配置已更新".to_string())
}

/// 获取渐进式触发状态
#[tauri::command]
pub async fn get_progressive_trigger_status() -> Result<String, String> {
    use crate::shortcuts::ProgressiveTriggerConfig;
    
    let config = ProgressiveTriggerConfig::default();
    
    Ok(serde_json::json!({
        "monitoring": false, // TODO: 从全局状态获取
        "state": "idle",
        "config": {
            "shortcut": config.shortcut,
            "threshold_ms": config.long_press_threshold_ms,
            "enabled": config.enabled,
            "real_time_injection": config.enable_real_time_injection,
            "sound_enabled": config.trigger_sound_enabled,
            "auto_detect_app": config.auto_detect_target_app,
        }
    }).to_string())
}

/// 测试渐进式触发
#[tauri::command]
pub async fn test_progressive_trigger(
    app: tauri::AppHandle,
    target_bundle_id: Option<String>,
) -> Result<String, String> {
    println!("🧪 测试渐进式语音输入触发");
    
    // 直接调用渐进式语音输入
    match crate::commands::start_progressive_voice_input(
        target_bundle_id,
        app.clone(),
        Some(true), // 启用实时注入
    ).await {
        Ok(message) => {
            println!("✅ 测试成功: {}", message);
            
            // 发送测试事件
            if let Err(e) = app.emit_all("progressive_trigger_test_complete", serde_json::json!({
                "success": true,
                "message": message,
                "timestamp": chrono::Utc::now().timestamp_millis(),
            })) {
                eprintln!("发送测试事件失败: {}", e);
            }
            
            Ok(format!("渐进式触发测试成功: {}", message))
        }
        Err(e) => {
            println!("❌ 测试失败: {}", e);
            
            // 发送错误事件
            if let Err(emit_error) = app.emit_all("progressive_trigger_test_error", serde_json::json!({
                "success": false,
                "error": e.clone(),
                "timestamp": chrono::Utc::now().timestamp_millis(),
            })) {
                eprintln!("发送测试错误事件失败: {}", emit_error);
            }
            
            Err(format!("渐进式触发测试失败: {}", e))
        }
    }
}