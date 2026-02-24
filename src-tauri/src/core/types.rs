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
    // Existing fields
    pub openai_api_key: Option<String>,
    pub luyin_token: Option<String>,
    pub selected_model: String,
    pub auto_inject: bool,
    pub inject_delay_ms: u64,
    pub shortcut_key: Option<String>,

    // New: Interface settings
    #[serde(default = "default_display_style")]
    pub display_style: String,
    #[serde(default = "default_appearance")]
    pub appearance: String,
    #[serde(default = "default_ui_language")]
    pub ui_language: String,

    // New: Behavior settings
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default = "default_show_in_dock")]
    pub show_in_dock: bool,
    #[serde(default = "default_show_in_menu_bar")]
    pub show_in_menu_bar: bool,
    #[serde(default = "default_esc_to_cancel")]
    pub esc_to_cancel: bool,

    // New: Shortcut settings
    #[serde(default = "default_shortcut_preset")]
    pub shortcut_preset: String,
    #[serde(default)]
    pub custom_shortcut: Option<CustomShortcut>,
    #[serde(default = "default_activation_mode")]
    pub activation_mode: String,

    // New: Microphone priority
    #[serde(default)]
    pub microphone_priority: Vec<String>,

    // New: Onboarding
    #[serde(default)]
    pub onboarding_complete: bool,

    // New: Word replacements
    #[serde(default)]
    pub word_replacements: Vec<WordReplacement>,

    // New: Transcription settings per model
    #[serde(default = "default_transcription_language")]
    pub transcription_language: String,
    #[serde(default)]
    pub transcription_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomShortcut {
    pub r#type: String,
    pub modifiers: Vec<String>,
    pub key: String,
    pub display_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WordReplacement {
    pub id: String,
    pub from: String,
    pub to: String,
    pub enabled: bool,
}

// Default value functions for backward compatibility
fn default_display_style() -> String {
    "panel".to_string()
}

fn default_appearance() -> String {
    "system".to_string()
}

fn default_ui_language() -> String {
    "system".to_string()
}

fn default_show_in_dock() -> bool {
    true
}

fn default_show_in_menu_bar() -> bool {
    true
}

fn default_esc_to_cancel() -> bool {
    true
}

fn default_shortcut_preset() -> String {
    "none".to_string()
}

fn default_activation_mode() -> String {
    "hold-or-toggle".to_string()
}

fn default_transcription_language() -> String {
    "auto".to_string()
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
            display_style: default_display_style(),
            appearance: default_appearance(),
            ui_language: default_ui_language(),
            launch_at_login: false,
            show_in_dock: default_show_in_dock(),
            show_in_menu_bar: default_show_in_menu_bar(),
            esc_to_cancel: default_esc_to_cancel(),
            shortcut_preset: default_shortcut_preset(),
            custom_shortcut: None,
            activation_mode: default_activation_mode(),
            microphone_priority: vec![],
            onboarding_complete: false,
            word_replacements: vec![],
            transcription_language: default_transcription_language(),
            transcription_prompt: String::new(),
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
