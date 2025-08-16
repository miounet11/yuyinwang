use std::sync::{Arc, Mutex};
use tauri::{Manager, GlobalShortcutManager as TauriGSM};
use crate::errors::AppResult;
use crate::system::PermissionManager;

pub struct EnhancedShortcutManager {
    app_handle: tauri::AppHandle,
    registered_shortcuts: Arc<Mutex<Vec<String>>>,
}

impl EnhancedShortcutManager {
    pub fn new(app_handle: tauri::AppHandle) -> AppResult<Self> {
        Ok(Self {
            app_handle,
            registered_shortcuts: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    pub fn register_shortcuts(&self) -> AppResult<()> {
        println!("🔧 开始注册全局快捷键...");
        
        // 首先检查关键权限
        println!("🔍 检查权限状态...");
        let permission_status = PermissionManager::check_all_permissions()?;
        
        if !permission_status.input_monitoring {
            eprintln!("❌ 缺少输入监控权限！快捷键将无法工作");
            eprintln!("📋 请在系统偏好设置 > 安全性与隐私 > 隐私 > 输入监控 中添加本应用");
            
            // 尝试打开系统设置
            if let Err(e) = PermissionManager::open_system_preferences("input_monitoring") {
                eprintln!("⚠️ 无法自动打开系统设置: {}", e);
            }
            
            return Err(crate::errors::AppError::PermissionError(
                "输入监控权限缺失，快捷键功能不可用".to_string()
            ));
        }
        
        if !permission_status.accessibility {
            eprintln!("⚠️ 缺少辅助功能权限，部分快捷键功能可能受限");
        }
        
        println!("✅ 权限检查通过，继续注册快捷键");
        
        // 定义要注册的快捷键列表 - 使用不被系统拦截的组合
        let shortcuts = vec![
            ("Cmd+Shift+K", "Cmd + Shift + K"),
            ("Cmd+Alt+Space", "Cmd + Option + 空格"),
            ("Ctrl+Alt+Space", "Ctrl + Option + 空格"),
            ("Cmd+Shift+V", "Cmd + Shift + V"),
            ("Alt+Shift+Space", "Option + Shift + 空格"),
        ];
        
        let app_handle = self.app_handle.clone();
        let mut registered = Vec::new();
        
        for (shortcut, description) in shortcuts {
            let app_handle_clone = app_handle.clone();
            let shortcut_str = shortcut.to_string();
            
            match self.app_handle.global_shortcut_manager().register(
                shortcut,
                move || {
                    println!("🔑 快捷键触发: {}", shortcut_str);
                    eprintln!("🔑 快捷键触发: {}", shortcut_str);
                    
                    // 先检查所有窗口
                    let windows = app_handle_clone.windows();
                    println!("📱 当前所有窗口: {:?}", windows.keys().collect::<Vec<_>>());
                    
                    // 尝试找到悬浮输入窗口
                    if let Some(window) = app_handle_clone.get_window("floating-input") {
                        println!("✅ 找到悬浮输入窗口，开始显示");
                        if let Err(e) = window.show() {
                            eprintln!("❌ 显示窗口失败: {}", e);
                        }
                        if let Err(e) = window.set_focus() {
                            eprintln!("❌ 设置焦点失败: {}", e);
                        }
                        if let Err(e) = window.emit("floating_input_triggered", ()) {
                            eprintln!("❌ 发送事件失败: {}", e);
                        }
                        println!("✅ 悬浮输入窗口操作完成");
                    } else {
                        eprintln!("❌ 悬浮输入窗口未找到，尝试显示主窗口");
                        // 回退到主窗口
                        if let Some(main_window) = app_handle_clone.get_window("main") {
                            let _ = main_window.show();
                            let _ = main_window.set_focus();
                            println!("✅ 显示主窗口作为回退");
                        }
                    }
                }
            ) {
                Ok(_) => {
                    println!("✅ 成功注册快捷键: {} ({})", shortcut, description);
                    registered.push(shortcut.to_string());
                }
                Err(e) => {
                    eprintln!("⚠️ 注册快捷键 {} 失败: {}", shortcut, e);
                }
            }
        }
        
        // 保存已注册的快捷键
        *self.registered_shortcuts.lock().unwrap() = registered;
        
        println!("🎯 全局快捷键系统已启动");
        println!("📱 可用快捷键:");
        println!("   • Cmd+Shift+K - 主要触发键");
        println!("   • Cmd+Option+Space - 备用触发键");
        println!("   • Ctrl+Option+Space - 第三选择");
        println!("   • Cmd+Shift+V - 快速输入");
        println!("   • Option+Shift+Space - 语音输入");
        
        Ok(())
    }
    
    pub fn unregister_all(&self) -> AppResult<()> {
        let registered = self.registered_shortcuts.lock().unwrap();
        
        for shortcut in registered.iter() {
            if let Err(e) = self.app_handle.global_shortcut_manager().unregister(shortcut) {
                eprintln!("⚠️ 注销快捷键 {} 失败: {}", shortcut, e);
            }
        }
        
        println!("🛑 所有快捷键已注销");
        Ok(())
    }
}

// 用于前端调用的命令
#[tauri::command]
pub async fn test_global_shortcut(app: tauri::AppHandle) -> Result<String, String> {
    // 测试触发悬浮窗口
    println!("🧪 测试快捷键命令被调用");
    
    let windows = app.windows();
    println!("📱 当前所有窗口: {:?}", windows.keys().collect::<Vec<_>>());
    
    if let Some(window) = app.get_window("floating-input") {
        println!("✅ 找到悬浮输入窗口");
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        window.emit("floating_input_triggered", ()).map_err(|e| e.to_string())?;
        Ok("悬浮窗口已触发".to_string())
    } else {
        println!("❌ 悬浮窗口未找到");
        // 显示主窗口作为测试
        if let Some(main_window) = app.get_window("main") {
            main_window.show().map_err(|e| e.to_string())?;
            main_window.set_focus().map_err(|e| e.to_string())?;
            Ok("显示主窗口作为测试".to_string())
        } else {
            Err("所有窗口都未找到".to_string())
        }
    }
}

// 检查快捷键状态的命令
#[tauri::command]
pub async fn check_shortcut_status(app: tauri::AppHandle) -> Result<String, String> {
    let shortcuts_to_check = vec![
        "Alt+Space",
        "F1", 
        "Alt+V",
        "Cmd+Shift+Space"
    ];
    
    let mut status = String::new();
    status.push_str("🔍 快捷键状态检查:\n");
    
    for shortcut in shortcuts_to_check {
        let is_registered = app.global_shortcut_manager()
            .is_registered(shortcut)
            .unwrap_or(false);
        status.push_str(&format!("  {} - {}\n", 
            shortcut, 
            if is_registered { "✅ 已注册" } else { "❌ 未注册" }
        ));
    }
    
    Ok(status)
}