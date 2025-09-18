// 文本注入器 - 将转录结果注入到当前应用光标位置
// 跨平台支持，重点优化 macOS 体验

use crate::errors::{AppError, AppResult};
use std::process::Command;
use std::time::Duration;

#[cfg(target_os = "macos")]
use cocoa::foundation::{NSAutoreleasePool, NSString};

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

    /// 获取配置
    pub fn config(&self) -> &TextInjectionConfig {
        &self.config
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
            Ok(if content.is_empty() {
                None
            } else {
                Some(content)
            })
        } else {
            Err(AppError::SystemIntegrationError(
                "获取剪贴板内容失败".to_string(),
            ))
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
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| AppError::SystemIntegrationError(format!("写入pbcopy失败: {}", e)))?;
        }

        let status = child
            .wait()
            .map_err(|e| AppError::SystemIntegrationError(format!("等待pbcopy完成失败: {}", e)))?;

        if !status.success() {
            return Err(AppError::SystemIntegrationError(
                "设置剪贴板内容失败".to_string(),
            ));
        }

        Ok(())
    }

    /// macOS: 模拟 Cmd+V 快捷键
    async fn simulate_paste_shortcut_macos(&self) -> AppResult<()> {
        // 使用 AppleScript 模拟 Cmd+V
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
            return Err(AppError::SystemIntegrationError(format!(
                "模拟粘贴失败: {}",
                error_msg
            )));
        }

        Ok(())
    }

    /// macOS: 模拟单个按键
    async fn simulate_key_press_macos(&self, ch: char) -> AppResult<()> {
        // 对于简单字符，使用AppleScript输入
        let escaped_char = ch.to_string().replace("\"", "\\\"").replace("\\", "\\\\");
        let script = format!(
            r#"
            tell application "System Events"
                keystroke "{}"
            end tell
        "#,
            escaped_char
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| AppError::SystemIntegrationError(format!("执行AppleScript失败: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::SystemIntegrationError(format!(
                "模拟按键失败: {}",
                error_msg
            )));
        }

        Ok(())
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
            .map_err(|e| {
                AppError::SystemIntegrationError(format!("获取活动应用信息失败: {}", e))
            })?;

        if output.status.success() {
            let result_string = String::from_utf8_lossy(&output.stdout);
            let result = result_string.trim();
            let parts: Vec<&str> = result.split('|').collect();

            Ok(ApplicationInfo {
                name: parts.get(0).unwrap_or(&"Unknown").to_string(),
                bundle_id: parts
                    .get(1)
                    .filter(|s| !s.is_empty())
                    .unwrap_or(&"unknown")
                    .to_string(),
                process_id: 0, // Process ID would need additional AppleScript to retrieve
            })
        } else {
            Err(AppError::SystemIntegrationError(
                "获取活动应用信息失败".to_string(),
            ))
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
        Err(AppError::SystemIntegrationError(
            "Windows剪贴板功能未实现".to_string(),
        ))
    }

    async fn set_clipboard_content_windows(&self, _text: &str) -> AppResult<()> {
        Err(AppError::SystemIntegrationError(
            "Windows剪贴板功能未实现".to_string(),
        ))
    }

    async fn simulate_paste_shortcut_windows(&self) -> AppResult<()> {
        // 可以使用 enigo crate 或 Windows API
        Err(AppError::SystemIntegrationError(
            "Windows按键模拟未实现".to_string(),
        ))
    }

    async fn simulate_key_press_windows(&self, _ch: char) -> AppResult<()> {
        Err(AppError::SystemIntegrationError(
            "Windows按键模拟未实现".to_string(),
        ))
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
            Ok(if content.is_empty() {
                None
            } else {
                Some(content)
            })
        } else {
            Err(AppError::SystemIntegrationError(
                "获取剪贴板内容失败".to_string(),
            ))
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
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| AppError::SystemIntegrationError(format!("写入xclip失败: {}", e)))?;
        }

        let status = child
            .wait()
            .map_err(|e| AppError::SystemIntegrationError(format!("等待xclip完成失败: {}", e)))?;

        if !status.success() {
            return Err(AppError::SystemIntegrationError(
                "设置剪贴板内容失败".to_string(),
            ));
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

/// 注入环境状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InjectionEnvironmentStatus {
    pub has_accessibility_permission: bool,
    pub active_app_detected: bool,
    pub clipboard_available: bool,
    pub applescript_available: bool,
    pub errors: Vec<String>,
}

/// 注入结果详情
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InjectionResult {
    pub success: bool,
    pub text_length: usize,
    pub target_app: Option<String>,
    pub injection_method: String,
    pub duration_ms: u64,
    pub retry_count: u32,
    pub error_message: Option<String>,
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

    /// 智能文本注入（包含重复检测和重试机制）
    pub async fn smart_inject(&self, text: &str) -> AppResult<bool> {
        if !self.enabled || text.trim().is_empty() {
            return Ok(false);
        }

        // 检查是否与上次注入时间太近
        {
            let mut last_time = self.last_injection_time.lock().unwrap();
            if let Some(last) = *last_time {
                if last.elapsed() < Duration::from_millis(500) {
                    println!(
                        "🚫 注入频率过高，跳过 ({}ms间隔)",
                        last.elapsed().as_millis()
                    );
                    return Ok(false);
                }
            }
            *last_time = Some(std::time::Instant::now());
        }

        // 权限检查（带重试）
        let has_permission = self.check_permission_with_retry().await?;
        if !has_permission {
            return Err(AppError::SystemIntegrationError(
                "缺少辅助功能权限，请在系统偏好设置中启用Recording King的辅助功能权限".to_string(),
            ));
        }

        // 获取当前应用信息（带错误恢复）
        let app_info = match self.injector.get_active_application_info().await {
            Ok(info) => {
                println!("🎯 当前活动应用: {} ({})", info.name, info.bundle_id);
                Some(info)
            }
            Err(e) => {
                println!("⚠️ 无法获取应用信息，继续注入: {}", e);
                None
            }
        };

        // 执行注入（带重试机制）
        self.inject_with_retry(text, app_info).await
    }

    /// 带重试的权限检查
    async fn check_permission_with_retry(&self) -> AppResult<bool> {
        let max_retries = 3;
        let mut retry_count = 0;

        while retry_count < max_retries {
            match self.injector.check_accessibility_permission() {
                Ok(has_permission) => return Ok(has_permission),
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(e);
                    }
                    println!(
                        "⚠️ 权限检查失败，重试 {}/{}: {}",
                        retry_count, max_retries, e
                    );
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }

        Ok(false)
    }

    /// 带重试的文本注入
    async fn inject_with_retry(
        &self,
        text: &str,
        app_info: Option<ApplicationInfo>,
    ) -> AppResult<bool> {
        let max_retries = 3;
        let mut retry_count = 0;
        let mut last_error = None;

        while retry_count < max_retries {
            match self.injector.inject_text(text).await {
                Ok(_) => {
                    println!("✅ 文本注入成功 ({}字符)", text.len());
                    if let Some(ref info) = app_info {
                        println!("📋 目标应用: {}", info.name);
                    }
                    return Ok(true);
                }
                Err(e) => {
                    retry_count += 1;
                    last_error = Some(e.clone());

                    if retry_count >= max_retries {
                        println!("❌ 文本注入最终失败 (重试{}次): {}", max_retries, e);
                        return Err(e);
                    }

                    println!(
                        "⚠️ 文本注入失败，重试 {}/{}: {}",
                        retry_count, max_retries, e
                    );

                    // 根据错误类型调整重试延迟
                    let delay = match e {
                        AppError::SystemIntegrationError(_) => Duration::from_millis(200),
                        _ => Duration::from_millis(100),
                    };

                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| AppError::SystemIntegrationError("未知注入错误".to_string())))
    }

    /// 验证注入环境
    pub async fn validate_injection_environment(&self) -> AppResult<InjectionEnvironmentStatus> {
        let mut status = InjectionEnvironmentStatus {
            has_accessibility_permission: false,
            active_app_detected: false,
            clipboard_available: false,
            applescript_available: false,
            errors: Vec::new(),
        };

        // 检查辅助功能权限
        match self.injector.check_accessibility_permission() {
            Ok(has_permission) => status.has_accessibility_permission = has_permission,
            Err(e) => status.errors.push(format!("权限检查失败: {}", e)),
        }

        // 检查活动应用
        match self.injector.get_active_application_info().await {
            Ok(_) => status.active_app_detected = true,
            Err(e) => status.errors.push(format!("应用检测失败: {}", e)),
        }

        // 检查剪贴板功能
        match self.injector.get_clipboard_content().await {
            Ok(_) => status.clipboard_available = true,
            Err(e) => status.errors.push(format!("剪贴板访问失败: {}", e)),
        }

        // 检查AppleScript可用性 (macOS)
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg("return \"test\"")
                .output();

            match output {
                Ok(result) if result.status.success() => status.applescript_available = true,
                _ => status.errors.push("AppleScript不可用".to_string()),
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            status.applescript_available = true; // 非macOS平台不需要AppleScript
        }

        Ok(status)
    }

    /// 获取注入器引用
    pub fn injector(&self) -> &TextInjector {
        &self.injector
    }
}
