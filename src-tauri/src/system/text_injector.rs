// 文本注入器 - 将转录结果注入到当前应用光标位置
// 跨平台支持，重点优化 macOS 体验

use std::time::Duration;
use std::process::Command;

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSAutoreleasePool, NSString};
#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventFlags, CGKeyCode};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use crate::errors::{AppError, AppResult};

/// 文本注入配置
#[derive(Debug, Clone)]
pub struct TextInjectionConfig {
    /// 是否启用自动注入
    pub auto_inject_enabled: bool,
    /// 注入前延迟时间
    pub inject_delay: std::time::Duration,
    /// 是否使用键盘模拟而不是剪贴板
    pub use_keyboard_simulation: bool,
    /// 是否保留剪贴板内容
    pub preserve_clipboard: bool,
    /// 是否启用重复检测
    pub duplicate_detection: bool,
    /// 快捷键延迟
    pub shortcut_delay: std::time::Duration,
    /// 目标应用过滤器
    pub target_app_filter: Vec<String>,
}

impl Default for TextInjectionConfig {
    fn default() -> Self {
        Self {
            auto_inject_enabled: true,
            inject_delay: std::time::Duration::from_millis(100),
            use_keyboard_simulation: false,
            preserve_clipboard: true,
            duplicate_detection: true,
            shortcut_delay: std::time::Duration::from_millis(50),
            target_app_filter: Vec::new(),
        }
    }
}

/// 文本注入器
#[derive(Debug)]
pub struct TextInjector {
    config: TextInjectionConfig,
}

impl TextInjector {
    /// 创建新的文本注入器
    pub fn new(config: TextInjectionConfig) -> Self {
        Self { config }
    }
    
    /// 创建默认文本注入器
    pub fn default() -> Self {
        Self::new(TextInjectionConfig::default())
    }
    
    /// 注入文本到当前活动应用
    pub async fn inject_text(&self, text: &str) -> AppResult<()> {
        if text.is_empty() {
            return Ok(());
        }
        
        println!("📝 准备注入文本到当前应用: {} 字符", text.len());
        
        // 添加延迟确保用户切换到目标应用
        if !self.config.inject_delay.is_zero() {
            tokio::time::sleep(self.config.inject_delay).await;
        }
        
        // 根据配置选择注入方式
        if self.config.use_keyboard_simulation {
            self.inject_via_keyboard_simulation(text).await
        } else {
            self.inject_via_clipboard(text).await
        }
    }
    
    /// 通过剪贴板注入文本（推荐方式）
    async fn inject_via_clipboard(&self, text: &str) -> AppResult<()> {
        // 1. 备份当前剪贴板内容
        let original_clipboard = self.get_clipboard_content().await?;
        
        // 2. 将文本复制到剪贴板
        self.set_clipboard_content(text).await?;
        
        // 3. 模拟 Cmd+V 粘贴
        self.simulate_paste_shortcut().await?;
        
        // 4. 等待粘贴完成
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // 5. 恢复原剪贴板内容（如果配置要求）
        if self.config.preserve_clipboard {
            if let Some(original) = original_clipboard {
                self.set_clipboard_content(&original).await?;
            }
        }
        
        println!("✅ 文本注入完成（剪贴板方式）");
        Ok(())
    }
    
    /// 通过键盘模拟注入文本
    async fn inject_via_keyboard_simulation(&self, text: &str) -> AppResult<()> {
        println!("⌨️ 使用键盘模拟方式注入文本");
        
        for ch in text.chars() {
            self.simulate_key_press(ch).await?;
            
            if !self.config.shortcut_delay.is_zero() {
                tokio::time::sleep(self.config.shortcut_delay).await;
            }
        }
        
        println!("✅ 文本注入完成（键盘模拟方式）");
        Ok(())
    }
    
    /// 获取剪贴板内容
    async fn get_clipboard_content(&self) -> AppResult<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            self.get_clipboard_content_macos().await
        }
        #[cfg(target_os = "windows")]
        {
            self.get_clipboard_content_windows().await
        }
        #[cfg(target_os = "linux")]
        {
            self.get_clipboard_content_linux().await
        }
    }
    
    /// 设置剪贴板内容
    async fn set_clipboard_content(&self, text: &str) -> AppResult<()> {
        #[cfg(target_os = "macos")]
        {
            self.set_clipboard_content_macos(text).await
        }
        #[cfg(target_os = "windows")]
        {
            self.set_clipboard_content_windows(text).await
        }
        #[cfg(target_os = "linux")]
        {
            self.set_clipboard_content_linux(text).await
        }
    }
    
    /// 模拟粘贴快捷键
    async fn simulate_paste_shortcut(&self) -> AppResult<()> {
        #[cfg(target_os = "macos")]
        {
            self.simulate_paste_shortcut_macos().await
        }
        #[cfg(target_os = "windows")]
        {
            self.simulate_paste_shortcut_windows().await
        }
        #[cfg(target_os = "linux")]
        {
            self.simulate_paste_shortcut_linux().await
        }
    }
    
    /// 模拟按键
    async fn simulate_key_press(&self, ch: char) -> AppResult<()> {
        #[cfg(target_os = "macos")]
        {
            self.simulate_key_press_macos(ch).await
        }
        #[cfg(target_os = "windows")]
        {
            self.simulate_key_press_windows(ch).await
        }
        #[cfg(target_os = "linux")]
        {
            self.simulate_key_press_linux(ch).await
        }
    }
    
    /// 检查是否有辅助功能权限（macOS）
    pub fn check_accessibility_permission(&self) -> AppResult<bool> {
        #[cfg(target_os = "macos")]
        {
            self.check_accessibility_permission_macos()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(true) // 非macOS平台假设有权限
        }
    }
    
    /// 获取当前活动应用信息
    pub async fn get_active_application_info(&self) -> AppResult<ApplicationInfo> {
        #[cfg(target_os = "macos")]
        {
            self.get_active_application_info_macos().await
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(ApplicationInfo {
                name: "Unknown".to_string(),
                bundle_id: "unknown".to_string(),
                process_id: 0,
            })
        }
    }
    
    /// 获取当前活动应用信息 (别名方法)
    pub async fn get_active_app_info(&self) -> AppResult<ApplicationInfo> {
        self.get_active_application_info().await
    }
}

/// macOS 平台实现
#[cfg(target_os = "macos")]
impl TextInjector {
    /// macOS: 获取剪贴板内容
    async fn get_clipboard_content_macos(&self) -> AppResult<Option<String>> {
        let output = Command::new("pbpaste")
            .output()
            .map_err(|e| AppError::SystemIntegrationError(format!("执行pbpaste失败: {}", e)))?;
        
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(if content.is_empty() { None } else { Some(content) })
        } else {
            Err(AppError::SystemIntegrationError("获取剪贴板内容失败".to_string()))
        }
    }
    
    /// macOS: 设置剪贴板内容
    async fn set_clipboard_content_macos(&self, text: &str) -> AppResult<()> {
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::SystemIntegrationError(format!("启动pbcopy失败: {}", e)))?;
        
        use std::io::Write;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())
                .map_err(|e| AppError::SystemIntegrationError(format!("写入pbcopy失败: {}", e)))?;
        }
        
        let status = child.wait()
            .map_err(|e| AppError::SystemIntegrationError(format!("等待pbcopy完成失败: {}", e)))?;
        
        if !status.success() {
            return Err(AppError::SystemIntegrationError("设置剪贴板内容失败".to_string()));
        }
        
        Ok(())
    }
    
    /// macOS: 模拟 Cmd+V 快捷键
    async fn simulate_paste_shortcut_macos(&self) -> AppResult<()> {
        // 使用 CGEvent 而不是 AppleScript
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil};
            use cocoa::foundation::NSAutoreleasePool;
            use core_graphics::event::{CGEvent, CGEventFlags};
            use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
            
            unsafe {
                let pool = NSAutoreleasePool::new(nil);
                
                // 创建事件源
                match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
                    Ok(source) => {
                        // 增加延迟确保应用完全获得焦点
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        
                        // 发送 Cmd+V 按键事件
                        if let Ok(key_down) = CGEvent::new_keyboard_event(source.clone(), 9, true) { // 9 是 V 键的 keycode
                            key_down.set_flags(CGEventFlags::CGEventFlagCommand);
                            key_down.post(core_graphics::event::CGEventTapLocation::HID);
                            
                            // 适当延迟
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            
                            // 释放按键
                            if let Ok(key_up) = CGEvent::new_keyboard_event(source, 9, false) {
                                key_up.set_flags(CGEventFlags::CGEventFlagCommand);
                                key_up.post(core_graphics::event::CGEventTapLocation::HID);
                                
                                pool.drain();
                                return Ok(());
                            }
                        }
                        
                        pool.drain();
                        Err(AppError::SystemIntegrationError("无法创建键盘事件".to_string()))
                    }
                    Err(_) => {
                        pool.drain();
                        Err(AppError::SystemIntegrationError("无法创建CGEvent源".to_string()))
                    }
                }
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            // 对于非macOS平台，使用原来的实现
            let script = r#"
                tell application "System Events"
                    key code 9 using {command down}
                end tell
            "#;
            
            let output = Command::new("osascript")
                .arg("-e")
                .arg(script)
                .output()
                .map_err(|e| AppError::SystemIntegrationError(format!("执行AppleScript失败: {}", e)))?;
            
            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::SystemIntegrationError(format!("模拟粘贴失败: {}", error_msg)));
            }
            
            Ok(())
        }
    }
    
    /// macOS: 模拟单个按键
    async fn simulate_key_press_macos(&self, ch: char) -> AppResult<()> {
        // 使用 CGEvent 而不是 AppleScript
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            
            // 创建事件源
            match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
                Ok(source) => {
                    // 将字符转换为字符串
                    let text_string = NSString::alloc(nil).init_str(&ch.to_string());
                    
                    // 创建键盘事件（使用Unicode文本而不是keycode）
                    if let Ok(mut event) = CGEvent::new_keyboard_event(source.clone(), 0, true) {
                        // 设置要输入的文本
                        event.set_string(&ch.to_string());
                        
                        // 发送事件
                        event.post(core_graphics::event::CGEventTapLocation::HID);
                        
                        // 短暂延迟
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        
                        pool.drain();
                        return Ok(());
                    }
                    
                    pool.drain();
                    Err(AppError::SystemIntegrationError("无法创建键盘事件".to_string()))
                }
                Err(_) => {
                    pool.drain();
                    Err(AppError::SystemIntegrationError("无法创建CGEvent源".to_string()))
                }
            }
        }
    }
    
    /// macOS: 检查辅助功能权限
    fn check_accessibility_permission_macos(&self) -> AppResult<bool> {
        let script = r#"
            tell application "System Events"
                try
                    get name of first process
                    return true
                on error
                    return false
                end try
            end tell
        "#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| AppError::SystemIntegrationError(format!("检查权限失败: {}", e)))?;
        
        if output.status.success() {
            let result_string = String::from_utf8_lossy(&output.stdout);
            let result = result_string.trim();
            Ok(result == "true")
        } else {
            Ok(false)
        }
    }
    
    /// macOS: 获取当前活动应用信息
    async fn get_active_application_info_macos(&self) -> AppResult<ApplicationInfo> {
        let script = r#"
            tell application "System Events"
                set frontApp to first application process whose frontmost is true
                set appName to name of frontApp
                try
                    set appBundle to bundle identifier of frontApp
                on error
                    set appBundle to ""
                end try
                return appName & "|" & appBundle
            end tell
        "#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| AppError::SystemIntegrationError(format!("获取活动应用信息失败: {}", e)))?;
        
        if output.status.success() {
            let result_string = String::from_utf8_lossy(&output.stdout);
            let result = result_string.trim();
            let parts: Vec<&str> = result.split('|').collect();
            
            Ok(ApplicationInfo {
                name: parts.get(0).unwrap_or(&"Unknown").to_string(),
                bundle_id: parts.get(1).filter(|s| !s.is_empty()).unwrap_or(&"unknown").to_string(),
                process_id: 0, // Process ID would need additional AppleScript to retrieve
            })
        } else {
            Err(AppError::SystemIntegrationError("获取活动应用信息失败".to_string()))
        }
    }
}

/// Windows 平台实现
#[cfg(target_os = "windows")]
impl TextInjector {
    async fn get_clipboard_content_windows(&self) -> AppResult<Option<String>> {
        // Windows 剪贴板实现
        // 可以使用 winapi 或 clipboard-win crate
        // 这里提供基础实现框架
        Err(AppError::SystemIntegrationError("Windows剪贴板功能未实现".to_string()))
    }
    
    async fn set_clipboard_content_windows(&self, _text: &str) -> AppResult<()> {
        Err(AppError::SystemIntegrationError("Windows剪贴板功能未实现".to_string()))
    }
    
    async fn simulate_paste_shortcut_windows(&self) -> AppResult<()> {
        // 可以使用 enigo crate 或 Windows API
        Err(AppError::SystemIntegrationError("Windows按键模拟未实现".to_string()))
    }
    
    async fn simulate_key_press_windows(&self, _ch: char) -> AppResult<()> {
        Err(AppError::SystemIntegrationError("Windows按键模拟未实现".to_string()))
    }
}

/// Linux 平台实现
#[cfg(target_os = "linux")]
impl TextInjector {
    async fn get_clipboard_content_linux(&self) -> AppResult<Option<String>> {
        // 使用 xclip 或 wl-clipboard
        let output = Command::new("xclip")
            .args(&["-selection", "clipboard", "-o"])
            .output()
            .map_err(|e| AppError::SystemIntegrationError(format!("执行xclip失败: {}", e)))?;
        
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(if content.is_empty() { None } else { Some(content) })
        } else {
            Err(AppError::SystemIntegrationError("获取剪贴板内容失败".to_string()))
        }
    }
    
    async fn set_clipboard_content_linux(&self, text: &str) -> AppResult<()> {
        let mut child = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::SystemIntegrationError(format!("启动xclip失败: {}", e)))?;
        
        use std::io::Write;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())
                .map_err(|e| AppError::SystemIntegrationError(format!("写入xclip失败: {}", e)))?;
        }
        
        let status = child.wait()
            .map_err(|e| AppError::SystemIntegrationError(format!("等待xclip完成失败: {}", e)))?;
        
        if !status.success() {
            return Err(AppError::SystemIntegrationError("设置剪贴板内容失败".to_string()));
        }
        
        Ok(())
    }
    
    async fn simulate_paste_shortcut_linux(&self) -> AppResult<()> {
        // 使用 xdotool 模拟 Ctrl+V
        let output = Command::new("xdotool")
            .args(&["key", "ctrl+v"])
            .output()
            .map_err(|e| AppError::SystemIntegrationError(format!("执行xdotool失败: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::SystemIntegrationError("模拟粘贴失败".to_string()));
        }
        
        Ok(())
    }
    
    async fn simulate_key_press_linux(&self, ch: char) -> AppResult<()> {
        // 使用 xdotool 模拟按键
        let output = Command::new("xdotool")
            .args(&["type", &ch.to_string()])
            .output()
            .map_err(|e| AppError::SystemIntegrationError(format!("执行xdotool失败: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::SystemIntegrationError("模拟按键失败".to_string()));
        }
        
        Ok(())
    }
}

/// 应用信息结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    pub bundle_id: String,
    pub process_id: u32,
}

/// 文本注入管理器
#[derive(Debug)]
pub struct TextInjectionManager {
    injector: TextInjector,
    enabled: bool,
    last_injection_time: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}

impl TextInjectionManager {
    /// 创建新的文本注入管理器
    pub fn new(config: TextInjectionConfig) -> Self {
        Self {
            injector: TextInjector::new(config),
            enabled: true,
            last_injection_time: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    /// 启用/禁用文本注入
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// 智能文本注入（包含重复检测）
    pub async fn smart_inject(&self, text: &str) -> AppResult<bool> {
        if !self.enabled || text.is_empty() {
            return Ok(false);
        }
        
        // 检查是否与上次注入时间太近
        {
            let mut last_time = self.last_injection_time.lock().unwrap();
            if let Some(last) = *last_time {
                if last.elapsed() < Duration::from_millis(500) {
                    println!("🚫 注入频率过高，跳过");
                    return Ok(false);
                }
            }
            *last_time = Some(std::time::Instant::now());
        }
        
        // 检查权限
        if !self.injector.check_accessibility_permission()? {
            return Err(AppError::SystemIntegrationError(
                "缺少辅助功能权限，无法进行文本注入".to_string()
            ));
        }
        
        // 获取当前应用信息
        let app_info = self.injector.get_active_application_info().await?;
        println!("🎯 当前活动应用: {}", app_info.name);
        
        // 执行注入
        self.injector.inject_text(text).await?;
        
        Ok(true)
    }
    
    /// 获取注入器引用
    pub fn injector(&self) -> &TextInjector {
        &self.injector
    }
}