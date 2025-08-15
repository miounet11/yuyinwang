// 快捷键管理相关的Tauri命令
use tauri::{GlobalShortcutManager, Manager, State};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfigDto {
    pub key: String,
    pub action: String,
    pub description: String,
    pub enabled: bool,
    pub global: bool,
}

/// 快捷键管理器状态
pub struct ShortcutManager {
    registered_shortcuts: Arc<Mutex<HashMap<String, String>>>, // key -> action
}

impl ShortcutManager {
    pub fn new() -> Self {
        Self {
            registered_shortcuts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_shortcut(&self, key: String, action: String) {
        let mut shortcuts = self.registered_shortcuts.lock();
        shortcuts.insert(key, action);
    }

    pub fn remove_shortcut(&self, key: &str) {
        let mut shortcuts = self.registered_shortcuts.lock();
        shortcuts.remove(key);
    }

    pub fn get_action(&self, key: &str) -> Option<String> {
        let shortcuts = self.registered_shortcuts.lock();
        shortcuts.get(key).cloned()
    }

    pub fn list_shortcuts(&self) -> HashMap<String, String> {
        let shortcuts = self.registered_shortcuts.lock();
        shortcuts.clone()
    }
}

/// 注册全局快捷键
#[tauri::command]
pub async fn register_global_shortcut(
    app_handle: tauri::AppHandle,
    shortcut: String,
    action: String,
) -> Result<bool, String> {
    println!("🔧 尝试注册全局快捷键: {} -> {}", shortcut, action);
    
    let mut shortcut_manager = app_handle.global_shortcut_manager();
    
    // 如果快捷键已经注册，先取消注册
    if shortcut_manager.is_registered(&shortcut).map_err(|e| e.to_string())? {
        if let Err(e) = shortcut_manager.unregister(&shortcut) {
            eprintln!("❌ 取消注册现有快捷键失败: {}", e);
            return Err(format!("取消注册现有快捷键失败: {}", e));
        }
    }
    
    // 注册新的快捷键
    let app_handle_clone = app_handle.clone();
    let action_clone = action.clone();
    let shortcut_clone = shortcut.clone();
    
    match shortcut_manager.register(&shortcut, move || {
        println!("🔥 全局快捷键被触发: {} -> {}", shortcut_clone, action_clone);
        
        // 发送事件到前端
        if let Err(e) = app_handle_clone.emit_all("shortcut_pressed", serde_json::json!({
            "shortcut": shortcut_clone,
            "action": action_clone,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        })) {
            eprintln!("❌ 发送快捷键事件失败: {}", e);
        } else {
            println!("✅ 快捷键事件已发送到前端");
        }
    }) {
        Ok(_) => {
            println!("✅ 全局快捷键注册成功: {}", shortcut);
            
            // 记录到快捷键管理器
            if let Some(manager) = app_handle.try_state::<ShortcutManager>() {
                manager.add_shortcut(shortcut, action);
            }
            
            Ok(true)
        }
        Err(e) => {
            eprintln!("❌ 全局快捷键注册失败: {}", e);
            Err(format!("注册快捷键失败: {}", e))
        }
    }
}

/// 取消注册全局快捷键
#[tauri::command]
pub async fn unregister_global_shortcut(
    app_handle: tauri::AppHandle,
    shortcut: String,
) -> Result<bool, String> {
    println!("🔧 尝试取消注册全局快捷键: {}", shortcut);
    
    let mut shortcut_manager = app_handle.global_shortcut_manager();
    
    match shortcut_manager.unregister(&shortcut) {
        Ok(_) => {
            println!("✅ 全局快捷键取消注册成功: {}", shortcut);
            
            // 从快捷键管理器中移除
            if let Some(manager) = app_handle.try_state::<ShortcutManager>() {
                manager.remove_shortcut(&shortcut);
            }
            
            Ok(true)
        }
        Err(e) => {
            eprintln!("❌ 全局快捷键取消注册失败: {}", e);
            Err(format!("取消注册快捷键失败: {}", e))
        }
    }
}

/// 检查快捷键是否已注册
#[tauri::command]
pub async fn is_shortcut_registered(
    app_handle: tauri::AppHandle,
    shortcut: String,
) -> Result<bool, String> {
    let shortcut_manager = app_handle.global_shortcut_manager();
    Ok(shortcut_manager.is_registered(&shortcut).map_err(|e| e.to_string())?)
}

/// 获取所有已注册的快捷键
#[tauri::command]
pub async fn get_registered_shortcuts(
    shortcut_manager: State<'_, ShortcutManager>,
) -> Result<HashMap<String, String>, String> {
    Ok(shortcut_manager.list_shortcuts())
}

/// 批量注册快捷键
#[tauri::command]
pub async fn register_multiple_shortcuts(
    app_handle: tauri::AppHandle,
    shortcuts: Vec<ShortcutConfigDto>,
) -> Result<Vec<bool>, String> {
    let mut results = Vec::new();
    
    for shortcut_config in shortcuts {
        if shortcut_config.enabled && shortcut_config.global {
            let result = register_global_shortcut(
                app_handle.clone(),
                shortcut_config.key.clone(),
                shortcut_config.action.clone(),
            ).await;
            
            results.push(result.unwrap_or(false));
        } else {
            results.push(false);
        }
    }
    
    Ok(results)
}

/// 取消注册所有快捷键
#[tauri::command]
pub async fn unregister_all_shortcuts(
    app_handle: tauri::AppHandle,
    shortcut_manager: State<'_, ShortcutManager>,
) -> Result<bool, String> {
    let shortcuts = shortcut_manager.list_shortcuts();
    let mut all_success = true;
    
    for (key, _) in shortcuts {
        if let Err(e) = unregister_global_shortcut(app_handle.clone(), key.clone()).await {
            eprintln!("❌ 取消注册快捷键失败 {}: {}", key, e);
            all_success = false;
        }
    }
    
    Ok(all_success)
}

/// 验证快捷键格式
#[tauri::command]
pub async fn validate_shortcut_format(shortcut: String) -> Result<bool, String> {
    // 基本的快捷键格式验证
    let valid_modifiers = ["CommandOrControl", "Command", "Control", "Alt", "Shift", "Meta"];
    let parts: Vec<&str> = shortcut.split('+').collect();
    
    if parts.len() < 2 {
        return Ok(false);
    }
    
    // 检查修饰键
    for part in &parts[..parts.len()-1] {
        if !valid_modifiers.contains(part) {
            return Ok(false);
        }
    }
    
    // 检查主键
    let main_key = parts.last().unwrap();
    if main_key.is_empty() {
        return Ok(false);
    }
    
    Ok(true)
}

/// 获取快捷键冲突信息
#[tauri::command]
pub async fn check_shortcut_conflicts(
    app_handle: tauri::AppHandle,
    new_shortcuts: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut conflicts = Vec::new();
    let shortcut_manager = app_handle.global_shortcut_manager();
    
    for shortcut in new_shortcuts {
        if shortcut_manager.is_registered(&shortcut).map_err(|e| e.to_string())? {
            conflicts.push(shortcut);
        }
    }
    
    Ok(conflicts)
}

/// 测试快捷键（模拟触发）
#[tauri::command]
pub async fn test_shortcut(
    app_handle: tauri::AppHandle,
    shortcut: String,
    action: String,
) -> Result<bool, String> {
    println!("🧪 测试快捷键: {} -> {}", shortcut, action);
    
    // 发送测试事件到前端
    match app_handle.emit_all("shortcut_pressed", serde_json::json!({
        "shortcut": shortcut,
        "action": action,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        "test": true
    })) {
        Ok(_) => {
            println!("✅ 快捷键测试事件已发送");
            Ok(true)
        }
        Err(e) => {
            eprintln!("❌ 发送快捷键测试事件失败: {}", e);
            Err(format!("测试快捷键失败: {}", e))
        }
    }
}