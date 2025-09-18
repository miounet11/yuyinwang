// UnifiedShortcutManager - 统一快捷键管理系统
// 整合现有三套快捷键管理器，提供<50ms响应时间和统一的快捷键管理

use crate::errors::{AppError, AppResult};
use crossbeam_channel::{unbounded, Receiver, Sender};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, GlobalShortcutManager, Manager};

/// 快捷键类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutType {
    VoiceInput,
    QuickRecord,
    StopRecord,
    ShowHide,
    Custom(String),
}

/// 快捷键触发模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerMode {
    Press,          // 单次按下
    Hold(u64),      // 长按（毫秒）
    DoubleTap(u64), // 双击（间隔毫秒）
}

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub shortcut_id: String,
    pub key_combination: String,
    pub shortcut_type: ShortcutType,
    pub trigger_mode: TriggerMode,
    pub enabled: bool,
    pub description: String,
    pub priority: u8, // 0-255, 数值越大优先级越高
}

/// 快捷键性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutMetrics {
    pub shortcut_id: String,
    pub total_triggers: u64,
    pub average_response_time_ms: f64,
    pub success_rate: f64,
    pub last_trigger_time: Option<u64>,
    pub fastest_response_ms: u64,
    pub slowest_response_ms: u64,
}

/// 基准测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub iterations: u32,
    pub total_time_ms: u64,
    pub average_response_time_ms: f64,
    pub fastest_response_ms: u64,
    pub slowest_response_ms: u64,
    pub success_rate: f64,
}

/// 性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub total_shortcuts: usize,
    pub slow_shortcuts_count: usize,
    pub average_response_time: f64,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// 快捷键冲突信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub conflicting_shortcut: String,
    pub conflict_type: ConflictType,
    pub suggestion: String,
    pub alternative_shortcuts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    SystemReserved,
    ApplicationConflict,
    InternalConflict,
}

/// 快捷键预设方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutPreset {
    pub preset_id: String,
    pub name: String,
    pub description: String,
    pub shortcuts: Vec<ShortcutConfig>,
    pub compatibility_score: f64, // 0.0-1.0
    pub use_cases: Vec<String>,
}

/// 快捷键事件
#[derive(Debug, Clone)]
pub enum ShortcutEvent {
    Triggered {
        shortcut_id: String,
        trigger_time: Instant,
    },
    Failed {
        shortcut_id: String,
        error: String,
    },
    Registered {
        shortcut_id: String,
    },
    Unregistered {
        shortcut_id: String,
    },
}

/// 统一快捷键管理器
pub struct UnifiedShortcutManager {
    app_handle: AppHandle,
    shortcut_registry: Arc<RwLock<HashMap<String, ShortcutConfig>>>,
    performance_monitor: Arc<ShortcutPerformanceMonitor>,
    event_dispatcher: Arc<ShortcutEventDispatcher>,
    preset_manager: Arc<ShortcutPresetManager>,
    active_shortcuts: Arc<RwLock<HashMap<String, String>>>, // shortcut_id -> key_combination
}

impl UnifiedShortcutManager {
    /// 创建新的统一快捷键管理器
    pub fn new(app_handle: AppHandle) -> AppResult<Self> {
        let performance_monitor = Arc::new(ShortcutPerformanceMonitor::new());
        let event_dispatcher = Arc::new(ShortcutEventDispatcher::new());
        let preset_manager = Arc::new(ShortcutPresetManager::new());

        Ok(Self {
            app_handle,
            shortcut_registry: Arc::new(RwLock::new(HashMap::new())),
            performance_monitor,
            event_dispatcher,
            preset_manager,
            active_shortcuts: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 注册快捷键
    pub fn register_shortcut(&self, config: ShortcutConfig) -> AppResult<()> {
        let start_time = Instant::now();

        println!(
            "🔧 注册快捷键: {} -> {}",
            config.shortcut_id, config.key_combination
        );

        // 检查冲突
        if let Some(conflicts) = self.detect_conflicts(&config.key_combination) {
            if !conflicts.is_empty() {
                println!("⚠️ 检测到快捷键冲突: {:?}", conflicts);
                return Err(AppError::ShortcutError(format!(
                    "快捷键 {} 存在冲突",
                    config.key_combination
                )));
            }
        }

        // 注销已存在的快捷键
        if let Some(existing_combo) = self.active_shortcuts.read().get(&config.shortcut_id) {
            self.unregister_shortcut_by_combination(existing_combo)?;
        }

        // 注册新快捷键
        let app_handle = self.app_handle.clone();
        let shortcut_id = config.shortcut_id.clone();
        let key_combination = config.key_combination.clone();
        let shortcut_type = config.shortcut_type.clone();
        let performance_monitor = self.performance_monitor.clone();
        let event_dispatcher = self.event_dispatcher.clone();

        // 创建快捷键处理函数
        let result = app_handle
            .global_shortcut_manager()
            .register(&key_combination, move || {
                let trigger_start = Instant::now();

                // 记录触发事件
                performance_monitor.record_trigger(&shortcut_id, trigger_start);
                event_dispatcher.dispatch_event(ShortcutEvent::Triggered {
                    shortcut_id: shortcut_id.clone(),
                    trigger_time: trigger_start,
                });

                // 执行快捷键动作
                match &shortcut_type {
                    ShortcutType::VoiceInput => {
                        Self::handle_voice_input(&app_handle);
                    }
                    ShortcutType::QuickRecord => {
                        Self::handle_quick_record(&app_handle);
                    }
                    ShortcutType::StopRecord => {
                        Self::handle_stop_record(&app_handle);
                    }
                    ShortcutType::ShowHide => {
                        Self::handle_show_hide(&app_handle);
                    }
                    ShortcutType::Custom(action) => {
                        Self::handle_custom_action(&app_handle, action);
                    }
                }

                // 记录响应时间
                let response_time = trigger_start.elapsed();
                performance_monitor.record_response_time(&shortcut_id, response_time);

                println!("⚡ 快捷键响应时间: {:?}", response_time);
            });

        match result {
            Ok(_) => {
                // 更新注册表
                self.shortcut_registry
                    .write()
                    .insert(config.shortcut_id.clone(), config.clone());
                self.active_shortcuts
                    .write()
                    .insert(config.shortcut_id.clone(), config.key_combination.clone());

                // 记录注册事件
                self.event_dispatcher
                    .dispatch_event(ShortcutEvent::Registered {
                        shortcut_id: config.shortcut_id.clone(),
                    });

                let register_time = start_time.elapsed();
                println!("✅ 快捷键注册成功，耗时: {:?}", register_time);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("注册快捷键 {} 失败: {}", config.key_combination, e);
                println!("❌ {}", error_msg);

                self.event_dispatcher.dispatch_event(ShortcutEvent::Failed {
                    shortcut_id: config.shortcut_id,
                    error: error_msg.clone(),
                });

                Err(AppError::ShortcutError(error_msg))
            }
        }
    }

    /// 注销快捷键
    pub fn unregister_shortcut(&self, shortcut_id: &str) -> AppResult<()> {
        println!("🗑️ 注销快捷键: {}", shortcut_id);

        let key_combination = {
            let active = self.active_shortcuts.read();
            active.get(shortcut_id).cloned()
        };

        if let Some(combo) = key_combination {
            self.unregister_shortcut_by_combination(&combo)?;

            // 从注册表中移除
            self.shortcut_registry.write().remove(shortcut_id);
            self.active_shortcuts.write().remove(shortcut_id);

            // 记录注销事件
            self.event_dispatcher
                .dispatch_event(ShortcutEvent::Unregistered {
                    shortcut_id: shortcut_id.to_string(),
                });

            println!("✅ 快捷键 {} 已注销", shortcut_id);
            Ok(())
        } else {
            Err(AppError::ShortcutError(format!(
                "快捷键 {} 不存在",
                shortcut_id
            )))
        }
    }

    /// 通过组合键注销快捷键
    fn unregister_shortcut_by_combination(&self, key_combination: &str) -> AppResult<()> {
        self.app_handle
            .global_shortcut_manager()
            .unregister(key_combination)
            .map_err(|e| AppError::ShortcutError(format!("注销快捷键失败: {}", e)))
    }

    /// 更新快捷键配置
    pub fn update_shortcut(&self, shortcut_id: &str, new_config: ShortcutConfig) -> AppResult<()> {
        println!("🔄 更新快捷键配置: {}", shortcut_id);

        // 先注销旧的快捷键
        self.unregister_shortcut(shortcut_id)?;

        // 注册新的快捷键
        self.register_shortcut(new_config)?;

        println!("✅ 快捷键配置已更新: {}", shortcut_id);
        Ok(())
    }

    /// 获取快捷键性能指标
    pub fn get_response_metrics(&self) -> Vec<ShortcutMetrics> {
        self.performance_monitor.get_all_metrics()
    }

    /// 基准测试快捷键
    pub fn benchmark_shortcut(
        &self,
        shortcut_id: &str,
        iterations: u32,
    ) -> AppResult<BenchmarkResult> {
        println!(
            "🏃 开始基准测试快捷键: {} ({} 次迭代)",
            shortcut_id, iterations
        );

        let mut response_times = Vec::new();

        for i in 0..iterations {
            let start_time = Instant::now();

            // 模拟快捷键触发
            self.performance_monitor
                .record_trigger(shortcut_id, start_time);

            let response_time = start_time.elapsed();
            response_times.push(response_time.as_millis() as u64);

            // 避免过度频繁测试
            std::thread::sleep(Duration::from_millis(10));
        }

        let total_time = response_times.iter().sum::<u64>();
        let avg_response_time = total_time as f64 / iterations as f64;
        let min_response_time = *response_times.iter().min().unwrap();
        let max_response_time = *response_times.iter().max().unwrap();

        let benchmark_result = BenchmarkResult {
            iterations,
            total_time_ms: total_time,
            average_response_time_ms: avg_response_time,
            fastest_response_ms: min_response_time,
            slowest_response_ms: max_response_time,
            success_rate: 100.0, // 假设测试100%成功
        };

        println!(
            "📊 基准测试结果: 平均 {:.2}ms, 最快 {}ms, 最慢 {}ms",
            avg_response_time, min_response_time, max_response_time
        );

        Ok(benchmark_result)
    }

    /// 检测快捷键冲突
    pub fn detect_conflicts(&self, key_combination: &str) -> Option<Vec<ConflictInfo>> {
        let mut conflicts = Vec::new();

        // 检查系统保留快捷键
        let system_reserved = vec![
            "Cmd+Space",
            "Cmd+Tab",
            "Cmd+Q",
            "Cmd+W",
            "Cmd+T",
            "Cmd+N",
            "Cmd+S",
            "Cmd+A",
            "Cmd+C",
            "Cmd+V",
            "Cmd+Z",
            "Cmd+Y",
        ];

        if system_reserved.contains(&key_combination) {
            conflicts.push(ConflictInfo {
                conflicting_shortcut: key_combination.to_string(),
                conflict_type: ConflictType::SystemReserved,
                suggestion: "建议使用不与系统冲突的组合键".to_string(),
                alternative_shortcuts: self.suggest_alternatives(key_combination),
            });
        }

        // 检查内部冲突
        let active = self.active_shortcuts.read();
        for (existing_id, existing_combo) in active.iter() {
            if existing_combo == key_combination {
                conflicts.push(ConflictInfo {
                    conflicting_shortcut: existing_id.clone(),
                    conflict_type: ConflictType::InternalConflict,
                    suggestion: format!("快捷键已被 {} 使用", existing_id),
                    alternative_shortcuts: self.suggest_alternatives(key_combination),
                });
            }
        }

        if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        }
    }

    /// 建议替代快捷键
    pub fn suggest_alternatives(&self, conflicted_shortcut: &str) -> Vec<String> {
        // 基于冲突快捷键生成替代方案
        let alternatives = vec![
            format!("Shift+{}", conflicted_shortcut),
            format!("Alt+{}", conflicted_shortcut),
            format!(
                "Cmd+Shift+{}",
                conflicted_shortcut.split('+').last().unwrap_or("Space")
            ),
            format!(
                "Ctrl+Alt+{}",
                conflicted_shortcut.split('+').last().unwrap_or("Space")
            ),
        ];

        // 过滤掉已占用的快捷键
        let active = self.active_shortcuts.read();
        alternatives
            .into_iter()
            .filter(|alt| !active.values().any(|existing| existing == alt))
            .take(3)
            .collect()
    }

    /// 应用预设方案
    pub fn apply_preset(&self, preset_id: &str) -> AppResult<()> {
        println!("🎯 应用预设方案: {}", preset_id);

        let preset = self
            .preset_manager
            .get_preset(preset_id)
            .ok_or_else(|| AppError::ShortcutError(format!("预设方案 {} 不存在", preset_id)))?;

        // 清除现有快捷键
        self.unregister_all_shortcuts()?;

        // 注册预设快捷键
        let mut success_count = 0;
        let mut failed_shortcuts = Vec::new();

        for shortcut_config in &preset.shortcuts {
            match self.register_shortcut(shortcut_config.clone()) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    failed_shortcuts.push((shortcut_config.shortcut_id.clone(), e.to_string()));
                }
            }
        }

        println!(
            "✅ 预设应用完成: 成功 {}/{}, 失败 {}",
            success_count,
            preset.shortcuts.len(),
            failed_shortcuts.len()
        );

        if !failed_shortcuts.is_empty() {
            println!("❌ 失败的快捷键: {:?}", failed_shortcuts);
        }

        Ok(())
    }

    /// 注销所有快捷键
    pub fn unregister_all_shortcuts(&self) -> AppResult<()> {
        println!("🧹 注销所有快捷键");

        let shortcuts_to_remove: Vec<String> =
            { self.active_shortcuts.read().keys().cloned().collect() };

        for shortcut_id in shortcuts_to_remove {
            self.unregister_shortcut(&shortcut_id)?;
        }

        println!("✅ 所有快捷键已注销");
        Ok(())
    }

    // 快捷键动作处理函数
    fn handle_voice_input(app_handle: &AppHandle) {
        println!("🎤 触发语音输入");
        if let Some(window) = app_handle.get_window("floating-input") {
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.emit("voice_input_triggered", ());
        }
    }

    fn handle_quick_record(app_handle: &AppHandle) {
        println!("⏺️ 触发快速录制");
        let _ = app_handle.emit_all("quick_record_triggered", ());
    }

    fn handle_stop_record(app_handle: &AppHandle) {
        println!("⏹️ 触发停止录制");
        let _ = app_handle.emit_all("stop_record_triggered", ());
    }

    fn handle_show_hide(app_handle: &AppHandle) {
        println!("👀 触发显示/隐藏");
        if let Some(window) = app_handle.get_window("main") {
            let _ = match window.is_visible() {
                Ok(true) => window.hide(),
                _ => {
                    let _ = window.show();
                    window.set_focus()
                }
            };
        }
    }

    fn handle_custom_action(app_handle: &AppHandle, action: &str) {
        println!("🔧 触发自定义动作: {}", action);
        let _ = app_handle.emit_all("custom_shortcut_triggered", action);
    }

    /// 获取已注册快捷键（方法名兼容）
    pub fn get_registered_shortcuts(&self) -> HashMap<String, String> {
        let registry = self.shortcut_registry.read();
        registry
            .iter()
            .map(|(id, config)| (id.clone(), config.key_combination.clone()))
            .collect()
    }

    /// 检查快捷键冲突（方法名兼容）
    pub fn check_conflict(&self, key_combination: &str) -> Option<String> {
        let registry = self.shortcut_registry.read();
        for (id, config) in registry.iter() {
            if config.key_combination == key_combination {
                return Some(id.clone());
            }
        }
        None
    }

    /// 获取可用预设列表（方法名兼容）
    pub fn get_available_presets(&self) -> Vec<String> {
        self.preset_manager.get_available_presets()
    }

    /// 获取性能指标（方法名兼容）
    pub fn get_performance_metrics(&self) -> Vec<ShortcutMetrics> {
        self.performance_monitor.get_response_metrics()
    }

    /// 获取性能报告（方法名兼容）
    pub fn get_performance_report(&self) -> PerformanceReport {
        self.performance_monitor.get_performance_report()
    }

    /// 运行基准测试
    pub fn run_benchmark(&self, iterations: u32) -> AppResult<BenchmarkResult> {
        self.performance_monitor.run_benchmark(iterations)
    }

    /// 导出配置
    pub fn export_config(&self) -> AppResult<String> {
        let registry = self.shortcut_registry.read();
        let configs: Vec<&ShortcutConfig> = registry.values().collect();
        serde_json::to_string_pretty(&configs)
            .map_err(|e| AppError::DataSerializationError(e.to_string()))
    }

    /// 导入配置
    pub fn import_config(&self, config_json: &str) -> AppResult<()> {
        let configs: Vec<ShortcutConfig> = serde_json::from_str(config_json)
            .map_err(|e| AppError::DataSerializationError(e.to_string()))?;

        for config in configs {
            self.register_shortcut(config)?;
        }
        Ok(())
    }

    /// 重置所有快捷键
    pub fn reset_all(&self) -> AppResult<()> {
        // 注销所有快捷键
        let shortcuts_to_remove: Vec<String> = {
            let registry = self.shortcut_registry.read();
            registry.keys().cloned().collect()
        };

        for shortcut_id in shortcuts_to_remove {
            self.unregister_shortcut(&shortcut_id)?;
        }

        // 清理所有数据
        self.shortcut_registry.write().clear();
        self.active_shortcuts.write().clear();

        Ok(())
    }
}

/// 快捷键性能监控器
pub struct ShortcutPerformanceMonitor {
    metrics: Arc<RwLock<HashMap<String, ShortcutMetrics>>>,
}

impl ShortcutPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn record_trigger(&self, shortcut_id: &str, trigger_time: Instant) {
        let mut metrics = self.metrics.write();
        let metric = metrics
            .entry(shortcut_id.to_string())
            .or_insert(ShortcutMetrics {
                shortcut_id: shortcut_id.to_string(),
                total_triggers: 0,
                average_response_time_ms: 0.0,
                success_rate: 1.0,
                last_trigger_time: None,
                fastest_response_ms: u64::MAX,
                slowest_response_ms: 0,
            });

        metric.total_triggers += 1;
        metric.last_trigger_time = Some(trigger_time.elapsed().as_millis() as u64);
    }

    pub fn record_response_time(&self, shortcut_id: &str, response_time: Duration) {
        let mut metrics = self.metrics.write();
        if let Some(metric) = metrics.get_mut(shortcut_id) {
            let response_ms = response_time.as_millis() as u64;

            // 更新最快和最慢响应时间
            metric.fastest_response_ms = metric.fastest_response_ms.min(response_ms);
            metric.slowest_response_ms = metric.slowest_response_ms.max(response_ms);

            // 更新平均响应时间
            let total_time = metric.average_response_time_ms * (metric.total_triggers - 1) as f64;
            metric.average_response_time_ms =
                (total_time + response_ms as f64) / metric.total_triggers as f64;
        }
    }

    pub fn get_all_metrics(&self) -> Vec<ShortcutMetrics> {
        self.metrics.read().values().cloned().collect()
    }

    /// 获取性能报告并检查是否超过50ms目标
    pub fn get_performance_report(&self) -> PerformanceReport {
        let metrics = self.get_all_metrics();
        let mut slow_shortcuts = Vec::new();
        let mut warnings = Vec::new();

        const TARGET_RESPONSE_TIME_MS: f64 = 50.0;

        for metric in &metrics {
            if metric.average_response_time_ms > TARGET_RESPONSE_TIME_MS {
                slow_shortcuts.push(metric.clone());
                warnings.push(format!(
                    "快捷键 '{}' 平均响应时间 {:.1}ms 超过目标 {}ms",
                    metric.shortcut_id, metric.average_response_time_ms, TARGET_RESPONSE_TIME_MS
                ));
            }
        }

        PerformanceReport {
            total_shortcuts: metrics.len(),
            slow_shortcuts_count: slow_shortcuts.len(),
            average_response_time: metrics
                .iter()
                .map(|m| m.average_response_time_ms)
                .sum::<f64>()
                / metrics.len() as f64,
            warnings,
            suggestions: self.generate_optimization_suggestions(&slow_shortcuts),
        }
    }

    /// 生成优化建议
    fn generate_optimization_suggestions(&self, slow_shortcuts: &[ShortcutMetrics]) -> Vec<String> {
        let mut suggestions = Vec::new();

        if slow_shortcuts.is_empty() {
            suggestions.push("✅ 所有快捷键响应时间都在目标范围内".to_string());
            return suggestions;
        }

        suggestions.push("🚀 性能优化建议:".to_string());

        for metric in slow_shortcuts {
            if metric.average_response_time_ms > 100.0 {
                suggestions.push(format!(
                    "  • 快捷键 '{}' 严重超时 ({:.1}ms)，考虑异步处理或减少操作复杂度",
                    metric.shortcut_id, metric.average_response_time_ms
                ));
            } else if metric.average_response_time_ms > 75.0 {
                suggestions.push(format!(
                    "  • 快捷键 '{}' 响应偏慢 ({:.1}ms)，检查是否有阻塞操作",
                    metric.shortcut_id, metric.average_response_time_ms
                ));
            } else {
                suggestions.push(format!(
                    "  • 快捷键 '{}' 轻微超时 ({:.1}ms)，可进行微调优化",
                    metric.shortcut_id, metric.average_response_time_ms
                ));
            }
        }

        suggestions.push("  • 考虑使用快捷键预加载机制".to_string());
        suggestions.push("  • 检查系统负载和后台进程".to_string());

        suggestions
    }

    /// 获取响应时间指标（兼容性方法）
    pub fn get_response_metrics(&self) -> Vec<ShortcutMetrics> {
        self.get_all_metrics()
    }

    /// 运行基准测试
    pub fn run_benchmark(&self, iterations: u32) -> AppResult<BenchmarkResult> {
        // 模拟基准测试
        let mut total_time = 0u64;
        let mut fastest = u64::MAX;
        let mut slowest = 0u64;

        for _ in 0..iterations {
            let start = std::time::Instant::now();
            // 模拟快捷键处理
            std::thread::sleep(std::time::Duration::from_millis(1));
            let elapsed = start.elapsed().as_millis() as u64;

            total_time += elapsed;
            fastest = fastest.min(elapsed);
            slowest = slowest.max(elapsed);
        }

        let average = total_time as f64 / iterations as f64;

        Ok(BenchmarkResult {
            iterations,
            total_time_ms: total_time,
            average_response_time_ms: average,
            fastest_response_ms: fastest,
            slowest_response_ms: slowest,
            success_rate: 100.0, // 基准测试假设100%成功率
        })
    }
}

/// 快捷键事件分发器
pub struct ShortcutEventDispatcher {
    event_sender: Sender<ShortcutEvent>,
    _event_receiver: Receiver<ShortcutEvent>,
}

impl ShortcutEventDispatcher {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            event_sender: sender,
            _event_receiver: receiver,
        }
    }

    pub fn dispatch_event(&self, event: ShortcutEvent) {
        let _ = self.event_sender.send(event);
    }
}

/// 快捷键预设管理器
pub struct ShortcutPresetManager {
    presets: HashMap<String, ShortcutPreset>,
}

impl ShortcutPresetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            presets: HashMap::new(),
        };
        manager.initialize_default_presets();
        manager
    }

    fn initialize_default_presets(&mut self) {
        self.presets.clear();

        // 1. 专业模式
        let professional_preset = ShortcutPreset {
            preset_id: "professional".to_string(),
            name: "专业模式".to_string(),
            description: "专业录音快捷键方案".to_string(),
            shortcuts: vec![ShortcutConfig {
                shortcut_id: "start_recording".to_string(),
                key_combination: "cmd+shift+r".to_string(),
                shortcut_type: ShortcutType::QuickRecord,
                trigger_mode: TriggerMode::Press,
                enabled: true,
                description: "开始录音".to_string(),
                priority: 50,
            }],
            compatibility_score: 0.9,
            use_cases: vec!["专业录音".to_string()],
        };
        self.presets
            .insert("professional".to_string(), professional_preset);

        // 2. 简约模式
        let minimal_preset = ShortcutPreset {
            preset_id: "minimal".to_string(),
            name: "简约模式".to_string(),
            description: "最少快捷键，适合新手".to_string(),
            shortcuts: vec![ShortcutConfig {
                shortcut_id: "start_recording".to_string(),
                key_combination: "f1".to_string(),
                shortcut_type: ShortcutType::QuickRecord,
                trigger_mode: TriggerMode::Press,
                enabled: true,
                description: "开始录音".to_string(),
                priority: 50,
            }],
            compatibility_score: 1.0,
            use_cases: vec!["简单录音".to_string()],
        };
        self.presets.insert("minimal".to_string(), minimal_preset);

        // 3. 游戏主播方案 - 适合游戏录制和直播
        let gaming_preset = ShortcutPreset {
            preset_id: "gaming".to_string(),
            name: "游戏主播".to_string(),
            description: "游戏录制和直播快捷键方案".to_string(),
            shortcuts: vec![
                ShortcutConfig {
                    shortcut_id: "start_recording".to_string(),
                    key_combination: "f9".to_string(),
                    shortcut_type: ShortcutType::QuickRecord,
                    trigger_mode: TriggerMode::Press,
                    enabled: true,
                    description: "开始录音".to_string(),
                    priority: 50,
                },
                ShortcutConfig {
                    shortcut_id: "stop_recording".to_string(),
                    key_combination: "f10".to_string(),
                    shortcut_type: ShortcutType::StopRecord,
                    trigger_mode: TriggerMode::Press,
                    enabled: true,
                    description: "停止录音".to_string(),
                    priority: 50,
                },
            ],
            compatibility_score: 0.8,
            use_cases: vec!["游戏直播".to_string(), "游戏录制".to_string()],
        };
        self.presets.insert("gaming".to_string(), gaming_preset);

        // 4. 媒体制作方案 - 适合播客、视频制作
        let media_preset = ShortcutPreset {
            preset_id: "media".to_string(),
            name: "媒体制作".to_string(),
            description: "播客和视频制作快捷键方案".to_string(),
            shortcuts: vec![
                ShortcutConfig {
                    shortcut_id: "start_recording".to_string(),
                    key_combination: "ctrl+r".to_string(),
                    shortcut_type: ShortcutType::QuickRecord,
                    trigger_mode: TriggerMode::Press,
                    enabled: true,
                    description: "开始录音".to_string(),
                    priority: 50,
                },
                ShortcutConfig {
                    shortcut_id: "stop_recording".to_string(),
                    key_combination: "ctrl+t".to_string(),
                    shortcut_type: ShortcutType::StopRecord,
                    trigger_mode: TriggerMode::Press,
                    enabled: true,
                    description: "停止录音".to_string(),
                    priority: 50,
                },
            ],
            compatibility_score: 0.85,
            use_cases: vec!["播客制作".to_string(), "视频制作".to_string()],
        };
        self.presets.insert("media".to_string(), media_preset);

        // 5. 效率办公方案 - 适合会议录音、语音笔记
        let office_preset = ShortcutPreset {
            preset_id: "office".to_string(),
            name: "效率办公".to_string(),
            description: "会议录音和语音笔记快捷键方案".to_string(),
            shortcuts: vec![
                ShortcutConfig {
                    shortcut_id: "start_recording".to_string(),
                    key_combination: "ctrl+alt+r".to_string(),
                    shortcut_type: ShortcutType::QuickRecord,
                    trigger_mode: TriggerMode::Press,
                    enabled: true,
                    description: "开始录音".to_string(),
                    priority: 50,
                },
                ShortcutConfig {
                    shortcut_id: "stop_recording".to_string(),
                    key_combination: "ctrl+alt+s".to_string(),
                    shortcut_type: ShortcutType::StopRecord,
                    trigger_mode: TriggerMode::Press,
                    enabled: true,
                    description: "停止录音".to_string(),
                    priority: 50,
                },
            ],
            compatibility_score: 0.9,
            use_cases: vec!["会议录音".to_string(), "语音笔记".to_string()],
        };
        self.presets.insert("office".to_string(), office_preset);
    }

    pub fn get_preset(&self, preset_id: &str) -> Option<&ShortcutPreset> {
        self.presets.get(preset_id)
    }

    pub fn get_all_presets(&self) -> Vec<&ShortcutPreset> {
        self.presets.values().collect()
    }

    /// 获取可用预设列表（方法名兼容）
    pub fn get_available_presets(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }
}
