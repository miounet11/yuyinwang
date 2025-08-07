use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig, SampleFormat};
use hound::{WavWriter, WavSpec};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;

pub struct AudioRecorder {
    device: Option<Device>,
    stream_config: Option<StreamConfig>,
    is_recording: Arc<Mutex<bool>>,
    audio_data: Arc<Mutex<Vec<f32>>>,
}

impl AudioRecorder {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        
        let config = device.default_input_config()?;
        println!("🎤 Default input config: {:?}", config);
        
        Ok(AudioRecorder {
            device: Some(device),
            stream_config: Some(config.into()),
            is_recording: Arc::new(Mutex::new(false)),
            audio_data: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.device.as_ref()
            .ok_or("No device available")?;
        
        let config = self.stream_config.as_ref()
            .ok_or("No stream config available")?;

        let is_recording = Arc::clone(&self.is_recording);
        let audio_data = Arc::clone(&self.audio_data);
        
        *is_recording.lock() = true;
        audio_data.lock().clear();

        println!("🔴 开始录音...");

        let stream = match SampleFormat::F32 {
            SampleFormat::F32 => {
                device.build_input_stream(
                    config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if *is_recording.lock() {
                            let mut buffer = audio_data.lock();
                            buffer.extend_from_slice(data);
                        }
                    },
                    |err| eprintln!("录音错误: {}", err),
                    None,
                )?
            },
            SampleFormat::I16 => {
                device.build_input_stream(
                    config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if *is_recording.lock() {
                            let mut buffer = audio_data.lock();
                            // 转换 i16 到 f32
                            for sample in data {
                                buffer.push(*sample as f32 / i16::MAX as f32);
                            }
                        }
                    },
                    |err| eprintln!("录音错误: {}", err),
                    None,
                )?
            },
            SampleFormat::U16 => {
                device.build_input_stream(
                    config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        if *is_recording.lock() {
                            let mut buffer = audio_data.lock();
                            // 转换 u16 到 f32
                            for sample in data {
                                buffer.push((*sample as f32 - 32768.0) / 32768.0);
                            }
                        }
                    },
                    |err| eprintln!("录音错误: {}", err),
                    None,
                )?
            },
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;
        
        // 这里需要保持stream存活，在实际应用中需要更好的生命周期管理
        std::mem::forget(stream);
        
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        *self.is_recording.lock() = false;
        println!("⏹️ 停止录音...");
        
        let audio_data = self.audio_data.lock().clone();
        println!("📊 录制了 {} 个音频样本", audio_data.len());
        
        Ok(audio_data)
    }

    pub fn save_to_wav(&self, audio_data: &[f32], file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.stream_config.as_ref()
            .ok_or("No stream config available")?;

        let spec = WavSpec {
            channels: config.channels,
            sample_rate: config.sample_rate.0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(file_path, spec)?;
        
        for sample in audio_data {
            let sample_i16 = (*sample * i16::MAX as f32) as i16;
            writer.write_sample(sample_i16)?;
        }
        
        writer.finalize()?;
        println!("💾 音频已保存到: {:?}", file_path);
        
        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock()
    }

    pub fn get_available_devices() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        
        // 添加默认设备
        if let Some(default_device) = host.default_input_device() {
            if let Ok(name) = default_device.name() {
                devices.push((format!("Default: {}", name), "default".to_string()));
            }
        }
        
        // 添加所有输入设备
        if let Ok(input_devices) = host.input_devices() {
            for (i, device) in input_devices.enumerate() {
                if let Ok(name) = device.name() {
                    devices.push((name, format!("device_{}", i)));
                }
            }
        }
        
        Ok(devices)
    }
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| AudioRecorder {
            device: None,
            stream_config: None,
            is_recording: Arc::new(Mutex::new(false)),
            audio_data: Arc::new(Mutex::new(Vec::new())),
        })
    }
}