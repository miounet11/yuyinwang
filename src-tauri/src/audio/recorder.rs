use crate::errors::{AppError, AppResult};
use crate::types::RecordingConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use crossbeam_channel;
use hound::{WavSpec, WavWriter};
use parking_lot::Mutex;
use ringbuf::{ring_buffer::RbBase, HeapRb, Rb};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

/// 音频质量指标
#[derive(Debug, Clone, Serialize)]
#[serde(default)]
pub struct AudioQualityMetrics {
    pub volume_db: f64,
    pub snr_db: Option<f64>,
    pub noise_level_db: f64,
    pub clarity_score: f64,
    pub recommended_actions: Vec<Recommendation>,
    #[serde(skip)]
    pub timestamp: std::time::Instant,
}

impl Default for AudioQualityMetrics {
    fn default() -> Self {
        Self {
            volume_db: 0.0,
            snr_db: None,
            noise_level_db: 0.0,
            clarity_score: 0.0,
            recommended_actions: Vec::new(),
            timestamp: Instant::now(),
        }
    }
}

/// 音频质量建议
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Recommendation {
    IncreaseVolume,
    DecreaseVolume,
    ReduceNoise,
    ImproveClarity,
    MoveCloserToMic,
    MoveToQuieterEnvironment,
    CheckMicrophoneConnection,
}

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    audio_data: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    config: RecordingConfig,
    // 新增：实时音频流支持
    realtime_buffer: Arc<Mutex<ringbuf::HeapRb<f32>>>,
    stream_listeners: Arc<Mutex<Vec<crossbeam_channel::Sender<Vec<f32>>>>>,
    // 设备管理
    current_device_id: Arc<Mutex<Option<String>>>,
    device_change_listeners: Arc<Mutex<Vec<crossbeam_channel::Sender<String>>>>,
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
            current_device_id: Arc::new(Mutex::new(config.device_id.clone())),
            device_change_listeners: Arc::new(Mutex::new(Vec::new())),
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
        let usage_percent = if capacity > 0 {
            used as f32 / capacity as f32 * 100.0
        } else {
            0.0
        };
        (used, capacity, usage_percent)
    }

    /// 获取最新的音频数据（非阻塞）
    /// 获取最新音频数据 - 智能块大小版本
    pub fn get_latest_audio_data(&self) -> Vec<f32> {
        let mut buffer = self.realtime_buffer.lock();
        let available = buffer.len();

        // 智能块大小：确保有足够的数据但不过度延迟
        let optimal_chunk_size = self.calculate_optimal_chunk_size();
        let to_read = available.min(optimal_chunk_size);

        let mut data = Vec::with_capacity(to_read);
        for _ in 0..to_read {
            if let Some(sample) = buffer.pop() {
                data.push(sample);
            }
        }

        // 通知实时监听器
        if !data.is_empty() {
            self.notify_stream_listeners(&data);
        }

        data
    }

    /// 获取指定大小的音频数据（兼容旧接口）
    pub fn get_latest_audio_data_sized(&self, samples_count: usize) -> Vec<f32> {
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

    /// 计算最优音频块大小
    fn calculate_optimal_chunk_size(&self) -> usize {
        let sample_rate = *self.sample_rate.lock();

        // 目标：1.5秒的音频块用于转录
        let target_duration_seconds = 1.5;
        let target_chunk_size = (sample_rate as f32 * target_duration_seconds) as usize;

        // 但不要超过缓冲区容量的一半
        let buffer_capacity = self.realtime_buffer.lock().capacity();
        let max_chunk_size = buffer_capacity / 2;

        target_chunk_size.min(max_chunk_size)
    }

    /// 通知所有流监听器
    fn notify_stream_listeners(&self, data: &[f32]) {
        let mut listeners = self.stream_listeners.lock();

        // 清理断开的监听器
        listeners.retain(|sender| sender.try_send(data.to_vec()).is_ok());
    }

    /// 音频质量实时分析
    pub fn analyze_audio_quality(&self, samples: &[f32]) -> AudioQualityMetrics {
        let volume_db = self.calculate_volume_db(samples);
        let noise_level_db = self.calculate_noise_level(samples);
        let clarity_score = self.calculate_clarity_score(samples);
        let snr_db = self.calculate_snr(samples);
        let recommended_actions =
            self.generate_recommendations(volume_db, snr_db, clarity_score, noise_level_db);

        AudioQualityMetrics {
            volume_db,
            snr_db,
            noise_level_db,
            clarity_score,
            recommended_actions,
            timestamp: std::time::Instant::now(),
        }
    }

    /// 计算音量（dB）
    fn calculate_volume_db(&self, samples: &[f32]) -> f64 {
        if samples.is_empty() {
            return -80.0; // 静音
        }

        // 计算RMS音量
        let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();

        if rms > 0.0 {
            20.0 * (rms as f64).log10()
        } else {
            -80.0
        }
    }

    /// 计算噪声级别
    fn calculate_noise_level(&self, samples: &[f32]) -> f64 {
        if samples.is_empty() {
            return -80.0;
        }

        // 简化的噪声检测：计算低能量部分的平均值
        let mut low_energy_samples = Vec::new();
        let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
        let threshold = rms * 0.3; // 30%的RMS作为噪声阈值

        for &sample in samples {
            if sample.abs() < threshold {
                low_energy_samples.push(sample);
            }
        }

        if low_energy_samples.is_empty() {
            return -60.0;
        }

        let noise_rms = (low_energy_samples.iter().map(|&x| x * x).sum::<f32>()
            / low_energy_samples.len() as f32)
            .sqrt();

        if noise_rms > 0.0 {
            20.0 * (noise_rms as f64).log10()
        } else {
            -80.0
        }
    }

    /// 计算语音清晰度评分 (0.0-1.0)
    fn calculate_clarity_score(&self, samples: &[f32]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }

        // 简化的清晰度评估：基于频率内容和动态范围
        let mut high_freq_energy = 0.0f32;
        let mut mid_freq_energy = 0.0f32;

        // 简单的频率分析（在时域中近似）
        for window in samples.windows(8) {
            let high_freq = window.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f32>();
            let mid_freq = window.iter().map(|&x| x.abs()).sum::<f32>();

            high_freq_energy += high_freq;
            mid_freq_energy += mid_freq;
        }

        // 动态范围
        let max_sample = samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        let avg_sample = samples.iter().map(|&x| x.abs()).sum::<f32>() / samples.len() as f32;
        let dynamic_range = if avg_sample > 0.0 {
            max_sample / avg_sample
        } else {
            1.0
        };

        // 清晰度评分组合
        let freq_ratio = if mid_freq_energy > 0.0 {
            (high_freq_energy / mid_freq_energy).min(1.0)
        } else {
            0.0
        };

        let dynamic_score = ((dynamic_range - 1.0) / 10.0).clamp(0.0, 1.0);

        ((freq_ratio + dynamic_score) / 2.0).clamp(0.0, 1.0) as f64
    }

    /// 计算信噪比（SNR）
    fn calculate_snr(&self, samples: &[f32]) -> Option<f64> {
        if samples.is_empty() {
            return None;
        }

        let volume_db = self.calculate_volume_db(samples);
        let noise_db = self.calculate_noise_level(samples);

        Some(volume_db - noise_db)
    }

    /// 生成音频质量建议
    fn generate_recommendations(
        &self,
        volume_db: f64,
        snr_db: Option<f64>,
        clarity_score: f64,
        noise_level_db: f64,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // 音量检查
        if volume_db < -40.0 {
            recommendations.push(Recommendation::IncreaseVolume);
            recommendations.push(Recommendation::MoveCloserToMic);
        } else if volume_db > -6.0 {
            recommendations.push(Recommendation::DecreaseVolume);
        }

        // 信噪比检查
        if let Some(snr) = snr_db {
            if snr < 10.0 {
                recommendations.push(Recommendation::ReduceNoise);
                recommendations.push(Recommendation::MoveToQuieterEnvironment);
            }
        }

        // 噪声级别检查
        if noise_level_db > -30.0 {
            recommendations.push(Recommendation::ReduceNoise);
            recommendations.push(Recommendation::MoveToQuieterEnvironment);
        }

        // 清晰度检查
        if clarity_score < 0.5 {
            recommendations.push(Recommendation::ImproveClarity);
            if volume_db < -30.0 {
                recommendations.push(Recommendation::MoveCloserToMic);
            }
        }

        // 极低音量可能是设备问题
        if volume_db < -60.0 {
            recommendations.push(Recommendation::CheckMicrophoneConnection);
        }

        // 去重
        recommendations.sort_by_key(|r| format!("{:?}", r));
        recommendations.dedup();

        recommendations
    }

    /// 添加设备变更监听器
    pub fn add_device_change_listener(&self) -> crossbeam_channel::Receiver<String> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.device_change_listeners.lock().push(sender);
        receiver
    }

    /// 动态切换音频设备
    pub fn switch_audio_device(&self, new_device_id: Option<String>) -> AppResult<()> {
        let was_recording = self.is_recording.load(Ordering::Relaxed);

        // 如果正在录音，需要先停止
        if was_recording {
            println!("🔄 检测到设备切换，暂停录音进行设备切换");
            // 这里需要实现更复杂的逻辑来暂停和恢复录音
            // 目前简化处理：提示用户手动重启
            return Err(AppError::AudioRecordingError(
                "设备切换需要停止当前录音，请先停止录音后再切换设备".to_string(),
            ));
        }

        // 更新设备配置
        *self.current_device_id.lock() = new_device_id.clone();

        // 通知所有监听器
        let device_name = new_device_id.unwrap_or_else(|| "默认设备".to_string());
        let mut listeners = self.device_change_listeners.lock();
        listeners.retain(|sender| sender.try_send(device_name.clone()).is_ok());

        println!("🎤 音频设备已切换至: {}", device_name);
        Ok(())
    }

    /// 获取当前音频设备
    pub fn get_current_device(&self) -> Option<String> {
        self.current_device_id.lock().clone()
    }

    /// 检测可用音频设备
    pub fn detect_available_devices(&self) -> AppResult<Vec<String>> {
        use cpal::traits::HostTrait;

        let host = cpal::default_host();
        let devices: Result<Vec<String>, _> = host
            .input_devices()
            .map_err(|e| AppError::AudioRecordingError(format!("获取音频设备失败: {}", e)))?
            .map(|device| {
                device
                    .name()
                    .map_err(|e| AppError::AudioRecordingError(format!("获取设备名称失败: {}", e)))
            })
            .collect();

        devices
    }

    /// 检测设备变化（热插拔检测）
    pub fn monitor_device_changes(&self) -> AppResult<()> {
        let device_change_listeners = self.device_change_listeners.clone();
        let current_device_id = self.current_device_id.clone();

        // 启动设备监控线程
        std::thread::spawn(move || {
            let mut last_devices = Vec::new();

            loop {
                std::thread::sleep(Duration::from_secs(2)); // 每2秒检查一次

                match Self::get_available_devices_static() {
                    Ok(current_devices) => {
                        // 检测设备变化
                        if current_devices != last_devices {
                            println!("🔍 检测到音频设备变化");

                            // 检查当前设备是否仍然可用
                            let current_device = current_device_id.lock().clone();
                            if let Some(ref device_id) = current_device {
                                if !current_devices.contains(device_id) {
                                    println!("⚠️ 当前音频设备已断开: {}", device_id);

                                    // 通知监听器设备断开
                                    let mut listeners = device_change_listeners.lock();
                                    listeners.retain(|sender| {
                                        sender.try_send(format!("设备断开: {}", device_id)).is_ok()
                                    });
                                }
                            }

                            last_devices = current_devices;
                        }
                    }
                    Err(e) => {
                        eprintln!("设备监控错误: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// 静态方法获取可用设备（用于监控线程）
    fn get_available_devices_static() -> AppResult<Vec<String>> {
        use cpal::traits::HostTrait;

        let host = cpal::default_host();
        let devices: Result<Vec<String>, _> = host
            .input_devices()
            .map_err(|e| AppError::AudioRecordingError(format!("获取音频设备失败: {}", e)))?
            .map(|device| {
                device
                    .name()
                    .map_err(|e| AppError::AudioRecordingError(format!("获取设备名称失败: {}", e)))
            })
            .collect();

        devices
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
                    is_recording.clone(),
                ),
                SampleFormat::I16 => build_input_stream::<i16>(
                    &device,
                    &config.into(),
                    audio_data.clone(),
                    realtime_buffer.clone(),
                    stream_listeners.clone(),
                    is_recording.clone(),
                ),
                SampleFormat::U16 => build_input_stream::<u16>(
                    &device,
                    &config.into(),
                    audio_data.clone(),
                    realtime_buffer.clone(),
                    stream_listeners.clone(),
                    is_recording.clone(),
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
            writer
                .write_sample(sample)
                .map_err(|e| AppError::AudioRecordingError(format!("写入音频样本失败: {}", e)))?;
        }

        writer
            .finalize()
            .map_err(|e| AppError::AudioRecordingError(format!("完成WAV文件失败: {}", e)))?;

        // 保持文件不被删除
        temp_file
            .persist(&temp_path)
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
                        listeners.retain(|sender| sender.try_send(data.clone()).is_ok());
                    }
                }
            }
        },
        move |err| {
            eprintln!("音频流发生错误: {}", err);
        },
        None,
    )
}
