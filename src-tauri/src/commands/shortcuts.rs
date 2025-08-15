use tauri::{State, Manager};
use std::sync::Arc;
use crate::shortcuts::{ShortcutManager, insert_text_to_active_app};

#[cfg(target_os = "macos")]
use objc::{sel, sel_impl};

/// 注册快速语音输入快捷键
#[tauri::command]
pub async fn register_voice_shortcut(
    shortcut_manager: State<'_, Arc<ShortcutManager>>,
    shortcut: String,
) -> Result<(), String> {
    shortcut_manager
        .register_voice_input_shortcut(&shortcut)
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
            .register_voice_input_shortcut(&config.shortcut)
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VoiceShortcutConfig {
    pub enabled: bool,
    pub shortcut: String,
    pub auto_insert: bool,
    pub use_floating_window: bool,
    pub preferred_model: String,
}

impl Default for VoiceShortcutConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shortcut: "CmdOrCtrl+Shift+Space".to_string(),
            auto_insert: true,
            use_floating_window: true,
            preferred_model: "luyingwang-online".to_string(),
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