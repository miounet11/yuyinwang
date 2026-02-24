use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Transcription error: {0}")]
    Transcription(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Audio encoding error: {0}")]
    Hound(#[from] hound::Error),

    #[error("Audio device error: {0}")]
    CpalDevices(#[from] cpal::DevicesError),

    #[error("Audio build error: {0}")]
    CpalBuild(#[from] cpal::BuildStreamError),

    #[error("Audio play error: {0}")]
    CpalPlay(#[from] cpal::PlayStreamError),

    #[error("Audio config error: {0}")]
    CpalConfig(#[from] cpal::DefaultStreamConfigError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
