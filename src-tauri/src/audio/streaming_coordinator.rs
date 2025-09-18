use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
/// 流式转录协调器
///
/// 这是Story 1.3的核心组件，负责协调：
/// - 实时音频流处理
/// - 音频质量监控
/// - 流式转录处理
/// - UI事件分发
/// - 性能监控和优化
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

use super::realtime_streamer::{RealtimeAudioStreamer, RealtimeEvent};
use crate::errors::{AppError, AppResult};
use crate::transcription::TranscriptionService;
use crate::types::TranscriptionConfig;

/// UI事件类型 - 发送给前端的事件
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum UIEvent {
    /// 转录文本更新
    TranscriptionUpdate {
        text: String,
        confidence: f64,
        is_final: bool,
        chunk_id: u64,
        timestamp: u64,
    },
    /// 音频质量指标更新
    AudioQualityUpdate {
        volume_db: f64,
        snr_db: Option<f64>,
        noise_level_db: f64,
        clarity_score: f64,
        recommendations: Vec<String>,
    },
    /// 录音状态变化
    RecordingStatusChanged {
        is_recording: bool,
        session_id: Option<String>,
    },
    /// 缓冲区状态更新
    BufferStatusUpdate { used_percent: u8, latency_ms: u64 },
    /// 性能警告
    PerformanceWarning {
        message: String,
        severity: String, // "low", "medium", "high"
    },
    /// 设备错误
    DeviceError {
        error: String,
        suggested_action: String,
    },
    /// 会话统计
    SessionStats {
        duration_seconds: u64,
        chunks_processed: u64,
        average_latency_ms: f64,
        success_rate: f64,
    },
}

/// 转录会话状态
#[derive(Debug, Clone, Serialize)]
#[serde(default)]
pub struct TranscriptionSession {
    pub session_id: String,
    #[serde(skip)]
    pub started_at: Instant,
    pub chunks_processed: u64,
    pub total_transcribed_text: String,
    pub average_confidence: f64,
    pub processing_times: Vec<Duration>,
    pub error_count: u64,
}

impl Default for TranscriptionSession {
    fn default() -> Self {
        Self {
            session_id: "default".to_string(),
            started_at: Instant::now(),
            chunks_processed: 0,
            total_transcribed_text: String::new(),
            average_confidence: 0.0,
            processing_times: Vec::new(),
            error_count: 0,
        }
    }
}

/// 流式转录协调器
pub struct StreamingTranscriptionCoordinator {
    // 核心组件
    app_handle: AppHandle,
    audio_streamer: Arc<Mutex<RealtimeAudioStreamer>>,
    transcription_service: Arc<TranscriptionService>,

    // 会话管理
    current_session: Arc<Mutex<Option<TranscriptionSession>>>,
    is_active: Arc<AtomicBool>,

    // 事件通道
    ui_event_sender: Arc<Mutex<Option<UnboundedSender<UIEvent>>>>,

    // 性能监控
    performance_monitor: Arc<Mutex<PerformanceMonitor>>,

    // 配置
    config: TranscriptionConfig,
}

/// 性能监控器
#[derive(Debug)]
struct PerformanceMonitor {
    latency_samples: Vec<u64>,
    quality_samples: Vec<f64>,
    error_count: u64,
    warning_count: u64,
    last_performance_check: Instant,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            latency_samples: Vec::with_capacity(100),
            quality_samples: Vec::with_capacity(100),
            error_count: 0,
            warning_count: 0,
            last_performance_check: Instant::now(),
        }
    }

    fn record_latency(&mut self, latency_ms: u64) {
        self.latency_samples.push(latency_ms);
        if self.latency_samples.len() > 100 {
            self.latency_samples.remove(0);
        }
    }

    fn record_quality(&mut self, quality_score: f64) {
        self.quality_samples.push(quality_score);
        if self.quality_samples.len() > 100 {
            self.quality_samples.remove(0);
        }
    }

    fn get_average_latency(&self) -> f64 {
        if self.latency_samples.is_empty() {
            return 0.0;
        }
        self.latency_samples.iter().sum::<u64>() as f64 / self.latency_samples.len() as f64
    }

    fn get_average_quality(&self) -> f64 {
        if self.quality_samples.is_empty() {
            return 1.0;
        }
        self.quality_samples.iter().sum::<f64>() / self.quality_samples.len() as f64
    }

    fn should_warn_latency(&self) -> bool {
        self.get_average_latency() > 2000.0 // 超过2秒警告
    }

    fn should_warn_quality(&self) -> bool {
        self.get_average_quality() < 0.5 // 质量分数低于0.5警告
    }
}

impl StreamingTranscriptionCoordinator {
    /// 创建新的流式转录协调器
    pub fn new(
        app_handle: AppHandle,
        transcription_service: Arc<TranscriptionService>,
        config: TranscriptionConfig,
    ) -> AppResult<Self> {
        let audio_streamer = Arc::new(Mutex::new(RealtimeAudioStreamer::new(
            transcription_service.clone(),
            config.clone(),
        )?));

        Ok(Self {
            app_handle,
            audio_streamer,
            transcription_service,
            current_session: Arc::new(Mutex::new(None)),
            is_active: Arc::new(AtomicBool::new(false)),
            ui_event_sender: Arc::new(Mutex::new(None)),
            performance_monitor: Arc::new(Mutex::new(PerformanceMonitor::new())),
            config,
        })
    }

    /// 开始流式转录会话
    pub async fn start_streaming_session(&self) -> AppResult<UnboundedReceiver<UIEvent>> {
        if self.is_active.load(Ordering::Relaxed) {
            return Err(AppError::AudioRecordingError(
                "流式转录会话已在进行中".to_string(),
            ));
        }

        // 创建UI事件通道
        let (ui_tx, ui_rx) = tokio::sync::mpsc::unbounded_channel();
        *self.ui_event_sender.lock().await = Some(ui_tx.clone());

        // 创建新的转录会话
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = TranscriptionSession {
            session_id: session_id.clone(),
            started_at: Instant::now(),
            chunks_processed: 0,
            total_transcribed_text: String::new(),
            average_confidence: 0.0,
            processing_times: Vec::new(),
            error_count: 0,
        };
        *self.current_session.lock().await = Some(session);

        // 启动音频流处理
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let audio_event_rx = {
            let mut streamer = self.audio_streamer.lock().await;
            streamer.start_streaming(rx).await?
        };

        self.is_active.store(true, Ordering::Relaxed);

        // 发送录音状态变化事件
        let _ = ui_tx.send(UIEvent::RecordingStatusChanged {
            is_recording: true,
            session_id: Some(session_id),
        });

        // 启动事件处理循环
        self.start_event_processing_loop(audio_event_rx, ui_tx.clone())
            .await?;

        // 启动性能监控循环
        self.start_performance_monitoring_loop(ui_tx.clone())
            .await?;

        // 启动统计报告循环
        self.start_stats_reporting_loop(ui_tx.clone()).await?;

        println!(
            "🎙️ 流式转录协调器已启动，会话ID: {}",
            self.current_session
                .lock()
                .await
                .as_ref()
                .unwrap()
                .session_id
        );

        Ok(ui_rx)
    }

    /// 停止流式转录会话
    pub async fn stop_streaming_session(&self) -> AppResult<TranscriptionSession> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(AppError::AudioRecordingError(
                "没有活动的流式转录会话".to_string(),
            ));
        }

        self.is_active.store(false, Ordering::Relaxed);

        // 停止音频流处理
        {
            let mut streamer = self.audio_streamer.lock().await;
            streamer.stop_streaming().await?;
        }

        // 发送录音状态变化事件
        if let Some(sender) = self.ui_event_sender.lock().await.as_ref() {
            let _ = sender.send(UIEvent::RecordingStatusChanged {
                is_recording: false,
                session_id: None,
            });
        }

        // 获取会话统计
        let session = self
            .current_session
            .lock()
            .await
            .take()
            .ok_or_else(|| AppError::AudioRecordingError("会话数据丢失".to_string()))?;

        // 发送最终统计
        if let Some(sender) = self.ui_event_sender.lock().await.as_ref() {
            let duration = session.started_at.elapsed().as_secs();
            let avg_latency = if !session.processing_times.is_empty() {
                session
                    .processing_times
                    .iter()
                    .sum::<Duration>()
                    .as_millis() as f64
                    / session.processing_times.len() as f64
            } else {
                0.0
            };
            let success_rate = if session.chunks_processed > 0 {
                1.0 - (session.error_count as f64 / session.chunks_processed as f64)
            } else {
                1.0
            };

            let _ = sender.send(UIEvent::SessionStats {
                duration_seconds: duration,
                chunks_processed: session.chunks_processed,
                average_latency_ms: avg_latency,
                success_rate,
            });
        }

        // 清理事件发送器
        *self.ui_event_sender.lock().await = None;

        println!("🛑 流式转录协调器已停止");

        Ok(session)
    }

    /// 启动事件处理循环
    async fn start_event_processing_loop(
        &self,
        mut audio_event_rx: UnboundedReceiver<RealtimeEvent>,
        ui_sender: UnboundedSender<UIEvent>,
    ) -> AppResult<()> {
        let is_active = self.is_active.clone();
        let current_session = self.current_session.clone();
        let performance_monitor = self.performance_monitor.clone();
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            while is_active.load(Ordering::Relaxed) {
                match audio_event_rx.recv().await {
                    Some(event) => {
                        match event {
                            RealtimeEvent::PartialTranscription {
                                text,
                                chunk_id,
                                confidence,
                                timestamp,
                            } => {
                                let _ = ui_sender.send(UIEvent::TranscriptionUpdate {
                                    text,
                                    confidence,
                                    is_final: false,
                                    chunk_id,
                                    timestamp: timestamp.elapsed().as_millis() as u64,
                                });
                            }

                            RealtimeEvent::FinalTranscription {
                                text,
                                chunk_id,
                                confidence,
                                duration,
                            } => {
                                // 更新会话统计
                                if let Some(ref mut session) = current_session.lock().await.as_mut()
                                {
                                    session.chunks_processed += 1;
                                    session.total_transcribed_text.push_str(&text);
                                    session.total_transcribed_text.push(' ');
                                    session.processing_times.push(duration);

                                    // 更新平均置信度
                                    let total_confidence = session.average_confidence
                                        * (session.chunks_processed - 1) as f64
                                        + confidence;
                                    session.average_confidence =
                                        total_confidence / session.chunks_processed as f64;
                                }

                                // 记录性能指标
                                performance_monitor
                                    .lock()
                                    .await
                                    .record_latency(duration.as_millis() as u64);

                                let _ = ui_sender.send(UIEvent::TranscriptionUpdate {
                                    text: text.clone(),
                                    confidence,
                                    is_final: true,
                                    chunk_id,
                                    timestamp: duration.as_millis() as u64,
                                });

                                // 发送到前端的全局事件（兼容现有系统）
                                let _ = app_handle.emit_all(
                                    "transcription_result",
                                    serde_json::json!({
                                        "text": text,
                                        "confidence": confidence,
                                        "chunk_id": chunk_id,
                                        "is_streaming": true
                                    }),
                                );
                            }

                            RealtimeEvent::AudioQuality {
                                volume_db,
                                snr_db,
                                noise_level_db,
                                clarity_score,
                                recommendations,
                            } => {
                                // 记录质量指标
                                performance_monitor
                                    .lock()
                                    .await
                                    .record_quality(clarity_score);

                                let recommendation_strings: Vec<String> =
                                    recommendations.iter().map(|r| format!("{:?}", r)).collect();

                                let _ = ui_sender.send(UIEvent::AudioQualityUpdate {
                                    volume_db,
                                    snr_db,
                                    noise_level_db,
                                    clarity_score,
                                    recommendations: recommendation_strings,
                                });
                            }

                            RealtimeEvent::BufferStatus {
                                used_samples: _,
                                capacity_samples: _,
                                usage_percent,
                            } => {
                                let _ = ui_sender.send(UIEvent::BufferStatusUpdate {
                                    used_percent: usage_percent,
                                    latency_ms: performance_monitor
                                        .lock()
                                        .await
                                        .get_average_latency()
                                        as u64,
                                });
                            }

                            RealtimeEvent::AudioDeviceError { error } => {
                                let _ = ui_sender.send(UIEvent::DeviceError {
                                    error: error.clone(),
                                    suggested_action: "请检查麦克风连接并重新启动录音".to_string(),
                                });
                            }

                            RealtimeEvent::TranscriptionError { error, chunk_id } => {
                                eprintln!("转录错误 #{}: {}", chunk_id, error);

                                // 更新错误计数
                                if let Some(ref mut session) = current_session.lock().await.as_mut()
                                {
                                    session.error_count += 1;
                                }

                                let _ = ui_sender.send(UIEvent::PerformanceWarning {
                                    message: format!("转录块 #{} 失败: {}", chunk_id, error),
                                    severity: "medium".to_string(),
                                });
                            }

                            _ => {
                                // 其他事件类型的处理
                            }
                        }
                    }
                    None => {
                        println!("音频事件流已关闭");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// 启动性能监控循环
    async fn start_performance_monitoring_loop(
        &self,
        ui_sender: UnboundedSender<UIEvent>,
    ) -> AppResult<()> {
        let is_active = self.is_active.clone();
        let performance_monitor = self.performance_monitor.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5)); // 每5秒检查一次

            while is_active.load(Ordering::Relaxed) {
                interval.tick().await;

                let monitor = performance_monitor.lock().await;

                // 检查延迟警告
                if monitor.should_warn_latency() {
                    let _ = ui_sender.send(UIEvent::PerformanceWarning {
                        message: format!("转录延迟过高: {:.1}ms", monitor.get_average_latency()),
                        severity: "high".to_string(),
                    });
                }

                // 检查质量警告
                if monitor.should_warn_quality() {
                    let _ = ui_sender.send(UIEvent::PerformanceWarning {
                        message: format!("音频质量较低: {:.2}", monitor.get_average_quality()),
                        severity: "medium".to_string(),
                    });
                }
            }
        });

        Ok(())
    }

    /// 启动统计报告循环
    async fn start_stats_reporting_loop(
        &self,
        ui_sender: UnboundedSender<UIEvent>,
    ) -> AppResult<()> {
        let is_active = self.is_active.clone();
        let current_session = self.current_session.clone();
        let performance_monitor = self.performance_monitor.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // 每30秒报告一次

            while is_active.load(Ordering::Relaxed) {
                interval.tick().await;

                if let Some(ref session) = current_session.lock().await.as_ref() {
                    let duration = session.started_at.elapsed().as_secs();
                    let avg_latency = performance_monitor.lock().await.get_average_latency();
                    let success_rate = if session.chunks_processed > 0 {
                        1.0 - (session.error_count as f64 / session.chunks_processed as f64)
                    } else {
                        1.0
                    };

                    let _ = ui_sender.send(UIEvent::SessionStats {
                        duration_seconds: duration,
                        chunks_processed: session.chunks_processed,
                        average_latency_ms: avg_latency,
                        success_rate,
                    });
                }
            }
        });

        Ok(())
    }

    /// 获取当前会话状态
    pub async fn get_session_status(&self) -> Option<TranscriptionSession> {
        self.current_session.lock().await.clone()
    }

    /// 获取性能报告
    pub async fn get_performance_report(&self) -> serde_json::Value {
        let monitor = self.performance_monitor.lock().await;
        serde_json::json!({
            "average_latency_ms": monitor.get_average_latency(),
            "average_quality": monitor.get_average_quality(),
            "error_count": monitor.error_count,
            "warning_count": monitor.warning_count,
            "is_active": self.is_active.load(Ordering::Relaxed)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcription::MockTranscriptionService;
    use std::time::Duration;
    use tauri::test::{mock_app, MockRuntime};
    use tokio::time::timeout;

    /// 创建测试用的 AppHandle
    fn create_test_app() -> AppHandle<MockRuntime> {
        mock_app().handle()
    }

    /// 创建测试用的转录服务
    fn create_test_transcription_service() -> Arc<TranscriptionService> {
        Arc::new(TranscriptionService::new_mock())
    }

    /// 创建测试用的转录配置
    fn create_test_config() -> TranscriptionConfig {
        TranscriptionConfig {
            language: "zh-CN".to_string(),
            model: "whisper-1".to_string(),
            temperature: 0.0,
            response_format: "json".to_string(),
            enable_voice_activity_detection: true,
            chunk_length_ms: 2000,
            overlap_length_ms: 200,
            silence_threshold: -40.0,
            min_speech_duration_ms: 500,
            max_speech_duration_ms: 30000,
            enable_noise_reduction: true,
            enable_echo_cancellation: true,
            enable_auto_gain_control: true,
            sample_rate: 16000,
            channels: 1,
            bit_depth: 16,
        }
    }

    #[tokio::test]
    async fn test_coordinator_creation() {
        let app = create_test_app();
        let service = create_test_transcription_service();
        let config = create_test_config();

        let coordinator = StreamingTranscriptionCoordinator::new(app, service, config);
        assert!(coordinator.is_ok());

        let coordinator = coordinator.unwrap();
        assert!(!coordinator.is_active.load(Ordering::Relaxed));
        assert!(coordinator.current_session.lock().await.is_none());
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let app = create_test_app();
        let service = create_test_transcription_service();
        let config = create_test_config();
        let coordinator = StreamingTranscriptionCoordinator::new(app, service, config).unwrap();

        // 测试启动会话
        let ui_rx = coordinator.start_streaming_session().await;
        assert!(ui_rx.is_ok());
        assert!(coordinator.is_active.load(Ordering::Relaxed));

        // 验证会话已创建
        let session = coordinator.get_session_status().await;
        assert!(session.is_some());
        let session = session.unwrap();
        assert!(!session.session_id.is_empty());
        assert_eq!(session.chunks_processed, 0);

        // 测试停止会话
        let final_session = coordinator.stop_streaming_session().await;
        assert!(final_session.is_ok());
        assert!(!coordinator.is_active.load(Ordering::Relaxed));

        let final_session = final_session.unwrap();
        assert_eq!(final_session.session_id, session.session_id);
    }

    #[tokio::test]
    async fn test_duplicate_session_start() {
        let app = create_test_app();
        let service = create_test_transcription_service();
        let config = create_test_config();
        let coordinator = StreamingTranscriptionCoordinator::new(app, service, config).unwrap();

        // 启动第一个会话
        let _ui_rx1 = coordinator.start_streaming_session().await.unwrap();

        // 尝试启动第二个会话应该失败
        let result = coordinator.start_streaming_session().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已在进行中"));
    }

    #[tokio::test]
    async fn test_stop_without_active_session() {
        let app = create_test_app();
        let service = create_test_transcription_service();
        let config = create_test_config();
        let coordinator = StreamingTranscriptionCoordinator::new(app, service, config).unwrap();

        // 尝试停止不存在的会话应该失败
        let result = coordinator.stop_streaming_session().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("没有活动"));
    }

    #[tokio::test]
    async fn test_ui_event_reception() {
        let app = create_test_app();
        let service = create_test_transcription_service();
        let config = create_test_config();
        let coordinator = StreamingTranscriptionCoordinator::new(app, service, config).unwrap();

        let mut ui_rx = coordinator.start_streaming_session().await.unwrap();

        // 应该接收到录音状态变化事件
        let event = timeout(Duration::from_millis(100), ui_rx.recv()).await;
        assert!(event.is_ok());
        let event = event.unwrap();
        assert!(event.is_some());

        if let UIEvent::RecordingStatusChanged {
            is_recording,
            session_id,
        } = event.unwrap()
        {
            assert!(is_recording);
            assert!(session_id.is_some());
        } else {
            panic!("Expected RecordingStatusChanged event");
        }

        let _ = coordinator.stop_streaming_session().await;
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();

        // 测试延迟记录
        monitor.record_latency(1000);
        monitor.record_latency(2000);
        monitor.record_latency(3000);

        assert_eq!(monitor.get_average_latency(), 2000.0);
        assert!(!monitor.should_warn_latency()); // 2000ms < 2000ms 阈值

        // 测试质量记录
        monitor.record_quality(0.8);
        monitor.record_quality(0.9);
        monitor.record_quality(0.7);

        assert_eq!(monitor.get_average_quality(), 0.8);
        assert!(!monitor.should_warn_quality()); // 0.8 > 0.5 阈值

        // 测试警告阈值
        monitor.record_latency(5000); // 超过阈值
        assert!(monitor.should_warn_latency());

        monitor.record_quality(0.3); // 低于阈值
        assert!(monitor.should_warn_quality());
    }

    #[tokio::test]
    async fn test_performance_monitor_capacity_limit() {
        let mut monitor = PerformanceMonitor::new();

        // 添加超过容量限制的样本
        for i in 0..150 {
            monitor.record_latency(i);
        }

        // 应该只保留最后100个样本
        assert_eq!(monitor.latency_samples.len(), 100);
        assert_eq!(monitor.latency_samples[0], 50); // 前50个被移除
        assert_eq!(monitor.latency_samples[99], 149);
    }

    #[tokio::test]
    async fn test_session_statistics() {
        let app = create_test_app();
        let service = create_test_transcription_service();
        let config = create_test_config();
        let coordinator = StreamingTranscriptionCoordinator::new(app, service, config).unwrap();

        let _ui_rx = coordinator.start_streaming_session().await.unwrap();

        // 模拟一些处理
        {
            let mut session = coordinator.current_session.lock().await;
            if let Some(ref mut s) = session.as_mut() {
                s.chunks_processed = 10;
                s.error_count = 2;
                s.processing_times.push(Duration::from_millis(100));
                s.processing_times.push(Duration::from_millis(200));
                s.average_confidence = 0.85;
            }
        }

        let session = coordinator.get_session_status().await.unwrap();
        assert_eq!(session.chunks_processed, 10);
        assert_eq!(session.error_count, 2);
        assert_eq!(session.processing_times.len(), 2);
        assert_eq!(session.average_confidence, 0.85);

        let final_session = coordinator.stop_streaming_session().await.unwrap();
        assert_eq!(final_session.chunks_processed, 10);
        assert_eq!(final_session.error_count, 2);
    }

    #[tokio::test]
    async fn test_performance_report() {
        let app = create_test_app();
        let service = create_test_transcription_service();
        let config = create_test_config();
        let coordinator = StreamingTranscriptionCoordinator::new(app, service, config).unwrap();

        // 添加一些性能数据
        {
            let mut monitor = coordinator.performance_monitor.lock().await;
            monitor.record_latency(1500);
            monitor.record_quality(0.75);
            monitor.error_count = 3;
            monitor.warning_count = 1;
        }

        let report = coordinator.get_performance_report().await;
        assert!(report.is_object());
        assert_eq!(report["average_latency_ms"], 1500.0);
        assert_eq!(report["average_quality"], 0.75);
        assert_eq!(report["error_count"], 3);
        assert_eq!(report["warning_count"], 1);
        assert_eq!(report["is_active"], false);
    }

    #[test]
    fn test_transcription_session_default() {
        let session = TranscriptionSession::default();
        assert_eq!(session.session_id, "default");
        assert_eq!(session.chunks_processed, 0);
        assert!(session.total_transcribed_text.is_empty());
        assert_eq!(session.average_confidence, 0.0);
        assert!(session.processing_times.is_empty());
        assert_eq!(session.error_count, 0);
    }

    #[test]
    fn test_ui_event_serialization() {
        let event = UIEvent::TranscriptionUpdate {
            text: "测试文本".to_string(),
            confidence: 0.95,
            is_final: true,
            chunk_id: 123,
            timestamp: 1000,
        };

        let serialized = serde_json::to_string(&event);
        assert!(serialized.is_ok());

        let json: serde_json::Value = serde_json::from_str(&serialized.unwrap()).unwrap();
        assert_eq!(json["type"], "TranscriptionUpdate");
        assert_eq!(json["text"], "测试文本");
        assert_eq!(json["confidence"], 0.95);
        assert_eq!(json["is_final"], true);
        assert_eq!(json["chunk_id"], 123);
        assert_eq!(json["timestamp"], 1000);
    }
}
