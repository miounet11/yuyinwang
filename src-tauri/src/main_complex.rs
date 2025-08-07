// Spokenly Clone - 完整功能版本
// 集成真实音频录制、API调用和数据持久化

use tauri::{Manager, GlobalShortcutManager, AppHandle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use uuid::Uuid;

mod audio;
mod api;
mod storage;

use audio::AudioRecorder;
use api::{TranscriptionService, ApiConfig};
use storage::{StorageManager, AppSettings, TranscriptionEntry};

#[derive(Debug)]
pub struct AppState {
    pub is_recording: bool,
    pub current_recording_data: Option<Vec<f32>>,
    pub audio_recorder: AudioRecorder,
    pub transcription_service: TranscriptionService,
    pub storage_manager: StorageManager,
    pub settings: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
    pub is_default: bool,
    pub is_available: bool,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let storage_manager = StorageManager::new()?;
        let settings = storage_manager.load_settings().await.unwrap_or_default();
        let audio_recorder = AudioRecorder::new().unwrap_or_default();
        let transcription_service = TranscriptionService::new(settings.api_config.clone());
        
        Ok(Self {
            is_recording: false,
            current_recording_data: None,
            audio_recorder,
            transcription_service,
            storage_manager,
            settings,
        })
    }
}

// Tauri命令 - 真实音频录制
#[tauri::command]
async fn start_recording(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut app_state = state.lock();
    
    if app_state.is_recording {
        return Err("Already recording".to_string());
    }
    
    match app_state.audio_recorder.start_recording() {
        Ok(_) => {
            app_state.is_recording = true;
            println!("🎙️ 开始真实音频录制...");
            Ok("Real recording started".to_string())
        },
        Err(e) => {
            eprintln!("录音失败: {}", e);
            Err(format!("录音失败: {}", e))
        }
    }
}

#[tauri::command]
async fn stop_recording(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_handle: AppHandle
) -> Result<String, String> {
    let (audio_data, selected_model) = {
        let mut app_state = state.lock();
        
        if !app_state.is_recording {
            return Err("Not recording".to_string());
        }
        
        match app_state.audio_recorder.stop_recording() {
            Ok(data) => {
                app_state.is_recording = false;
                app_state.current_recording_data = Some(data.clone());
                (data, app_state.settings.selected_model.clone())
            },
            Err(e) => {
                eprintln!("停止录音失败: {}", e);
                return Err(format!("停止录音失败: {}", e));
            }
        }
    };
    
    println!("⏹️ 停止录音，开始转录...");
    
    // 在后台进行转录
    let state_clone = Arc::clone(&state);
    let app_handle_clone = app_handle.clone();
    
    tokio::spawn(async move {
        match process_transcription(audio_data, selected_model, state_clone).await {
            Ok(entry) => {
                // 发送转录完成事件到前端
                let _ = app_handle_clone.emit_all("transcription_result", &entry);
            },
            Err(e) => {
                eprintln!("转录失败: {}", e);
                let _ = app_handle_clone.emit_all("transcription_error", &e.to_string());
            }
        }
    });
    
    Ok("Recording stopped, transcription started".to_string())
}

// 处理转录的异步函数
async fn process_transcription(
    audio_data: Vec<f32>,
    model: String,
    state: Arc<Mutex<AppState>>
) -> Result<TranscriptionEntry, Box<dyn std::error::Error + Send + Sync>> {
    let entry_id = Uuid::new_v4().to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    // 将音频数据转换为WAV格式
    let (audio_bytes, sample_rate) = {
        let app_state = state.lock();
        let temp_path = std::env::temp_dir().join(format!("{}.wav", entry_id));
        
        match app_state.audio_recorder.save_to_wav(&audio_data, &temp_path) {
            Ok(_) => {
                let bytes = std::fs::read(&temp_path)?;
                let _ = std::fs::remove_file(&temp_path);
                (bytes, 44100) // 假设采样率为44.1kHz
            },
            Err(e) => return Err(e.into())
        }
    };
    
    // 保存音频文件
    let audio_file_path = {
        let app_state = state.lock();
        app_state.storage_manager.save_audio_file(&entry_id, &audio_bytes).await?
    };
    
    // 调用转录服务
    let transcription_result = {
        let app_state = state.lock();
        app_state.transcription_service.transcribe_from_bytes(&audio_bytes, &model).await?
    };
    
    let entry = TranscriptionEntry {
        id: entry_id,
        text: transcription_result.text,
        timestamp,
        duration: transcription_result.duration,
        model: transcription_result.model,
        confidence: transcription_result.confidence,
        audio_file_path: Some(audio_file_path),
    };
    
    // 保存到历史记录
    {
        let app_state = state.lock();
        app_state.storage_manager.save_transcription_entry(&entry).await?;
    }
    
    println!("✅ 转录完成: {}", entry.text);
    Ok(entry)
}

#[tauri::command]
fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    println!("🔍 获取真实音频设备列表...");
    
    match AudioRecorder::get_available_devices() {
        Ok(devices) => {
            let audio_devices: Vec<AudioDevice> = devices.into_iter().map(|(name, id)| AudioDevice {
                name,
                id: id.clone(),
                is_default: id == "default",
                is_available: true,
            }).collect();
            
            println!("找到 {} 个音频设备", audio_devices.len());
            Ok(audio_devices)
        },
        Err(e) => {
            eprintln!("获取设备失败: {}", e);
            // 返回模拟设备作为后备
            Ok(vec![
                AudioDevice {
                    name: "MacBook Pro麦克风".to_string(),
                    id: "builtin".to_string(),
                    is_default: true,
                    is_available: true,
                },
            ])
        }
    }
}

#[tauri::command]
async fn get_transcription_history(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<TranscriptionEntry>, String> {
    let storage_manager = {
        let app_state = state.lock();
        app_state.storage_manager.clone()
    };
    
    match storage_manager.load_transcription_history().await {
        Ok(history) => Ok(history),
        Err(e) => {
            eprintln!("加载历史记录失败: {}", e);
            Ok(Vec::new())
        }
    }
}

#[tauri::command]
async fn update_settings(
    language: String,
    hotkey: String,
    device: Option<String>,
    selected_model: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let (storage_manager, settings) = {
        let mut app_state = state.lock();
        app_state.settings.language = language;
        app_state.settings.hotkey = hotkey;
        app_state.settings.selected_device = device;
        app_state.settings.selected_model = selected_model;
        (app_state.storage_manager.clone(), app_state.settings.clone())
    };
    
    match storage_manager.save_settings(&settings).await {
        Ok(_) => {
            println!("⚙️ 设置已更新并保存");
            Ok(())
        },
        Err(e) => {
            eprintln!("保存设置失败: {}", e);
            Err(format!("保存设置失败: {}", e))
        }
    }
}

#[tauri::command]
async fn update_api_config(
    openai_key: Option<String>,
    deepgram_key: Option<String>,
    mistral_key: Option<String>,
    elevenlabs_key: Option<String>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let (storage_manager, settings) = {
        let mut app_state = state.lock();
        app_state.settings.api_config = ApiConfig {
            openai_api_key: openai_key,
            deepgram_api_key: deepgram_key,
            mistral_api_key: mistral_key,
            elevenlabs_api_key: elevenlabs_key,
        };
        
        // 更新转录服务配置
        app_state.transcription_service = TranscriptionService::new(app_state.settings.api_config.clone());
        
        (app_state.storage_manager.clone(), app_state.settings.clone())
    };
    
    match storage_manager.save_settings(&settings).await {
        Ok(_) => {
            println!("🔑 API配置已更新");
            Ok(())
        },
        Err(e) => {
            eprintln!("保存API配置失败: {}", e);
            Err(format!("保存API配置失败: {}", e))
        }
    }
}

#[tauri::command]
async fn clear_history(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let storage_manager = {
        let app_state = state.lock();
        app_state.storage_manager.clone()
    };
    
    match storage_manager.clear_transcription_history().await {
        Ok(_) => {
            println!("🗑️ 历史记录已清空");
            Ok(())
        },
        Err(e) => {
            eprintln!("清空历史记录失败: {}", e);
            Err(format!("清空历史记录失败: {}", e))
        }
    }
}

#[tauri::command]
async fn export_history(
    file_path: String,
    format: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let storage_manager = {
        let app_state = state.lock();
        app_state.storage_manager.clone()
    };
    
    let path = PathBuf::from(file_path);
    
    let result = match format.as_str() {
        "json" => storage_manager.export_history_to_json(&path).await,
        "csv" => storage_manager.export_history_to_csv(&path).await,
        _ => return Err("Unsupported export format".to_string()),
    };
    
    match result {
        Ok(_) => {
            println!("📤 历史记录已导出为 {} 格式", format);
            Ok(())
        },
        Err(e) => {
            eprintln!("导出失败: {}", e);
            Err(format!("导出失败: {}", e))
        }
    }
}

// 全局快捷键设置
fn setup_global_shortcut(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle();
    let mut shortcut_manager = app.global_shortcut_manager();
    
    shortcut_manager.register("CommandOrControl+Shift+Space", move || {
        println!("🔥 全局快捷键触发");
        // 这里可以触发录音开始/停止
        let _ = app_handle.emit_all("global_shortcut_triggered", ());
    })?;
    
    println!("⌨️ 全局快捷键已注册: Cmd+Shift+Space");
    Ok(())
}

#[tokio::main]
async fn main() {
    // 初始化应用状态
    let app_state = Arc::new(Mutex::new(
        AppState::new().await.expect("Failed to initialize app state")
    ));
    
    println!("🚀 启动 Spokenly 克隆应用（完整功能版）...");
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            get_audio_devices,
            get_transcription_history,
            update_settings,
            update_api_config,
            clear_history,
            export_history
        ])
        .setup(|app| {
            println!("✅ Tauri 应用已启动");
            
            // 设置全局快捷键
            if let Err(e) = setup_global_shortcut(app) {
                eprintln!("设置全局快捷键失败: {}", e);
            }
            
            let window = app.get_window("main").unwrap();
            window.show().unwrap();
            
            // 启动后台清理任务
            let app_handle = app.handle();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(24 * 60 * 60)).await; // 每天运行一次
                    
                    // 清理30天前的音频文件
                    if let Some(state) = app_handle.try_state::<Arc<Mutex<AppState>>>() {
                        let app_state = state.lock();
                        if let Err(e) = app_state.storage_manager.cleanup_old_audio_files(30).await {
                            eprintln!("清理旧文件失败: {}", e);
                        }
                    }
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}