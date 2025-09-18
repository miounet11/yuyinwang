use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    pub id: i64,
    pub model_name: String,
    pub usage_count: i64,
    pub total_duration: f64,
    pub average_confidence: f64,
    pub last_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entries: Vec<crate::types::TranscriptionEntry>,
    pub total_count: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_transcriptions: i64,
    pub total_duration: f64,
    pub most_used_model: Option<String>,
    pub average_confidence: f64,
    pub database_size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub model: Option<String>,
    pub min_confidence: Option<f64>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub min_duration: Option<f64>,
    pub max_duration: Option<f64>,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            model: None,
            min_confidence: None,
            start_date: None,
            end_date: None,
            tags: None,
            min_duration: None,
            max_duration: None,
        }
    }
}

// Local Whisper Model Management Structures - Story 1.4

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    NotDownloaded,
    Downloading { progress: f64 },
    Downloaded,
    Corrupted,
    UpdateAvailable,
}

impl Default for DownloadStatus {
    fn default() -> Self {
        DownloadStatus::NotDownloaded
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    pub model_id: String,
    pub model_name: String, // "whisper-small", "whisper-medium", etc.
    pub file_path: Option<PathBuf>,
    pub size_bytes: u64,
    pub download_status: DownloadStatus,
    pub accuracy_score: Option<f64>,     // Estimated accuracy
    pub performance_rating: Option<f64>, // Processing speed rating
    pub supported_languages: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_verified: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value, // Additional model metadata
}

impl Default for LocalModelInfo {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            model_name: String::new(),
            file_path: None,
            size_bytes: 0,
            download_status: DownloadStatus::default(),
            accuracy_score: None,
            performance_rating: None,
            supported_languages: vec!["en".to_string()], // Default to English
            created_at: Utc::now(),
            last_verified: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub model_id: String,
    pub average_processing_time: Duration,
    pub gpu_acceleration_used: bool,
    pub memory_usage_mb: u32,
    pub cpu_usage_percent: f64,
    pub accuracy_samples: Vec<f64>,
    pub last_benchmark: DateTime<Utc>,
}

impl Default for ModelPerformanceMetrics {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            average_processing_time: Duration::from_secs(0),
            gpu_acceleration_used: false,
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
            accuracy_samples: Vec::new(),
            last_benchmark: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModelConfig {
    pub model_id: String,
    pub download_url: String,
    pub sha256_checksum: String,
    pub size_mb: u32,
    pub languages: Vec<String>,
    pub description: String,
    pub recommended_use: String,
}

// Constants for available Whisper models
pub fn get_available_whisper_models() -> Vec<WhisperModelConfig> {
    vec![
        WhisperModelConfig {
            model_id: "whisper-tiny".to_string(),
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
                .to_string(),
            sha256_checksum: "".to_string(), // To be filled with actual checksums
            size_mb: 39,
            languages: vec![
                "en".to_string(),
                "zh".to_string(),
                "es".to_string(),
                "fr".to_string(),
            ],
            description: "Fastest model, lower accuracy".to_string(),
            recommended_use: "Real-time transcription, testing".to_string(),
        },
        WhisperModelConfig {
            model_id: "whisper-base".to_string(),
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
                .to_string(),
            sha256_checksum: "".to_string(),
            size_mb: 142,
            languages: vec![
                "en".to_string(),
                "zh".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
            ],
            description: "Good balance of speed and accuracy".to_string(),
            recommended_use: "General purpose transcription".to_string(),
        },
        WhisperModelConfig {
            model_id: "whisper-small".to_string(),
            download_url:
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
                    .to_string(),
            sha256_checksum: "".to_string(),
            size_mb: 244,
            languages: vec![
                "en".to_string(),
                "zh".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "ja".to_string(),
            ],
            description: "Better accuracy, moderate speed".to_string(),
            recommended_use: "High quality transcription".to_string(),
        },
        WhisperModelConfig {
            model_id: "whisper-medium".to_string(),
            download_url:
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
                    .to_string(),
            sha256_checksum: "".to_string(),
            size_mb: 769,
            languages: vec![
                "en".to_string(),
                "zh".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "ja".to_string(),
                "ko".to_string(),
            ],
            description: "High accuracy, slower processing".to_string(),
            recommended_use: "Professional transcription, accuracy critical".to_string(),
        },
        WhisperModelConfig {
            model_id: "whisper-large".to_string(),
            download_url:
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                    .to_string(),
            sha256_checksum: "".to_string(),
            size_mb: 1550,
            languages: vec![
                "en".to_string(),
                "zh".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "ja".to_string(),
                "ko".to_string(),
                "ar".to_string(),
                "ru".to_string(),
            ],
            description: "Highest accuracy, slowest processing".to_string(),
            recommended_use: "Maximum accuracy needs, offline professional use".to_string(),
        },
    ]
}
