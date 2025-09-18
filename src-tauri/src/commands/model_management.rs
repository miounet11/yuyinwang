// Story 1.4: Tauri Commands for Local Whisper Model Management

use crate::database::models::DownloadStatus;
use crate::transcription::{GPUCapabilities, LocalModelManager, ModelPriority, WhisperTranscriber};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use tauri::{State, Window};
use tokio::sync::Mutex as TokioMutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub model_name: String,
    pub size_mb: u32,
    pub download_status: String,
    pub download_progress: Option<f64>,
    pub is_downloaded: bool,
    pub file_path: Option<String>,
    pub description: String,
    pub supported_languages: Vec<String>,
    pub accuracy_score: Option<f64>,
    pub performance_rating: Option<f64>,
    pub recommended_use: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub models_directory: String,
    pub total_models: usize,
    pub downloaded_models: usize,
    pub total_downloaded_size_mb: u32,
    pub total_available_size_mb: u32,
    pub free_space_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model_id: String,
    pub reason: String,
    pub priority_match: String,
    pub estimated_performance: String,
}

/// List all available Whisper models (both local and downloadable)
#[tauri::command]
pub async fn list_whisper_models(
    model_manager: State<'_, Arc<LocalModelManager>>,
) -> Result<Vec<ModelInfo>, String> {
    println!("🔍 Listing available Whisper models...");

    let models = model_manager
        .list_available_models()
        .await
        .map_err(|e| format!("Failed to list models: {}", e))?;

    let model_infos: Vec<ModelInfo> = models
        .into_iter()
        .map(|model| {
            let (download_status, download_progress, is_downloaded) = match &model.download_status {
                DownloadStatus::NotDownloaded => ("not_downloaded".to_string(), None, false),
                DownloadStatus::Downloading { progress } => {
                    ("downloading".to_string(), Some(*progress), false)
                }
                DownloadStatus::Downloaded => ("downloaded".to_string(), Some(1.0), true),
                DownloadStatus::Corrupted => ("corrupted".to_string(), None, false),
                DownloadStatus::UpdateAvailable => {
                    ("update_available".to_string(), Some(1.0), true)
                }
            };

            let description = model
                .metadata
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("Whisper model for speech-to-text transcription")
                .to_string();

            let recommended_use = model
                .metadata
                .get("recommended_use")
                .and_then(|v| v.as_str())
                .unwrap_or("General purpose transcription")
                .to_string();

            ModelInfo {
                model_id: model.model_id.clone(),
                model_name: model.model_name.clone(),
                size_mb: (model.size_bytes / 1_000_000) as u32,
                download_status,
                download_progress,
                is_downloaded,
                file_path: model.file_path.map(|p| p.to_string_lossy().to_string()),
                description,
                supported_languages: model.supported_languages,
                accuracy_score: model.accuracy_score,
                performance_rating: model.performance_rating,
                recommended_use,
            }
        })
        .collect();

    println!("✅ Found {} models", model_infos.len());
    Ok(model_infos)
}

/// Download a specific Whisper model
#[tauri::command]
pub async fn download_whisper_model(
    model_id: String,
    model_manager: State<'_, Arc<LocalModelManager>>,
    window: Window,
) -> Result<String, String> {
    println!("📥 Starting download for model: {}", model_id);

    // Subscribe to download events
    let mut event_rx = model_manager.subscribe_events();
    let window_clone = window.clone();
    let model_id_clone = model_id.clone();

    // Start event forwarding task
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                crate::transcription::model_manager::ModelManagerEvent::DownloadProgress {
                    progress,
                } => {
                    if progress.model_id == model_id_clone {
                        let _ = window_clone.emit("model_download_progress", &progress);
                    }
                }
                crate::transcription::model_manager::ModelManagerEvent::DownloadCompleted {
                    model_id: completed_id,
                    ..
                } => {
                    if completed_id == model_id_clone {
                        let _ = window_clone.emit("model_download_completed", &completed_id);
                        break;
                    }
                }
                crate::transcription::model_manager::ModelManagerEvent::DownloadFailed {
                    model_id: failed_id,
                    error,
                } => {
                    if failed_id == model_id_clone {
                        let _ = window_clone.emit(
                            "model_download_failed",
                            &serde_json::json!({
                                "model_id": failed_id,
                                "error": error
                            }),
                        );
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    // Start the download
    match model_manager.download_model(&model_id).await {
        Ok(file_path) => {
            let success_message = format!(
                "Model {} downloaded successfully to {:?}",
                model_id, file_path
            );
            println!("✅ {}", success_message);
            Ok(success_message)
        }
        Err(e) => {
            let error_message = format!("Failed to download model {}: {}", model_id, e);
            eprintln!("❌ {}", error_message);
            Err(error_message)
        }
    }
}

/// Delete a downloaded model
#[tauri::command]
pub async fn delete_whisper_model(
    model_id: String,
    model_manager: State<'_, Arc<LocalModelManager>>,
) -> Result<String, String> {
    println!("🗑️ Deleting model: {}", model_id);

    model_manager
        .delete_model(&model_id)
        .await
        .map_err(|e| format!("Failed to delete model: {}", e))?;

    let success_message = format!("Model {} deleted successfully", model_id);
    println!("✅ {}", success_message);
    Ok(success_message)
}

/// Verify integrity of a downloaded model
#[tauri::command]
pub async fn verify_whisper_model(
    model_id: String,
    model_manager: State<'_, Arc<LocalModelManager>>,
) -> Result<bool, String> {
    println!("🔍 Verifying model: {}", model_id);

    // Get model info
    let model_info = model_manager
        .database_manager
        .get_local_model(&model_id)
        .await
        .map_err(|e| format!("Failed to get model info: {}", e))?;

    if let Some(model) = model_info {
        if let Some(file_path) = model.file_path {
            let is_valid = model_manager
                .verify_model_integrity(&model_id, &file_path)
                .await
                .map_err(|e| format!("Failed to verify model: {}", e))?;

            println!(
                "✅ Model {} verification: {}",
                model_id,
                if is_valid { "PASSED" } else { "FAILED" }
            );
            Ok(is_valid)
        } else {
            Ok(false)
        }
    } else {
        Err(format!("Model {} not found", model_id))
    }
}

/// Get storage information for all models
#[tauri::command]
pub async fn get_models_storage_info(
    model_manager: State<'_, Arc<LocalModelManager>>,
) -> Result<StorageInfo, String> {
    println!("📊 Getting models storage information...");

    let storage_data = model_manager
        .get_storage_info()
        .await
        .map_err(|e| format!("Failed to get storage info: {}", e))?;

    let storage_info = StorageInfo {
        models_directory: storage_data["models_directory"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        total_models: storage_data["total_models"].as_u64().unwrap_or(0) as usize,
        downloaded_models: storage_data["downloaded_models"].as_u64().unwrap_or(0) as usize,
        total_downloaded_size_mb: (storage_data["total_downloaded_size_bytes"]
            .as_u64()
            .unwrap_or(0)
            / 1_000_000) as u32,
        total_available_size_mb: (storage_data["total_available_size_bytes"]
            .as_u64()
            .unwrap_or(0)
            / 1_000_000) as u32,
        free_space_mb: storage_data["free_space_mb"].as_u64().unwrap_or(0),
    };

    println!("✅ Storage info retrieved");
    Ok(storage_info)
}

/// Detect GPU capabilities for Whisper acceleration
#[tauri::command]
pub async fn detect_gpu_capabilities(
    whisper_transcriber: State<'_, Arc<TokioMutex<WhisperTranscriber>>>,
) -> Result<GPUCapabilities, String> {
    println!("🔍 Detecting GPU capabilities...");

    let capabilities = {
        let transcriber = whisper_transcriber.lock().await;
        transcriber
            .initialize_gpu()
            .await
            .map_err(|e| format!("Failed to detect GPU: {}", e))?
    };

    println!("✅ GPU detection completed");
    Ok(capabilities)
}

/// Get current GPU capabilities (cached)
#[tauri::command]
pub async fn get_gpu_capabilities(
    whisper_transcriber: State<'_, Arc<TokioMutex<WhisperTranscriber>>>,
) -> Result<Option<GPUCapabilities>, String> {
    let transcriber = whisper_transcriber.lock().await;
    Ok(transcriber.get_gpu_capabilities())
}

/// Recommend best model based on requirements
#[tauri::command]
pub async fn recommend_whisper_model(
    language: Option<String>,
    priority: String, // "speed", "accuracy", or "balanced"
    whisper_transcriber: State<'_, Arc<TokioMutex<WhisperTranscriber>>>,
) -> Result<Option<ModelRecommendation>, String> {
    println!(
        "🎯 Recommending model for language: {:?}, priority: {}",
        language, priority
    );

    let model_priority = match priority.as_str() {
        "speed" => ModelPriority::Speed,
        "accuracy" => ModelPriority::Accuracy,
        "balanced" => ModelPriority::Balanced,
        _ => ModelPriority::Balanced,
    };

    let transcriber = whisper_transcriber.lock().await;
    let recommended_model_id = transcriber
        .recommend_best_local_model(language.as_deref(), model_priority)
        .await
        .map_err(|e| format!("Failed to get recommendation: {}", e))?;

    if let Some(model_id) = recommended_model_id {
        let reason = match model_priority {
            ModelPriority::Speed => "Optimized for fast transcription with reasonable accuracy",
            ModelPriority::Accuracy => "Provides highest accuracy for professional use",
            ModelPriority::Balanced => "Good balance between speed and accuracy",
        };

        let estimated_performance = match model_priority {
            ModelPriority::Speed => "2-5x faster than large models",
            ModelPriority::Accuracy => "Best possible accuracy, slower processing",
            ModelPriority::Balanced => "Moderate speed with good accuracy",
        };

        let recommendation = ModelRecommendation {
            model_id,
            reason: reason.to_string(),
            priority_match: priority,
            estimated_performance: estimated_performance.to_string(),
        };

        println!("✅ Recommended model: {}", recommendation.model_id);
        Ok(Some(recommendation))
    } else {
        println!("⚠️ No suitable model found for the given criteria");
        Ok(None)
    }
}

/// Check if a specific model is available locally
#[tauri::command]
pub async fn is_model_available(
    model_id: String,
    model_manager: State<'_, Arc<LocalModelManager>>,
) -> Result<bool, String> {
    let is_available = model_manager
        .is_model_available(&model_id)
        .await
        .map_err(|e| format!("Failed to check model availability: {}", e))?;

    Ok(is_available)
}

/// Load a specific model for use
#[tauri::command]
pub async fn load_whisper_model(
    model_id: String,
    whisper_transcriber: State<'_, Arc<TokioMutex<WhisperTranscriber>>>,
) -> Result<String, String> {
    println!("🔄 Loading model: {}", model_id);

    let transcriber = whisper_transcriber.lock().await;
    transcriber
        .load_local_model(&model_id)
        .await
        .map_err(|e| format!("Failed to load model: {}", e))?;

    let success_message = format!("Model {} loaded successfully", model_id);
    println!("✅ {}", success_message);
    Ok(success_message)
}
