use crate::core::{error::Result, local_whisper};
use tauri::{AppHandle, Manager};

#[derive(serde::Serialize)]
pub struct ModelStatus {
    pub model_id: String,
    pub downloaded: bool,
}

/// 获取所有本地模型的下载状态
#[tauri::command]
pub fn get_local_model_status(app: AppHandle) -> Result<Vec<ModelStatus>> {
    let app_data_dir = app
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| crate::core::error::AppError::Other("无法获取应用数据目录".into()))?;

    let all_models = [
        "whisper-tiny",
        "whisper-base",
        "whisper-small",
        "whisper-medium",
        "whisper-large-v3",
        "whisper-large-v3-turbo",
    ];

    Ok(all_models
        .iter()
        .map(|id| ModelStatus {
            model_id: id.to_string(),
            downloaded: local_whisper::is_model_downloaded(&app_data_dir, id),
        })
        .collect())
}

/// 下载本地模型（通过事件推送进度）
#[tauri::command]
pub async fn download_local_model(app: AppHandle, model_id: String) -> Result<String> {
    let app_data_dir = app
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| crate::core::error::AppError::Other("无法获取应用数据目录".into()))?;

    let app_clone = app.clone();
    let model_id_clone = model_id.clone();

    let model_path = local_whisper::download_model(&app_data_dir, &model_id, move |progress| {
        let _ = app_clone.emit_all(
            "model-download-progress",
            serde_json::json!({
                "model_id": model_id_clone,
                "progress": progress,
            }),
        );
    })
    .await?;

    Ok(model_path.to_string_lossy().to_string())
}

/// 删除本地模型
#[tauri::command]
pub async fn delete_local_model(app: AppHandle, model_id: String) -> Result<()> {
    let app_data_dir = app
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| crate::core::error::AppError::Other("无法获取应用数据目录".into()))?;

    local_whisper::delete_model(&app_data_dir, &model_id).await
}
