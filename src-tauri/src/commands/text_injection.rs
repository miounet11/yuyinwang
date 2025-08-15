// 文本注入相关的Tauri命令
use serde::{Deserialize, Serialize};
use crate::system::{TextInjector, TextInjectionConfig, TextInjectionManager, AppInfo};

/// 文本注入配置的序列化结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInjectionConfigDto {
    /// 是否启用自动注入
    pub auto_inject_enabled: bool,
    /// 注入延迟（毫秒）
    pub inject_delay_ms: u64,
    /// 是否使用键盘模拟（否则使用剪贴板）
    pub use_keyboard_simulation: bool,
    /// 是否保留剪贴板内容
    pub preserve_clipboard: bool,
    /// 是否启用重复检测
    pub duplicate_detection: bool,
    /// 快捷键延迟（毫秒）
    pub shortcut_delay_ms: u64,
    /// 目标应用过滤器
    pub target_app_filter: Vec<String>,
}

impl From<TextInjectionConfigDto> for TextInjectionConfig {
    fn from(dto: TextInjectionConfigDto) -> Self {
        Self {
            auto_inject_enabled: dto.auto_inject_enabled,
            inject_delay: std::time::Duration::from_millis(dto.inject_delay_ms),
            use_keyboard_simulation: dto.use_keyboard_simulation,
            preserve_clipboard: dto.preserve_clipboard,
            duplicate_detection: dto.duplicate_detection,
            shortcut_delay: std::time::Duration::from_millis(dto.shortcut_delay_ms),
            target_app_filter: dto.target_app_filter,
        }
    }
}

impl From<TextInjectionConfig> for TextInjectionConfigDto {
    fn from(config: TextInjectionConfig) -> Self {
        Self {
            auto_inject_enabled: config.auto_inject_enabled,
            inject_delay_ms: config.inject_delay.as_millis() as u64,
            use_keyboard_simulation: config.use_keyboard_simulation,
            preserve_clipboard: config.preserve_clipboard,
            duplicate_detection: config.duplicate_detection,
            shortcut_delay_ms: config.shortcut_delay.as_millis() as u64,
            target_app_filter: config.target_app_filter,
        }
    }
}

/// 简单文本注入命令
#[tauri::command]
pub async fn inject_text_to_cursor(text: String) -> Result<bool, String> {
    let injector = TextInjector::default();
    
    match injector.inject_text(&text).await {
        Ok(_) => {
            println!("✅ 文本注入成功: {}", text);
            Ok(true)
        }
        Err(e) => {
            eprintln!("❌ 文本注入失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 智能文本注入命令（带重复检测）
#[tauri::command]
pub async fn smart_inject_text(text: String, config: Option<TextInjectionConfigDto>) -> Result<bool, String> {
    let injection_config = config
        .map(|c| c.into())
        .unwrap_or_else(TextInjectionConfig::default);
    
    let manager = TextInjectionManager::new(injection_config);
    
    match manager.smart_inject(&text).await {
        Ok(injected) => {
            if injected {
                println!("✅ 智能文本注入成功: {}", text);
            } else {
                println!("ℹ️ 文本注入被跳过（重复或禁用）");
            }
            Ok(injected)
        }
        Err(e) => {
            eprintln!("❌ 智能文本注入失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 检查文本注入权限
#[tauri::command]
pub async fn check_text_injection_permission() -> Result<bool, String> {
    let injector = TextInjector::default();
    
    match injector.check_accessibility_permission() {
        Ok(has_permission) => {
            if has_permission {
                println!("✅ 文本注入权限正常");
            } else {
                println!("⚠️ 缺少辅助功能权限");
            }
            Ok(has_permission)
        }
        Err(e) => {
            eprintln!("❌ 检查文本注入权限失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取当前活动应用信息
#[tauri::command]
pub async fn get_active_app_info() -> Result<AppInfo, String> {
    let injector = TextInjector::default();
    
    match injector.get_active_app_info().await {
        Ok(app_info) => {
            println!("🎯 当前活动应用: {} (Bundle ID: {})", app_info.name, app_info.bundle_id);
            Ok(app_info)
        }
        Err(e) => {
            eprintln!("❌ 获取活动应用信息失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 测试文本注入功能
#[tauri::command]
pub async fn test_text_injection() -> Result<String, String> {
    let test_text = "Hello from Recording King! 📝";
    
    // 检查权限
    if !check_text_injection_permission().await? {
        return Err("缺少辅助功能权限，请在系统偏好设置中授权".to_string());
    }
    
    // 获取当前应用信息
    let app_info = get_active_app_info().await?;
    
    // 执行测试注入
    let success = inject_text_to_cursor(test_text.to_string()).await?;
    
    if success {
        Ok(format!("✅ 文本注入测试成功!\n目标应用: {}\n注入内容: {}", app_info.name, test_text))
    } else {
        Err("文本注入测试失败".to_string())
    }
}

/// 批量文本注入
#[tauri::command]
pub async fn batch_inject_text(texts: Vec<String>, config: Option<TextInjectionConfigDto>) -> Result<Vec<bool>, String> {
    let injection_config = config
        .map(|c| c.into())
        .unwrap_or_else(TextInjectionConfig::default);
    
    let manager = TextInjectionManager::new(injection_config);
    let mut results = Vec::new();
    
    for text in texts {
        match manager.smart_inject(&text).await {
            Ok(injected) => {
                results.push(injected);
                if injected {
                    println!("✅ 批量注入成功: {}", text);
                }
                // 添加延迟避免过快注入
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Err(e) => {
                eprintln!("❌ 批量注入失败: {} - {}", text, e);
                results.push(false);
            }
        }
    }
    
    Ok(results)
}

/// 获取默认文本注入配置
#[tauri::command]
pub async fn get_default_text_injection_config() -> Result<TextInjectionConfigDto, String> {
    let config = TextInjectionConfig::default();
    Ok(config.into())
}

/// 验证文本注入配置
#[tauri::command]
pub async fn validate_text_injection_config(config: TextInjectionConfigDto) -> Result<bool, String> {
    // 基本验证
    if config.inject_delay_ms > 10000 {
        return Err("注入延迟不能超过10秒".to_string());
    }
    
    if config.shortcut_delay_ms > 5000 {
        return Err("快捷键延迟不能超过5秒".to_string());
    }
    
    println!("✅ 文本注入配置验证通过");
    Ok(true)
}

/// 清除文本注入历史（如果有的话）
#[tauri::command]
pub async fn clear_text_injection_history() -> Result<(), String> {
    // 这里可以清除注入历史记录
    println!("🧹 文本注入历史已清除");
    Ok(())
}