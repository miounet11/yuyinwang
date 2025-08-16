// 优化后的文本注入系统
// 基于 tech-lead-reviewer 和 ux-reviewer 的专业建议

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::errors::{AppResult, AppError};
use crate::system::{PermissionManager};

/// 应用信息结构（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub bundle_id: String,
}

/// 文本注入适配器类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InjectionAdapterType {
    Clipboard,           // 剪贴板方式：快速，通用性强
    Accessibility,       // 辅助功能：中速，兼容性好
    KeyboardSimulation,  // 键盘模拟：慢速，兼容性最好
}

/// 注入速度级别
#[derive(Debug, Clone, PartialEq)]
pub enum InjectionSpeed {
    Fast,    // < 100ms
    Medium,  // 100-500ms
    Slow,    // > 500ms
}

/// 应用特化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSpecificConfig {
    pub bundle_id: String,
    pub preferred_adapter: InjectionAdapterType,
    pub pre_inject_delay_ms: u64,
    pub post_inject_delay_ms: u64,
    pub requires_focus: bool,
    pub supports_batch_inject: bool,
}

/// 优化后的文本注入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedTextInjectionConfig {
    pub default_adapter: InjectionAdapterType,
    pub auto_fallback_enabled: bool,
    pub preserve_clipboard: bool,
    pub batch_inject_enabled: bool,
    pub batch_delay_ms: u64,
    pub max_text_length: usize,
    pub app_specific_configs: Vec<AppSpecificConfig>,
    pub performance_monitoring: bool,
}

impl Default for OptimizedTextInjectionConfig {
    fn default() -> Self {
        Self {
            default_adapter: InjectionAdapterType::Clipboard,
            auto_fallback_enabled: true,
            preserve_clipboard: true,
            batch_inject_enabled: true,
            batch_delay_ms: 50,
            max_text_length: 10000,
            app_specific_configs: Self::get_default_app_configs(),
            performance_monitoring: true,
        }
    }
}

impl OptimizedTextInjectionConfig {
    /// 获取常用应用的默认配置
    fn get_default_app_configs() -> Vec<AppSpecificConfig> {
        vec![
            // 代码编辑器 - 使用辅助功能获得更好的性能
            AppSpecificConfig {
                bundle_id: "com.microsoft.VSCode".to_string(),
                preferred_adapter: InjectionAdapterType::Accessibility,
                pre_inject_delay_ms: 10,
                post_inject_delay_ms: 10,
                requires_focus: true,
                supports_batch_inject: true,
            },
            AppSpecificConfig {
                bundle_id: "com.apple.dt.Xcode".to_string(),
                preferred_adapter: InjectionAdapterType::Accessibility,
                pre_inject_delay_ms: 20,
                post_inject_delay_ms: 20,
                requires_focus: true,
                supports_batch_inject: true,
            },
            // 终端应用 - 使用键盘模拟确保兼容性
            AppSpecificConfig {
                bundle_id: "com.apple.Terminal".to_string(),
                preferred_adapter: InjectionAdapterType::KeyboardSimulation,
                pre_inject_delay_ms: 50,
                post_inject_delay_ms: 50,
                requires_focus: true,
                supports_batch_inject: false,
            },
            AppSpecificConfig {
                bundle_id: "com.googlecode.iterm2".to_string(),
                preferred_adapter: InjectionAdapterType::KeyboardSimulation,
                pre_inject_delay_ms: 30,
                post_inject_delay_ms: 30,
                requires_focus: true,
                supports_batch_inject: false,
            },
            // 文档编辑器 - 使用剪贴板获得最快速度
            AppSpecificConfig {
                bundle_id: "com.microsoft.Word".to_string(),
                preferred_adapter: InjectionAdapterType::Clipboard,
                pre_inject_delay_ms: 10,
                post_inject_delay_ms: 10,
                requires_focus: true,
                supports_batch_inject: true,
            },
            AppSpecificConfig {
                bundle_id: "com.apple.TextEdit".to_string(),
                preferred_adapter: InjectionAdapterType::Clipboard,
                pre_inject_delay_ms: 5,
                post_inject_delay_ms: 5,
                requires_focus: true,
                supports_batch_inject: true,
            },
            // 网页浏览器 - 使用辅助功能平衡性能和兼容性
            AppSpecificConfig {
                bundle_id: "com.google.Chrome".to_string(),
                preferred_adapter: InjectionAdapterType::Accessibility,
                pre_inject_delay_ms: 20,
                post_inject_delay_ms: 20,
                requires_focus: true,
                supports_batch_inject: true,
            },
            AppSpecificConfig {
                bundle_id: "com.apple.Safari".to_string(),
                preferred_adapter: InjectionAdapterType::Accessibility,
                pre_inject_delay_ms: 15,
                post_inject_delay_ms: 15,
                requires_focus: true,
                supports_batch_inject: true,
            },
        ]
    }
}

/// 文本注入适配器 trait
#[async_trait::async_trait]
pub trait TextInjectionAdapter: Send + Sync {
    async fn inject_text(&self, text: &str, target: &AppInfo) -> AppResult<()>;
    fn get_supported_apps(&self) -> Vec<String>;
    fn get_injection_speed(&self) -> InjectionSpeed;
    fn get_adapter_type(&self) -> InjectionAdapterType;
    async fn test_compatibility(&self, target: &AppInfo) -> AppResult<bool>;
}

/// 剪贴板适配器 - 最快速度
pub struct ClipboardAdapter {
    preserve_original: bool,
}

#[async_trait::async_trait]
impl TextInjectionAdapter for ClipboardAdapter {
    async fn inject_text(&self, text: &str, target: &AppInfo) -> AppResult<()> {
        let original_clipboard = if self.preserve_original {
            self.get_clipboard_content().await.ok()
        } else {
            None
        };

        // 设置剪贴板内容
        self.set_clipboard_content(text).await?;
        
        // 等待剪贴板更新
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // 发送 Cmd+V
        self.send_paste_command().await?;
        
        // 恢复原始剪贴板内容
        if let Some(original) = original_clipboard {
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.set_clipboard_content(&original).await?;
        }
        
        Ok(())
    }

    fn get_supported_apps(&self) -> Vec<String> {
        vec!["*".to_string()] // 支持所有应用
    }

    fn get_injection_speed(&self) -> InjectionSpeed {
        InjectionSpeed::Fast
    }

    fn get_adapter_type(&self) -> InjectionAdapterType {
        InjectionAdapterType::Clipboard
    }

    async fn test_compatibility(&self, _target: &AppInfo) -> AppResult<bool> {
        // 剪贴板方式几乎兼容所有应用
        Ok(true)
    }
}

impl ClipboardAdapter {
    pub fn new(preserve_original: bool) -> Self {
        Self { preserve_original }
    }

    async fn get_clipboard_content(&self) -> AppResult<String> {
        let output = tokio::process::Command::new("pbpaste")
            .output()
            .await
            .map_err(|e| AppError::InjectionError(format!("获取剪贴板失败: {}", e)))?;
        
        String::from_utf8(output.stdout)
            .map_err(|e| AppError::InjectionError(format!("剪贴板内容编码错误: {}", e)))
    }

    async fn set_clipboard_content(&self, text: &str) -> AppResult<()> {
        let mut child = tokio::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::InjectionError(format!("启动pbcopy失败: {}", e)))?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(text.as_bytes()).await
                .map_err(|e| AppError::InjectionError(format!("写入剪贴板失败: {}", e)))?;
        }
        
        child.wait().await
            .map_err(|e| AppError::InjectionError(format!("pbcopy执行失败: {}", e)))?;
        
        Ok(())
    }

    async fn send_paste_command(&self) -> AppResult<()> {
        let script = r#"tell application "System Events" to keystroke "v" using command down"#;
        
        tokio::process::Command::new("osascript")
            .args(&["-e", script])
            .output()
            .await
            .map_err(|e| AppError::InjectionError(format!("发送粘贴命令失败: {}", e)))?;
        
        Ok(())
    }
}

/// 辅助功能适配器 - 平衡性能和兼容性
pub struct AccessibilityAdapter;

#[async_trait::async_trait]
impl TextInjectionAdapter for AccessibilityAdapter {
    async fn inject_text(&self, text: &str, target: &AppInfo) -> AppResult<()> {
        // 使用 macOS Accessibility API 直接插入文本
        self.inject_via_accessibility(text).await
    }

    fn get_supported_apps(&self) -> Vec<String> {
        vec![
            "com.microsoft.VSCode".to_string(),
            "com.apple.dt.Xcode".to_string(),
            "com.google.Chrome".to_string(),
            "com.apple.Safari".to_string(),
            "com.microsoft.Word".to_string(),
        ]
    }

    fn get_injection_speed(&self) -> InjectionSpeed {
        InjectionSpeed::Medium
    }

    fn get_adapter_type(&self) -> InjectionAdapterType {
        InjectionAdapterType::Accessibility
    }

    async fn test_compatibility(&self, target: &AppInfo) -> AppResult<bool> {
        // 检查目标应用是否支持辅助功能
        Ok(self.get_supported_apps().contains(&target.bundle_id) || 
           target.bundle_id.contains("editor") || 
           target.bundle_id.contains("text"))
    }
}

impl AccessibilityAdapter {
    async fn inject_via_accessibility(&self, text: &str) -> AppResult<()> {
        // 这里应该使用 Cocoa 的 Accessibility API
        // 目前使用 AppleScript 作为临时实现
        let escaped_text = text.replace("\"", "\\\"").replace("\n", "\\n");
        let script = format!(
            r#"tell application "System Events" to keystroke "{}""#,
            escaped_text
        );
        
        tokio::process::Command::new("osascript")
            .args(&["-e", &script])
            .output()
            .await
            .map_err(|e| AppError::InjectionError(format!("辅助功能注入失败: {}", e)))?;
        
        Ok(())
    }
}

/// 键盘模拟适配器 - 最高兼容性
pub struct KeyboardSimulationAdapter {
    typing_speed_ms: u64,
}

#[async_trait::async_trait]
impl TextInjectionAdapter for KeyboardSimulationAdapter {
    async fn inject_text(&self, text: &str, target: &AppInfo) -> AppResult<()> {
        // 逐字符模拟键盘输入
        for char in text.chars() {
            self.simulate_key_press(char).await?;
            tokio::time::sleep(Duration::from_millis(self.typing_speed_ms)).await;
        }
        Ok(())
    }

    fn get_supported_apps(&self) -> Vec<String> {
        vec!["*".to_string()] // 支持所有应用
    }

    fn get_injection_speed(&self) -> InjectionSpeed {
        InjectionSpeed::Slow
    }

    fn get_adapter_type(&self) -> InjectionAdapterType {
        InjectionAdapterType::KeyboardSimulation
    }

    async fn test_compatibility(&self, _target: &AppInfo) -> AppResult<bool> {
        // 键盘模拟兼容所有应用
        Ok(true)
    }
}

impl KeyboardSimulationAdapter {
    pub fn new(typing_speed_ms: u64) -> Self {
        Self { typing_speed_ms }
    }

    async fn simulate_key_press(&self, char: char) -> AppResult<()> {
        let script = if char == '\n' {
            r#"tell application "System Events" to key code 36"#.to_string() // Return key
        } else {
            format!(r#"tell application "System Events" to keystroke "{}""#, char)
        };
        
        tokio::process::Command::new("osascript")
            .args(&["-e", &script])
            .output()
            .await
            .map_err(|e| AppError::InjectionError(format!("键盘模拟失败: {}", e)))?;
        
        Ok(())
    }
}

/// 性能监控数据
#[derive(Debug, Clone, Serialize)]
pub struct InjectionPerformanceMetrics {
    pub adapter_type: InjectionAdapterType,
    pub injection_time_ms: u64,
    pub text_length: usize,
    pub success: bool,
    pub target_app: String,
    pub timestamp: u64,
}

/// 智能文本注入器 - 主要管理类
pub struct SmartTextInjector {
    config: Arc<Mutex<OptimizedTextInjectionConfig>>,
    adapters: HashMap<InjectionAdapterType, Box<dyn TextInjectionAdapter>>,
    app_preferences: Arc<Mutex<HashMap<String, InjectionAdapterType>>>,
    performance_metrics: Arc<Mutex<Vec<InjectionPerformanceMetrics>>>,
    permission_cache: Arc<Mutex<HashMap<String, (bool, Instant)>>>,
}

impl SmartTextInjector {
    pub fn new(config: OptimizedTextInjectionConfig) -> Self {
        let mut adapters: HashMap<InjectionAdapterType, Box<dyn TextInjectionAdapter>> = HashMap::new();
        
        // 初始化所有适配器
        adapters.insert(
            InjectionAdapterType::Clipboard,
            Box::new(ClipboardAdapter::new(config.preserve_clipboard))
        );
        adapters.insert(
            InjectionAdapterType::Accessibility,
            Box::new(AccessibilityAdapter)
        );
        adapters.insert(
            InjectionAdapterType::KeyboardSimulation,
            Box::new(KeyboardSimulationAdapter::new(50)) // 50ms per character
        );

        // 构建应用偏好映射
        let mut app_preferences = HashMap::new();
        for app_config in &config.app_specific_configs {
            app_preferences.insert(app_config.bundle_id.clone(), app_config.preferred_adapter.clone());
        }

        Self {
            config: Arc::new(Mutex::new(config)),
            adapters,
            app_preferences: Arc::new(Mutex::new(app_preferences)),
            performance_metrics: Arc::new(Mutex::new(Vec::new())),
            permission_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 智能文本注入 - 自动选择最优适配器
    pub async fn smart_inject(&self, text: &str, target_app: Option<AppInfo>) -> AppResult<bool> {
        if text.is_empty() {
            return Ok(false);
        }

        // 检查文本长度限制
        let config = self.config.lock().unwrap();
        if text.len() > config.max_text_length {
            return Err(AppError::InjectionError(
                format!("文本长度超过限制: {} > {}", text.len(), config.max_text_length)
            ));
        }
        drop(config);

        // 获取目标应用信息
        let target = match target_app {
            Some(app) => app,
            None => self.get_active_app_info().await?,
        };

        // 检查权限
        self.check_injection_permissions(&target).await?;

        // 选择最优适配器
        let adapter_type = self.select_optimal_adapter(&target).await?;
        
        // 执行注入
        let start_time = Instant::now();
        let result = self.execute_injection(text, &target, adapter_type.clone()).await;
        let injection_time = start_time.elapsed().as_millis() as u64;

        // 记录性能指标
        if self.config.lock().unwrap().performance_monitoring {
            self.record_performance_metrics(InjectionPerformanceMetrics {
                adapter_type: adapter_type.clone(),
                injection_time_ms: injection_time,
                text_length: text.len(),
                success: result.is_ok(),
                target_app: target.name.clone(),
                timestamp: chrono::Utc::now().timestamp() as u64,
            });
        }

        match result {
            Ok(_) => {
                println!("✅ 智能文本注入成功: {} -> {} ({}ms, {:?})", 
                    text.chars().take(20).collect::<String>(),
                    target.name,
                    injection_time,
                    adapter_type
                );
                Ok(true)
            }
            Err(e) => {
                // 如果启用了自动降级，尝试其他适配器
                if self.config.lock().unwrap().auto_fallback_enabled {
                    println!("⚠️ 主适配器失败，尝试降级: {}", e);
                    self.try_fallback_injection(text, &target, adapter_type).await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// 选择最优适配器
    async fn select_optimal_adapter(&self, target: &AppInfo) -> AppResult<InjectionAdapterType> {
        // 首先检查应用特定配置
        let app_preferences = self.app_preferences.lock().unwrap();
        if let Some(preferred) = app_preferences.get(&target.bundle_id) {
            return Ok(preferred.clone());
        }
        drop(app_preferences);

        // 测试适配器兼容性并选择最优的
        let config = self.config.lock().unwrap();
        let default_adapter = config.default_adapter.clone();
        drop(config);

        // 测试默认适配器
        if let Some(adapter) = self.adapters.get(&default_adapter) {
            if adapter.test_compatibility(target).await.unwrap_or(false) {
                return Ok(default_adapter);
            }
        }

        // 降级到其他适配器
        for (adapter_type, adapter) in &self.adapters {
            if adapter.test_compatibility(target).await.unwrap_or(false) {
                return Ok(adapter_type.clone());
            }
        }

        // 最后降级到键盘模拟
        Ok(InjectionAdapterType::KeyboardSimulation)
    }

    /// 执行文本注入
    async fn execute_injection(
        &self,
        text: &str,
        target: &AppInfo,
        adapter_type: InjectionAdapterType
    ) -> AppResult<()> {
        let adapter = self.adapters.get(&adapter_type)
            .ok_or_else(|| AppError::InjectionError(
                format!("适配器不存在: {:?}", adapter_type)
            ))?;

        // 获取应用特定配置
        let app_config = self.get_app_specific_config(&target.bundle_id);
        
        // 预注入延迟
        if let Some(config) = &app_config {
            tokio::time::sleep(Duration::from_millis(config.pre_inject_delay_ms)).await;
        }

        // 确保应用获得焦点
        if app_config.as_ref().map(|c| c.requires_focus).unwrap_or(true) {
            self.ensure_app_focus(target).await?;
        }

        // 执行注入
        adapter.inject_text(text, target).await?;

        // 后注入延迟
        if let Some(config) = &app_config {
            tokio::time::sleep(Duration::from_millis(config.post_inject_delay_ms)).await;
        }

        Ok(())
    }

    /// 降级注入尝试
    async fn try_fallback_injection(
        &self,
        text: &str,
        target: &AppInfo,
        failed_adapter: InjectionAdapterType
    ) -> AppResult<bool> {
        // 定义降级顺序
        let fallback_order = match failed_adapter {
            InjectionAdapterType::Clipboard => vec![
                InjectionAdapterType::Accessibility,
                InjectionAdapterType::KeyboardSimulation
            ],
            InjectionAdapterType::Accessibility => vec![
                InjectionAdapterType::Clipboard,
                InjectionAdapterType::KeyboardSimulation
            ],
            InjectionAdapterType::KeyboardSimulation => vec![
                InjectionAdapterType::Clipboard,
                InjectionAdapterType::Accessibility
            ],
        };

        for adapter_type in fallback_order {
            println!("🔄 尝试降级适配器: {:?}", adapter_type);
            
            match self.execute_injection(text, target, adapter_type.clone()).await {
                Ok(_) => {
                    println!("✅ 降级注入成功: {:?}", adapter_type);
                    
                    // 更新应用偏好
                    self.app_preferences.lock().unwrap()
                        .insert(target.bundle_id.clone(), adapter_type);
                    
                    return Ok(true);
                }
                Err(e) => {
                    println!("❌ 降级注入失败: {:?} - {}", adapter_type, e);
                }
            }
        }

        Err(AppError::InjectionError("所有适配器都失败".to_string()))
    }

    /// 获取当前活动应用信息
    async fn get_active_app_info(&self) -> AppResult<AppInfo> {
        // 这里应该调用系统 API 或现有的函数  
        // 暂时返回模拟数据
        Ok(AppInfo {
            name: "Current Application".to_string(),
            bundle_id: "com.unknown.app".to_string(),
        })
    }

    /// 检查注入权限
    async fn check_injection_permissions(&self, target: &AppInfo) -> AppResult<()> {
        let cache_key = format!("permission_{}", target.bundle_id);
        let mut cache = self.permission_cache.lock().unwrap();
        
        // 检查缓存
        if let Some((has_permission, timestamp)) = cache.get(&cache_key) {
            if timestamp.elapsed() < Duration::from_secs(300) { // 5分钟缓存
                if *has_permission {
                    return Ok(());
                } else {
                    return Err(AppError::PermissionError("缓存显示权限不足".to_string()));
                }
            }
        }

        // 执行权限检查
        let permission_status = PermissionManager::check_all_permissions()
            .map_err(|e| AppError::PermissionError(e.to_string()))?;

        let has_permission = permission_status.accessibility;
        cache.insert(cache_key, (has_permission, Instant::now()));
        drop(cache);

        if has_permission {
            Ok(())
        } else {
            Err(AppError::PermissionError("缺少辅助功能权限".to_string()))
        }
    }

    /// 确保应用获得焦点
    async fn ensure_app_focus(&self, target: &AppInfo) -> AppResult<()> {
        let script = format!(
            r#"tell application "{}" to activate"#,
            target.name.replace("\"", "\\\"")
        );
        
        tokio::process::Command::new("osascript")
            .args(&["-e", &script])
            .output()
            .await
            .map_err(|e| AppError::InjectionError(format!("激活应用失败: {}", e)))?;
        
        // 等待应用激活
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// 获取应用特定配置
    fn get_app_specific_config(&self, bundle_id: &str) -> Option<AppSpecificConfig> {
        let config = self.config.lock().unwrap();
        config.app_specific_configs
            .iter()
            .find(|c| c.bundle_id == bundle_id)
            .cloned()
    }

    /// 记录性能指标
    fn record_performance_metrics(&self, metrics: InjectionPerformanceMetrics) {
        let mut performance_metrics = self.performance_metrics.lock().unwrap();
        performance_metrics.push(metrics);
        
        // 保持最近1000条记录
        if performance_metrics.len() > 1000 {
            performance_metrics.remove(0);
        }
    }

    /// 获取性能报告
    pub fn get_performance_report(&self) -> Vec<InjectionPerformanceMetrics> {
        self.performance_metrics.lock().unwrap().clone()
    }

    /// 批量文本注入
    pub async fn batch_inject(&self, texts: Vec<String>, target_app: Option<AppInfo>) -> AppResult<Vec<bool>> {
        let config = self.config.lock().unwrap();
        if !config.batch_inject_enabled {
            return Err(AppError::InjectionError("批量注入功能已禁用".to_string()));
        }
        let batch_delay = config.batch_delay_ms;
        drop(config);

        let mut results = Vec::new();
        
        for (i, text) in texts.iter().enumerate() {
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(batch_delay)).await;
            }
            
            match self.smart_inject(text, target_app.clone()).await {
                Ok(success) => results.push(success),
                Err(_) => results.push(false),
            }
        }

        Ok(results)
    }
}

// Tauri 命令接口
#[tauri::command]
pub async fn optimized_inject_text(
    text: String,
    injector: tauri::State<'_, Arc<SmartTextInjector>>
) -> Result<bool, String> {
    injector.smart_inject(&text, None).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn batch_inject_optimized_text(
    texts: Vec<String>,
    injector: tauri::State<'_, Arc<SmartTextInjector>>
) -> Result<Vec<bool>, String> {
    injector.batch_inject(texts, None).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_injection_performance_report(
    injector: tauri::State<'_, Arc<SmartTextInjector>>
) -> Result<Vec<InjectionPerformanceMetrics>, String> {
    Ok(injector.get_performance_report())
}

#[tauri::command]
pub async fn test_optimized_injection() -> Result<String, String> {
    println!("🧪 测试优化后的文本注入系统");
    
    let config = OptimizedTextInjectionConfig::default();
    let injector = SmartTextInjector::new(config);
    
    let test_text = "Hello from optimized text injector! 🚀";
    
    match injector.smart_inject(test_text, None).await {
        Ok(success) => {
            if success {
                Ok("优化文本注入测试成功！".to_string())
            } else {
                Ok("文本注入被跳过（可能是重复检测）".to_string())
            }
        }
        Err(e) => Err(format!("优化文本注入测试失败: {}", e))
    }
}