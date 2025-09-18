use crate::errors::{AppError, AppResult};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

/// 用户体验事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UXEventType {
    ButtonClick,
    MenuOpen,
    DialogOpen,
    FileOperation,
    AudioOperation,
    TranscriptionStart,
    TranscriptionComplete,
    PermissionCheck,
    ShortcutTrigger,
    WindowSwitch,
    DatabaseQuery,
    NetworkRequest,
}

/// 用户体验事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UXEvent {
    pub event_id: String,
    pub event_type: UXEventType,
    pub component: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration_ms: Option<u64>,
    pub user_action: String,
    pub context: HashMap<String, String>,
}

/// 性能阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub ui_response_ms: u64,         // UI响应时间阈值 (目标: <100ms)
    pub file_operation_ms: u64,      // 文件操作阈值 (目标: <500ms)
    pub audio_operation_ms: u64,     // 音频操作阈值 (目标: <200ms)
    pub transcription_start_ms: u64, // 转录启动阈值 (目标: <1000ms)
    pub database_query_ms: u64,      // 数据库查询阈值 (目标: <100ms)
    pub network_request_ms: u64,     // 网络请求阈值 (目标: <3000ms)
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            ui_response_ms: 100,
            file_operation_ms: 500,
            audio_operation_ms: 200,
            transcription_start_ms: 1000,
            database_query_ms: 100,
            network_request_ms: 3000,
        }
    }
}

/// 用户体验指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UXMetrics {
    pub total_events: u64,
    pub slow_events_count: u64,
    pub average_response_time: f64,
    pub p95_response_time: u64,
    pub user_satisfaction_score: f64, // 0.0-1.0
    pub problematic_components: Vec<String>,
    pub performance_trends: Vec<f64>,
}

/// 用户体验监控器
pub struct UXMonitor {
    app_handle: AppHandle,
    events: Arc<RwLock<VecDeque<UXEvent>>>,
    active_events: Arc<RwLock<HashMap<String, UXEvent>>>,
    thresholds: Arc<RwLock<PerformanceThresholds>>,
    component_metrics: Arc<RwLock<HashMap<String, ComponentMetrics>>>,
    max_events_history: usize,
}

/// 组件性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub component_name: String,
    pub total_events: u64,
    pub slow_events: u64,
    pub average_response_time: f64,
    pub last_update: Instant,
}

impl UXMonitor {
    pub fn new(app_handle: AppHandle) -> AppResult<Self> {
        Ok(Self {
            app_handle,
            events: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            active_events: Arc::new(RwLock::new(HashMap::new())),
            thresholds: Arc::new(RwLock::new(PerformanceThresholds::default())),
            component_metrics: Arc::new(RwLock::new(HashMap::new())),
            max_events_history: 1000,
        })
    }

    /// 开始跟踪用户体验事件
    pub fn start_event(
        &self,
        event_type: UXEventType,
        component: &str,
        user_action: &str,
        context: HashMap<String, String>,
    ) -> String {
        let event_id = uuid::Uuid::new_v4().to_string();

        let event = UXEvent {
            event_id: event_id.clone(),
            event_type,
            component: component.to_string(),
            start_time: Instant::now(),
            end_time: None,
            duration_ms: None,
            user_action: user_action.to_string(),
            context,
        };

        self.active_events.write().insert(event_id.clone(), event);

        println!(
            "🔍 UX事件开始: {} - {} - {}",
            component, user_action, event_id
        );

        event_id
    }

    /// 结束跟踪用户体验事件
    pub fn end_event(&self, event_id: &str) -> AppResult<()> {
        let mut active_events = self.active_events.write();

        if let Some(mut event) = active_events.remove(event_id) {
            let end_time = Instant::now();
            let duration = end_time.duration_since(event.start_time);

            event.end_time = Some(end_time);
            event.duration_ms = Some(duration.as_millis() as u64);

            // 检查是否超过阈值
            let is_slow = self.is_event_slow(&event);

            if is_slow {
                println!(
                    "⚠️  慢速UX事件: {} - {} - {}ms",
                    event.component,
                    event.user_action,
                    event.duration_ms.unwrap()
                );

                // 发送慢速事件警告到前端
                let _ = self.app_handle.emit_all("ux_slow_event", &event);
            }

            // 更新组件指标
            self.update_component_metrics(&event);

            // 添加到历史记录
            let mut events = self.events.write();
            if events.len() >= self.max_events_history {
                events.pop_front();
            }
            events.push_back(event.clone());

            println!(
                "✅ UX事件完成: {} - {}ms",
                event.component,
                event.duration_ms.unwrap()
            );

            // 发送事件完成通知到前端
            let _ = self.app_handle.emit_all("ux_event_completed", &event);
        }

        Ok(())
    }

    /// 检查事件是否超过性能阈值
    fn is_event_slow(&self, event: &UXEvent) -> bool {
        let thresholds = self.thresholds.read();
        let duration_ms = event.duration_ms.unwrap_or(0);

        match event.event_type {
            UXEventType::ButtonClick | UXEventType::MenuOpen | UXEventType::DialogOpen => {
                duration_ms > thresholds.ui_response_ms
            }
            UXEventType::FileOperation => duration_ms > thresholds.file_operation_ms,
            UXEventType::AudioOperation => duration_ms > thresholds.audio_operation_ms,
            UXEventType::TranscriptionStart => duration_ms > thresholds.transcription_start_ms,
            UXEventType::DatabaseQuery => duration_ms > thresholds.database_query_ms,
            UXEventType::NetworkRequest => duration_ms > thresholds.network_request_ms,
            _ => duration_ms > thresholds.ui_response_ms,
        }
    }

    /// 更新组件性能指标
    fn update_component_metrics(&self, event: &UXEvent) {
        let mut metrics = self.component_metrics.write();

        let component_metric = metrics
            .entry(event.component.clone())
            .or_insert(ComponentMetrics {
                component_name: event.component.clone(),
                total_events: 0,
                slow_events: 0,
                average_response_time: 0.0,
                last_update: Instant::now(),
            });

        component_metric.total_events += 1;

        if self.is_event_slow(event) {
            component_metric.slow_events += 1;
        }

        // 更新平均响应时间
        let duration_ms = event.duration_ms.unwrap_or(0) as f64;
        component_metric.average_response_time = (component_metric.average_response_time
            * (component_metric.total_events - 1) as f64
            + duration_ms)
            / component_metric.total_events as f64;

        component_metric.last_update = Instant::now();
    }

    /// 获取用户体验指标
    pub fn get_ux_metrics(&self) -> UXMetrics {
        let events = self.events.read();
        let thresholds = self.thresholds.read();

        if events.is_empty() {
            return UXMetrics {
                total_events: 0,
                slow_events_count: 0,
                average_response_time: 0.0,
                p95_response_time: 0,
                user_satisfaction_score: 1.0,
                problematic_components: vec![],
                performance_trends: vec![],
            };
        }

        let total_events = events.len() as u64;
        let mut durations: Vec<u64> = events.iter().filter_map(|e| e.duration_ms).collect();

        durations.sort();

        let slow_events_count = events.iter().filter(|e| self.is_event_slow(e)).count() as u64;

        let average_response_time = if !durations.is_empty() {
            durations.iter().sum::<u64>() as f64 / durations.len() as f64
        } else {
            0.0
        };

        let p95_response_time = if !durations.is_empty() {
            let p95_index = (durations.len() as f64 * 0.95) as usize;
            durations
                .get(p95_index.min(durations.len() - 1))
                .copied()
                .unwrap_or(0)
        } else {
            0
        };

        // 计算用户满意度分数 (0.0-1.0)
        let user_satisfaction_score = if total_events > 0 {
            let slow_ratio = slow_events_count as f64 / total_events as f64;
            (1.0 - slow_ratio).max(0.0)
        } else {
            1.0
        };

        // 找出有问题的组件
        let component_metrics = self.component_metrics.read();
        let problematic_components: Vec<String> = component_metrics
            .values()
            .filter(|metrics| {
                metrics.total_events > 5
                    && (metrics.slow_events as f64 / metrics.total_events as f64) > 0.3
            })
            .map(|metrics| metrics.component_name.clone())
            .collect();

        // 性能趋势 (最近10个事件的平均响应时间)
        let performance_trends: Vec<f64> = events
            .iter()
            .rev()
            .take(10)
            .filter_map(|e| e.duration_ms.map(|d| d as f64))
            .collect();

        UXMetrics {
            total_events,
            slow_events_count,
            average_response_time,
            p95_response_time,
            user_satisfaction_score,
            problematic_components,
            performance_trends,
        }
    }

    /// 获取组件性能报告
    pub fn get_component_report(&self) -> Vec<ComponentMetrics> {
        self.component_metrics.read().values().cloned().collect()
    }

    /// 设置性能阈值
    pub fn set_thresholds(&self, thresholds: PerformanceThresholds) {
        *self.thresholds.write() = thresholds;
        println!("📊 性能阈值已更新");
    }

    /// 清理旧事件
    pub fn cleanup_old_events(&self, max_age: Duration) {
        let mut events = self.events.write();
        let cutoff_time = Instant::now() - max_age;

        events.retain(|event| event.start_time > cutoff_time);

        println!("🧹 清理了旧的UX事件，当前事件数: {}", events.len());
    }

    /// 导出性能报告
    pub fn export_performance_report(&self) -> AppResult<String> {
        let metrics = self.get_ux_metrics();
        let component_report = self.get_component_report();

        let report = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "overall_metrics": metrics,
            "component_metrics": component_report,
            "thresholds": *self.thresholds.read()
        });

        serde_json::to_string_pretty(&report)
            .map_err(|e| AppError::SerializationError(format!("导出报告失败: {}", e)))
    }
}
