use hound::{WavSpec, WavWriter};
use std::path::Path;
use crate::errors::{AppError, AppResult};

pub struct AudioProcessor {
    sample_rate: u32,
    channels: u16,
}

impl AudioProcessor {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }

    /// 将f32音频数据保存为WAV文件
    pub fn save_to_wav<P: AsRef<Path>>(&self, audio_data: &[f32], output_path: P) -> AppResult<()> {
        let spec = WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output_path.as_ref(), spec)
            .map_err(|e| AppError::AudioProcessingError(format!("创建WAV文件失败: {}", e)))?;

        // 转换 f32 到 i16
        for &sample in audio_data {
            let amplitude = self.clamp_and_convert_to_i16(sample);
            writer.write_sample(amplitude)
                .map_err(|e| AppError::AudioProcessingError(format!("写入采样失败: {}", e)))?;
        }

        writer.finalize()
            .map_err(|e| AppError::AudioProcessingError(format!("完成WAV文件失败: {}", e)))?;

        println!("💾 音频已保存到: {:?}", output_path.as_ref());
        Ok(())
    }

    /// 音频降噪处理（简单版本）
    pub fn apply_noise_reduction(&self, audio_data: &mut Vec<f32>) -> AppResult<()> {
        if audio_data.is_empty() {
            return Ok(());
        }

        // 简单的高通滤波器去除低频噪音
        let alpha = 0.95_f32; // 滤波器参数
        let mut prev_input = 0.0_f32;
        let mut prev_output = 0.0_f32;

        for sample in audio_data.iter_mut() {
            let current_input = *sample;
            let current_output = alpha * (prev_output + current_input - prev_input);
            
            *sample = current_output;
            prev_input = current_input;
            prev_output = current_output;
        }

        Ok(())
    }

    /// 音频标准化
    pub fn normalize_audio(&self, audio_data: &mut Vec<f32>) -> AppResult<()> {
        if audio_data.is_empty() {
            return Ok(());
        }

        // 找到最大振幅
        let max_amplitude = audio_data
            .iter()
            .map(|&x| x.abs())
            .fold(0.0_f32, f32::max);

        if max_amplitude == 0.0 {
            return Ok(());
        }

        // 标准化到 0.9 倍最大值，避免削波
        let scale_factor = 0.9 / max_amplitude;
        for sample in audio_data.iter_mut() {
            *sample *= scale_factor;
        }

        Ok(())
    }

    /// 音频音量调整
    pub fn adjust_volume(&self, audio_data: &mut Vec<f32>, volume_factor: f32) -> AppResult<()> {
        if volume_factor < 0.0 {
            return Err(AppError::ValidationError("音量因子不能为负数".to_string()));
        }

        for sample in audio_data.iter_mut() {
            *sample *= volume_factor;
            // 防止削波
            *sample = sample.clamp(-1.0, 1.0);
        }

        Ok(())
    }

    /// 音频静音检测
    pub fn detect_silence(&self, audio_data: &[f32], threshold: f32, min_duration_ms: u64) -> Vec<(usize, usize)> {
        let mut silence_ranges = Vec::new();
        let samples_per_ms = self.sample_rate as f64 / 1000.0;
        let min_samples = (min_duration_ms as f64 * samples_per_ms) as usize;
        
        let mut silence_start = None;
        
        for (i, &sample) in audio_data.iter().enumerate() {
            let is_silent = sample.abs() < threshold;
            
            match (silence_start, is_silent) {
                (None, true) => {
                    silence_start = Some(i);
                }
                (Some(start), false) => {
                    if i - start >= min_samples {
                        silence_ranges.push((start, i));
                    }
                    silence_start = None;
                }
                _ => {}
            }
        }
        
        // 处理结尾的静音
        if let Some(start) = silence_start {
            if audio_data.len() - start >= min_samples {
                silence_ranges.push((start, audio_data.len()));
            }
        }
        
        silence_ranges
    }

    /// 音频分段（基于静音）
    pub fn segment_by_silence(&self, audio_data: &[f32], silence_threshold: f32, min_silence_ms: u64) -> Vec<Vec<f32>> {
        let silence_ranges = self.detect_silence(audio_data, silence_threshold, min_silence_ms);
        let mut segments = Vec::new();
        let mut last_end = 0;
        
        for (start, end) in silence_ranges {
            if start > last_end {
                segments.push(audio_data[last_end..start].to_vec());
            }
            last_end = end;
        }
        
        // 添加最后一段
        if last_end < audio_data.len() {
            segments.push(audio_data[last_end..].to_vec());
        }
        
        // 过滤太短的段落
        segments.into_iter()
            .filter(|segment| segment.len() > self.sample_rate as usize / 10) // 至少0.1秒
            .collect()
    }

    /// 转换f32到i16，包含削波保护
    fn clamp_and_convert_to_i16(&self, sample: f32) -> i16 {
        let clamped = sample.clamp(-1.0, 1.0);
        (clamped * i16::MAX as f32) as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_audio() {
        let processor = AudioProcessor::new(16000, 1);
        let mut audio_data = vec![0.1, -0.5, 0.8, -0.2];
        
        processor.normalize_audio(&mut audio_data).unwrap();
        
        // 最大值应该接近0.9
        let max_val = audio_data.iter().map(|&x| x.abs()).fold(0.0, f32::max);
        assert!((max_val - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_volume_adjustment() {
        let processor = AudioProcessor::new(16000, 1);
        let mut audio_data = vec![0.1, -0.2, 0.3, -0.4];
        let original_data = audio_data.clone();
        
        processor.adjust_volume(&mut audio_data, 2.0).unwrap();
        
        for (i, &sample) in audio_data.iter().enumerate() {
            assert!((sample - original_data[i] * 2.0).abs() < 0.001);
        }
    }
}