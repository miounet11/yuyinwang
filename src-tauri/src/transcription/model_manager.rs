// Local Whisper Model Manager - Story 1.4
// Handles downloading, caching, and managing local Whisper models

use crate::database::models::{get_available_whisper_models, DownloadStatus, LocalModelInfo};
use crate::errors::{AppError, AppResult};
use chrono::Utc;
use directories::ProjectDirs;
use futures_util::StreamExt;
use parking_lot::Mutex;
use reqwest::Client;
use sha2::Digest;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelDownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub progress: f64,
    pub speed_bytes_per_sec: Option<u64>,
    pub eta_seconds: Option<u64>,
}

#[derive(Debug)]
pub enum ModelManagerEvent {
    DownloadStarted {
        model_id: String,
    },
    DownloadProgress {
        progress: ModelDownloadProgress,
    },
    DownloadCompleted {
        model_id: String,
        file_path: PathBuf,
    },
    DownloadFailed {
        model_id: String,
        error: String,
    },
    ModelVerified {
        model_id: String,
        is_valid: bool,
    },
    ModelLoaded {
        model_id: String,
    },
}

pub struct LocalModelManager {
    models_dir: PathBuf,
    http_client: Client,
    event_sender: Arc<Mutex<Option<UnboundedSender<ModelManagerEvent>>>>,
    pub database_manager: Arc<crate::database::DatabaseManager>,
}

impl LocalModelManager {
    pub fn new(
        database_manager: Arc<crate::database::DatabaseManager>,
        _data_dir: PathBuf,
        _app_handle: tauri::AppHandle,
    ) -> AppResult<Self> {
        let models_dir = Self::get_models_directory()?;

        // Create models directory if it doesn't exist
        if !models_dir.exists() {
            fs::create_dir_all(&models_dir).map_err(|e| {
                AppError::FileSystemError(format!("Failed to create models directory: {}", e))
            })?;
        }

        let http_client = Client::builder()
            .user_agent("RecordingKing/3.4.3")
            .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout
            .build()
            .map_err(|e| AppError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            models_dir,
            http_client,
            event_sender: Arc::new(Mutex::new(None)),
            database_manager,
        })
    }

    /// Get the standard models directory for the application
    fn get_models_directory() -> AppResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "RecordingKing", "RecordingKing") {
            let models_dir = proj_dirs.data_dir().join("models");
            Ok(models_dir)
        } else {
            // Fallback to home directory
            let home = std::env::var("HOME").map_err(|_| {
                AppError::FileSystemError("Cannot determine home directory".to_string())
            })?;
            Ok(PathBuf::from(home).join(".recording-king").join("models"))
        }
    }

    /// Subscribe to model manager events
    pub fn subscribe_events(&self) -> UnboundedReceiver<ModelManagerEvent> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        *self.event_sender.lock() = Some(tx);
        rx
    }

    /// Get list of all available models (both local and downloadable)
    pub async fn list_available_models(&self) -> AppResult<Vec<LocalModelInfo>> {
        let mut models: Vec<LocalModelInfo> = Vec::new();

        // Get models from database
        let db_models = self.database_manager.get_local_models().await?;

        // If database is empty, populate with default configurations
        if db_models.is_empty() {
            self.populate_default_models().await?;
            return self.database_manager.get_local_models().await;
        }

        Ok(db_models)
    }

    /// Get list of only downloaded/available local models
    pub async fn list_downloaded_models(&self) -> AppResult<Vec<LocalModelInfo>> {
        let all_models = self.list_available_models().await?;
        Ok(all_models
            .into_iter()
            .filter(|model| model.download_status == DownloadStatus::Downloaded)
            .collect())
    }

    /// Check if a model is downloaded and ready to use
    pub async fn is_model_available(&self, model_id: &str) -> AppResult<bool> {
        let model = self.database_manager.get_local_model(model_id).await?;
        match model {
            Some(model_info) => {
                if model_info.download_status == DownloadStatus::Downloaded {
                    // Verify file still exists
                    if let Some(file_path) = &model_info.file_path {
                        Ok(file_path.exists())
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }

    /// Download a specific model
    pub async fn download_model(&self, model_id: &str) -> AppResult<PathBuf> {
        println!("📥 Starting download for model: {}", model_id);

        // Find model configuration
        let available_models = get_available_whisper_models();
        let model_config = available_models
            .iter()
            .find(|m| m.model_id == model_id)
            .ok_or_else(|| AppError::ValidationError(format!("Unknown model: {}", model_id)))?;

        // Check if already downloaded
        if self.is_model_available(model_id).await? {
            let model_info = self
                .database_manager
                .get_local_model(model_id)
                .await?
                .unwrap();
            return Ok(model_info.file_path.unwrap());
        }

        // Send download started event
        self.send_event(ModelManagerEvent::DownloadStarted {
            model_id: model_id.to_string(),
        });

        // Update status to downloading
        self.update_model_status(model_id, DownloadStatus::Downloading { progress: 0.0 })
            .await?;

        // Prepare download
        let file_name = format!("{}.bin", model_id);
        let file_path = self.models_dir.join(&file_name);
        let temp_path = self.models_dir.join(format!("{}.tmp", file_name));

        // Start download
        let response = self
            .http_client
            .get(&model_config.download_url)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(format!("Failed to start download: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to create temp file: {}", e)))?;

        let mut stream = response.bytes_stream();
        let start_time = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| AppError::NetworkError(format!("Failed to read chunk: {}", e)))?;

            file.write_all(&chunk)
                .map_err(|e| AppError::FileSystemError(format!("Failed to write chunk: {}", e)))?;

            downloaded += chunk.len() as u64;

            // Calculate progress and send event
            let progress = if total_size > 0 {
                downloaded as f64 / total_size as f64
            } else {
                0.0
            };

            let elapsed = start_time.elapsed();
            let speed = if elapsed.as_secs() > 0 {
                Some(downloaded / elapsed.as_secs())
            } else {
                None
            };

            let eta = if let Some(speed) = speed {
                if speed > 0 && total_size > downloaded {
                    Some((total_size - downloaded) / speed)
                } else {
                    None
                }
            } else {
                None
            };

            self.send_event(ModelManagerEvent::DownloadProgress {
                progress: ModelDownloadProgress {
                    model_id: model_id.to_string(),
                    downloaded_bytes: downloaded,
                    total_bytes: total_size,
                    progress,
                    speed_bytes_per_sec: speed,
                    eta_seconds: eta,
                },
            });

            // Update database progress
            self.update_model_status(model_id, DownloadStatus::Downloading { progress })
                .await?;
        }

        // Move temp file to final location
        fs::rename(&temp_path, &file_path).map_err(|e| {
            AppError::FileSystemError(format!("Failed to move downloaded file: {}", e))
        })?;

        // Update model info in database
        self.update_model_after_download(model_id, &file_path, downloaded)
            .await?;

        // Verify model integrity
        let is_valid = self.verify_model_integrity(model_id, &file_path).await?;

        if is_valid {
            self.update_model_status(model_id, DownloadStatus::Downloaded)
                .await?;
            self.send_event(ModelManagerEvent::DownloadCompleted {
                model_id: model_id.to_string(),
                file_path: file_path.clone(),
            });
            println!("✅ Model {} downloaded successfully", model_id);
            Ok(file_path)
        } else {
            fs::remove_file(&file_path).ok(); // Clean up invalid file
            self.update_model_status(model_id, DownloadStatus::Corrupted)
                .await?;
            self.send_event(ModelManagerEvent::DownloadFailed {
                model_id: model_id.to_string(),
                error: "Model verification failed".to_string(),
            });
            Err(AppError::ValidationError(
                "Downloaded model failed verification".to_string(),
            ))
        }
    }

    /// Verify model file integrity
    pub async fn verify_model_integrity(
        &self,
        model_id: &str,
        file_path: &Path,
    ) -> AppResult<bool> {
        if !file_path.exists() {
            return Ok(false);
        }

        // For now, just check if file is not empty and is reasonable size
        let metadata = fs::metadata(file_path).map_err(|e| {
            AppError::FileSystemError(format!("Failed to read file metadata: {}", e))
        })?;

        let file_size = metadata.len();

        // Basic sanity check - model should be at least 10MB
        if file_size < 10_000_000 {
            return Ok(false);
        }

        // TODO: Implement SHA256 checksum verification when checksums are available
        // For now, we'll consider the file valid if it's the right size

        let available_models = get_available_whisper_models();
        let model_config = available_models.iter().find(|m| m.model_id == model_id);

        if let Some(config) = model_config {
            let expected_size = config.size_mb as u64 * 1_000_000;
            let size_diff_percent = if expected_size > 0 {
                ((file_size as f64 - expected_size as f64) / expected_size as f64).abs()
            } else {
                1.0
            };

            // Allow 10% variance in file size
            let is_valid = size_diff_percent < 0.1;

            self.send_event(ModelManagerEvent::ModelVerified {
                model_id: model_id.to_string(),
                is_valid,
            });

            Ok(is_valid)
        } else {
            Ok(true) // Unknown model, assume valid
        }
    }

    /// Delete a downloaded model
    pub async fn delete_model(&self, model_id: &str) -> AppResult<()> {
        let model_info = self.database_manager.get_local_model(model_id).await?;

        if let Some(model) = model_info {
            if let Some(file_path) = &model.file_path {
                if file_path.exists() {
                    fs::remove_file(file_path).map_err(|e| {
                        AppError::FileSystemError(format!("Failed to delete model file: {}", e))
                    })?;
                }
            }

            // Update status in database
            self.update_model_status(model_id, DownloadStatus::NotDownloaded)
                .await?;

            // Clear file path
            self.database_manager
                .update_model_file_path(model_id, None)
                .await?;

            println!("🗑️ Model {} deleted successfully", model_id);
        }

        Ok(())
    }

    /// Get storage info for all models
    pub async fn get_storage_info(&self) -> AppResult<serde_json::Value> {
        let models = self.list_available_models().await?;
        let mut total_downloaded_size = 0u64;
        let mut total_available_size = 0u64;

        for model in &models {
            total_available_size += model.size_bytes;
            if model.download_status == DownloadStatus::Downloaded {
                total_downloaded_size += model.size_bytes;
            }
        }

        // Get free space in models directory
        let free_space = match fs2::free_space(&self.models_dir) {
            Ok(free) => free,
            Err(_) => 0,
        };

        Ok(serde_json::json!({
            "models_directory": self.models_dir,
            "total_models": models.len(),
            "downloaded_models": models.iter().filter(|m| m.download_status == DownloadStatus::Downloaded).count(),
            "total_downloaded_size_bytes": total_downloaded_size,
            "total_available_size_bytes": total_available_size,
            "free_space_bytes": free_space,
            "free_space_mb": free_space / 1_000_000,
        }))
    }

    // Private helper methods

    fn send_event(&self, event: ModelManagerEvent) {
        if let Some(sender) = self.event_sender.lock().as_ref() {
            let _ = sender.send(event);
        }
    }

    async fn update_model_status(&self, model_id: &str, status: DownloadStatus) -> AppResult<()> {
        self.database_manager
            .update_model_status(model_id, status)
            .await
    }

    async fn update_model_after_download(
        &self,
        model_id: &str,
        file_path: &Path,
        size_bytes: u64,
    ) -> AppResult<()> {
        self.database_manager
            .update_model_file_path(model_id, Some(file_path.to_path_buf()))
            .await?;
        self.database_manager
            .update_model_size(model_id, size_bytes)
            .await?;
        self.database_manager
            .update_model_last_verified(model_id, Some(Utc::now()))
            .await?;
        Ok(())
    }

    async fn populate_default_models(&self) -> AppResult<()> {
        let available_models = get_available_whisper_models();
        for model_config in &available_models {
            let model_info = LocalModelInfo {
                model_id: model_config.model_id.clone(),
                model_name: model_config.model_id.clone(),
                file_path: None,
                size_bytes: model_config.size_mb as u64 * 1_000_000,
                download_status: DownloadStatus::NotDownloaded,
                accuracy_score: None,
                performance_rating: None,
                supported_languages: model_config.languages.clone(),
                created_at: Utc::now(),
                last_verified: None,
                metadata: serde_json::json!({
                    "description": model_config.description,
                    "recommended_use": model_config.recommended_use,
                    "source": "huggingface",
                    "download_url": model_config.download_url
                }),
            };

            self.database_manager
                .insert_local_model(&model_info)
                .await?;
        }

        Ok(())
    }
}

// Additional required dependency for free space check
// Add to Cargo.toml: fs2 = "0.4"
