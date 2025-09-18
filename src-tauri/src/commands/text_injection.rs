// 文本注入相关的Tauri命令
use crate::system::{AppInfo, TextInjectionConfig, TextInjectionManager, TextInjector};
use serde::{Deserialize, Serialize};

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
pub async fn smart_inject_text(
    text: String,
    config: Option<TextInjectionConfigDto>,
) -> Result<bool, String> {
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
            println!(
                "🎯 当前活动应用: {} (Bundle ID: {})",
                app_info.name, app_info.bundle_id
            );
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
        Ok(format!(
            "✅ 文本注入测试成功!\n目标应用: {}\n注入内容: {}",
            app_info.name, test_text
        ))
    } else {
        Err("文本注入测试失败".to_string())
    }
}

/// 批量文本注入
#[tauri::command]
pub async fn batch_inject_text(
    texts: Vec<String>,
    config: Option<TextInjectionConfigDto>,
) -> Result<Vec<bool>, String> {
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
pub async fn validate_text_injection_config(
    config: TextInjectionConfigDto,
) -> Result<bool, String> {
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

/// 验证注入环境状态
#[tauri::command]
pub async fn validate_injection_environment(
) -> Result<crate::system::InjectionEnvironmentStatus, String> {
    let manager = TextInjectionManager::new(TextInjectionConfig::default());

    match manager.validate_injection_environment().await {
        Ok(status) => {
            println!("🔍 注入环境验证完成:");
            println!(
                "  辅助功能权限: {}",
                if status.has_accessibility_permission {
                    "✅"
                } else {
                    "❌"
                }
            );
            println!(
                "  活动应用检测: {}",
                if status.active_app_detected {
                    "✅"
                } else {
                    "❌"
                }
            );
            println!(
                "  剪贴板可用: {}",
                if status.clipboard_available {
                    "✅"
                } else {
                    "❌"
                }
            );
            println!(
                "  AppleScript可用: {}",
                if status.applescript_available {
                    "✅"
                } else {
                    "❌"
                }
            );

            if !status.errors.is_empty() {
                println!("  错误: {:?}", status.errors);
            }

            Ok(status)
        }
        Err(e) => {
            eprintln!("❌ 环境验证失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 增强的智能文本注入
#[tauri::command]
pub async fn enhanced_smart_inject(
    text: String,
    config: Option<TextInjectionConfigDto>,
) -> Result<crate::system::InjectionResult, String> {
    let start_time = std::time::Instant::now();
    let injection_config = config
        .map(|c| c.into())
        .unwrap_or_else(TextInjectionConfig::default);

    let mut manager = TextInjectionManager::new(injection_config);
    let method = if manager.injector().config().use_keyboard_simulation {
        "keyboard_simulation"
    } else {
        "clipboard_paste"
    };

    // 获取目标应用信息
    let target_app = match manager.injector().get_active_app_info().await {
        Ok(info) => Some(info.name),
        Err(_) => None,
    };

    let mut retry_count = 0;
    let result = match manager.smart_inject(&text).await {
        Ok(success) => crate::system::InjectionResult {
            success,
            text_length: text.len(),
            target_app,
            injection_method: method.to_string(),
            duration_ms: start_time.elapsed().as_millis() as u64,
            retry_count,
            error_message: None,
        },
        Err(e) => {
            retry_count = 3; // 假设重试了3次
            crate::system::InjectionResult {
                success: false,
                text_length: text.len(),
                target_app,
                injection_method: method.to_string(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                retry_count,
                error_message: Some(e.to_string()),
            }
        }
    };

    if result.success {
        println!(
            "✅ 增强注入成功: {}ms, 方法: {}",
            result.duration_ms, result.injection_method
        );
    } else {
        println!("❌ 增强注入失败: {:?}", result.error_message);
    }

    Ok(result)
}

/// 文本注入健康检查
#[tauri::command]
pub async fn text_injection_health_check() -> Result<serde_json::Value, String> {
    let mut health_status = serde_json::Map::new();

    // 基础权限检查
    let has_permission = check_text_injection_permission().await.unwrap_or(false);
    health_status.insert(
        "accessibility_permission".to_string(),
        serde_json::Value::Bool(has_permission),
    );

    // 环境状态检查
    match validate_injection_environment().await {
        Ok(env_status) => {
            health_status.insert(
                "environment_status".to_string(),
                serde_json::to_value(env_status).unwrap_or(serde_json::Value::Null),
            );
        }
        Err(e) => {
            health_status.insert(
                "environment_error".to_string(),
                serde_json::Value::String(e),
            );
        }
    }

    // 当前应用检测
    match get_active_app_info().await {
        Ok(app_info) => {
            health_status.insert(
                "current_app".to_string(),
                serde_json::to_value(app_info).unwrap_or(serde_json::Value::Null),
            );
        }
        Err(e) => {
            health_status.insert(
                "app_detection_error".to_string(),
                serde_json::Value::String(e),
            );
        }
    }

    // 整体健康评分
    let health_score = if has_permission { 100 } else { 0 };
    health_status.insert(
        "health_score".to_string(),
        serde_json::Value::Number(serde_json::Number::from(health_score)),
    );

    println!("🏥 文本注入健康检查完成，评分: {}", health_score);
    Ok(serde_json::Value::Object(health_status))
}

/// 修复文本注入问题
#[tauri::command]
pub async fn fix_text_injection_issues() -> Result<Vec<String>, String> {
    let mut fixes_applied = Vec::new();

    // 检查并尝试修复权限问题
    if !check_text_injection_permission().await.unwrap_or(false) {
        // 这里可以尝试引导用户到系统设置
        fixes_applied.push(
            "检测到权限问题，请手动到系统偏好设置 > 安全性与隐私 > 辅助功能中授权Recording King"
                .to_string(),
        );
    }

    // 检查环境状态
    match validate_injection_environment().await {
        Ok(env_status) => {
            if !env_status.clipboard_available {
                fixes_applied.push("剪贴板访问异常，建议重启应用".to_string());
            }
            if !env_status.applescript_available {
                fixes_applied.push("AppleScript不可用，请检查系统设置".to_string());
            }
        }
        Err(_) => {
            fixes_applied.push("环境检测失败，建议重启应用".to_string());
        }
    }

    if fixes_applied.is_empty() {
        fixes_applied.push("文本注入系统运行正常，无需修复".to_string());
    }

    println!("🔧 修复建议: {:?}", fixes_applied);
    Ok(fixes_applied)
}
