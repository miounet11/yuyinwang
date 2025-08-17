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
    current_level: Arc<Mutex<f32>>,
    silence_duration: Arc<Mutex<std::time::Duration>>,
    last_sound_time: Arc<Mutex<std::time::Instant>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            audio_data: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(44100)),
            current_level: Arc::new(Mutex::new(0.0)),
            silence_duration: Arc::new(Mutex::new(std::time::Duration::from_secs(0))),
            last_sound_time: Arc::new(Mutex::new(std::time::Instant::now())),
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
        let current_level = self.current_level.clone();
        let silence_duration = self.silence_duration.clone();
        let last_sound_time = self.last_sound_time.clone();
        
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
                SampleFormat::F32 => build_input_stream::<f32>(&device, &config.into(), audio_data.clone(), is_recording.clone(), current_level.clone(), silence_duration.clone(), last_sound_time.clone()),
                SampleFormat::I16 => build_input_stream::<i16>(&device, &config.into(), audio_data.clone(), is_recording.clone(), current_level.clone(), silence_duration.clone(), last_sound_time.clone()),
                SampleFormat::U16 => build_input_stream::<u16>(&device, &config.into(), audio_data.clone(), is_recording.clone(), current_level.clone(), silence_duration.clone(), last_sound_time.clone()),
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
    
    pub fn get_sample_rate(&self) -> u32 {
        *self.sample_rate.lock()
    }
    
    pub fn get_current_audio_level(&self) -> Option<f32> {
        if self.is_recording.load(Ordering::Relaxed) {
            Some(*self.current_level.lock())
        } else {
            None
        }
    }
    
    pub fn get_silence_duration(&self) -> std::time::Duration {
        *self.silence_duration.lock()
    }
    
    pub fn reset_silence_detection(&self) {
        *self.silence_duration.lock() = std::time::Duration::from_secs(0);
        *self.last_sound_time.lock() = std::time::Instant::now();
    }
    
    pub fn force_reset(&mut self) {
        self.is_recording.store(false, Ordering::Relaxed);
        self.audio_data.lock().clear();
        *self.current_level.lock() = 0.0;
        self.reset_silence_detection();
    }
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_data: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    current_level: Arc<Mutex<f32>>,
    silence_duration: Arc<Mutex<std::time::Duration>>,
    last_sound_time: Arc<Mutex<std::time::Instant>>,
) -> Result<cpal::Stream, String>
where
    T: Sample + Send + 'static + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let err_fn = |err| eprintln!("Audio stream error: {}", err);
    
    const SILENCE_THRESHOLD: f32 = 0.01;  // 静音阈值
    
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            if is_recording.load(Ordering::Relaxed) {
                let mut audio = audio_data.lock();
                let mut max_level = 0.0f32;
                
                for &sample in data {
                    let f32_sample = sample.to_sample::<f32>();
                    audio.push(f32_sample);
                    max_level = max_level.max(f32_sample.abs());
                }
                
                // 更新当前音频电平
                *current_level.lock() = max_level;
                
                // 静音检测
                if max_level < SILENCE_THRESHOLD {
                    // 如果是静音，更新静音持续时间
                    let now = std::time::Instant::now();
                    let last_sound = *last_sound_time.lock();
                    *silence_duration.lock() = now.duration_since(last_sound);
                } else {
                    // 如果有声音，重置静音计时
                    *last_sound_time.lock() = std::time::Instant::now();
                    *silence_duration.lock() = std::time::Duration::from_secs(0);
                }
            }
        },
        err_fn,
        None
    ).map_err(|e| format!("Failed to build input stream: {}", e))?;

    Ok(stream)
}