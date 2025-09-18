// 实时音频流处理器
// 负责协调音频捕获、分块处理和实时转录

use super::AudioRecorder;
use crate::errors::{AppError, AppResult};
use crate::transcription::TranscriptionService;
use crate::types::TranscriptionConfig;
use parking_lot::Mutex;
use ringbuf::{HeapRb, Rb};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;

// 音频块大小配置 - 优化后的参数
const CHUNK_SIZE_SECONDS: f32 = 1.5; // 基础块大小，确保转录质量
const CHUNK_OVERLAP_SECONDS: f32 = 0.3; // 重叠时间，保证连续性
const SAMPLE_RATE: u32 = 16000; // 标准采样率
const BUFFER_CAPACITY: usize = SAMPLE_RATE as usize * 15; // 15秒缓冲，支持长句处理
const MIN_CHUNK_SIZE_SAMPLES: usize = SAMPLE_RATE as usize / 2; // 最小0.5秒块
const MAX_PROCESSING_LATENCY_MS: u64 = 500; // 最大处理延迟500ms

/// 实时转录事件类型
#[derive(Debug, Clone)]
pub enum RealtimeEvent {
    /// 部分转录结果（实时更新）
    PartialTranscription {
        text: String,
        chunk_id: u64,
        confidence: f64,
        timestamp: Instant,
    },
    /// 最终转录结果
    FinalTranscription {
        text: String,
        chunk_id: u64,
        confidence: f64,
        duration: Duration,
    },
    /// 转录错误
    TranscriptionError { error: String, chunk_id: u64 },
    /// 录音状态变化
    RecordingStatusChanged { is_recording: bool },
    /// 缓冲区状态
    BufferStatus {
        used_samples: usize,
        capacity_samples: usize,
        usage_percent: u8,
    },
    /// 音频质量监控
    AudioQuality {
        volume_db: f64,
        snr_db: Option<f64>,
        noise_level_db: f64,
        clarity_score: f64,
        recommendations: Vec<super::recorder::Recommendation>,
    },
    /// 音频设备错误
    AudioDeviceError { error: String },
}

/// 实时音频流处理器
pub struct RealtimeAudioStreamer {
    // 核心组件
    audio_recorder: Arc<Mutex<AudioRecorder>>,
    transcription_service: Arc<TranscriptionService>,
    buffer_manager: Arc<LocalBufferManager>,
    chunk_processor: Arc<LocalAudioChunkProcessor>,

    // 状态管理
    is_streaming: Arc<AtomicBool>,
    chunk_counter: Arc<Mutex<u64>>,

    // 配置
    config: TranscriptionConfig,
    chunk_size_samples: usize,
    overlap_samples: usize,

    // 通信通道
    event_sender: Arc<Mutex<Option<mpsc::UnboundedSender<RealtimeEvent>>>>,

    // 性能监控
    last_chunk_time: Arc<Mutex<Option<Instant>>>,
    processing_times: Arc<Mutex<Vec<Duration>>>,
}

impl RealtimeAudioStreamer {
    /// 创建新的实时音频流处理器
    pub fn new(
        transcription_service: Arc<TranscriptionService>,
        config: TranscriptionConfig,
    ) -> AppResult<Self> {
        // 创建录音器配置
        let recording_config = crate::types::RecordingConfig {
            sample_rate: SAMPLE_RATE,
            channels: 1,
            device_id: None,
            duration_seconds: None,
            buffer_duration: Some(3.0),
        };

        let audio_recorder = Arc::new(Mutex::new(AudioRecorder::new(recording_config)));
        let buffer_manager = Arc::new(LocalBufferManager::new(BUFFER_CAPACITY)?);
        let chunk_processor = Arc::new(LocalAudioChunkProcessor::new(SAMPLE_RATE)?);

        let chunk_size_samples = (CHUNK_SIZE_SECONDS * SAMPLE_RATE as f32) as usize;
        let overlap_samples = (CHUNK_OVERLAP_SECONDS * SAMPLE_RATE as f32) as usize;

        Ok(Self {
            audio_recorder,
            transcription_service,
            buffer_manager,
            chunk_processor,
            is_streaming: Arc::new(AtomicBool::new(false)),
            chunk_counter: Arc::new(Mutex::new(0)),
            config,
            chunk_size_samples,
            overlap_samples,
            event_sender: Arc::new(Mutex::new(None)),
            last_chunk_time: Arc::new(Mutex::new(None)),
            processing_times: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// 开始实时音频流处理
    pub async fn start_streaming(
        &mut self,
        _event_receiver: mpsc::UnboundedReceiver<RealtimeEvent>,
    ) -> AppResult<mpsc::UnboundedReceiver<RealtimeEvent>> {
        if self.is_streaming.load(Ordering::Relaxed) {
            return Err(AppError::AudioRecordingError(
                "已经在进行实时流处理".to_string(),
            ));
        }

        // 创建事件通道
        let (event_tx, event_rx) = mpsc::unbounded_channel::<RealtimeEvent>();
        *self.event_sender.lock() = Some(event_tx.clone());

        // 启动音频录制
        self.audio_recorder.lock().start_recording()?;
        self.is_streaming.store(true, Ordering::Relaxed);

        // 发送录音状态变化事件
        let _ = event_tx.send(RealtimeEvent::RecordingStatusChanged { is_recording: true });

        // 启动音频处理循环
        self.start_audio_processing_loop(event_tx.clone()).await?;

        // 启动转录处理循环
        self.start_transcription_processing_loop(event_tx.clone())
            .await?;

        // 启动状态监控循环
        self.start_monitoring_loop(event_tx).await?;

        println!("🎙️ 实时音频流处理已启动");
        Ok(event_rx)
    }

    /// 停止实时音频流处理
    pub async fn stop_streaming(&mut self) -> AppResult<()> {
        if !self.is_streaming.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.is_streaming.store(false, Ordering::Relaxed);

        // 停止录音
        let final_audio = self.audio_recorder.lock().stop_recording()?;

        // 处理最后的音频数据
        if !final_audio.is_empty() {
            self.buffer_manager.write_samples(&final_audio)?;
        }

        // 发送录音状态变化事件
        if let Some(sender) = self.event_sender.lock().as_ref() {
            let _ = sender.send(RealtimeEvent::RecordingStatusChanged {
                is_recording: false,
            });
        }

        // 清理事件发送器
        *self.event_sender.lock() = None;

        println!("🛑 实时音频流处理已停止");
        Ok(())
    }

    /// 检查是否正在流处理
    pub fn is_streaming(&self) -> bool {
        self.is_streaming.load(Ordering::Relaxed)
    }

    /// 获取处理统计信息
    pub fn get_processing_stats(&self) -> ProcessingStats {
        let processing_times = self.processing_times.lock();
        let avg_processing_time = if processing_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = processing_times.iter().sum();
            total / processing_times.len() as u32
        };

        ProcessingStats {
            total_chunks_processed: *self.chunk_counter.lock(),
            average_processing_time: avg_processing_time,
            buffer_utilization: self.buffer_manager.utilization(),
            is_streaming: self.is_streaming(),
        }
    }

    /// 优化后的音频处理循环 - 智能分块和质量监控
    async fn start_audio_processing_loop(
        &self,
        event_sender: mpsc::UnboundedSender<RealtimeEvent>,
    ) -> AppResult<()> {
        let is_streaming = self.is_streaming.clone();
        let audio_recorder = self.audio_recorder.clone();
        let buffer_manager = self.buffer_manager.clone();
        let last_chunk_time = self.last_chunk_time.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(50)); // 50ms检查一次，更高频率
            let mut last_quality_check = Instant::now();
            let mut consecutive_empty_reads = 0;

            while is_streaming.load(Ordering::Relaxed) {
                interval.tick().await;

                // 从录音器获取最新音频数据
                if let Some(recorder) = audio_recorder.try_lock() {
                    if recorder.is_recording() {
                        // 使用新的智能数据获取方法
                        let new_audio_data = recorder.get_latest_audio_data();

                        if !new_audio_data.is_empty() {
                            consecutive_empty_reads = 0;

                            // 写入缓冲区
                            if let Err(e) = buffer_manager.write_samples(&new_audio_data) {
                                eprintln!("写入音频缓冲区失败: {}", e);
                                continue;
                            }

                            // 定期进行音频质量分析（每500ms一次）
                            if last_quality_check.elapsed() > Duration::from_millis(500) {
                                let quality_metrics =
                                    recorder.analyze_audio_quality(&new_audio_data);

                                // 发送音频质量事件
                                let _ = event_sender.send(RealtimeEvent::AudioQuality {
                                    volume_db: quality_metrics.volume_db,
                                    snr_db: quality_metrics.snr_db,
                                    noise_level_db: quality_metrics.noise_level_db,
                                    clarity_score: quality_metrics.clarity_score,
                                    recommendations: quality_metrics.recommended_actions,
                                });

                                last_quality_check = Instant::now();
                            }

                            // 更新最后处理时间
                            *last_chunk_time.lock() = Some(Instant::now());

                            // 发送缓冲区状态更新
                            let (used, capacity) = recorder.get_buffer_status();
                            let _ = event_sender.send(RealtimeEvent::BufferStatus {
                                used_samples: used,
                                capacity_samples: capacity,
                                usage_percent: (used as f32 / capacity as f32 * 100.0) as u8,
                            });
                        } else {
                            consecutive_empty_reads += 1;

                            // 如果连续太多次没有读到数据，可能是音频设备问题
                            if consecutive_empty_reads > 200 {
                                // 10秒没有数据
                                eprintln!(
                                    "⚠️ 警告：连续{}次音频读取为空，可能是设备问题",
                                    consecutive_empty_reads
                                );
                                let _ = event_sender.send(RealtimeEvent::AudioDeviceError {
                                    error: "音频设备可能断开连接".to_string(),
                                });
                                consecutive_empty_reads = 0; // 重置计数器
                            }
                        }
                    }
                }
            }

            println!("🔄 音频处理循环已停止");
        });

        Ok(())
    }

    /// 优化后的转录处理循环 - 智能分块和流式处理
    async fn start_transcription_processing_loop(
        &self,
        event_sender: mpsc::UnboundedSender<RealtimeEvent>,
    ) -> AppResult<()> {
        let is_streaming = self.is_streaming.clone();
        let buffer_manager = self.buffer_manager.clone();
        let chunk_processor = self.chunk_processor.clone();
        let transcription_service = self.transcription_service.clone();
        let chunk_counter = self.chunk_counter.clone();
        let config = self.config.clone();
        let chunk_size_samples = self.chunk_size_samples;
        let overlap_samples = self.overlap_samples;
        let processing_times = self.processing_times.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(750)); // 750ms检查一次，更积极的转录
            let mut last_chunk_overlap: Option<Vec<f32>> = None;

            while is_streaming.load(Ordering::Relaxed) {
                interval.tick().await;

                // 智能检查：有足够数据或者超过最大等待时间
                let available = buffer_manager.available_samples();
                let should_process = available >= chunk_size_samples
                    || (available >= MIN_CHUNK_SIZE_SAMPLES
                        && interval.period().as_millis() > MAX_PROCESSING_LATENCY_MS as u128);

                if should_process {
                    let start_time = Instant::now();

                    // 动态块大小：根据可用数据调整
                    let actual_chunk_size = if available >= chunk_size_samples {
                        chunk_size_samples
                    } else {
                        available.max(MIN_CHUNK_SIZE_SAMPLES)
                    };

                    // 读取音频块（带重叠处理）
                    match buffer_manager.read_chunk(actual_chunk_size) {
                        Ok(mut audio_chunk) => {
                            let chunk_id = {
                                let mut counter = chunk_counter.lock();
                                *counter += 1;
                                *counter
                            };

                            // 应用重叠处理，确保转录连续性
                            if let Some(ref overlap) = last_chunk_overlap {
                                if overlap.len() >= overlap_samples {
                                    // 在音频块前面添加重叠部分
                                    let overlap_start = overlap.len() - overlap_samples;
                                    let mut overlapped_chunk =
                                        Vec::with_capacity(overlap_samples + audio_chunk.len());
                                    overlapped_chunk.extend_from_slice(&overlap[overlap_start..]);
                                    overlapped_chunk.extend_from_slice(&audio_chunk);
                                    audio_chunk = overlapped_chunk;
                                }
                            }

                            // 保存当前块的末尾作为下次的重叠
                            if audio_chunk.len() >= overlap_samples {
                                last_chunk_overlap = Some(audio_chunk.clone());
                            }

                            println!(
                                "🎵 处理音频块 #{} ({} 样本, {:.2}秒)",
                                chunk_id,
                                audio_chunk.len(),
                                audio_chunk.len() as f32 / SAMPLE_RATE as f32
                            );

                            // 异步处理音频块
                            let chunk_processor_clone = chunk_processor.clone();
                            let transcription_service_clone = transcription_service.clone();
                            let config_clone = config.clone();
                            let event_sender_clone = event_sender.clone();
                            let processing_times_clone = processing_times.clone();

                            tokio::spawn(async move {
                                // 优化的音频块处理和转录流水线
                                match chunk_processor_clone.process_chunk(&audio_chunk).await {
                                    Ok(processed_audio) => {
                                        // 立即发送部分转录事件（占位符）
                                        let _ = event_sender_clone.send(
                                            RealtimeEvent::PartialTranscription {
                                                text: "正在转录...".to_string(),
                                                chunk_id,
                                                confidence: 0.0,
                                                timestamp: Instant::now(),
                                            },
                                        );

                                        // 流式转录处理
                                        match chunk_processor_clone
                                            .save_chunk_to_file(&processed_audio)
                                            .await
                                        {
                                            Ok(temp_file_path) => {
                                                let transcription_start = Instant::now();

                                                // 高效转录：直接调用转录服务
                                                match transcription_service_clone
                                                    .transcribe_audio(
                                                        &temp_file_path,
                                                        &config_clone,
                                                    )
                                                    .await
                                                {
                                                    Ok(result) => {
                                                        let total_processing_time =
                                                            start_time.elapsed();
                                                        let transcription_time =
                                                            transcription_start.elapsed();

                                                        // 记录性能指标
                                                        processing_times_clone
                                                            .lock()
                                                            .push(total_processing_time);

                                                        println!("✅ 转录完成 #{}: '{}' (总耗时: {:.2}s, 转录耗时: {:.2}s)",
                                                            chunk_id,
                                                            result.text.chars().take(50).collect::<String>(),
                                                            total_processing_time.as_secs_f64(),
                                                            transcription_time.as_secs_f64()
                                                        );

                                                        // 发送最终转录结果
                                                        let _ = event_sender_clone.send(
                                                            RealtimeEvent::FinalTranscription {
                                                                text: result.text,
                                                                chunk_id,
                                                                confidence: result
                                                                    .confidence
                                                                    .unwrap_or(0.9),
                                                                duration: total_processing_time,
                                                            },
                                                        );

                                                        // 性能警告检查
                                                        if total_processing_time.as_millis() > 2000
                                                        {
                                                            eprintln!(
                                                                "⚠️ 块 #{} 处理时间过长: {:.2}s",
                                                                chunk_id,
                                                                total_processing_time.as_secs_f64()
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "❌ 转录失败 #{}: {}",
                                                            chunk_id, e
                                                        );
                                                        let _ = event_sender_clone.send(
                                                            RealtimeEvent::TranscriptionError {
                                                                error: format!("转录失败: {}", e),
                                                                chunk_id,
                                                            },
                                                        );
                                                    }
                                                }

                                                // 异步清理临时文件
                                                tokio::spawn(async move {
                                                    if let Err(e) =
                                                        std::fs::remove_file(temp_file_path)
                                                    {
                                                        eprintln!("清理临时文件失败: {}", e);
                                                    }
                                                });
                                            }
                                            Err(e) => {
                                                eprintln!("❌ 保存音频块失败 #{}: {}", chunk_id, e);
                                                let _ = event_sender_clone.send(
                                                    RealtimeEvent::TranscriptionError {
                                                        error: format!("保存音频块失败: {}", e),
                                                        chunk_id,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("❌ 音频块处理失败 #{}: {}", chunk_id, e);
                                        let _ = event_sender_clone.send(
                                            RealtimeEvent::TranscriptionError {
                                                error: format!("处理音频块失败: {}", e),
                                                chunk_id,
                                            },
                                        );
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("读取音频块失败: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// 启动监控循环
    async fn start_monitoring_loop(
        &self,
        event_sender: mpsc::UnboundedSender<RealtimeEvent>,
    ) -> AppResult<()> {
        let is_streaming = self.is_streaming.clone();
        let buffer_manager = self.buffer_manager.clone();
        let chunk_counter = self.chunk_counter.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5)); // 每5秒报告一次状态

            while is_streaming.load(Ordering::Relaxed) {
                interval.tick().await;

                // 发送缓冲区状态
                let used = buffer_manager.used_samples();
                let capacity = buffer_manager.capacity();
                let usage_percent = (used as f32 / capacity as f32 * 100.0) as u8;
                let _ = event_sender.send(RealtimeEvent::BufferStatus {
                    used_samples: used,
                    capacity_samples: capacity,
                    usage_percent,
                });
            }
        });

        Ok(())
    }
}

/// 处理统计信息
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub total_chunks_processed: u64,
    pub average_processing_time: Duration,
    pub buffer_utilization: f64,
    pub is_streaming: bool,
}

/// 缓冲区管理器
pub struct LocalBufferManager {
    ring_buffer: Mutex<HeapRb<f32>>,
    capacity: usize,
}

impl LocalBufferManager {
    pub fn new(capacity: usize) -> AppResult<Self> {
        let ring_buffer = HeapRb::<f32>::new(capacity);
        Ok(Self {
            ring_buffer: Mutex::new(ring_buffer),
            capacity,
        })
    }

    pub fn write_samples(&self, samples: &[f32]) -> AppResult<()> {
        let mut buffer = self.ring_buffer.lock();

        if samples.len() > buffer.free_len() {
            return Err(AppError::AudioProcessingError("缓冲区空间不足".to_string()));
        }

        for &sample in samples {
            if buffer.push(sample).is_err() {
                return Err(AppError::AudioProcessingError("写入缓冲区失败".to_string()));
            }
        }

        Ok(())
    }

    pub fn read_chunk(&self, size: usize) -> AppResult<Vec<f32>> {
        let mut buffer = self.ring_buffer.lock();

        if buffer.len() < size {
            return Err(AppError::AudioProcessingError("缓冲区数据不足".to_string()));
        }

        let mut chunk = Vec::with_capacity(size);
        for _ in 0..size {
            if let Some(sample) = buffer.pop() {
                chunk.push(sample);
            } else {
                return Err(AppError::AudioProcessingError("读取缓冲区失败".to_string()));
            }
        }

        Ok(chunk)
    }

    pub fn available_samples(&self) -> usize {
        self.ring_buffer.lock().len()
    }

    pub fn used_samples(&self) -> usize {
        self.ring_buffer.lock().len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn utilization(&self) -> f64 {
        self.used_samples() as f64 / self.capacity as f64
    }

    pub fn clear(&self) {
        self.ring_buffer.lock().clear();
    }
}

/// 音频块处理器
pub struct LocalAudioChunkProcessor {
    sample_rate: u32,
    temp_dir: std::path::PathBuf,
}

impl LocalAudioChunkProcessor {
    pub fn new(sample_rate: u32) -> AppResult<Self> {
        let temp_dir = std::env::temp_dir().join("recording_king_chunks");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::FileSystemError(format!("创建临时目录失败: {}", e)))?;

        Ok(Self {
            sample_rate,
            temp_dir,
        })
    }

    /// 处理音频块（格式转换、降噪等）
    pub async fn process_chunk(&self, audio_data: &[f32]) -> AppResult<Vec<f32>> {
        // 这里可以添加音频预处理逻辑
        // - 降噪
        // - 增益调整
        // - 格式标准化

        // 目前简单返回原数据
        Ok(audio_data.to_vec())
    }

    /// 将音频块保存为临时WAV文件
    pub async fn save_chunk_to_file(&self, audio_data: &[f32]) -> AppResult<std::path::PathBuf> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let file_path = self.temp_dir.join(format!("chunk_{}.wav", timestamp));

        // 创建WAV文件
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(&file_path, spec)
            .map_err(|e| AppError::AudioProcessingError(format!("创建WAV文件失败: {}", e)))?;

        // 将f32转换为i16并写入
        for &sample in audio_data {
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| AppError::AudioProcessingError(format!("写入WAV数据失败: {}", e)))?;
        }

        writer
            .finalize()
            .map_err(|e| AppError::AudioProcessingError(format!("完成WAV文件失败: {}", e)))?;

        Ok(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcription::MockTranscriptionService;
    use std::time::Duration;
    use tokio::time::timeout;

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

    /// 创建测试用的转录服务
    fn create_test_transcription_service() -> Arc<TranscriptionService> {
        Arc::new(TranscriptionService::new_mock())
    }

    #[tokio::test]
    async fn test_realtime_streamer_creation() {
        let service = create_test_transcription_service();
        let config = create_test_config();

        let streamer = RealtimeAudioStreamer::new(service, config);
        assert!(streamer.is_ok());

        let streamer = streamer.unwrap();
        assert!(!streamer.is_streaming());
        assert_eq!(streamer.get_processing_stats().total_chunks_processed, 0);
    }

    #[tokio::test]
    async fn test_streaming_lifecycle() {
        let service = create_test_transcription_service();
        let config = create_test_config();
        let mut streamer = RealtimeAudioStreamer::new(service, config).unwrap();

        // 创建空的事件接收器用于启动
        let (_tx, rx) = mpsc::unbounded_channel();

        // 测试启动流处理
        let event_rx = streamer.start_streaming(rx).await;
        assert!(event_rx.is_ok());
        assert!(streamer.is_streaming());

        // 等待一小段时间让系统启动
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试停止流处理
        let stop_result = streamer.stop_streaming().await;
        assert!(stop_result.is_ok());
        assert!(!streamer.is_streaming());
    }

    #[tokio::test]
    async fn test_duplicate_streaming_start() {
        let service = create_test_transcription_service();
        let config = create_test_config();
        let mut streamer = RealtimeAudioStreamer::new(service, config).unwrap();

        let (_tx, rx1) = mpsc::unbounded_channel();
        let (_tx, rx2) = mpsc::unbounded_channel();

        // 启动第一个流处理会话
        let _event_rx1 = streamer.start_streaming(rx1).await.unwrap();

        // 尝试启动第二个会话应该失败
        let result = streamer.start_streaming(rx2).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已经在进行"));
    }

    #[tokio::test]
    async fn test_event_reception() {
        let service = create_test_transcription_service();
        let config = create_test_config();
        let mut streamer = RealtimeAudioStreamer::new(service, config).unwrap();

        let (_tx, rx) = mpsc::unbounded_channel();
        let mut event_rx = streamer.start_streaming(rx).await.unwrap();

        // 应该接收到录音状态变化事件
        let event = timeout(Duration::from_millis(200), event_rx.recv()).await;
        assert!(event.is_ok());
        let event = event.unwrap();
        assert!(event.is_some());

        if let RealtimeEvent::RecordingStatusChanged { is_recording } = event.unwrap() {
            assert!(is_recording);
        } else {
            panic!("Expected RecordingStatusChanged event");
        }

        let _ = streamer.stop_streaming().await;
    }

    #[tokio::test]
    async fn test_processing_stats() {
        let service = create_test_transcription_service();
        let config = create_test_config();
        let streamer = RealtimeAudioStreamer::new(service, config).unwrap();

        let initial_stats = streamer.get_processing_stats();
        assert_eq!(initial_stats.total_chunks_processed, 0);
        assert_eq!(
            initial_stats.average_processing_time,
            Duration::from_millis(0)
        );
        assert!(!initial_stats.is_streaming);

        // 模拟添加一些处理时间
        {
            let mut times = streamer.processing_times.lock();
            times.push(Duration::from_millis(100));
            times.push(Duration::from_millis(200));
        }

        let updated_stats = streamer.get_processing_stats();
        assert_eq!(
            updated_stats.average_processing_time,
            Duration::from_millis(150)
        );
    }

    #[tokio::test]
    async fn test_buffer_manager() {
        let buffer_manager = LocalBufferManager::new(1000).unwrap();

        // 测试初始状态
        assert_eq!(buffer_manager.capacity(), 1000);
        assert_eq!(buffer_manager.used_samples(), 0);
        assert_eq!(buffer_manager.available_samples(), 0);
        assert_eq!(buffer_manager.utilization(), 0.0);

        // 测试写入样本
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let write_result = buffer_manager.write_samples(&samples);
        assert!(write_result.is_ok());

        assert_eq!(buffer_manager.used_samples(), 5);
        assert_eq!(buffer_manager.available_samples(), 5);
        assert_eq!(buffer_manager.utilization(), 0.005);

        // 测试读取块
        let read_result = buffer_manager.read_chunk(3);
        assert!(read_result.is_ok());
        let chunk = read_result.unwrap();
        assert_eq!(chunk.len(), 3);
        assert_eq!(chunk[0], 0.1);
        assert_eq!(chunk[1], 0.2);
        assert_eq!(chunk[2], 0.3);

        // 检查缓冲区状态更新
        assert_eq!(buffer_manager.used_samples(), 2);
        assert_eq!(buffer_manager.available_samples(), 2);

        // 测试清空缓冲区
        buffer_manager.clear();
        assert_eq!(buffer_manager.used_samples(), 0);
        assert_eq!(buffer_manager.available_samples(), 0);
    }

    #[tokio::test]
    async fn test_buffer_manager_overflow() {
        let buffer_manager = LocalBufferManager::new(10).unwrap();

        // 尝试写入超过容量的数据
        let large_samples: Vec<f32> = (0..20).map(|i| i as f32 * 0.1).collect();
        let write_result = buffer_manager.write_samples(&large_samples);
        assert!(write_result.is_err());
        assert!(write_result
            .unwrap_err()
            .to_string()
            .contains("缓冲区空间不足"));
    }

    #[tokio::test]
    async fn test_buffer_manager_underflow() {
        let buffer_manager = LocalBufferManager::new(100).unwrap();

        // 尝试从空缓冲区读取数据
        let read_result = buffer_manager.read_chunk(10);
        assert!(read_result.is_err());
        assert!(read_result
            .unwrap_err()
            .to_string()
            .contains("缓冲区数据不足"));

        // 添加少量数据后尝试读取更多
        let samples = vec![0.1, 0.2, 0.3];
        buffer_manager.write_samples(&samples).unwrap();

        let read_result = buffer_manager.read_chunk(10);
        assert!(read_result.is_err());
    }

    #[tokio::test]
    async fn test_audio_chunk_processor_creation() {
        let processor = LocalAudioChunkProcessor::new(16000);
        assert!(processor.is_ok());

        let processor = processor.unwrap();
        assert_eq!(processor.sample_rate, 16000);
        assert!(processor.temp_dir.exists());
    }

    #[tokio::test]
    async fn test_audio_chunk_processing() {
        let processor = LocalAudioChunkProcessor::new(16000).unwrap();

        // 测试音频块处理
        let input_audio = vec![0.1, 0.2, -0.3, 0.4, -0.5];
        let processed = processor.process_chunk(&input_audio).await;
        assert!(processed.is_ok());

        let processed_audio = processed.unwrap();
        assert_eq!(processed_audio.len(), input_audio.len());
        assert_eq!(processed_audio, input_audio); // 当前实现是直接返回
    }

    #[tokio::test]
    async fn test_audio_chunk_file_saving() {
        let processor = LocalAudioChunkProcessor::new(16000).unwrap();

        // 创建测试音频数据
        let audio_data: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.001).sin()).collect(); // 0.1秒的正弦波

        // 保存为文件
        let file_result = processor.save_chunk_to_file(&audio_data).await;
        assert!(file_result.is_ok());

        let file_path = file_result.unwrap();
        assert!(file_path.exists());
        assert!(file_path.extension().unwrap() == "wav");

        // 验证文件大小（应该不为空）
        let metadata = std::fs::metadata(&file_path).unwrap();
        assert!(metadata.len() > 0);

        // 清理测试文件
        std::fs::remove_file(file_path).unwrap();
    }

    #[tokio::test]
    async fn test_empty_audio_chunk_saving() {
        let processor = LocalAudioChunkProcessor::new(16000).unwrap();

        // 测试空音频数据
        let empty_audio: Vec<f32> = vec![];
        let file_result = processor.save_chunk_to_file(&empty_audio).await;
        assert!(file_result.is_ok());

        let file_path = file_result.unwrap();
        assert!(file_path.exists());

        // 即使是空音频，WAV文件也应该有头部信息
        let metadata = std::fs::metadata(&file_path).unwrap();
        assert!(metadata.len() > 44); // WAV头部至少44字节

        // 清理测试文件
        std::fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_realtime_event_variants() {
        // 测试各种事件类型的创建和基本属性
        let partial_event = RealtimeEvent::PartialTranscription {
            text: "测试".to_string(),
            chunk_id: 1,
            confidence: 0.95,
            timestamp: Instant::now(),
        };

        if let RealtimeEvent::PartialTranscription {
            text,
            chunk_id,
            confidence,
            ..
        } = partial_event
        {
            assert_eq!(text, "测试");
            assert_eq!(chunk_id, 1);
            assert_eq!(confidence, 0.95);
        } else {
            panic!("Wrong event type");
        }

        let final_event = RealtimeEvent::FinalTranscription {
            text: "最终文本".to_string(),
            chunk_id: 2,
            confidence: 0.98,
            duration: Duration::from_millis(500),
        };

        if let RealtimeEvent::FinalTranscription { text, duration, .. } = final_event {
            assert_eq!(text, "最终文本");
            assert_eq!(duration, Duration::from_millis(500));
        } else {
            panic!("Wrong event type");
        }

        let error_event = RealtimeEvent::TranscriptionError {
            error: "测试错误".to_string(),
            chunk_id: 3,
        };

        if let RealtimeEvent::TranscriptionError { error, chunk_id } = error_event {
            assert_eq!(error, "测试错误");
            assert_eq!(chunk_id, 3);
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_processing_stats_structure() {
        let stats = ProcessingStats {
            total_chunks_processed: 42,
            average_processing_time: Duration::from_millis(150),
            buffer_utilization: 0.75,
            is_streaming: true,
        };

        assert_eq!(stats.total_chunks_processed, 42);
        assert_eq!(stats.average_processing_time, Duration::from_millis(150));
        assert_eq!(stats.buffer_utilization, 0.75);
        assert!(stats.is_streaming);
    }

    #[tokio::test]
    async fn test_concurrent_buffer_operations() {
        let buffer_manager = Arc::new(LocalBufferManager::new(1000).unwrap());

        // 并发写入和读取测试
        let buffer_clone1 = buffer_manager.clone();
        let buffer_clone2 = buffer_manager.clone();

        let write_task = tokio::spawn(async move {
            for i in 0..10 {
                let samples: Vec<f32> = (0..10).map(|j| (i * 10 + j) as f32 * 0.01).collect();
                if buffer_clone1.write_samples(&samples).is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        let read_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await; // 等待一些数据写入
            for _ in 0..5 {
                if buffer_clone2.available_samples() >= 10 {
                    let _ = buffer_clone2.read_chunk(10);
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        });

        let (write_result, read_result) = tokio::join!(write_task, read_task);
        assert!(write_result.is_ok());
        assert!(read_result.is_ok());
    }
}
