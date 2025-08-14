// 命令模块 - 统一管理所有Tauri命令

use tauri::State;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::errors::{AppError, AppResult};
use crate::types::*;
use crate::{AppState, ai_agent};

pub mod history;
pub mod transcription;
pub mod subtitle;

pub use history::*;
pub use transcription::*;
pub use subtitle::*;

// 基础功能命令

#[tauri::command]
pub async fn transcribe_file(
    state: State<'_, AppState>,
    file_path: String,
    model: String,
) -> Result<TranscriptionResult, String> {
    let config = TranscriptionConfig {
        model_name: model.clone(),
        language: Some("auto".to_string()),
        temperature: Some(0.0),
        is_local: model.starts_with("whisper-") && model != "whisper-1",
        api_endpoint: None,
    };
    
    match state.transcription_service.transcribe_audio(&file_path, &config).await {
        Ok(result) => {
            println!("✅ 转录成功: {}", result.text);
            
            // 保存到数据库
            let entry = TranscriptionEntry {
                id: uuid::Uuid::new_v4().to_string(),
                text: result.text.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                duration: 0.0,
                model: model,
                confidence: 0.95,
                audio_file_path: Some(file_path),
                created_at: None,
                updated_at: None,
                tags: None,
                metadata: None,
            };
            
            if let Err(e) = state.database.insert_transcription(&entry) {
                eprintln!("保存转录记录失败: {}", e);
            }
            
            Ok(result)
        },
        Err(e) => {
            eprintln!("转录失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_transcription_history(
    state: State<'_, AppState>,
) -> Result<Vec<TranscriptionEntry>, String> {
    match state.database.get_all_transcriptions() {
        Ok(history) => Ok(history),
        Err(e) => {
            eprintln!("获取转录历史失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn process_ai_agent(
    state: State<'_, AppState>,
    request: AgentRequest,
) -> Result<AgentResponse, String> {
    let ai_request = ai_agent::AIAgentRequest {
        text: request.input_text,
        agent_type: match request.agent_type.as_str() {
            "text-enhancer" => ai_agent::AIAgentType::TextEnhancement,
            "translator" => ai_agent::AIAgentType::Translation,
            "summarizer" => ai_agent::AIAgentType::Summarization,
            "grammar-check" => ai_agent::AIAgentType::GrammarCorrection,
            _ => ai_agent::AIAgentType::Custom,
        },
        options: request.additional_context.unwrap_or_default(),
        context: None,
    };
    
    let service = state.ai_agent_service.clone();
    let agent_type = request.agent_type.clone();
    
    // 在tokio任务中处理以避免Send问题
    match tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            service.lock().process_agent_request(ai_request).await
        })
    }).await {
        Ok(Ok(response)) => {
            Ok(AgentResponse {
                success: response.success,
                output_text: response.processed_text,
                agent_type,
                processing_time_ms: response.processing_time_ms,
                error: response.error,
            })
        },
        Ok(Err(e)) => {
            eprintln!("AI代理处理失败: {}", e);
            Err(e.to_string())
        },
        Err(e) => {
            eprintln!("AI代理任务执行失败: {}", e);
            Err(format!("任务执行失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_audio_devices(
    state: State<'_, AppState>,
) -> Result<Vec<AudioDevice>, String> {
    match state.audio_device_manager.get_input_devices() {
        Ok(devices) => Ok(devices),
        Err(e) => {
            eprintln!("获取音频设备失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn start_recording(
    state: State<'_, AppState>,
    device_id: Option<String>,
) -> Result<String, String> {
    let mut is_recording = state.is_recording.lock();
    if *is_recording {
        return Err("已在录音中".to_string());
    }
    
    let config = RecordingConfig {
        device_id,
        sample_rate: 16000,
        channels: 1,
        duration_seconds: None,
        buffer_duration: Some(3.0), // 默认3秒缓冲区
    };
    
    // 这里需要实际的录音实现
    *is_recording = true;
    Ok("录音已开始".to_string())
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut is_recording = state.is_recording.lock();
    if !*is_recording {
        return Err("当前没有在录音".to_string());
    }
    
    *is_recording = false;
    Ok("录音已停止".to_string())
}

#[tauri::command]
pub async fn get_app_settings(
    state: State<'_, AppState>,
) -> Result<crate::config::AppSettings, String> {
    Ok(state.settings.lock().clone())
}

#[tauri::command]
pub async fn update_app_settings(
    state: State<'_, AppState>,
    settings: crate::config::AppSettings,
) -> Result<(), String> {
    match settings.save() {
        Ok(_) => {
            *state.settings.lock() = settings;
            Ok(())
        },
        Err(e) => {
            eprintln!("保存设置失败: {}", e);
            Err(e.to_string())
        }
    }
}