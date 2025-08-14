use serde::{Deserialize, Serialize};

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