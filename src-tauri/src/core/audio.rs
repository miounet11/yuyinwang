use crate::core::{error::Result, types::*};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

// 最大录音时长：5 分钟
const MAX_RECORDING_DURATION_SECS: u64 = 300;
// 最大 buffer 大小：5 分钟 * 16kHz = 4.8M samples ≈ 19MB
const MAX_BUFFER_SAMPLES: usize = 16000 * MAX_RECORDING_DURATION_SECS as usize;

pub struct AudioRecorder {
    config: RecordingConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Arc<Mutex<Option<cpal::Stream>>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    buffer_overflow: Arc<AtomicBool>,
}

impl AudioRecorder {
    pub fn new(config: RecordingConfig) -> Self {
        Self {
            config,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 10))), // 预分配 10 秒
            stream: Arc::new(Mutex::new(None)),
            start_time: Arc::new(Mutex::new(None)),
            buffer_overflow: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> Result<()> {
        let host = cpal::default_host();
        let device = if let Some(device_id) = &self.config.device_id {
            host.input_devices()?
                .find(|d| d.name().ok().as_deref() == Some(device_id))
                .ok_or_else(|| crate::core::error::AppError::Audio("Device not found".into()))?
        } else {
            host.default_input_device()
                .ok_or_else(|| crate::core::error::AppError::Audio("No input device".into()))?
        };

        let config = cpal::StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = self.buffer.clone();
        let buffer_overflow = self.buffer_overflow.clone();
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buf = buffer.lock();
                // 检查是否超过最大 buffer 大小
                if buf.len() + data.len() > MAX_BUFFER_SAMPLES {
                    buffer_overflow.store(true, Ordering::Relaxed);
                    eprintln!("⚠️ 录音 buffer 溢出，已达到最大时长 {} 秒", MAX_RECORDING_DURATION_SECS);
                    return; // 停止接收新数据
                }
                buf.extend_from_slice(data);
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        stream.play()?;
        *self.stream.lock() = Some(stream);
        *self.start_time.lock() = Some(Instant::now());
        self.buffer_overflow.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn stop(&self) -> Result<Vec<f32>> {
        if let Some(stream) = self.stream.lock().take() {
            drop(stream);
            // 等待 stream 完全停止，确保所有回调完成
            std::thread::sleep(Duration::from_millis(20));
        }

        let data: Vec<f32> = self.buffer.lock().drain(..).collect();
        let duration = self.start_time.lock().take()
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        // 检查是否发生 buffer 溢出
        if self.buffer_overflow.load(Ordering::Relaxed) {
            eprintln!("⚠️ 录音已达到最大时长 {} 秒，已自动截断", MAX_RECORDING_DURATION_SECS);
        }

        println!("🎤 录音停止: {:.2}s, {} samples", duration, data.len());
        Ok(data)
    }

    pub fn is_recording(&self) -> bool {
        self.stream.lock().is_some()
    }
}

pub fn list_audio_devices() -> Result<Vec<AudioDevice>> {
    let host = cpal::default_host();
    let default_device = host.default_input_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());

    let mut devices = Vec::new();
    for device in host.input_devices()? {
        if let Ok(name) = device.name() {
            devices.push(AudioDevice {
                id: name.clone(),
                name: name.clone(),
                is_default: Some(&name) == default_name.as_ref(),
                is_available: true,
            });
        }
    }
    Ok(devices)
}

pub fn save_audio_to_wav(samples: &[f32], sample_rate: u32, path: &str) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;
    for &sample in samples {
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer.write_sample(amplitude)?;
    }
    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_config_default() {
        let config = RecordingConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
        assert!(config.device_id.is_none());
    }

    #[test]
    fn test_recording_config_validation() {
        // Valid config
        let config = RecordingConfig {
            device_id: None,
            sample_rate: 16000,
            channels: 1,
        };
        assert_eq!(config.sample_rate, 16000);

        // High sample rate
        let config = RecordingConfig {
            device_id: None,
            sample_rate: 48000,
            channels: 2,
        };
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
    }

    #[test]
    fn test_audio_recorder_creation() {
        let config = RecordingConfig::default();
        let recorder = AudioRecorder::new(config);
        assert!(!recorder.is_recording());
    }
}
