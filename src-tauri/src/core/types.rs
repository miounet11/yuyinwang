use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    pub device_id: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            device_id: None,
            sample_rate: 16000,
            channels: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub timestamp: i64,
    pub duration: f64,
    pub model: String,
    pub confidence: f32,
    pub audio_file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    pub openai_api_key: Option<String>,
    pub luyin_token: Option<String>,
    pub selected_model: String,
    pub auto_inject: bool,
    pub inject_delay_ms: u64,
    pub shortcut_key: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            luyin_token: None,
            selected_model: "luyin-free".to_string(),
            auto_inject: false,
            inject_delay_ms: 100,
            shortcut_key: None,
        }
    }
}

/// 模型提供商分类
#[derive(Debug, Clone, PartialEq)]
pub enum ModelProvider {
    LuYinWang,
    OpenAI,
    Deepgram,
    Mistral,
    ElevenLabs,
    LocalWhisper,
}

impl ModelProvider {
    /// 根据 model id 判断提供商
    pub fn from_model_id(id: &str) -> Self {
        match id {
            "luyin-free" => Self::LuYinWang,
            "gpt-4o-mini-transcribe" => Self::OpenAI,
            "deepgram-nova3" => Self::Deepgram,
            "voxtral-mini" => Self::Mistral,
            "elevenlabs-scribe" => Self::ElevenLabs,
            s if s.starts_with("whisper-") => Self::LocalWhisper,
            _ => Self::LuYinWang, // fallback
        }
    }

    /// 该提供商需要哪个 key
    pub fn required_key(&self) -> &'static str {
        match self {
            Self::LuYinWang => "luyin_token",
            Self::OpenAI => "openai_api_key",
            Self::Deepgram => "openai_api_key",
            Self::Mistral => "openai_api_key",
            Self::ElevenLabs => "openai_api_key",
            Self::LocalWhisper => "",
        }
    }
}
