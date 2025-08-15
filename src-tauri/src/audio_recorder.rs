use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use hound::{WavSpec, WavWriter};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    audio_data: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            audio_data: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(44100)),
        }
    }

    pub fn start_recording(&mut self) -> Result<(), String> {
        if self.is_recording.load(Ordering::Relaxed) {
            return Err("Already recording".to_string());
        }

        // 清空之前的音频数据
        self.audio_data.lock().clear();
        
        let is_recording = self.is_recording.clone();
        let audio_data = self.audio_data.clone();
        let sample_rate = self.sample_rate.clone();
        
        // 在新线程中处理音频流，避免 Send 问题
        std::thread::spawn(move || {
            // 获取默认音频输入设备
            let host = cpal::default_host();
            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    eprintln!("No input device available");
                    is_recording.store(false, Ordering::Relaxed); // 确保状态重置
                    return;
                }
            };

            // 获取默认配置
            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to get default input config: {}", e);
                    is_recording.store(false, Ordering::Relaxed); // 确保状态重置
                    return;
                }
            };

            *sample_rate.lock() = config.sample_rate().0;
            
            is_recording.store(true, Ordering::Relaxed);
            
            // 创建音频流
            let stream = match config.sample_format() {
                SampleFormat::F32 => build_input_stream::<f32>(&device, &config.into(), audio_data.clone(), is_recording.clone()),
                SampleFormat::I16 => build_input_stream::<i16>(&device, &config.into(), audio_data.clone(), is_recording.clone()),
                SampleFormat::U16 => build_input_stream::<u16>(&device, &config.into(), audio_data.clone(), is_recording.clone()),
                _ => {
                    eprintln!("Unsupported sample format");
                    is_recording.store(false, Ordering::Relaxed); // 确保状态重置
                    return;
                }
            };

            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to create stream: {}", e);
                    is_recording.store(false, Ordering::Relaxed);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("Failed to play stream: {}", e);
                is_recording.store(false, Ordering::Relaxed);
                return;
            }

            println!("🎤 Recording started with sample rate: {} Hz", sample_rate.lock());

            // 保持流活跃直到停止录音
            while is_recording.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            drop(stream);
            println!("⏹️ Recording thread stopped");
        });

        // 等待一下确保线程启动
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<Vec<f32>, String> {
        if !self.is_recording.load(Ordering::Relaxed) {
            return Err("Not recording".to_string());
        }

        self.is_recording.store(false, Ordering::Relaxed);
        
        // 等待录音线程结束
        std::thread::sleep(std::time::Duration::from_millis(200));

        // 获取录制的音频数据
        let audio_data = self.audio_data.lock().clone();
        
        println!("⏹️ Recording stopped. Captured {} samples", audio_data.len());
        Ok(audio_data)
    }

    pub fn save_to_wav(&self, audio_data: &[f32], output_path: &PathBuf) -> Result<(), String> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: *self.sample_rate.lock(),
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output_path, spec)
            .map_err(|e| format!("Failed to create WAV file: {}", e))?;

        // 转换 f32 到 i16
        for &sample in audio_data {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer.finalize()
            .map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

        println!("💾 Audio saved to: {:?}", output_path);
        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_data: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
) -> Result<cpal::Stream, String>
where
    T: Sample + Send + 'static + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let err_fn = |err| eprintln!("Audio stream error: {}", err);
    
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            if is_recording.load(Ordering::Relaxed) {
                let mut audio = audio_data.lock();
                for &sample in data {
                    audio.push(sample.to_sample::<f32>());
                }
            }
        },
        err_fn,
        None
    ).map_err(|e| format!("Failed to build input stream: {}", e))?;

    Ok(stream)
}