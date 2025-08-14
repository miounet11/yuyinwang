use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// 实时转录事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionEvent {
    pub event_type: String,
    pub chunk_id: u64,
    pub text: Option<String>,
    pub confidence: Option<f64>,
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub timestamp: i64,
    pub duration: f64,
    pub model: String,
    pub confidence: f64,
    pub audio_file_path: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub tags: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
    pub is_default: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPrompt {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agent_type: String,
    pub prompt_text: String,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent_type: String,
    pub input_text: String,
    pub prompt_id: Option<String>,
    pub additional_context: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub output_text: String,
    pub agent_type: String,
    pub processing_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: Option<f64>,
    pub duration: Option<std::time::Duration>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    pub device_id: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_seconds: Option<u64>,
    /// 缓冲区持续时间（秒），动态计算缓冲区大小
    pub buffer_duration: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    pub model_name: String,
    pub language: Option<String>,
    pub temperature: Option<f32>,
    pub is_local: bool,
    pub api_endpoint: Option<String>,
}