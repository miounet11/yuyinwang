use crate::core::{error::Result, types::*};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct AudioRecorder {
    config: RecordingConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Arc<Mutex<Option<cpal::Stream>>>,
}

impl AudioRecorder {
    pub fn new(config: RecordingConfig) -> Self {
        Self {
            config,
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: Arc::new(Mutex::new(None)),
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
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                buffer.lock().extend_from_slice(data);
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        stream.play()?;
        *self.stream.lock() = Some(stream);
        Ok(())
    }

    pub fn stop(&self) -> Result<Vec<f32>> {
        if let Some(stream) = self.stream.lock().take() {
            drop(stream);
        }
        let data = self.buffer.lock().drain(..).collect();
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
