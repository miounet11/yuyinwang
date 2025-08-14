// 实时音频流处理器
// 负责协调音频捕获、分块处理和实时转录

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use ringbuf::{HeapRb, Producer, Consumer, Rb};
use tokio::sync::mpsc;
use tokio::time::interval;
use crate::errors::{AppError, AppResult};
use crate::types::{TranscriptionResult, TranscriptionConfig, RealtimeTranscriptionEvent};
use super::AudioRecorder;
use crate::transcription::TranscriptionService;

// 音频块大小配置
const CHUNK_SIZE_SECONDS: f32 = 1.5;  // 基础块大小
const CHUNK_OVERLAP_SECONDS: f32 = 0.3;  // 重叠时间
const SAMPLE_RATE: u32 = 16000;  // 标准采样率
const BUFFER_CAPACITY: usize = SAMPLE_RATE as usize * 10;  // 10秒缓冲

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
    TranscriptionError { 
        error: String, 
        chunk_id: u64,
    },
    /// 录音状态变化
    RecordingStatusChanged { 
        is_recording: bool,
    },
    /// 缓冲区状态
    BufferStatus {
        used_samples: usize,
        capacity_samples: usize,
        processing_chunks: usize,
    }
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
        event_receiver: mpsc::UnboundedReceiver<RealtimeEvent>
    ) -> AppResult<mpsc::UnboundedReceiver<RealtimeEvent>> {
        if self.is_streaming.load(Ordering::Relaxed) {
            return Err(AppError::AudioRecordingError("已经在进行实时流处理".to_string()));
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
        self.start_transcription_processing_loop(event_tx.clone()).await?;
        
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
            let _ = sender.send(RealtimeEvent::RecordingStatusChanged { is_recording: false });
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
    
    /// 启动音频处理循环
    async fn start_audio_processing_loop(
        &self,
        event_sender: mpsc::UnboundedSender<RealtimeEvent>
    ) -> AppResult<()> {
        let is_streaming = self.is_streaming.clone();
        let audio_recorder = self.audio_recorder.clone();
        let buffer_manager = self.buffer_manager.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100)); // 每100ms检查一次
            
            while is_streaming.load(Ordering::Relaxed) {
                interval.tick().await;
                
                // 从录音器获取新的音频数据
                if let Some(recorder) = audio_recorder.try_lock() {
                    if recorder.is_recording() {
                        // 这里需要修改AudioRecorder以支持获取实时数据
                        // 目前的实现不支持，需要重构
                        // let new_audio_data = recorder.get_latest_audio_data();
                        // if !new_audio_data.is_empty() {
                        //     let _ = buffer_manager.write_samples(&new_audio_data);
                        // }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// 启动转录处理循环
    async fn start_transcription_processing_loop(
        &self,
        event_sender: mpsc::UnboundedSender<RealtimeEvent>
    ) -> AppResult<()> {
        let is_streaming = self.is_streaming.clone();
        let buffer_manager = self.buffer_manager.clone();
        let chunk_processor = self.chunk_processor.clone();
        let transcription_service = self.transcription_service.clone();
        let chunk_counter = self.chunk_counter.clone();
        let config = self.config.clone();
        let chunk_size_samples = self.chunk_size_samples;
        let processing_times = self.processing_times.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(1000)); // 每秒检查一次
            
            while is_streaming.load(Ordering::Relaxed) {
                interval.tick().await;
                
                // 检查是否有足够的音频数据
                if buffer_manager.available_samples() >= chunk_size_samples {
                    let start_time = Instant::now();
                    
                    // 读取音频块
                    match buffer_manager.read_chunk(chunk_size_samples) {
                        Ok(audio_chunk) => {
                            let chunk_id = {
                                let mut counter = chunk_counter.lock();
                                *counter += 1;
                                *counter
                            };
                            
                            // 处理音频块
                            match chunk_processor.process_chunk(&audio_chunk).await {
                                Ok(processed_audio) => {
                                    // 异步转录
                                    let transcription_service = transcription_service.clone();
                                    let config = config.clone();
                                    let sender = event_sender.clone();
                                    let processing_times = processing_times.clone();
                                    let chunk_processor_clone = chunk_processor.clone();
                                    
                                    tokio::spawn(async move {
                                        // 创建临时音频文件
                                        match chunk_processor_clone.save_chunk_to_file(&processed_audio).await {
                                            Ok(temp_file_path) => {
                                                // 转录音频块
                                                match transcription_service.transcribe_audio(&temp_file_path, &config).await {
                                                    Ok(result) => {
                                                        let processing_time = start_time.elapsed();
                                                        processing_times.lock().push(processing_time);
                                                        
                                                        // 发送最终转录结果
                                                        let _ = sender.send(RealtimeEvent::FinalTranscription {
                                                            text: result.text,
                                                            chunk_id,
                                                            confidence: result.confidence.unwrap_or(0.0),
                                                            duration: processing_time,
                                                        });
                                                    }
                                                    Err(e) => {
                                                        let _ = sender.send(RealtimeEvent::TranscriptionError {
                                                            error: e.to_string(),
                                                            chunk_id,
                                                        });
                                                    }
                                                }
                                                
                                                // 清理临时文件
                                                let _ = std::fs::remove_file(temp_file_path);
                                            }
                                            Err(e) => {
                                                let _ = sender.send(RealtimeEvent::TranscriptionError {
                                                    error: format!("保存音频块失败: {}", e),
                                                    chunk_id,
                                                });
                                            }
                                        }
                                    });
                                }
                                Err(e) => {
                                    let _ = event_sender.send(RealtimeEvent::TranscriptionError {
                                        error: format!("处理音频块失败: {}", e),
                                        chunk_id,
                                    });
                                }
                            }
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
        event_sender: mpsc::UnboundedSender<RealtimeEvent>
    ) -> AppResult<()> {
        let is_streaming = self.is_streaming.clone();
        let buffer_manager = self.buffer_manager.clone();
        let chunk_counter = self.chunk_counter.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5)); // 每5秒报告一次状态
            
            while is_streaming.load(Ordering::Relaxed) {
                interval.tick().await;
                
                // 发送缓冲区状态
                let _ = event_sender.send(RealtimeEvent::BufferStatus {
                    used_samples: buffer_manager.used_samples(),
                    capacity_samples: buffer_manager.capacity(),
                    processing_chunks: *chunk_counter.lock() as usize,
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
            writer.write_sample(sample_i16)
                .map_err(|e| AppError::AudioProcessingError(format!("写入WAV数据失败: {}", e)))?;
        }
        
        writer.finalize()
            .map_err(|e| AppError::AudioProcessingError(format!("完成WAV文件失败: {}", e)))?;
        
        Ok(file_path)
    }
}