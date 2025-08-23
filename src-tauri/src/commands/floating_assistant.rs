use tauri::{AppHandle, Manager};

/// 显示主窗口
#[tauri::command]
pub async fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 显示设置窗口
#[tauri::command]
pub async fn show_settings(app_handle: AppHandle) -> Result<(), String> {
    // 向主窗口发送事件打开设置
    app_handle
        .emit_all("open_settings", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 打开快速笔记
#[tauri::command]
pub async fn open_quick_note(app_handle: AppHandle) -> Result<(), String> {
    // 创建或显示快速笔记窗口
    app_handle
        .emit_all("open_quick_note", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 显示剪贴板历史
#[tauri::command]
pub async fn show_clipboard_history(app_handle: AppHandle) -> Result<(), String> {
    app_handle
        .emit_all("show_clipboard_history", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 显示搜索
#[tauri::command]
pub async fn show_search(app_handle: AppHandle) -> Result<(), String> {
    app_handle
        .emit_all("show_search", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 切换悬浮助手
#[tauri::command]
pub async fn toggle_floating_assistant(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_window("floating-assistant") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
        }
    } else {
        // 创建新窗口
        super::super::create_floating_input_window(&app_handle)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 获取音频电平（用于可视化）
#[tauri::command]
pub async fn get_audio_level(state: tauri::State<'_, crate::AppState>) -> Result<f32, String> {
    let level = {
        let recorder = state.audio_recorder.lock();
        recorder.get_current_audio_level().unwrap_or(0.0)
    };
    Ok(level)
}

/// 停止录音并转录
#[tauri::command]
pub async fn stop_recording_and_transcribe(
    state: tauri::State<'_, crate::AppState>,
    model: Option<String>,
) -> Result<String, String> {
    // 停止录音并获取WAV文件路径
    let audio_path = {
        let mut recorder = state.audio_recorder.lock();
        recorder.stop().map_err(|e| e.to_string())?
    };
    
    // 如果没有录音文件，返回错误
    let audio_path = audio_path.ok_or("没有录音文件")?;
    
    // 创建转录配置
    let config = crate::types::TranscriptionConfig {
        model_name: model.unwrap_or_else(|| "luyingwang-online".to_string()),
        language: Some("zh".to_string()),
        temperature: Some(0.0),
        is_local: false,
        api_endpoint: None,
    };
    
    // 执行转录
    let result = state
        .transcription_service
        .transcribe_audio(&audio_path, &config)
        .await
        .map_err(|e| e.to_string())?;
    
    // 清理临时文件
    if let Err(e) = std::fs::remove_file(&audio_path) {
        eprintln!("清理临时文件失败: {}", e);
    }
    
    Ok(result.text)
}