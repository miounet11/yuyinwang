我将为您提供一个完整的音频处理管线优化方案，专门针对Spokenly克隆的需求。

## 1. 音频处理算法实现

### Cargo.toml
```toml
[package]
name = "spokenly-audio"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
cpal = "0.15"
hound = "3.5"
rustfft = "6.1"
candle-core = "0.3"
candle-nn = "0.3"
candle-transformers = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
crossbeam-channel = "0.5"
ringbuf = "0.3"
dasp = { version = "0.11", features = ["signal", "sample", "interpolate"] }
webrtc-vad = "0.4"
nnnoiseless = "0.5"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "audio_processing"
harness = false
```

### src/audio/config.rs
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub chunk_duration_ms: u64,
    pub vad_threshold: f32,
    pub noise_suppression: bool,
    pub whisper_model: WhisperConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperConfig {
    pub model_path: String,
    pub language: Option<String>,
    pub temperature: f32,
    pub beam_size: usize,
    pub best_of: usize,
    pub use_gpu: bool,
    pub chunk_length: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            buffer_size: 1024,
            chunk_duration_ms: 30,
            vad_threshold: 0.5,
            noise_suppression: true,
            whisper_model: WhisperConfig::default(),
        }
    }
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            model_path: "models/whisper-base.safetensors".to_string(),
            language: Some("en".to_string()),
            temperature: 0.0,
            beam_size: 5,
            best_of: 5,
            use_gpu: true,
            chunk_length: 30,
        }
    }
}
```

### src/audio/capture.rs
```rust
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, Sample, SampleFormat, SampleRate, Stream, StreamConfig,
};
use crossbeam_channel::{Receiver, Sender};
use ringbuf::{HeapRb, Rb};
use std::sync::{Arc, Mutex};
use tokio::time::Instant;
use tracing::{error, info, warn};

use super::config::AudioConfig;

pub struct AudioCapture {
    config: AudioConfig,
    stream: Option<Stream>,
    audio_sender: Sender<Vec<f32>>,
    audio_receiver: Receiver<Vec<f32>>,
    ring_buffer: Arc<Mutex<HeapRb<f32>>>,
}

impl AudioCapture {
    pub fn new(config: AudioConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let buffer_size = config.sample_rate as usize * 2; // 2 seconds buffer
        let ring_buffer = Arc::new(Mutex::new(HeapRb::<f32>::new(buffer_size)));
        let (audio_sender, audio_receiver) = crossbeam_channel::unbounded();

        Ok(Self {
            config,
            stream: None,
            audio_sender,
            audio_receiver,
            ring_buffer,
        })
    }

    pub fn start_capture(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        info!("Using input device: {}", device.name()?);

        let config = self.get_optimal_config(&device)?;
        let stream = self.build_input_stream(&device, &config)?;

        stream.play()?;
        self.stream = Some(stream);

        info!("Audio capture started with config: {:?}", config);
        Ok(())
    }

    fn get_optimal_config(&self, device: &Device) -> Result<StreamConfig, Box<dyn std::error::Error>> {
        let supported_configs = device.supported_input_configs()?;
        
        // Find the best matching configuration
        let mut best_config = None;
        let mut best_score = 0;

        for supported_config in supported_configs {
            let sample_rate = supported_config.max_sample_rate().0;
            let channels = supported_config.channels();
            
            // Score based on how close to our desired config
            let mut score = 0;
            if sample_rate >= self.config.sample_rate {
                score += 10;
            }
            if channels >= self.config.channels {
                score += 5;
            }
            if supported_config.sample_format() == SampleFormat::F32 {
                score += 15;
            }

            if score > best_score {
                best_score = score;
                best_config = Some(supported_config.with_max_sample_rate());
            }
        }

        let config = best_config.ok_or("No suitable audio configuration found")?;
        
        Ok(StreamConfig {
            channels: self.config.channels,
            sample_rate: SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(self.config.buffer_size as u32),
        })
    }

    fn build_input_stream(
        &self,
        device: &Device,
        config: &StreamConfig,
    ) -> Result<Stream, Box<dyn std::error::Error>> {
        let ring_buffer = Arc::clone(&self.ring_buffer);
        let sender = self.audio_sender.clone();
        let chunk_size = (self.config.sample_rate as f64 
            * self.config.chunk_duration_ms as f64 / 1000.0) as usize;

        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let start_time = Instant::now();
                
                // Write to ring buffer
                if let Ok(mut rb) = ring_buffer.lock() {
                    for &sample in data {
                        if rb.is_full() {
                            rb.pop(); // Remove oldest sample
                        }
                        rb.push(sample).ok();
                    }

                    // Check if we have enough data for a chunk
                    if rb.len() >= chunk_size {
                        let mut chunk = Vec::with_capacity(chunk_size);
                        for _ in 0..chunk_size {
                            if let Some(sample) = rb.pop() {
                                chunk.push(sample);
                            }
                        }
                        
                        if let Err(e) = sender.try_send(chunk) {
                            warn!("Failed to send audio chunk: {}", e);
                        }
                    }
                }

                let latency = start_time.elapsed();
                if latency.as_millis() > 20 {
                    warn!("Audio capture latency: {}ms", latency.as_millis());
                }
            },
            |err| error!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }

    pub fn get_audio_receiver(&self) -> Receiver<Vec<f32>> {
        self.audio_receiver.clone()
    }

    pub fn stop_capture(&mut self) {
        if let Some(stream) = self.stream.take() {
            drop(stream);
            info!("Audio capture stopped");
        }
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop_capture();
    }
}
```

### src/audio/preprocessing.rs
```rust
use dasp::{interpolate::linear::Linear, signal, Signal};
use nnnoiseless::DenoiseState;
use rustfft::{num_complex::Complex, FftPlanner};
use std::collections::VecDeque;
use webrtc_vad::{Vad, VadMode};

use super::config::AudioConfig;

pub struct AudioPreprocessor {
    config: AudioConfig,
    vad: Vad,
    denoise_state: Option<DenoiseState<'static>>,
    high_pass_filter: HighPassFilter,
    energy_buffer: VecDeque<f32>,
    silence_counter: usize,
}

impl AudioPreprocessor {
    pub fn new(config: AudioConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let vad = Vad::new_with_rate_and_mode(
            webrtc_vad::SampleRate::Rate16kHz,
            VadMode::VeryAggressive,
        )?;

        let denoise_state = if config.noise_suppression {
            Some(DenoiseState::new())
        } else {
            None
        };

        Ok(Self {
            config: config.clone(),
            vad,
            denoise_state,
            high_pass_filter: HighPassFilter::new(config.sample_rate as f32, 80.0),
            energy_buffer: VecDeque::with_capacity(10),
            silence_counter: 0,
        })
    }

    pub fn process_chunk(&mut self, mut audio_data: Vec<f32>) -> ProcessedAudio {
        let start_time = std::time::Instant::now();

        // Apply high-pass filter to remove low-frequency noise
        self.high_pass_filter.process(&mut audio_data);

        // Apply noise suppression if enabled
        if let Some(ref mut denoise_state) = self.denoise_state {
            self.apply_noise_suppression(denoise_state, &mut audio_data);
        }

        // Normalize audio
        self.normalize_audio(&mut audio_data);

        // Voice Activity Detection
        let has_voice = self.detect_voice_activity(&audio_data);
        
        // Calculate audio energy
        let energy = self.calculate_energy(&audio_data);
        self.energy_buffer.push_back(energy);
        if self.energy_buffer.len() > 10 {
            self.energy_buffer.pop_front();
        }

        // Update silence counter
        if has_voice {
            self.silence_counter = 0;
        } else {
            self.silence_counter += 1;
        }

        let processing_time = start_time.elapsed();

        ProcessedAudio {
            data: audio_data,
            has_voice,
            energy,
            avg_energy: self.energy_buffer.iter().sum::<f32>() / self.energy_buffer.len() as f32,
            silence_duration: self.silence_counter * self.config.chunk_duration_ms as usize,
            processing_time_us: processing_time.as_micros() as u64,
        }
    }

    fn apply_noise_suppression(&self, denoise_state: &mut DenoiseState, audio_data: &mut [f32]) {
        // RNNoise expects 480 samples at 48kHz, so we need to resample
        if self.config.sample_rate != 48000 {
            // For simplicity, we'll apply a basic spectral subtraction
            self.spectral_subtraction(audio_data);
        } else {
            // Apply RNNoise directly
            for chunk in audio_data.chunks_mut(480) {
                if chunk.len() == 480 {
                    denoise_state.process_frame(chunk);
                }
            }
        }
    }

    fn spectral_subtraction(&self, audio_data: &mut [f32]) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(audio_data.len());
        let ifft = planner.plan_fft_inverse(audio_data.len());

        // Convert to complex
        let mut complex_data: Vec<Complex<f32>> = audio_data
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        // Forward FFT
        fft.process(&mut complex_data);

        // Apply spectral subtraction
        for sample in complex_data.iter_mut() {
            let magnitude = sample.norm();
            let phase = sample.arg();
            
            // Simple noise reduction: reduce magnitude by a factor
            let reduced_magnitude = (magnitude - 0.1 * magnitude).max(0.1 * magnitude);
            *sample = Complex::from_polar(reduced_magnitude, phase);
        }

        // Inverse FFT
        ifft.process(&mut complex_data);

        // Convert back to real
        for (i, complex_sample) in complex_data.iter().enumerate() {
            audio_data[i] = complex_sample.re / audio_data.len() as f32;
        }
    }

    fn normalize_audio(&self, audio_data: &mut [f32]) {
        let max_amplitude = audio_data.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
        
        if max_amplitude > 0.0 && max_amplitude < 0.95 {
            let gain = 0.95 / max_amplitude;
            for sample in audio_data.iter_mut() {
                *sample *= gain;
            }
        }
    }

    fn detect_voice_activity(&mut self, audio_data: &[f32]) -> bool {
        // Convert f32 to i16 for WebRTC VAD
        let audio_i16: Vec<i16> = audio_data
            .iter()
            .map(|&x| (x * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();

        // WebRTC VAD expects specific frame sizes
        let frame_size = match self.config.sample_rate {
            8000 => 80,   // 10ms at 8kHz
            16000 => 160, // 10ms at 16kHz
            32000 => 320, // 10ms at 32kHz
            48000 => 480, // 10ms at 48kHz
            _ => 160,     // Default to 16kHz
        };

        let mut has_voice = false;
        for chunk in audio_i16.chunks(frame_size) {
            if chunk.len() == frame_size {
                if let Ok(is_voice) = self.vad.is_voice_segment(chunk) {
                    if is_voice {
                        has_voice = true;
                        break;
                    }
                }
            }
        }

        // Additional energy-based detection
        let energy = self.