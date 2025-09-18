use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 音频可视化数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVisualizationData {
    /// 当前振幅级别 (0.0-1.0)
    pub amplitude: f32,
    /// 频率域数据用于波形显示
    pub frequency_data: Vec<f32>,
    /// 时间戳数组用于波形历史
    pub time_stamps: Vec<f32>,
    /// 语音活动检测
    pub voice_activity_detected: bool,
    /// 背景噪声级别
    pub noise_level: f32,
    /// 峰值检测
    pub peak_detected: bool,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
}

/// 波形渲染配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformConfig {
    /// 缓冲区大小
    pub buffer_size: usize,
    /// 采样率
    pub sample_rate: u32,
    /// 渲染模式
    pub render_mode: WaveformRenderMode,
    /// 颜色方案
    pub color_scheme: WaveformColorScheme,
    /// 响应时间要求（毫秒）
    pub max_response_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaveformRenderMode {
    RealTime,
    Static,
    Miniature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformColorScheme {
    pub low_amplitude: String,  // 低振幅颜色
    pub mid_amplitude: String,  // 中振幅颜色
    pub high_amplitude: String, // 高振幅颜色
    pub background: String,     // 背景颜色
    pub peak_indicator: String, // 峰值指示器颜色
}

impl Default for WaveformColorScheme {
    fn default() -> Self {
        Self {
            low_amplitude: "#4caf50".to_string(),  // 绿色
            mid_amplitude: "#ff9800".to_string(),  // 橙色
            high_amplitude: "#f44336".to_string(), // 红色
            background: "#1a1a1a".to_string(),     // 深灰色
            peak_indicator: "#ffeb3b".to_string(), // 黄色
        }
    }
}

/// 音频可视化管理器
#[derive(Debug)]
pub struct AudioVisualizationManager {
    /// 音频数据历史缓冲区
    amplitude_history: Arc<Mutex<VecDeque<f32>>>,
    /// 频率数据缓冲区
    frequency_history: Arc<Mutex<VecDeque<Vec<f32>>>>,
    /// 配置
    config: WaveformConfig,
    /// 上次更新时间
    last_update: Arc<Mutex<std::time::Instant>>,
    /// 语音活动检测阈值
    voice_activity_threshold: f32,
    /// 噪声级别计算窗口
    noise_calculation_window: usize,
}

impl AudioVisualizationManager {
    /// 创建新的音频可视化管理器
    pub fn new(config: WaveformConfig) -> Self {
        Self {
            amplitude_history: Arc::new(Mutex::new(VecDeque::with_capacity(config.buffer_size))),
            frequency_history: Arc::new(Mutex::new(VecDeque::with_capacity(config.buffer_size))),
            config,
            last_update: Arc::new(Mutex::new(std::time::Instant::now())),
            voice_activity_threshold: 0.1, // 10%的振幅阈值
            noise_calculation_window: 100, // 100个采样点的窗口
        }
    }

    /// 处理新的音频数据
    pub async fn process_audio_data(&self, audio_samples: &[f32]) -> AudioVisualizationData {
        let start_time = std::time::Instant::now();

        // 计算当前振幅（RMS）
        let amplitude = self.calculate_rms_amplitude(audio_samples);

        // 计算频率域数据（简化FFT）
        let frequency_data = self.calculate_frequency_data(audio_samples).await;

        // 更新历史数据
        self.update_history(amplitude, frequency_data.clone()).await;

        // 语音活动检测
        let voice_activity_detected = amplitude > self.voice_activity_threshold;

        // 计算噪声级别
        let noise_level = self.calculate_noise_level().await;

        // 峰值检测
        let peak_detected = self.detect_peak(amplitude).await;

        // 获取时间戳数组
        let time_stamps = self.get_time_stamps().await;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        AudioVisualizationData {
            amplitude,
            frequency_data,
            time_stamps,
            voice_activity_detected,
            noise_level,
            peak_detected,
            response_time_ms,
        }
    }

    /// 计算RMS振幅
    fn calculate_rms_amplitude(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = samples.iter().map(|&sample| sample * sample).sum();
        let rms = (sum_squares / samples.len() as f32).sqrt();

        // 归一化到0-1范围
        (rms * 10.0).min(1.0)
    }

    /// 计算频率域数据（简化的频谱分析）
    async fn calculate_frequency_data(&self, samples: &[f32]) -> Vec<f32> {
        // 简化的频率分析，分为10个频段
        const NUM_BANDS: usize = 10;
        let band_size = samples.len() / NUM_BANDS;

        let mut frequency_data = Vec::with_capacity(NUM_BANDS);

        for i in 0..NUM_BANDS {
            let start = i * band_size;
            let end = ((i + 1) * band_size).min(samples.len());

            if start < end {
                let band_amplitude = self.calculate_rms_amplitude(&samples[start..end]);
                frequency_data.push(band_amplitude);
            } else {
                frequency_data.push(0.0);
            }
        }

        frequency_data
    }

    /// 更新历史数据
    async fn update_history(&self, amplitude: f32, frequency_data: Vec<f32>) {
        let mut amplitude_history = self.amplitude_history.lock().await;
        let mut frequency_history = self.frequency_history.lock().await;

        // 添加新数据
        amplitude_history.push_back(amplitude);
        frequency_history.push_back(frequency_data);

        // 保持缓冲区大小
        while amplitude_history.len() > self.config.buffer_size {
            amplitude_history.pop_front();
        }
        while frequency_history.len() > self.config.buffer_size {
            frequency_history.pop_front();
        }

        // 更新时间戳
        *self.last_update.lock().await = std::time::Instant::now();
    }

    /// 计算噪声级别
    async fn calculate_noise_level(&self) -> f32 {
        let amplitude_history = self.amplitude_history.lock().await;

        if amplitude_history.len() < self.noise_calculation_window {
            return 0.0;
        }

        // 取最低的20%作为噪声级别
        let mut sorted_amplitudes: Vec<f32> = amplitude_history
            .iter()
            .rev()
            .take(self.noise_calculation_window)
            .cloned()
            .collect();

        sorted_amplitudes.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let noise_samples = sorted_amplitudes.len() / 5; // 20%
        if noise_samples > 0 {
            sorted_amplitudes[..noise_samples].iter().sum::<f32>() / noise_samples as f32
        } else {
            0.0
        }
    }

    /// 检测峰值
    async fn detect_peak(&self, current_amplitude: f32) -> bool {
        let amplitude_history = self.amplitude_history.lock().await;

        if amplitude_history.len() < 3 {
            return false;
        }

        // 检查当前振幅是否比前两个值都高，并且高于阈值
        if let (Some(&prev1), Some(&prev2)) = (
            amplitude_history.get(amplitude_history.len().saturating_sub(1)),
            amplitude_history.get(amplitude_history.len().saturating_sub(2)),
        ) {
            current_amplitude > prev1 && current_amplitude > prev2 && current_amplitude > 0.3
        } else {
            false
        }
    }

    /// 获取时间戳数组
    async fn get_time_stamps(&self) -> Vec<f32> {
        let amplitude_history = self.amplitude_history.lock().await;
        let len = amplitude_history.len();

        (0..len)
            .map(|i| i as f32 / self.config.sample_rate as f32)
            .collect()
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &WaveformConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, new_config: WaveformConfig) {
        self.config = new_config;
    }

    /// 清除历史数据
    pub async fn clear_history(&self) {
        self.amplitude_history.lock().await.clear();
        self.frequency_history.lock().await.clear();
    }

    /// 获取当前缓冲区使用情况
    pub async fn get_buffer_usage(&self) -> (usize, usize) {
        let amplitude_len = self.amplitude_history.lock().await.len();
        let frequency_len = self.frequency_history.lock().await.len();
        (amplitude_len, frequency_len)
    }
}

impl Default for WaveformConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            sample_rate: 44100,
            render_mode: WaveformRenderMode::RealTime,
            color_scheme: WaveformColorScheme::default(),
            max_response_time_ms: 16, // 60 FPS = ~16ms per frame
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_amplitude_calculation() {
        let manager = AudioVisualizationManager::new(WaveformConfig::default());

        // 测试静音
        let silent_samples = vec![0.0; 1024];
        let result = manager.process_audio_data(&silent_samples).await;
        assert!(result.amplitude < 0.01);

        // 测试有声音
        let loud_samples = vec![0.5; 1024];
        let result = manager.process_audio_data(&loud_samples).await;
        assert!(result.amplitude > 0.1);
    }

    #[tokio::test]
    async fn test_frequency_data() {
        let manager = AudioVisualizationManager::new(WaveformConfig::default());
        let samples = vec![0.1; 1024];

        let result = manager.process_audio_data(&samples).await;
        assert_eq!(result.frequency_data.len(), 10);
    }

    #[tokio::test]
    async fn test_response_time() {
        let mut config = WaveformConfig::default();
        config.max_response_time_ms = 100; // 100ms限制

        let manager = AudioVisualizationManager::new(config);
        let samples = vec![0.1; 1024];

        let result = manager.process_audio_data(&samples).await;
        assert!(result.response_time_ms < 100);
    }
}
