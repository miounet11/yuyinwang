// Spokenly Clone - 简化版本
// 移除系统托盘，先让基本功能运行

use tauri::Manager;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub is_recording: bool,
    pub selected_device: Option<String>,
    pub language: String,
    pub hotkey: String,
    pub selected_model: String,
    pub transcription_history: Vec<TranscriptionEntry>,
    pub mcp_config: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub timestamp: u64,
    pub duration: u64,
    pub model: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub server_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
    pub is_default: bool,
    pub is_available: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_recording: false,
            selected_device: None,
            language: "en".to_string(),
            hotkey: "CommandOrControl+Shift+Space".to_string(),
            selected_model: "gpt-4o-mini".to_string(),
            transcription_history: Vec::new(),
            mcp_config: McpConfig {
                enabled: true,
                server_url: "https://api.openai.com/v1".to_string(),
                api_key: String::new(),
                model: "whisper-1".to_string(),
            },
        }
    }
}

static RECORDING_STATE: AtomicBool = AtomicBool::new(false);

// Tauri命令
#[tauri::command]
async fn start_recording(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut app_state = state.lock().unwrap();
    app_state.is_recording = true;
    println!("🎤 开始录音...");
    Ok("Recording started".to_string())
}

#[tauri::command]
async fn stop_recording(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut app_state = state.lock().unwrap();
    app_state.is_recording = false;
    println!("⏹️ 停止录音...");
    Ok("Recording stopped".to_string())
}

#[tauri::command]
fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    println!("🔍 获取音频设备列表...");
    
    let host = cpal::default_host();
    let mut devices = Vec::new();
    
    // 获取默认输入设备
    if let Some(default_input) = host.default_input_device() {
        if let Ok(name) = default_input.name() {
            devices.push(AudioDevice {
                name: format!("Default: {}", name),
                id: "default".to_string(),
                is_default: true,
                is_available: true,
            });
        }
    }
    
    // 获取所有输入设备
    if let Ok(input_devices) = host.input_devices() {
        for (i, device) in input_devices.enumerate() {
            if let Ok(name) = device.name() {
                devices.push(AudioDevice {
                    name: name.clone(),
                    id: format!("device_{}", i),
                    is_default: false,
                    is_available: true,
                });
            }
        }
    }
    
    // 如果没有找到设备，返回模拟设备
    if devices.is_empty() {
        devices.push(AudioDevice {
            name: "MacBook Pro麦克风".to_string(),
            id: "builtin".to_string(),
            is_default: true,
            is_available: true,
        });
        devices.push(AudioDevice {
            name: "\"iPhone\"的麦克风".to_string(),
            id: "iphone".to_string(),
            is_default: false,
            is_available: false,
        });
    }
    
    Ok(devices)
}

#[tauri::command]
async fn get_transcription_result(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<TranscriptionEntry, String> {
    println!("📝 获取转录结果...");
    
    let app_state = state.lock().unwrap();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // 模拟转录结果
    let entry = TranscriptionEntry {
        id: format!("transcript_{}", timestamp),
        text: "This is a sample transcription result from Spokenly Clone.".to_string(),
        timestamp,
        duration: 5,
        model: app_state.selected_model.clone(),
        confidence: 0.95,
    };
    
    Ok(entry)
}

// MCP协议支持
#[tauri::command]
async fn transcribe_with_mcp(
    _audio_data: Vec<u8>, 
    model: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<TranscriptionEntry, String> {
    let mcp_enabled = {
        let app_state = state.lock().unwrap();
        app_state.mcp_config.enabled
    };
    
    if !mcp_enabled {
        return Err("MCP not enabled".to_string());
    }
    
    println!("🤖 使用MCP协议进行转录: {}", model);
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // 模拟MCP API调用
    match model.as_str() {
        "gpt-4o-mini" => {
            // 模拟OpenAI Whisper API调用
            println!("🔄 调用OpenAI GPT-4o mini转录API...");
            tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
            
            Ok(TranscriptionEntry {
                id: format!("mcp_{}", timestamp),
                text: "使用GPT-4o mini模型转录的高质量结果。这个模型在准确性方面表现卓越。".to_string(),
                timestamp,
                duration: 8,
                model: "gpt-4o-mini".to_string(),
                confidence: 0.98,
            })
        },
        "nova-3" => {
            println!("⚡ 调用Deepgram Nova-3实时转录API...");
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
            
            Ok(TranscriptionEntry {
                id: format!("nova_{}", timestamp),
                text: "Real-time transcription using Deepgram Nova-3 with excellent accuracy for English content.".to_string(),
                timestamp,
                duration: 6,
                model: "nova-3".to_string(),
                confidence: 0.96,
            })
        },
        "voxtral-mini" => {
            println!("🌟 调用Mistral Voxtral Mini转录API...");
            tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
            
            Ok(TranscriptionEntry {
                id: format!("voxtral_{}", timestamp),
                text: "Transcription result from Mistral Voxtral Mini with multilingual support and high quality output.".to_string(),
                timestamp,
                duration: 7,
                model: "voxtral-mini".to_string(),
                confidence: 0.92,
            })
        },
        "elevenlabs" => {
            println!("🔊 调用ElevenLabs Scribe转录API...");
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            
            Ok(TranscriptionEntry {
                id: format!("eleven_{}", timestamp),
                text: "High-quality transcription from ElevenLabs Scribe with advanced language recognition capabilities.".to_string(),
                timestamp,
                duration: 9,
                model: "elevenlabs".to_string(),
                confidence: 0.94,
            })
        },
        _ => {
            Err(format!("Unsupported model: {}", model))
        }
    }
}

#[tauri::command]
async fn get_transcription_history(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<TranscriptionEntry>, String> {
    let app_state = state.lock().unwrap();
    Ok(app_state.transcription_history.clone())
}

#[tauri::command]
async fn add_transcription_entry(
    entry: TranscriptionEntry,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let mut app_state = state.lock().unwrap();
    app_state.transcription_history.push(entry);
    println!("📝 添加转录记录到历史");
    Ok(())
}

#[tauri::command]
async fn update_mcp_config(
    config: McpConfig,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let mut app_state = state.lock().unwrap();
    app_state.mcp_config = config;
    println!("⚙️ 更新MCP配置");
    Ok(())
}

#[tauri::command]
fn update_settings(
    language: String,
    hotkey: String,
    device: Option<String>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let mut app_state = state.lock().unwrap();
    app_state.language = language;
    app_state.hotkey = hotkey;
    app_state.selected_device = device;
    println!("⚙️ 设置已更新");
    Ok(())
}

fn main() {
    let app_state = Arc::new(Mutex::new(AppState::default()));
    
    println!("🚀 启动Spokenly克隆应用...");
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            get_audio_devices,
            get_transcription_result,
            transcribe_with_mcp,
            get_transcription_history,
            add_transcription_entry,
            update_mcp_config,
            update_settings
        ])
        .setup(|app| {
            println!("✅ Tauri应用已启动");
            let window = app.get_window("main").unwrap();
            window.show().unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}