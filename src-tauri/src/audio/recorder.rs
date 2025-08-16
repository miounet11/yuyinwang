use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::errors::{AppError, AppResult};
use crate::types::RecordingConfig;
use ringbuf::{HeapRb, Rb, ring_buffer::RbBase};
use crossbeam_channel;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use hound::{WavWriter, WavSpec};
use tempfile::NamedTempFile;

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    audio_data: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    config: RecordingConfig,
    // 新增：实时音频流支持
    realtime_buffer: Arc<Mutex<ringbuf::HeapRb<f32>>>,
    stream_listeners: Arc<Mutex<Vec<crossbeam_channel::Sender<Vec<f32>>>>>,
}

impl AudioRecorder {
    pub fn new(config: RecordingConfig) -> Self {
        // 动态缓冲区大小：根据采样率和需求计算，默认3秒缓冲
        let buffer_duration_seconds = config.buffer_duration.unwrap_or(3.0);
        let realtime_buffer_size = (config.sample_rate as f32 * buffer_duration_seconds) as usize;
        
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            audio_data: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(config.sample_rate)),
            realtime_buffer: Arc::new(Mutex::new(HeapRb::new(realtime_buffer_size))),
            stream_listeners: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }
    
    /// 添加实时音频流监听器
    pub fn add_stream_listener(&self) -> crossbeam_channel::Receiver<Vec<f32>> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.stream_listeners.lock().push(sender);
        receiver
    }
    
    /// 动态调整缓冲区大小
    pub fn resize_buffer(&self, new_duration: f32) {
        let sample_rate = *self.sample_rate.lock();
        let new_size = (sample_rate as f32 * new_duration) as usize;
        
        // 只有在新大小明显不同时才调整（避免频繁调整）
        let current_capacity = self.realtime_buffer.lock().capacity();
        if (new_size as f32 - current_capacity as f32).abs() > current_capacity as f32 * 0.2 {
            *self.realtime_buffer.lock() = HeapRb::new(new_size);
        }
    }
    
    /// 获取当前缓冲区使用情况
    pub fn get_buffer_stats(&self) -> (usize, usize, f32) {
        let buffer = self.realtime_buffer.lock();
        let used = buffer.len();
        let capacity = buffer.capacity();
        let usage_percent = if capacity > 0 { used as f32 / capacity as f32 * 100.0 } else { 0.0 };
        (used, capacity, usage_percent)
    }

    /// 获取最新的音频数据（非阻塞）
    pub fn get_latest_audio_data(&self, samples_count: usize) -> Vec<f32> {
        let mut buffer = self.realtime_buffer.lock();
        let available = buffer.len();
        let to_read = samples_count.min(available);
        
        let mut data = Vec::with_capacity(to_read);
        for _ in 0..to_read {
            if let Some(sample) = buffer.pop() {
                data.push(sample);
            }
        }
        
        data
    }
    
    /// 获取实时缓冲区使用情况
    pub fn get_buffer_status(&self) -> (usize, usize) {
        let buffer = self.realtime_buffer.lock();
        (buffer.len(), buffer.capacity())
    }
    
    /// 清空实时缓冲区
    pub fn clear_realtime_buffer(&self) {
        self.realtime_buffer.lock().clear();
    }

    pub fn start_recording(&mut self) -> AppResult<()> {
        if self.is_recording.load(Ordering::Relaxed) {
            return Err(AppError::AudioRecordingError("已经在录音中".to_string()));
        }

        // 清空之前的音频数据
        self.audio_data.lock().clear();
        
        let is_recording = self.is_recording.clone();
        let audio_data = self.audio_data.clone();
        let sample_rate = self.sample_rate.clone();
        let realtime_buffer = self.realtime_buffer.clone();
        let stream_listeners = self.stream_listeners.clone();
        let device_id = self.config.device_id.clone();
        let channels = self.config.channels;
        let duration = self.config.duration_seconds;
        
        // 在新线程中处理音频流，避免 Send 问题
        std::thread::spawn(move || {
            // 获取音频输入设备
            let host = cpal::default_host();
            let device = if let Some(device_id) = device_id {
                // 使用指定设备（需要实现设备查找逻辑）
                host.default_input_device()
                    .ok_or_else(|| "指定的音频设备不可用")
            } else {
                host.default_input_device()
                    .ok_or_else(|| "没有可用的音频输入设备")
            };

            let device = match device {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("音频设备错误: {}", e);
                    return;
                }
            };

            // 获取配置
            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("获取默认输入配置失败: {}", e);
                    return;
                }
            };

            // 更新采样率
            *sample_rate.lock() = config.sample_rate().0;
            is_recording.store(true, Ordering::Relaxed);
            
            // 创建音频流
            let stream = match config.sample_format() {
                SampleFormat::F32 => build_input_stream::<f32>(
                    &device, 
                    &config.into(), 
                    audio_data.clone(), 
                    realtime_buffer.clone(),
                    stream_listeners.clone(),
                    is_recording.clone()
                ),
                SampleFormat::I16 => build_input_stream::<i16>(
                    &device, 
                    &config.into(), 
                    audio_data.clone(),
                    realtime_buffer.clone(),
                    stream_listeners.clone(), 
                    is_recording.clone()
                ),
                SampleFormat::U16 => build_input_stream::<u16>(
                    &device, 
                    &config.into(), 
                    audio_data.clone(),
                    realtime_buffer.clone(),
                    stream_listeners.clone(),
                    is_recording.clone()
                ),
                _ => {
                    eprintln!("不支持的采样格式");
                    is_recording.store(false, Ordering::Relaxed);
                    return;
                }
            };

            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("创建音频流失败: {}", e);
                    is_recording.store(false, Ordering::Relaxed);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("播放音频流失败: {}", e);
                is_recording.store(false, Ordering::Relaxed);
                return;
            }

            println!("🎤 开始录音，采样率: {} Hz", sample_rate.lock());

            // 处理限时录音
            let start_time = std::time::Instant::now();

            // 保持流活跃直到停止录音或达到时间限制
            while is_recording.load(Ordering::Relaxed) {
                if let Some(duration_sec) = duration {
                    if start_time.elapsed().as_secs() >= duration_sec {
                        is_recording.store(false, Ordering::Relaxed);
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            drop(stream);
            println!("⏹️ 录音线程已停止");
        });

        // 等待线程启动
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }

    pub fn stop_recording(&mut self) -> AppResult<Vec<f32>> {
        if !self.is_recording.load(Ordering::Relaxed) {
            return Err(AppError::AudioRecordingError("当前没有在录音".to_string()));
        }

        self.is_recording.store(false, Ordering::Relaxed);
        
        // 等待录音线程结束
        std::thread::sleep(std::time::Duration::from_millis(200));

        // 获取录制的音频数据
        let audio_data = self.audio_data.lock().clone();
        
        println!("⏹️ 录音已停止。捕获了 {} 个采样点", audio_data.len());
        Ok(audio_data)
    }

    /// 停止录音并保存为WAV文件
    pub fn stop(&mut self) -> AppResult<Option<PathBuf>> {
        if !self.is_recording.load(Ordering::Relaxed) {
            return Ok(None);
        }

        // 停止录音并获取音频数据
        let audio_data = self.stop_recording()?;
        
        if audio_data.is_empty() {
            return Ok(None);
        }

        // 保存为WAV文件
        let wav_path = self.save_to_wav(&audio_data)?;
        Ok(Some(wav_path))
    }

    /// 将音频数据保存为WAV文件
    fn save_to_wav(&self, samples: &[f32]) -> AppResult<PathBuf> {
        // 创建临时文件
        let temp_file = NamedTempFile::with_suffix(".wav")
            .map_err(|e| AppError::AudioRecordingError(format!("创建临时文件失败: {}", e)))?;
        
        let temp_path = temp_file.path().to_path_buf();
        
        // 配置WAV规格
        let spec = WavSpec {
            channels: self.config.channels,
            sample_rate: self.get_sample_rate(),
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        
        // 写入WAV文件
        let mut writer = WavWriter::create(&temp_path, spec)
            .map_err(|e| AppError::AudioRecordingError(format!("创建WAV文件失败: {}", e)))?;
        
        for &sample in samples {
            writer.write_sample(sample)
                .map_err(|e| AppError::AudioRecordingError(format!("写入音频样本失败: {}", e)))?;
        }
        
        writer.finalize()
            .map_err(|e| AppError::AudioRecordingError(format!("完成WAV文件失败: {}", e)))?;
        
        // 保持文件不被删除
        temp_file.persist(&temp_path)
            .map_err(|e| AppError::AudioRecordingError(format!("保存WAV文件失败: {}", e)))?;
        
        println!("💾 音频已保存到: {:?}", temp_path);
        Ok(temp_path)
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    pub fn get_sample_rate(&self) -> u32 {
        *self.sample_rate.lock()
    }

    /// 强制重置录音器状态，用于状态同步
    pub fn force_reset(&mut self) {
        println!("🔄 强制重置录音器状态");
        self.is_recording.store(false, Ordering::Relaxed);
        
        // 清空音频数据缓存
        self.audio_data.lock().clear();
        
        // 等待任何正在运行的线程结束
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        println!("✅ 录音器状态已重置");
    }
}

// 辅助函数：构建输入流
fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_data: Arc<Mutex<Vec<f32>>>,
    realtime_buffer: Arc<Mutex<HeapRb<f32>>>,
    stream_listeners: Arc<Mutex<Vec<crossbeam_channel::Sender<Vec<f32>>>>>,
    is_recording: Arc<AtomicBool>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: Sample + cpal::SizedSample + Into<f32>,
{
    // 将状态放到Arc<Mutex>中以便在闭包间共享
    let chunk_buffer = Arc::new(Mutex::new(Vec::new()));
    let last_notify = Arc::new(Mutex::new(Instant::now()));
    const NOTIFY_INTERVAL: Duration = Duration::from_millis(100); // 每100ms通知一次
    
    let chunk_buffer_clone = chunk_buffer.clone();
    let last_notify_clone = last_notify.clone();
    
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            if is_recording.load(Ordering::Relaxed) {
                // 转换为f32并存储
                let samples: Vec<f32> = data.iter().map(|&sample| sample.into()).collect();
                
                // 更新完整的音频数据
                {
                    let mut audio_data_lock = audio_data.lock();
                    audio_data_lock.extend_from_slice(&samples);
                }
                
                // 更新实时缓冲区
                {
                    let mut buffer = realtime_buffer.lock();
                    for &sample in &samples {
                        // 如果缓冲区满了，丢弃旧数据
                        if buffer.is_full() {
                            buffer.pop();
                        }
                        let _ = buffer.push(sample);
                    }
                }
                
                // 积累样本用于块通知
                {
                    let mut chunk_buf = chunk_buffer_clone.lock();
                    chunk_buf.extend_from_slice(&samples);
                }
                
                // 定期通知监听器
                let now = Instant::now();
                let should_notify = {
                    let mut last_notify_lock = last_notify_clone.lock();
                    if now.duration_since(*last_notify_lock) >= NOTIFY_INTERVAL {
                        *last_notify_lock = now;
                        true
                    } else {
                        false
                    }
                };
                
                if should_notify {
                    let chunk_data = {
                        let mut chunk_buf = chunk_buffer_clone.lock();
                        if !chunk_buf.is_empty() {
                            let data = chunk_buf.clone();
                            chunk_buf.clear();
                            Some(data)
                        } else {
                            None
                        }
                    };
                    
                    if let Some(data) = chunk_data {
                        // 通知所有监听器
                        let mut listeners = stream_listeners.lock();
                        listeners.retain(|sender| {
                            sender.try_send(data.clone()).is_ok()
                        });
                    }
                }
            }
        },
        move |err| {
            eprintln!("音频流发生错误: {}", err);
        },
        None
    )
}