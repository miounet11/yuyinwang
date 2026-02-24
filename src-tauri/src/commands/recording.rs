use crate::core::{error::Result, transcription::TranscriptionService, types::*};
use crate::services::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<()> {
    state.start_recording().await?;
    Ok(())
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    model: String,
) -> Result<TranscriptionResult> {
    let samples = state.stop_recording().await?;

    let settings = {
        let mut s = state.settings.lock().clone();
        s.selected_model = model.clone();
        s
    };

    let mut service = TranscriptionService::new(settings);
    if let Some(dir) = app.path_resolver().app_data_dir() {
        service = service.with_app_data_dir(dir);
    }
    let result = service.transcribe_samples(&samples, 16000).await?;

    let entry = TranscriptionEntry {
        id: uuid::Uuid::new_v4().to_string(),
        text: result.text.clone(),
        timestamp: chrono::Utc::now().timestamp(),
        duration: result.duration.unwrap_or(0.0),
        model,
        confidence: 0.95,
        audio_file_path: None,
    };
    state.database.save_transcription(&entry)?;

    Ok(result)
}

#[tauri::command]
pub fn get_recording_state(state: State<'_, AppState>) -> Result<bool> {
    Ok(state.is_recording())
}

#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>> {
    crate::core::audio::list_audio_devices()
}

#[tauri::command]
pub async fn transcribe_file(
    file_path: String,
    model: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<TranscriptionResult> {
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(crate::core::error::AppError::Other(format!(
            "文件不存在: {}",
            file_path
        )));
    }

    let settings = {
        let mut s = state.settings.lock().clone();
        s.selected_model = model.clone();
        s
    };

    let mut service = TranscriptionService::new(settings);
    if let Some(dir) = app.path_resolver().app_data_dir() {
        service = service.with_app_data_dir(dir);
    }
    let result = service.transcribe_audio(path).await?;

    let entry = TranscriptionEntry {
        id: uuid::Uuid::new_v4().to_string(),
        text: result.text.clone(),
        timestamp: chrono::Utc::now().timestamp(),
        duration: result.duration.unwrap_or(0.0),
        model,
        confidence: 0.95,
        audio_file_path: Some(file_path),
    };
    state.database.save_transcription(&entry)?;

    Ok(result)
}
