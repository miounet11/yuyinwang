// 流式语音转录器 - Recording King v5.8+ 核心模块
// 实现如输入法般的实时转录体验

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, broadcast};
use serde::{Serialize, Deserialize};
use crate::transcription::TranscriptionService;
use crate::errors::{AppResult, AppError};

/// 转录事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptionEvent {
    /// 流式转录结果 (实时显示，如输入法)
    StreamingTranscription { 
        text: String, 
        is_partial: bool,
        confidence: f64,
        timestamp: u64,
    },
    /// 部分转录结果 (实时显示)
    PartialText { 
        text: String, 
        confidence: f64,
        timestamp: u64,
    },
    /// 最终转录结果 (完成的句子)
    FinalText { 
        text: String, 
        duration: Duration,
        timestamp: u64,
    },
    /// 流式转录完成
    StreamingComplete { 
        full_text: String,
        total_duration: Duration,
    },
    /// 转录错误
    TranscriptionError { 
        error: String,
        timestamp: u64,
    },
    /// 录音状态变化
    RecordingStatusChanged { 
        is_recording: bool,
        timestamp: u64,
    },
}

/// 流式音频配置
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub chunk_duration_ms: u64,    // 音频块持续时间 (默认100ms)
    pub overlap_duration_ms: u64,  // 重叠时间避免丢失边界词 (默认50ms)
    pub min_confidence: f64,        // 最小置信度阈值 (默认0.7)
    pub silence_timeout_ms: u64,   // 静音超时 (默认2000ms)
    pub max_partial_length: usize,  // 最大部分转录长度 (默认100字符)
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_duration_ms: 100,
            overlap_duration_ms: 50,
            min_confidence: 0.7,
            silence_timeout_ms: 2000,
            max_partial_length: 100,
        }
    }
}

/// 音频块数据
#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub data: Vec<f32>,
    pub sample_rate: u32,
    pub timestamp: Instant,
    pub chunk_id: u64,
}

/// 流式语音转录器
pub struct StreamingVoiceTranscriptor {
    config: StreamingConfig,
    transcription_service: Arc<TranscriptionService>,
    event_sender: broadcast::Sender<TranscriptionEvent>,
    audio_receiver: Option<mpsc::UnboundedReceiver<AudioChunk>>,
    is_active: Arc<Mutex<bool>>,
    current_chunk_id: Arc<Mutex<u64>>,
    partial_text_buffer: Arc<Mutex<String>>,
    last_activity: Arc<Mutex<Instant>>,
}

impl StreamingVoiceTranscriptor {
    /// 创建新的流式转录器
    pub fn new(
        config: StreamingConfig,
        transcription_service: Arc<TranscriptionService>,
    ) -> (Self, broadcast::Receiver<TranscriptionEvent>) {
        let (event_sender, event_receiver) = broadcast::channel(1000);
        
        let transcriptor = Self {
            config,
            transcription_service,
            event_sender,
            audio_receiver: None,
            is_active: Arc::new(Mutex::new(false)),
            current_chunk_id: Arc::new(Mutex::new(0)),
            partial_text_buffer: Arc::new(Mutex::new(String::new())),
            last_activity: Arc::new(Mutex::new(Instant::now())),
        };
        
        (transcriptor, event_receiver)
    }
    
    /// 启动流式转录
    pub async fn start_streaming(
        &mut self,
        mut audio_receiver: mpsc::UnboundedReceiver<AudioChunk>,
    ) -> AppResult<()> {
        {
            let mut is_active = self.is_active.lock().unwrap();
            if *is_active {
                return Err(AppError::StreamingError("流式转录已在运行中".to_string()));
            }
            *is_active = true;
        }
        
        // 发送录音状态变化事件
        let _ = self.event_sender.send(TranscriptionEvent::RecordingStatusChanged {
            is_recording: true,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        });
        
        println!("🎙️ 流式语音转录启动");
        
        // 启动主处理循环
        let event_sender = self.event_sender.clone();
        let transcription_service = self.transcription_service.clone();
        let config = self.config.clone();
        let is_active = self.is_active.clone();
        let current_chunk_id = self.current_chunk_id.clone();
        let partial_text_buffer = self.partial_text_buffer.clone();
        let last_activity = self.last_activity.clone();
        
        tokio::spawn(async move {
            Self::processing_loop(
                audio_receiver,
                event_sender,
                transcription_service,
                config,
                is_active,
                current_chunk_id,
                partial_text_buffer,
                last_activity,
            ).await;
        });
        
        Ok(())
    }
    
    /// 停止流式转录
    pub async fn stop_streaming(&mut self) -> AppResult<String> {
        {
            let mut is_active = self.is_active.lock().unwrap();
            if !*is_active {
                return Ok("流式转录未在运行".to_string());
            }
            *is_active = false;
        }
        
        // 获取最终的完整文本
        let final_text = {
            let buffer = self.partial_text_buffer.lock().unwrap();
            buffer.clone()
        };
        
        // 发送流式完成事件
        let _ = self.event_sender.send(TranscriptionEvent::StreamingComplete {
            full_text: final_text.clone(),
            total_duration: Duration::from_secs(0), // TODO: 计算实际持续时间
        });
        
        // 发送录音状态变化事件
        let _ = self.event_sender.send(TranscriptionEvent::RecordingStatusChanged {
            is_recording: false,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        });
        
        println!("🛑 流式语音转录停止");
        Ok(final_text)
    }
    
    /// 主要处理循环
    async fn processing_loop(
        mut audio_receiver: mpsc::UnboundedReceiver<AudioChunk>,
        event_sender: broadcast::Sender<TranscriptionEvent>,
        transcription_service: Arc<TranscriptionService>,
        config: StreamingConfig,
        is_active: Arc<Mutex<bool>>,
        current_chunk_id: Arc<Mutex<u64>>,
        partial_text_buffer: Arc<Mutex<String>>,
        last_activity: Arc<Mutex<Instant>>,
    ) {
        let mut accumulated_audio = Vec::new();
        let mut last_transcription_time = Instant::now();
        
        while let Some(chunk) = audio_receiver.recv().await {
            // 检查是否应该继续处理
            {
                let is_active = is_active.lock().unwrap();
                if !*is_active {
                    break;
                }
            }
            
            // 更新活动时间
            {
                let mut last_activity = last_activity.lock().unwrap();
                *last_activity = Instant::now();
            }
            
            // 累积音频数据
            accumulated_audio.extend_from_slice(&chunk.data);
            
            // 检查是否达到转录间隔
            if last_transcription_time.elapsed() >= Duration::from_millis(config.chunk_duration_ms) {
                // 处理累积的音频数据
                if !accumulated_audio.is_empty() {
                    Self::process_audio_chunk(
                        &accumulated_audio,
                        chunk.sample_rate,
                        &event_sender,
                        &transcription_service,
                        &config,
                        &current_chunk_id,
                        &partial_text_buffer,
                    ).await;
                    
                    // 保留重叠部分避免丢失边界词
                    let overlap_samples = (chunk.sample_rate as u64 * config.overlap_duration_ms / 1000) as usize;
                    if accumulated_audio.len() > overlap_samples {
                        accumulated_audio.drain(..accumulated_audio.len() - overlap_samples);
                    } else {
                        accumulated_audio.clear();
                    }
                }
                
                last_transcription_time = Instant::now();
            }
            
            // 检查静音超时
            let should_timeout = {
                let last_activity = last_activity.lock().unwrap();
                last_activity.elapsed() > Duration::from_millis(config.silence_timeout_ms)
            };
            
            if should_timeout {
                println!("🔇 检测到静音超时，停止流式转录");
                break;
            }
        }
        
        // 处理剩余的音频数据
        if !accumulated_audio.is_empty() {
            Self::process_audio_chunk(
                &accumulated_audio,
                16000, // 默认采样率
                &event_sender,
                &transcription_service,
                &config,
                &current_chunk_id,
                &partial_text_buffer,
            ).await;
        }
        
        println!("🔚 流式转录处理循环结束");
    }
    
    /// 处理音频块
    async fn process_audio_chunk(
        audio_data: &[f32],
        sample_rate: u32,
        event_sender: &broadcast::Sender<TranscriptionEvent>,
        transcription_service: &Arc<TranscriptionService>,
        config: &StreamingConfig,
        current_chunk_id: &Arc<Mutex<u64>>,
        partial_text_buffer: &Arc<Mutex<String>>,
    ) {
        // 生成chunk ID
        let chunk_id = {
            let mut id = current_chunk_id.lock().unwrap();
            *id += 1;
            *id
        };
        
        println!("🔄 处理音频块 #{}, 样本数: {}", chunk_id, audio_data.len());
        
        // 创建转录配置 - 使用快速模型实现实时响应
        let transcription_config = crate::types::TranscriptionConfig {
            model_name: "whisper-tiny".to_string(), // 使用最快的模型
            language: Some("zh".to_string()),
            temperature: Some(0.0),
            is_local: true,
            api_endpoint: None,
        };
        
        // 实际转录音频块
        match transcription_service.transcribe_audio_chunk(audio_data, sample_rate, &transcription_config).await {
            Ok(result) => {
                let confidence = result.confidence.unwrap_or(0.8);
                let text = result.text.trim().to_string();
                
                // 过滤空或无意义的转录结果
                if text.is_empty() || text.len() < 2 {
                    println!("⏭️ 跳过空转录结果");
                    return;
                }
                
                println!("📝 转录结果: '{}' (置信度: {:.2})", text, confidence);
                
                if confidence >= config.min_confidence {
                    // 发送流式转录事件 - 这是实时显示的文本
                    let _ = event_sender.send(TranscriptionEvent::StreamingTranscription {
                        text: text.clone(),
                        is_partial: confidence < 0.9, // 置信度低于0.9认为是部分结果
                        confidence,
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    });
                    
                    // 更新部分文本缓冲区
                    {
                        let mut buffer = partial_text_buffer.lock().unwrap();
                        if !buffer.is_empty() {
                            buffer.push(' ');
                        }
                        buffer.push_str(&text);
                        
                        // 限制缓冲区长度
                        if buffer.len() > config.max_partial_length {
                            let chars: Vec<char> = buffer.chars().collect();
                            let start_pos = chars.len().saturating_sub(config.max_partial_length);
                            *buffer = chars[start_pos..].iter().collect();
                        }
                    }
                    
                    // 如果置信度很高，也发送作为最终转录
                    if confidence >= 0.85 {
                        let _ = event_sender.send(TranscriptionEvent::FinalText {
                            text,
                            duration: std::time::Duration::from_millis(100), // 估计处理时间
                            timestamp: chrono::Utc::now().timestamp_millis() as u64,
                        });
                    }
                } else {
                    println!("⚠️ 转录置信度过低 ({:.2}), 跳过", confidence);
                }
            }
            Err(e) => {
                eprintln!("❌ 转录音频块失败: {}", e);
                let _ = event_sender.send(TranscriptionEvent::TranscriptionError {
                    error: e.to_string(),
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                });
            }
        }
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        *self.is_active.lock().unwrap()
    }
    
    /// 获取当前部分文本
    pub fn get_current_partial_text(&self) -> String {
        self.partial_text_buffer.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_streaming_transcriptor_creation() {
        let config = StreamingConfig::default();
        let transcription_service = Arc::new(
            TranscriptionService::new(reqwest::Client::new(), None)
        );
        
        let (transcriptor, mut receiver) = StreamingVoiceTranscriptor::new(
            config,
            transcription_service,
        );
        
        assert!(!transcriptor.is_running());
        assert_eq!(transcriptor.get_current_partial_text(), "");
    }
    
    #[tokio::test]
    async fn test_audio_chunk_processing() {
        let config = StreamingConfig::default();
        let transcription_service = Arc::new(
            TranscriptionService::new(reqwest::Client::new(), None)
        );
        
        let (mut transcriptor, mut receiver) = StreamingVoiceTranscriptor::new(
            config,
            transcription_service,
        );
        
        let (audio_sender, audio_receiver) = mpsc::unbounded_channel();
        
        // 发送测试音频块
        let test_chunk = AudioChunk {
            data: vec![0.1; 1600], // 100ms的16kHz音频
            sample_rate: 16000,
            timestamp: Instant::now(),
            chunk_id: 1,
        };
        
        audio_sender.send(test_chunk).unwrap();
        
        // 启动转录但不实际处理（由于我们还没有真正的转录服务）
        // 这里只是测试结构的正确性
        
        assert!(!transcriptor.is_running());
    }
}