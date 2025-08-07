use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub openai_api_key: Option<String>,
    pub deepgram_api_key: Option<String>,
    pub mistral_api_key: Option<String>,
    pub elevenlabs_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub model: String,
    pub duration: u64,
}

#[derive(Debug)]
pub struct TranscriptionService {
    client: Client,
    config: ApiConfig,
}

impl TranscriptionService {
    pub fn new(config: ApiConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    // OpenAI Whisper API 集成
    pub async fn transcribe_with_openai(&self, audio_file: &PathBuf) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        let api_key = self.config.openai_api_key.as_ref()
            .ok_or("OpenAI API key not configured")?;

        println!("🤖 调用 OpenAI Whisper API...");

        let audio_data = fs::read(audio_file).await?;
        let file_name = audio_file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav");

        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(audio_data)
                .file_name(file_name.to_string())
                .mime_str("audio/wav")?)
            .text("model", "whisper-1")
            .text("response_format", "verbose_json");

        let response = self.client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            
            Ok(TranscriptionResult {
                text: result["text"].as_str().unwrap_or("").to_string(),
                confidence: 0.95, // OpenAI 不提供置信度，使用默认值
                model: "openai-whisper".to_string(),
                duration: result["duration"].as_u64().unwrap_or(0),
            })
        } else {
            let error_text = response.text().await?;
            Err(format!("OpenAI API error: {}", error_text).into())
        }
    }

    // Deepgram Nova-3 API 集成
    pub async fn transcribe_with_deepgram(&self, audio_file: &PathBuf) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        let api_key = self.config.deepgram_api_key.as_ref()
            .ok_or("Deepgram API key not configured")?;

        println!("⚡ 调用 Deepgram Nova-3 API...");

        let audio_data = fs::read(audio_file).await?;

        let response = self.client
            .post("https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true")
            .header("Authorization", format!("Token {}", api_key))
            .header("Content-Type", "audio/wav")
            .body(audio_data)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            
            let transcript = result["results"]["channels"][0]["alternatives"][0]["transcript"]
                .as_str().unwrap_or("").to_string();
            
            let confidence = result["results"]["channels"][0]["alternatives"][0]["confidence"]
                .as_f64().unwrap_or(0.95) as f32;

            Ok(TranscriptionResult {
                text: transcript,
                confidence,
                model: "deepgram-nova-3".to_string(),
                duration: result["metadata"]["duration"].as_u64().unwrap_or(0),
            })
        } else {
            let error_text = response.text().await?;
            Err(format!("Deepgram API error: {}", error_text).into())
        }
    }

    // Mistral Voxtral Mini API 集成
    pub async fn transcribe_with_mistral(&self, audio_file: &PathBuf) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        let api_key = self.config.mistral_api_key.as_ref()
            .ok_or("Mistral API key not configured")?;

        println!("🌟 调用 Mistral Voxtral Mini API...");

        // 注意：这里使用假设的Mistral API端点，实际需要根据Mistral文档调整
        let audio_data = fs::read(audio_file).await?;

        let mut form_data = HashMap::new();
        form_data.insert("model", "voxtral-mini");
        
        let response = self.client
            .post("https://api.mistral.ai/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "multipart/form-data")
            .json(&form_data)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            
            Ok(TranscriptionResult {
                text: result["text"].as_str().unwrap_or("").to_string(),
                confidence: result["confidence"].as_f64().unwrap_or(0.92) as f32,
                model: "mistral-voxtral".to_string(),
                duration: 0, // 需要根据实际API响应调整
            })
        } else {
            let error_text = response.text().await?;
            Err(format!("Mistral API error: {}", error_text).into())
        }
    }

    // ElevenLabs Scribe API 集成
    pub async fn transcribe_with_elevenlabs(&self, audio_file: &PathBuf) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        let api_key = self.config.elevenlabs_api_key.as_ref()
            .ok_or("ElevenLabs API key not configured")?;

        println!("🔊 调用 ElevenLabs Scribe API...");

        let audio_data = fs::read(audio_file).await?;

        let response = self.client
            .post("https://api.elevenlabs.io/v1/speech-to-text")
            .header("xi-api-key", api_key)
            .header("Content-Type", "audio/wav")
            .body(audio_data)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            
            Ok(TranscriptionResult {
                text: result["text"].as_str().unwrap_or("").to_string(),
                confidence: result["confidence"].as_f64().unwrap_or(0.94) as f32,
                model: "elevenlabs-scribe".to_string(),
                duration: 0, // 需要根据实际API响应调整
            })
        } else {
            let error_text = response.text().await?;
            Err(format!("ElevenLabs API error: {}", error_text).into())
        }
    }

    // 统一转录接口
    pub async fn transcribe(&self, audio_file: &PathBuf, model: &str) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        match model {
            "gpt-4o-mini" | "openai-whisper" => self.transcribe_with_openai(audio_file).await,
            "nova-3" | "deepgram-nova" => self.transcribe_with_deepgram(audio_file).await,
            "voxtral-mini" | "mistral-voxtral" => self.transcribe_with_mistral(audio_file).await,
            "elevenlabs" | "elevenlabs-scribe" => self.transcribe_with_elevenlabs(audio_file).await,
            _ => Err(format!("Unsupported model: {}", model).into()),
        }
    }

    // 从音频数据直接转录（用于实时转录）
    pub async fn transcribe_from_bytes(&self, audio_data: &[u8], model: &str) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        // 创建临时文件
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("temp_audio_{}.wav", uuid::Uuid::new_v4()));
        
        fs::write(&temp_file, audio_data).await?;
        
        let result = self.transcribe(&temp_file, model).await;
        
        // 清理临时文件
        let _ = fs::remove_file(&temp_file).await;
        
        result
    }
}