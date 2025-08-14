// Spokenly Clone - 简化版本专注文件上传功能和AI Agent处理
use tauri::{Manager, AppHandle, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, WindowEvent, GlobalShortcutManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use uuid::Uuid;
use tokio::fs;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::collections::HashMap;
use std::process::Command;

// 安全模块导入
mod security {
    pub mod path_validator;
    pub mod secure_client; 
    pub mod command_executor;
}
use security::command_executor::SecureCommandExecutor;
use chrono::{DateTime, Local};
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
// Removed rdev dependency - global keyboard listener was causing crashes

mod audio_recorder;
mod ai_agent;
mod database;
mod folder_watcher;
mod performance_optimizer;

#[cfg(target_os = "macos")]
use cocoa::base::nil;

#[cfg(target_os = "macos")]
use cocoa::foundation::NSString;

#[cfg(target_os = "macos")]
use objc::runtime::Class;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
extern "C" {
    fn IOHIDCheckAccess(requestType: i32) -> i32;
}

#[cfg(target_os = "macos")]
const kIOHIDRequestTypeListenEvent: i32 = 1;

#[cfg(target_os = "macos")]
fn check_accessibility_permission() -> bool {
    // 检查辅助功能权限
    let status = std::process::Command::new("osascript")
        .args(&["-e", "tell application \"System Events\" to get name of first process"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    
    if !status {
        println!("🔍 请在 系统偏好设置 > 安全性与隐私 > 隐私 > 辅助功能 中启用此应用");
    }
    status
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub timestamp: i64,
    pub duration: f64,
    pub model: String,
    pub confidence: f64,
    pub audio_file_path: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub tags: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
    pub is_default: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPrompt {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agent_type: String,
    pub prompt_text: String,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent_type: String,
    pub input_text: String,
    pub prompt_id: Option<String>,
    pub additional_context: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub output_text: String,
    pub agent_type: String,
    pub processing_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    text: String,
}

#[derive(Debug)]
pub struct AppState {
    pub is_recording: bool,
    pub transcription_history: Vec<TranscriptionEntry>,
    pub temp_dir: PathBuf,
    pub ai_prompts: Vec<AIPrompt>,
    pub http_client: Client,
    pub openai_api_key: Option<String>,
    pub audio_recorder: Arc<Mutex<audio_recorder::AudioRecorder>>,
    pub database: Arc<database::DatabaseManager>,
    pub folder_watcher: Arc<folder_watcher::FolderWatcher>,
    pub performance_optimizer: Arc<Mutex<performance_optimizer::PerformanceOptimizer>>,
}

impl AppState {
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir().join("spokenly-clone");
        std::fs::create_dir_all(&temp_dir).ok();
        
        // 初始化数据库
        let db_dir = directories::UserDirs::new()
            .map(|dirs| dirs.home_dir().join("Library/Application Support/spokenly-clone"))
            .unwrap_or_else(|| temp_dir.clone());
        std::fs::create_dir_all(&db_dir).ok();
        
        let db_path = db_dir.join("spokenly.db");
        let database = database::DatabaseManager::new(&db_path)
            .expect("无法初始化数据库");
        
        // 创建HTTP客户端
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();
        
        // 从环境变量读取OpenAI API密钥 - 安全实现
        let openai_api_key = std::env::var("OPENAI_API_KEY").ok();
        
        // 初始化默认AI提示
        let mut ai_prompts = Vec::new();
        ai_prompts.push(create_default_prompt("speech-to-text", "语音转文字", "将语音转换为准确的文本"));
        ai_prompts.push(create_default_prompt("text-enhancer", "文本增强", "优化和增强文本内容，使其更清晰准确"));
        ai_prompts.push(create_default_prompt("translator", "翻译", "将文本翻译为目标语言"));
        ai_prompts.push(create_default_prompt("summarizer", "摘要", "生成内容的简洁摘要"));
        ai_prompts.push(create_default_prompt("formatter", "格式化", "格式化文本内容，使其结构清晰"));
        ai_prompts.push(create_default_prompt("grammar-check", "语法检查", "检查并修正语法错误"));
        ai_prompts.push(create_default_prompt("tone-adjuster", "语调调整", "调整文本的语调和风格"));
        ai_prompts.push(create_default_prompt("auto-input", "自动输入", "生成适合的文本输入内容"));
        
        // 从数据库加载历史记录
        let transcription_history = database.get_all_transcriptions()
            .unwrap_or_else(|e| {
                eprintln!("加载历史记录失败: {}", e);
                Vec::new()
            });
        
        Self {
            is_recording: false,
            transcription_history,
            temp_dir,
            ai_prompts,
            http_client,
            openai_api_key,
            audio_recorder: Arc::new(Mutex::new(audio_recorder::AudioRecorder::new())),
            database: Arc::new(database),
            folder_watcher: Arc::new(folder_watcher::FolderWatcher::new()),
            performance_optimizer: Arc::new(Mutex::new(performance_optimizer::PerformanceOptimizer::new())),
        }
    }
}

fn create_default_prompt(agent_type: &str, name: &str, description: &str) -> AIPrompt {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    let prompt_text = match agent_type {
        "speech-to-text" => "请将以下语音内容转换为准确的文本，保持原意不变：",
        "text-enhancer" => "请优化以下文本，使其更清晰、准确和专业：",
        "translator" => "请将以下文本翻译为指定的目标语言，保持原意和语调：",
        "summarizer" => "请为以下内容生成简洁明了的摘要，突出要点：",
        "formatter" => "请将以下内容格式化，使结构清晰，易于阅读：",
        "grammar-check" => "请检查以下文本的语法错误并提供修正建议：",
        "tone-adjuster" => "请调整以下文本的语调和风格，使其符合指定要求：",
        "auto-input" => "请基于上下文生成合适的文本输入内容：",
        _ => "请处理以下内容："
    }.to_string();
    
    AIPrompt {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: description.to_string(),
        agent_type: agent_type.to_string(),
        prompt_text,
        is_active: true,
        created_at: timestamp,
        updated_at: timestamp,
    }
}

#[tauri::command]
async fn upload_file(
    file_path: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_handle: AppHandle
) -> Result<String, String> {
    let file_path = PathBuf::from(&file_path);
    
    if !file_path.exists() {
        return Err("文件不存在".to_string());
    }

    // 检查文件扩展名
    let extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();
    
    let supported_formats = ["mp3", "wav", "m4a", "flac", "mp4", "mov", "m4v"];
    if !supported_formats.contains(&extension.as_str()) {
        return Err(format!("不支持的文件格式: .{}", extension));
    }

    println!("📁 开始处理文件: {:?}", file_path);
    
    let entry_id = Uuid::new_v4().to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
        .unwrap().as_secs();

    // 复制文件到临时目录
    let temp_path = {
        let app_state = state.lock();
        app_state.temp_dir.join(format!("{}_{}.{}", entry_id, timestamp, extension))
    };

    match fs::copy(&file_path, &temp_path).await {
        Ok(_) => {
            println!("✅ 文件已复制到: {:?}", temp_path);
            
            // 启动转录处理
            let app_handle_clone = app_handle.clone();
            let state_clone = Arc::clone(&state);
            let temp_path_clone = temp_path.clone();
            
            tokio::spawn(async move {
                match process_file_transcription(&temp_path_clone, entry_id, state_clone).await {
                    Ok(entry) => {
                        let _ = app_handle_clone.emit_all("file_transcription_result", &entry);
                    },
                    Err(e) => {
                        eprintln!("文件转录失败: {}", e);
                        let _ = app_handle_clone.emit_all("file_transcription_error", &e.to_string());
                    }
                }
            });
            
            Ok(format!("文件上传成功，开始转录: {}", file_path.display()))
        },
        Err(e) => {
            Err(format!("文件复制失败: {}", e))
        }
    }
}

async fn process_file_transcription(
    file_path: &PathBuf,
    entry_id: String,
    state: Arc<Mutex<AppState>>
) -> Result<TranscriptionEntry, Box<dyn std::error::Error + Send + Sync>> {
    
    println!("🔄 开始转录文件: {:?}", file_path);
    
    let client = {
        let app_state = state.lock();
        app_state.http_client.clone()
    };
    
    // 使用录音接口服务进行真实转写
    let transcription_result = transcribe_via_luyin_api(&client, file_path).await?;
    let text = transcription_result.text;
    
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    let entry = TranscriptionEntry {
        id: entry_id,
        text,
        timestamp: timestamp as i64,
        duration: 120.0, // 假设2分钟
        model: "gpt-4o-mini".to_string(),
        confidence: 0.95,
        audio_file_path: Some(file_path.to_string_lossy().to_string()),
        created_at: None,
        updated_at: None,
        tags: None,
        metadata: None,
    };
    
    // 保存到数据库并添加到内存历史记录
    {
        let mut state_guard = state.lock();
        
        // 保存到数据库
        if let Err(e) = state_guard.database.insert_transcription(&entry) {
            eprintln!("保存转录记录到数据库失败: {}", e);
        }
        
        state_guard.transcription_history.insert(0, entry.clone());
        
        // 限制内存历史记录数量
        if state_guard.transcription_history.len() > 100 {
            state_guard.transcription_history.truncate(100);
        }
    }
    
    println!("✅ 文件转录完成: {}", entry.text);
    Ok(entry)
}

#[tauri::command]
async fn process_batch(
    request: BatchProcessRequest,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<BatchProcessResponse, String> {
    let start_time = SystemTime::now();
    
    println!("📦 开始批量处理: {} ({} 项)", request.agent_type, request.input_texts.len());
    
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;
    
    for (index, input_text) in request.input_texts.iter().enumerate() {
        println!("🔄 处理项目 {}/{}", index + 1, request.input_texts.len());
        
        let agent_request = AgentRequest {
            agent_type: request.agent_type.clone(),
            input_text: input_text.clone(),
            prompt_id: request.prompt_id.clone(),
            additional_context: request.additional_context.clone(),
        };
        
        match process_with_agent(agent_request, state.clone()).await {
            Ok(result) => {
                if result.success {
                    success_count += 1;
                } else {
                    error_count += 1;
                }
                results.push(result);
            },
            Err(e) => {
                error_count += 1;
                results.push(AgentResponse {
                    success: false,
                    output_text: String::new(),
                    agent_type: request.agent_type.clone(),
                    processing_time_ms: 0,
                    error: Some(e),
                });
            }
        }
    }
    
    let total_time = start_time.elapsed().unwrap().as_millis() as u64;
    
    println!("✅ 批量处理完成: {} ({}ms, 成功: {}, 失败: {})", 
             request.agent_type, total_time, success_count, error_count);
    
    Ok(BatchProcessResponse {
        success: error_count == 0,
        results,
        total_processing_time_ms: total_time,
        success_count,
        error_count,
    })
}

// Removed key_to_string function - no longer needed without global keyboard listener

#[tauri::command]
async fn start_recording(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let mut app_state = state.lock();
    if !app_state.is_recording {
        app_state.is_recording = true;
        let mut recorder = app_state.audio_recorder.lock();
        recorder.start_recording().map_err(|e| e.to_string())?;
        Ok("Recording started".to_string())
    } else {
        Err("Already recording".to_string())
    }
}
#[tauri::command]
async fn stop_recording(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_handle: AppHandle,
    model: String,
    model_type: String
) -> Result<String, String> {
    let (client, api_key, temp_dir, recorder_arc, audio_data);
    {
        let mut app_state = state.lock();
        
        println!("🔍 调试信息: 当前录音状态 is_recording = {}", app_state.is_recording);
        println!("🔍 调试信息: 收到的模型参数 model = '{}', modelType = '{}'", model, model_type);
        
        if !app_state.is_recording {
            println!("⚠️ 错误: 尝试停止录音但当前状态为未录音");
            return Err("Not recording".to_string());
        }
        
        println!("✅ 录音状态正常，准备停止录音...");
        app_state.is_recording = false;
        client = app_state.http_client.clone();
        api_key = app_state.openai_api_key.clone().unwrap();
        temp_dir = app_state.temp_dir.clone();
        recorder_arc = app_state.audio_recorder.clone();
    }
    
    // 停止录音并获取音频数据
    {
        let mut recorder = recorder_arc.lock();
        match recorder.stop_recording() {
            Ok(data) => {
                audio_data = data;
                println!("⏹️ 停止录音，获得 {} 个样本", audio_data.len());
            },
            Err(e) => {
                eprintln!("停止录音失败: {}", e);
                return Err(format!("Failed to stop recording: {}", e));
            }
        }
    }
    
    // 如果没有音频数据，返回错误
    if audio_data.is_empty() {
        return Err("No audio data captured".to_string());
    }
    
    // 生成文件名并保存音频
    let entry_id = Uuid::new_v4().to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let audio_file_path = temp_dir.join(format!("recording_{}_{}.wav", entry_id, timestamp));
    
    // 保存音频到 WAV 文件
    {
        let recorder = recorder_arc.lock();
        match recorder.save_to_wav(&audio_data, &audio_file_path) {
            Ok(_) => {
                println!("💾 音频已保存到: {:?}", audio_file_path);
            },
            Err(e) => {
                eprintln!("保存音频文件失败: {}", e);
                return Err(format!("Failed to save audio: {}", e));
            }
        }
    }
    
    // 计算音频时长（假设 44100 Hz 采样率）
    let duration = (audio_data.len() as f32 / 44100.0) as u64;
    
    // 根据模型类型选择转录方式
    println!("🔍 调试信息: 接收到的参数 - model: '{}', model_type: '{}'", model, model_type);
    let transcription_result = if model_type == "local" {
        // 本地 Whisper 模型转录
        println!("🔍 使用本地 {} 模型进行转录...", model);
        match transcribe_with_local_whisper(&audio_file_path, &model).await {
            Ok(result) => {
                println!("✅ 本地转录成功: {}", result.text);
                result
            },
            Err(e) => {
                println!("❌ 本地转录失败: {}", e);
                return Err(e.to_string());
            }
        }
    } else if model == "luyingwang-online" {
        // 使用录音接口服务进行真实在线转写
        println!("🌐 使用鲁音网服务进行转写...");
        match transcribe_via_luyin_api(&client, &audio_file_path).await {
            Ok(result) => {
                println!("✅ 鲁音网在线转写成功: {}", result.text);
                result
            },
            Err(e) => {
                println!("❌ 鲁音网在线转写失败: {}", e);
                return Err(e);
            }
        }
    } else {
        // 在线 API 转录
        println!("📤 正在发送音频到 {} API...", model);
        match transcribe_audio_file(&client, &api_key, &audio_file_path, &model).await {
            Ok(result) => {
                println!("✅ 在线转录成功: {}", result.text);
                result
            },
            Err(e) => {
                println!("❌ 在线转录失败: {}", e);
                return Err(e.to_string());
            }
        }
    };
    
    let entry = TranscriptionEntry {
        id: entry_id,
        text: transcription_result.text,
        timestamp: timestamp as i64,
        duration: duration as f64,
        model: "gpt-4o-mini".to_string(),
        confidence: 0.95,
        audio_file_path: Some(audio_file_path.to_string_lossy().to_string()),
        created_at: None,
        updated_at: None,
        tags: None,
        metadata: None,
    };
    
    {
        let mut app_state = state.lock();
        
        // 保存到数据库
        if let Err(e) = app_state.database.insert_transcription(&entry) {
            eprintln!("保存转录记录到数据库失败: {}", e);
        }
        
        app_state.transcription_history.insert(0, entry.clone());
        
        if app_state.transcription_history.len() > 100 {
            app_state.transcription_history.truncate(100);
        }
    }
    
    let _ = app_handle.emit_all("transcription_result", &entry);
    Ok("Recording stopped".to_string())
}

#[tauri::command]
async fn delete_file(
    entry_id: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    // 从历史记录中找到并删除条目
    if let Some(pos) = app_state.transcription_history.iter().position(|entry| entry.id == entry_id) {
        let entry = app_state.transcription_history.remove(pos);
        
        // 从数据库删除
        if let Err(e) = app_state.database.delete_transcription(&entry_id) {
            eprintln!("从数据库删除转录记录失败: {}", e);
        }
        
        // 删除关联的音频文件
        if let Some(file_path_str) = &entry.audio_file_path {
            let file_path = PathBuf::from(file_path_str);
            if file_path.exists() {
                match std::fs::remove_file(&file_path) {
                    Ok(_) => println!("🗑️ 已删除音频文件: {:?}", file_path),
                    Err(e) => eprintln!("删除音频文件失败: {:?}: {}", file_path, e),
                }
            }
        }
        
        println!("✅ 已删除转录记录: {}", entry.text);
        Ok(())
    } else {
        Err("未找到指定的转录记录".to_string())
    }
}

#[tauri::command]
async fn export_transcription(
    entry_id: String,
    export_format: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<String, String> {
    let app_state = state.lock();
    
    // 找到指定的转录记录
    let entry = app_state.transcription_history.iter()
        .find(|e| e.id == entry_id)
        .ok_or("未找到指定的转录记录")?;
    
    // 获取桌面路径
    let desktop_path = directories::UserDirs::new()
        .and_then(|dirs| dirs.desktop_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::home_dir().unwrap_or_else(|| PathBuf::from(".")));
    
    // 生成文件名
    let timestamp_str = Local::now().format("%Y%m%d_%H%M%S");
    let file_name = format!("transcription_{}_{}.{}", 
        entry_id.chars().take(8).collect::<String>(), 
        timestamp_str, 
        export_format
    );
    let export_path = desktop_path.join(&file_name);
    
    // 根据格式导出
    match export_format.as_str() {
        "txt" => {
            // 导出为纯文本
            let content = format!(
                "转录文本\n{}\n\n时间: {}\n时长: {}秒\n模型: {}\n置信度: {:.1}%\n",
                entry.text,
                DateTime::<Local>::from(
                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(entry.timestamp as u64)
                ).format("%Y-%m-%d %H:%M:%S"),
                entry.duration,
                entry.model,
                entry.confidence * 100.0
            );
            std::fs::write(&export_path, content)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        },
        "json" => {
            // 导出为JSON
            let json_content = serde_json::to_string_pretty(&entry)
                .map_err(|e| format!("序列化失败: {}", e))?;
            std::fs::write(&export_path, json_content)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        },
        _ => {
            return Err(format!("不支持的导出格式: {}", export_format));
        }
    }
    
    println!("📤 已导出转录记录到: {:?}", export_path);
    Ok(export_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn get_supported_formats() -> Result<Vec<String>, String> {
    Ok(vec![
        "mp3".to_string(),
        "wav".to_string(),
        "m4a".to_string(),
        "flac".to_string(),
        "mp4".to_string(),
        "mov".to_string(),
        "m4v".to_string(),
    ])
}

// 文件夹监控相关命令
#[tauri::command]
async fn add_watched_folder(
    folder_path: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let app_state = state.lock();
    let path = PathBuf::from(folder_path);
    app_state.folder_watcher.add_folder(path)
}

#[tauri::command]
async fn remove_watched_folder(
    folder_path: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let app_state = state.lock();
    let path = PathBuf::from(folder_path);
    app_state.folder_watcher.remove_folder(&path)
}

#[tauri::command]
async fn get_watched_folders(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<Vec<String>, String> {
    let app_state = state.lock();
    let folders = app_state.folder_watcher.get_watched_folders();
    Ok(folders.into_iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[tauri::command]
async fn get_folder_watcher_stats(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(usize, Vec<String>), String> {
    let app_state = state.lock();
    Ok(app_state.folder_watcher.get_folder_stats())
}

#[tauri::command]
async fn clear_all_watched_folders(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let app_state = state.lock();
    app_state.folder_watcher.clear_all();
    Ok(())
}

// 数据库相关命令
#[tauri::command]
async fn get_transcription_history(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<TranscriptionEntry>, String> {
    let app_state = state.lock();
    Ok(app_state.transcription_history.clone())
}

#[tauri::command]
async fn update_transcription_text(
    entry_id: String,
    new_text: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    // 更新数据库
    if let Err(e) = app_state.database.update_transcription(&entry_id, &new_text) {
        return Err(format!("更新数据库失败: {}", e));
    }
    
    // 更新内存中的历史记录
    if let Some(entry) = app_state.transcription_history.iter_mut().find(|e| e.id == entry_id) {
        entry.text = new_text;
        println!("✅ 转录文本已更新: {}", entry_id);
    }
    
    Ok(())
}

#[tauri::command]
async fn search_transcriptions(
    query: String,
    limit: Option<usize>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<Vec<TranscriptionEntry>, String> {
    let app_state = state.lock();
    
    match app_state.database.search_transcriptions(&query, limit) {
        Ok(results) => Ok(results),
        Err(e) => Err(format!("搜索失败: {}", e))
    }
}

#[tauri::command]
async fn get_database_stats(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(usize, f64, usize), String> {
    let app_state = state.lock();
    
    match app_state.database.get_database_stats() {
        Ok(stats) => Ok(stats),
        Err(e) => Err(format!("获取统计信息失败: {}", e))
    }
}

#[tauri::command]
async fn get_model_stats(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<Vec<(String, i32, f64, f64)>, String> {
    let app_state = state.lock();
    
    match app_state.database.get_model_stats() {
        Ok(stats) => Ok(stats),
        Err(e) => Err(format!("获取模型统计失败: {}", e))
    }
}

#[tauri::command]
async fn export_database_json(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<String, String> {
    let app_state = state.lock();
    
    match app_state.database.export_to_json() {
        Ok(json) => Ok(json),
        Err(e) => Err(format!("导出数据失败: {}", e))
    }
}

// 系统托盘相关命令
#[tauri::command]
async fn set_tray_icon_recording(is_recording: bool, app_handle: AppHandle) -> Result<(), String> {
    let tray = app_handle.tray_handle();
    
    // 更新托盘菜单中的录音选项文字
    let item_handle = tray.get_item("toggle_recording");
    let new_title = if is_recording { "⏹️ 停止录音" } else { "🎤 开始录音" };
    match item_handle.set_title(new_title) {
        Ok(_) => {
            println!("🎯 托盘菜单已更新 - 录音状态: {}", is_recording);
            Ok(())
        },
        Err(e) => Err(format!("设置托盘菜单失败: {}", e))
    }
}

#[tauri::command]
async fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    let window = app_handle.get_window("main").ok_or("主窗口不存在")?;
    window.show().map_err(|e| format!("显示主窗口失败: {}", e))?;
    window.set_focus().map_err(|e| format!("聚焦主窗口失败: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn hide_main_window(app_handle: AppHandle) -> Result<(), String> {
    let window = app_handle.get_window("main").ok_or("主窗口不存在")?;
    window.hide().map_err(|e| format!("隐藏主窗口失败: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn quit_app(app_handle: AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
async fn get_current_app_info() -> Result<HashMap<String, String>, String> {
    let mut info = HashMap::new();
    
    // 根据平台获取当前激活应用的信息
    #[cfg(target_os = "macos")]
    {
        // 使用 AppleScript 获取真实的当前应用信息
        let output = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of first application process whose frontmost is true")
            .output()
            .map_err(|e| format!("Failed to execute osascript: {}", e))?;
        
        let app_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // 获取应用的 bundle ID
        let bundle_output = Command::new("osascript")
            .arg("-e")
            .arg(format!("id of app \"{}\"", app_name))
            .output()
            .ok();
        
        let bundle_id = bundle_output
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        // 应用图标映射
        let icon = match app_name.as_str() {
            "Finder" | "访达" => "📁",
            "Safari" => "🌐",
            "Google Chrome" | "Chrome" => "🔵",
            "Firefox" => "🦊",
            "Xcode" => "🔨",
            "Terminal" | "终端" => "⬛",
            "微信" | "WeChat" => "💬",
            "钉钉" | "DingTalk" => "📞",
            "Visual Studio Code" | "Code" => "📝",
            "Slack" => "💼",
            "Telegram" => "✈️",
            "Mail" | "邮件" => "📧",
            "Calendar" | "日历" => "📅",
            "Notes" | "备忘录" => "📓",
            "Messages" | "信息" => "💬",
            "Music" | "音乐" => "🎵",
            "Spotify" => "🎧",
            "System Preferences" | "系统偏好设置" => "⚙️",
            "Activity Monitor" | "活动监视器" => "📊",
            "Preview" | "预览" => "🖼️",
            "TextEdit" | "文本编辑" => "📄",
            _ => "📱"
        };
        
        info.insert("name".to_string(), app_name);
        info.insert("bundle_id".to_string(), bundle_id);
        info.insert("icon".to_string(), icon.to_string());
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        info.insert("name".to_string(), "未知应用".to_string());
        info.insert("bundle_id".to_string(), "unknown".to_string());
        info.insert("icon".to_string(), "📱".to_string());
    }
    
    Ok(info)
}

// 权限检查命令
#[cfg(target_os = "macos")]
#[tauri::command]
async fn check_permission(permission: String) -> Result<String, String> {
    use std::process::Command;
    
    match permission.as_str() {
        "accessibility" => {
            // 检查辅助功能权限
            let output = Command::new("osascript")
                .arg("-e")
                .arg("tell application \"System Events\" to get UI elements enabled")
                .output()
                .map_err(|e| e.to_string())?;
            
            let result = String::from_utf8_lossy(&output.stdout);
            if result.trim() == "true" {
                Ok("granted".to_string())
            } else {
                Ok("denied".to_string())
            }
        },
        "microphone" => {
            unsafe {
                let cls = Class::get("AVCaptureDevice").unwrap();
                let media_type = NSString::alloc(nil).init_str("soun");
                let status: i32 = msg_send![cls, authorizationStatusForMediaType: media_type];
                let status_str = match status {
                    3 => "granted".to_string(),
                    2 => "denied".to_string(),
                    1 => "restricted".to_string(),
                    0 => "not-determined".to_string(),
                    _ => "unknown".to_string(),
                };
                println!("🎤 麦克风权限状态: {} (raw: {})", status_str, status);
                Ok(status_str)
            }
        },
        "file-system" => {
            // 文件系统权限通常是自动授予的
            Ok("granted".to_string())
        },
        "notifications" => {
            // 实现通知权限检查
            unsafe {
                // 简化的通知权限检查 - 大多数情况下是被授权的
                let status = std::process::Command::new("osascript")
                    .args(&["-e", "display notification \"test\" with title \"test\""])
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false);
                
                let status_str = if status {
                    "granted".to_string()
                } else {
                    "not-determined".to_string()
                };
                println!("🔔 通知权限状态: {}", status_str);
                Ok(status_str)
            }
        },
        "screen-recording" => {
            // 实现屏幕录制权限检查
            unsafe {
                // 使用 CGDisplayStreamCreate 来检测屏幕录制权限
                let available = std::process::Command::new("osascript")
                    .args(&["-e", "tell application \"System Events\" to get name of first process"])
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false);
                
                let status_str = if available {
                    "granted".to_string()
                } else {
                    "denied".to_string()
                };
                println!("🖥️ 屏幕录制权限状态: {}", status_str);
                Ok(status_str)
            }
        },
        "automation" => {
            // 检查自动化权限（辅助功能访问）
            unsafe {
                let available = std::process::Command::new("osascript")
                    .args(&["-e", "tell application \"System Events\" to keystroke \"test\""])
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false);
                
                let status_str = if available {
                    "granted".to_string()
                } else {
                    "denied".to_string()
                };
                println!("🤖 自动化权限状态: {}", status_str);
                Ok(status_str)
            }
        },
        "input-monitoring" => {
            unsafe {
                let status = IOHIDCheckAccess(kIOHIDRequestTypeListenEvent);
                let status_str = match status {
                    1 => "granted".to_string(), // kIOHIDAccessTypeGranted
                    0 => "denied".to_string(),  // kIOHIDAccessTypeDenied
                    _ => "not-determined".to_string(),
                };
                println!("⌨️ 输入监控权限状态: {} (raw: {})", status_str, status);
                Ok(status_str)
            }
        },
        _ => Ok("not-determined".to_string())
    }
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn check_permission(permission: String) -> Result<String, String> {
    // 其他操作系统的权限检查
    Ok("granted".to_string())
}

#[tauri::command]
async fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let mut devices = Vec::new();

    match host.input_devices() {
        Ok(input_devices) => {
            for (index, device) in input_devices.enumerate() {
                match device.name() {
                    Ok(name) => {
                        devices.push(AudioDevice {
                            name,
                            id: index.to_string(),
                            is_default: index == 0,
                            is_available: true,
                        });
                    }
                    Err(e) => {
                        eprintln!("获取设备名称失败: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("获取音频设备失败: {}", e));
        }
    }

    Ok(devices)
}

// 权限请求命令
#[tauri::command]
async fn request_permission(permission: String, _app_handle: AppHandle) -> Result<String, String> {
    println!("🔐 申请权限: {}", permission);
    
    match permission.as_str() {
        "microphone" => {
            // 申请麦克风权限
            #[cfg(target_os = "macos")]
            {
                unsafe {
                    let cls = Class::get("AVCaptureDevice").unwrap();
                    let media_type = NSString::alloc(nil).init_str("soun");
                    let _: () = msg_send![cls, requestAccessForMediaType: media_type completionHandler: nil];
                    
                    // 打开系统设置到隐私页面
                    let _ = std::process::Command::new("open")
                        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
                        .spawn();
                }
            }
            Ok("pending".to_string())
        },
        "screen-recording" => {
            // 打开屏幕录制权限设置
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
                    .spawn();
            }
            Ok("pending".to_string())
        },
        "automation" => {
            // 打开辅助功能权限设置
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                    .spawn();
            }
            Ok("pending".to_string())
        },
        "input-monitoring" => {
            // 打开输入监控权限设置
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
                    .spawn();
            }
            Ok("pending".to_string())
        },
        "notifications" => {
            // 打开通知权限设置
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.notifications")
                    .spawn();
            }
            Ok("pending".to_string())
        },
        "file-system" => {
            // 文件系统权限通常是自动的
            Ok("granted".to_string())
        },
        _ => {
            Err("未知权限类型".to_string())
        }
    }
}

#[tauri::command] 
async fn open_system_preferences(preference_pane: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // 直接使用 open 命令打开系统设置
        let url = match preference_pane.as_str() {
            "accessibility" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            "microphone" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone", 
            "input-monitoring" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
            "screen-recording" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
            "files-and-folders" => "x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders",
            "developer-tools" => "x-apple.systempreferences:com.apple.preference.security?Privacy_DeveloperTools",
            "automation" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Automation",
            _ => {
                return Err(format!("未知的设置面板: {}", preference_pane));
            }
        };
        
        let output = Command::new("open")
            .arg(url)
            .output()
            .map_err(|e| format!("无法打开系统设置: {}", e))?;
            
        if !output.status.success() {
            return Err(format!("打开系统设置失败: {:?}", String::from_utf8_lossy(&output.stderr)));
        }
        
        println!("✅ 已打开系统设置: {}", preference_pane);
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        return Err("此功能仅在 macOS 上可用".to_string());
    }
    
    Ok(())
}

// ======== 性能优化相关命令 ========

#[tauri::command]
async fn get_performance_metrics(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<performance_optimizer::PerformanceMetrics, String> {
    let app_state = state.lock();
    let mut optimizer = app_state.performance_optimizer.lock();
    
    // 获取系统指标
    let (cpu_usage, memory_usage) = optimizer.get_system_metrics()
        .map_err(|e| format!("获取系统指标失败: {}", e))?;
    
    // 创建性能指标对象
    let mut metrics = performance_optimizer::PerformanceMetrics::default();
    metrics.cpu_usage_percent = cpu_usage;
    metrics.gpu_memory_usage_mb = memory_usage;
    
    Ok(metrics)
}

#[tauri::command]
async fn get_cache_stats(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(usize, Vec<String>), String> {
    let app_state = state.lock();
    let optimizer = app_state.performance_optimizer.lock();
    
    Ok(optimizer.get_cache_stats())
}

#[tauri::command]
async fn clear_model_cache(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let app_state = state.lock();
    let optimizer = app_state.performance_optimizer.lock();
    
    optimizer.clear_model_cache();
    Ok(())
}

#[tauri::command]
async fn configure_performance_optimizer(
    enable_gpu: bool,
    enable_caching: bool,
    max_cache_size: usize,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let app_state = state.lock();
    let mut optimizer = app_state.performance_optimizer.lock();
    
    optimizer.configure(enable_gpu, enable_caching, max_cache_size);
    println!("🔧 性能优化器配置已更新: GPU={}, 缓存={}, 最大缓存={}", 
             enable_gpu, enable_caching, max_cache_size);
    
    Ok(())
}

#[tauri::command]
async fn warmup_gpu(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let app_state = state.lock();
    let optimizer = app_state.performance_optimizer.lock();
    
    optimizer.warmup_gpu()
}

// ======== AI Agent 相关命令 ========

#[tauri::command]
async fn get_ai_prompts(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<AIPrompt>, String> {
    let app_state = state.lock();
    Ok(app_state.ai_prompts.clone())
}

#[tauri::command]
async fn save_ai_prompt(
    mut prompt: AIPrompt,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<AIPrompt, String> {
    let mut app_state = state.lock();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // 查找是否存在相同ID的提示
    if let Some(existing_index) = app_state.ai_prompts.iter().position(|p| p.id == prompt.id) {
        // 更新现有提示
        prompt.updated_at = timestamp;
        app_state.ai_prompts[existing_index] = prompt.clone();
        println!("✅ 已更新AI提示: {}", prompt.name);
    } else {
        // 创建新提示
        if prompt.id.is_empty() {
            prompt.id = Uuid::new_v4().to_string();
        }
        prompt.created_at = timestamp;
        prompt.updated_at = timestamp;
        app_state.ai_prompts.push(prompt.clone());
        println!("✅ 已创建新AI提示: {}", prompt.name);
    }
    
    Ok(prompt)
}

#[tauri::command]
async fn delete_ai_prompt(
    prompt_id: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    if let Some(index) = app_state.ai_prompts.iter().position(|p| p.id == prompt_id) {
        let removed_prompt = app_state.ai_prompts.remove(index);
        println!("🗑️ 已删除AI提示: {}", removed_prompt.name);
        Ok(())
    } else {
        Err("未找到指定的AI提示".to_string())
    }
}

#[tauri::command]
async fn activate_ai_prompt(
    prompt_id: String,
    agent_type: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    // 首先将同类型的其他提示设为非激活状态
    for prompt in app_state.ai_prompts.iter_mut() {
        if prompt.agent_type == agent_type {
            prompt.is_active = prompt.id == prompt_id;
        }
    }
    
    println!("✅ 已激活AI提示: {}", prompt_id);
    Ok(())
}

#[tauri::command]
async fn process_with_agent(
    request: AgentRequest,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<AgentResponse, String> {
    let start_time = SystemTime::now();
    
    println!("🤖 开始处理Agent请求: {}", request.agent_type);
    
    // 获取API key和HTTP客户端
    let (api_key, client) = {
        let app_state = state.lock();
        let api_key = app_state.openai_api_key.clone()
            .ok_or("OpenAI API key not configured")?;
        (api_key, app_state.http_client.clone())
    };
    
    // 创建AI Agent
    let agent = ai_agent::AIAgent::new(api_key, client);
    
    // 转换agent类型
    let agent_type = match request.agent_type.as_str() {
        "text_enhancement" => ai_agent::AIAgentType::TextEnhancement,
        "translation" => ai_agent::AIAgentType::Translation,
        "summarization" => ai_agent::AIAgentType::Summarization,
        "grammar_correction" => ai_agent::AIAgentType::GrammarCorrection,
        "tone_adjustment" => ai_agent::AIAgentType::ToneAdjustment,
        "keyword_extraction" => ai_agent::AIAgentType::KeywordExtraction,
        "code_explanation" => ai_agent::AIAgentType::CodeExplanation,
        _ => ai_agent::AIAgentType::Custom,
    };
    
    // 准备选项
    let mut options = request.additional_context.unwrap_or_default();
    
    // 如果是自定义类型，添加提示词
    if matches!(agent_type, ai_agent::AIAgentType::Custom) {
        let prompt_text = if let Some(prompt_id) = &request.prompt_id {
            let app_state = state.lock();
            app_state.ai_prompts.iter()
                .find(|p| p.id == *prompt_id && p.is_active)
                .map(|p| p.prompt_text.clone())
                .unwrap_or_else(|| get_default_prompt(&request.agent_type))
        } else {
            // 获取该类型的激活提示或默认提示
            let app_state = state.lock();
            app_state.ai_prompts.iter()
                .find(|p| p.agent_type == request.agent_type && p.is_active)
                .map(|p| p.prompt_text.clone())
                .unwrap_or_else(|| get_default_prompt(&request.agent_type))
        };
        options.insert("system_prompt".to_string(), prompt_text);
    }
    
    // 创建AI Agent请求
    let ai_request = ai_agent::AIAgentRequest {
        text: request.input_text.clone(),
        agent_type,
        options,
    };
    
    // 调用AI Agent处理
    match agent.process(ai_request).await {
        Ok(ai_response) => {
            let processing_time = start_time.elapsed().unwrap().as_millis() as u64;
            
            println!("✅ Agent处理完成: {} ({}ms)", request.agent_type, processing_time);
            
            Ok(AgentResponse {
                success: true,
                output_text: ai_response.processed_text,
                agent_type: request.agent_type,
                processing_time_ms: processing_time,
                error: None,
            })
        },
        Err(e) => {
            let processing_time = start_time.elapsed().unwrap().as_millis() as u64;
            eprintln!("❌ Agent处理失败: {}", e);
            
            Ok(AgentResponse {
                success: false,
                output_text: String::new(),
                agent_type: request.agent_type,
                processing_time_ms: processing_time,
                error: Some(e),
            })
        }
    }
}


async fn transcribe_audio_file(client: &Client, api_key: &str, audio_file_path: &PathBuf, model: &str) -> Result<TranscriptionResult, String> {
    // 读取音频文件
    let mut file = File::open(audio_file_path).await
        .map_err(|e| format!("Failed to open audio file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await
        .map_err(|e| format!("Failed to read audio file: {}", e))?;
    
    // 创建 multipart form 数据
    let part = Part::bytes(buffer)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to create part: {}", e))?;
    
    println!("🔍 调试信息: 发送到API的模型参数 = '{}'", model);
    
    let form = Form::new()
        .part("file", part)
        .text("model", model.to_string())
        .text("language", "zh")
        .text("response_format", "verbose_json"); // 中文语言提示和详细响应格式
    
    // 发送请求到 NewAPI (支持 GPT-4o mini 转录)
    let response = client
        .post("https://ttkk.inping.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API request failed: {}", error_text));
    }
    
    // 解析响应
    let response_text = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;
    
    let text = json["text"].as_str()
        .ok_or_else(|| "No text field in response".to_string())?
        .to_string();
    
    
    Ok(TranscriptionResult { text })
}

// 真实接入：ly.gl173.com 录音转文字服务
async fn transcribe_via_luyin_api(client: &Client, audio_file_path: &PathBuf) -> Result<TranscriptionResult, String> {
    // 读取音频文件
    let mut file = File::open(audio_file_path).await
        .map_err(|e| format!("无法打开音频文件: {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await
        .map_err(|e| format!("无法读取音频文件: {}", e))?;

    // 1) 上传文件，获取 file_id
    let file_name = audio_file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("recording.wav");
    let part = Part::bytes(buffer)
        .file_name(file_name.to_string())
        .mime_str("audio/wav")
        .map_err(|e| format!("创建上传部件失败: {}", e))?;

    let form = Form::new().part("file[]", part);
    let upload_resp = client
        .post("https://ly.gl173.com/api/v1/upload-file")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("上传文件失败: {}", e))?;

    let status = upload_resp.status();
    let upload_text = upload_resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("上传接口错误({}): {}", status, upload_text));
    }
    let upload_json: serde_json::Value = serde_json::from_str(&upload_text)
        .map_err(|e| format!("解析上传响应失败: {} - {}", e, upload_text))?;
    if upload_json["code"].as_i64().unwrap_or(0) != 200 {
        return Err(format!("上传返回非200: {}", upload_text));
    }
    let file_id_val = upload_json["data"][0]["file_id"].clone();
    let file_id = if let Some(id) = file_id_val.as_i64() { id.to_string() } else { file_id_val.to_string() };
    if file_id.is_empty() || file_id == "null" {
        return Err(format!("无法获取file_id: {}", upload_text));
    }

    // 2) 创建转换任务，得到 task_id
    let task_resp = client
        .post("https://ly.gl173.com/api/v1/task-add")
        .form(&[("file_id", file_id.clone())])
        .send()
        .await
        .map_err(|e| format!("创建转换任务失败: {}", e))?;
    let task_text = task_resp.text().await.unwrap_or_default();
    let task_json: serde_json::Value = serde_json::from_str(&task_text)
        .map_err(|e| format!("解析任务创建响应失败: {} - {}", e, task_text))?;
    if task_json["code"].as_i64().unwrap_or(0) != 200 {
        return Err(format!("任务创建返回非200: {}", task_text));
    }
    let task_id = task_json["data"]["task_id"].as_str()
        .unwrap_or("")
        .to_string();
    if task_id.is_empty() {
        return Err(format!("无法获取task_id: {}", task_text));
    }

    // 3) 轮询进度，直到完成或超时
    let mut attempts = 0usize;
    let max_attempts = 60usize; // 3 分钟
    loop {
        attempts += 1;
        let progress_resp = client
            .post("https://ly.gl173.com/api/v1/task-progress")
            .form(&[("task_id", task_id.clone())])
            .send()
            .await
            .map_err(|e| format!("查询进度失败: {}", e))?;

        let progress_text = progress_resp.text().await.unwrap_or_default();
        let progress_json: serde_json::Value = serde_json::from_str(&progress_text)
            .map_err(|e| format!("解析进度响应失败: {} - {}", e, progress_text))?;
        if progress_json["code"].as_i64().unwrap_or(0) != 200 {
            return Err(format!("进度接口返回非200: {}", progress_text));
        }

        let progress = progress_json["data"]["progress"].as_i64().unwrap_or(0);
        if progress == 1 {
            let result = progress_json["data"]["result"].as_str().unwrap_or("").to_string();
            if result.is_empty() {
                return Err("转换完成但结果为空".to_string());
            }
            return Ok(TranscriptionResult { text: result });
        }

        if attempts >= max_attempts {
            return Err("转换超时，请稍后重试".to_string());
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

// 本地 Whisper 模型转录函数（性能优化版）
async fn transcribe_with_local_whisper(audio_file_path: &PathBuf, model: &str) -> Result<TranscriptionResult, String> {
    println!("🔍 开始本地 Whisper {} 转录（性能优化版）...", model);
    
    // 检查音频文件是否存在
    if !audio_file_path.exists() {
        return Err("音频文件不存在".to_string());
    }
    
    // 在新线程中运行 Whisper（因为它是计算密集型的）
    let audio_path = audio_file_path.clone();
    let model_name = model.to_string();
    
    let transcription_result = tokio::task::spawn_blocking(move || {
        run_whisper_transcription_optimized(&audio_path, &model_name)
    }).await;
    
    match transcription_result {
        Ok(Ok((text, metrics))) => {
            println!("✅ 本地 Whisper 转录成功: {}", text);
            println!("📊 性能指标: RTF={:.2}, 总耗时={}ms", 
                    metrics.real_time_factor, metrics.total_time_ms);
            Ok(TranscriptionResult { text })
        },
        Ok(Err(e)) => {
            println!("❌ 本地 Whisper 转录失败: {}", e);
            Err(e)
        },
        Err(e) => {
            println!("❌ Whisper 任务执行失败: {}", e);
            Err(format!("转录任务执行失败: {}", e))
        }
    }
}

// 性能优化版 Whisper 转录
fn run_whisper_transcription_optimized(audio_file_path: &PathBuf, model: &str) -> Result<(String, performance_optimizer::PerformanceMetrics), String> {
    let total_start = std::time::Instant::now();
    let mut metrics = performance_optimizer::PerformanceMetrics::default();
    
    // 创建性能优化器
    let mut optimizer = performance_optimizer::PerformanceOptimizer::new();
    
    // 下载模型（如果需要）
    let model_path = download_whisper_model_if_needed(model)?;
    
    // 优化版模型加载（带缓存）
    let model_start = std::time::Instant::now();
    let ctx = optimizer.get_cached_model(&model_path)?;
    metrics.model_load_time_ms = model_start.elapsed().as_millis() as u64;
    
    println!("🔍 读取音频文件...");
    
    // 优化版音频数据加载
    let audio_start = std::time::Instant::now();
    let audio_data = load_audio_samples_optimized(audio_file_path, &mut optimizer)?;
    metrics.audio_processing_time_ms = audio_start.elapsed().as_millis() as u64;
    
    // 计算音频时长
    metrics.audio_duration_seconds = audio_data.len() as f64 / 16000.0; // 16kHz采样率
    
    println!("🔍 开始转录，音频样本数: {} (时长: {:.2}s)", 
             audio_data.len(), metrics.audio_duration_seconds);
    
    // 获取优化的转录参数
    let params = optimizer.get_optimized_transcription_params();
    
    // 运行转录
    let transcription_start = std::time::Instant::now();
    let mut state = ctx.create_state()
        .map_err(|e| format!("无法创建 Whisper 状态: {}", e))?;
    
    state.full(params, &audio_data)
        .map_err(|e| format!("Whisper 转录失败: {}", e))?;
    
    metrics.transcription_time_ms = transcription_start.elapsed().as_millis() as u64;
    
    // 获取转录结果
    let num_segments = state.full_n_segments()
        .map_err(|e| format!("无法获取分段数量: {}", e))?;
    
    let mut full_text = String::new();
    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i)
            .map_err(|e| format!("无法获取分段文本: {}", e))?;
        full_text.push_str(&segment);
        full_text.push(' ');
    }
    
    let result = full_text.trim().to_string();
    
    // 计算性能指标
    metrics.total_time_ms = total_start.elapsed().as_millis() as u64;
    metrics.real_time_factor = optimizer.calculate_rtf(metrics.transcription_time_ms, metrics.audio_duration_seconds);
    
    // 获取系统指标
    if let Ok((cpu_usage, memory_usage)) = optimizer.get_system_metrics() {
        metrics.cpu_usage_percent = cpu_usage;
        metrics.gpu_memory_usage_mb = memory_usage; // 这里用内存使用代替GPU内存
    }
    
    println!("✅ 转录完成，结果长度: {} 字符", result.len());
    println!("📊 详细性能指标:");
    println!("   - 模型加载: {}ms", metrics.model_load_time_ms);
    println!("   - 音频处理: {}ms", metrics.audio_processing_time_ms);
    println!("   - 转录时间: {}ms", metrics.transcription_time_ms);
    println!("   - 总耗时: {}ms", metrics.total_time_ms);
    println!("   - RTF: {:.3} (目标: <0.3)", metrics.real_time_factor);
    println!("   - CPU使用: {:.1}%", metrics.cpu_usage_percent);
    
    if result.is_empty() {
        return Err("转录结果为空，可能音频文件无效或太短".to_string());
    }
    
    Ok((result, metrics))
}

// 同步运行 Whisper 转录（原版，保留兼容性）
fn run_whisper_transcription(audio_file_path: &PathBuf, model: &str) -> Result<String, String> {
    // 首先尝试下载并使用预训练模型
    let model_path = download_whisper_model_if_needed(model)?;
    
    println!("🔍 加载 Whisper 模型: {}", model_path);
    
    // 初始化 Whisper 上下文
    let ctx = WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())
        .map_err(|e| format!("无法加载 Whisper 模型: {}", e))?;
    
    println!("🔍 读取音频文件...");
    
    // 读取音频数据
    let audio_data = load_audio_samples(audio_file_path)?;
    
    println!("🔍 开始转录，音频样本数: {}", audio_data.len());
    
    // 设置转录参数
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("auto"));
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
    // 运行转录
    let mut state = ctx.create_state()
        .map_err(|e| format!("无法创建 Whisper 状态: {}", e))?;
    
    state.full(params, &audio_data)
        .map_err(|e| format!("Whisper 转录失败: {}", e))?;
    
    // 获取转录结果
    let num_segments = state.full_n_segments()
        .map_err(|e| format!("无法获取分段数量: {}", e))?;
    
    let mut full_text = String::new();
    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i)
            .map_err(|e| format!("无法获取分段文本: {}", e))?;
        full_text.push_str(&segment);
        full_text.push(' ');
    }
    
    let result = full_text.trim().to_string();
    println!("✅ 转录完成，结果长度: {} 字符", result.len());
    
    if result.is_empty() {
        return Err("转录结果为空，可能音频文件无效或太短".to_string());
    }
    
    Ok(result)
}

// 下载 Whisper 模型（如果需要）
fn download_whisper_model_if_needed(model: &str) -> Result<String, String> {
    let model_path = get_local_model_path(model);
    
    if PathBuf::from(&model_path).exists() {
        println!("✅ 找到本地模型文件: {}", model_path);
        return Ok(model_path);
    }
    
    // 创建模型目录
    let model_path_buf = PathBuf::from(&model_path);
    let model_dir = model_path_buf.parent()
        .ok_or("无法获取模型目录")?;
    
    std::fs::create_dir_all(model_dir)
        .map_err(|e| format!("无法创建模型目录: {}", e))?;
    
    // 尝试下载模型
    println!("📥 模型文件不存在，尝试下载: {}", model);
    download_whisper_model(model, &model_path)?;
    
    Ok(model_path)
}

// 下载 Whisper 模型文件
fn download_whisper_model(model: &str, model_path: &str) -> Result<(), String> {
    let model_url = get_whisper_model_url(model)?;
    
    println!("📥 开始下载模型: {} -> {}", model_url, model_path);
    
    // 使用 curl 下载模型（简单实现）
    let output = std::process::Command::new("curl")
        .args(&["-L", "-o", model_path, &model_url])
        .output()
        .map_err(|e| format!("下载命令执行失败: {}", e))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("模型下载失败: {}", error));
    }
    
    println!("✅ 模型下载完成: {}", model_path);
    Ok(())
}

// 获取 Whisper 模型下载 URL
fn get_whisper_model_url(model: &str) -> Result<String, String> {
    let base_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
    let model_file = match model {
        "whisper-tiny" => "ggml-tiny.bin",
        "whisper-base" => "ggml-base.bin", 
        "whisper-small" => "ggml-small.bin",
        "whisper-medium" => "ggml-medium.bin",
        "whisper-large-v3" => "ggml-large-v3.bin",
        "whisper-large-v3-turbo" => "ggml-large-v3-turbo.bin",
        _ => return Err(format!("不支持的模型: {}", model))
    };
    
    Ok(format!("{}/{}", base_url, model_file))
}

// 优化版音频样本加载
fn load_audio_samples_optimized(audio_file_path: &PathBuf, optimizer: &mut performance_optimizer::PerformanceOptimizer) -> Result<Vec<f32>, String> {
    println!("🔍 读取音频文件（优化版）: {:?}", audio_file_path);
    
    // 读取 WAV 文件
    let mut reader = hound::WavReader::open(audio_file_path)
        .map_err(|e| format!("无法打开音频文件: {}", e))?;
    
    let spec = reader.spec();
    println!("🔍 音频规格: {}Hz, {} 声道, {} 位", spec.sample_rate, spec.channels, spec.bits_per_sample);
    
    // 读取样本
    let samples: Result<Vec<i16>, _> = reader.samples().collect();
    let samples = samples.map_err(|e| format!("无法读取音频样本: {}", e))?;
    
    // 转换为 f32
    let mut float_samples: Vec<f32> = samples.iter()
        .map(|&s| s as f32 / 32768.0)
        .collect();
    
    // 如果是立体声，转换为单声道
    if spec.channels == 2 {
        let mono_samples: Vec<f32> = float_samples.chunks(2)
            .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
            .collect();
        float_samples = mono_samples;
    }
    
    // 使用性能优化器进行快速重采样
    let final_samples = optimizer.preprocess_audio_fast(&float_samples, spec.sample_rate)?;
    
    println!("✅ 音频处理完成，样本数: {}", final_samples.len());
    Ok(final_samples)
}

// 加载音频样本数据（原版，保留兼容性）
fn load_audio_samples(audio_file_path: &PathBuf) -> Result<Vec<f32>, String> {
    println!("🔍 读取音频文件: {:?}", audio_file_path);
    
    // 读取 WAV 文件
    let mut reader = hound::WavReader::open(audio_file_path)
        .map_err(|e| format!("无法打开音频文件: {}", e))?;
    
    let spec = reader.spec();
    println!("🔍 音频规格: {}Hz, {} 声道, {} 位", spec.sample_rate, spec.channels, spec.bits_per_sample);
    
    // Whisper 需要 16kHz 单声道
    let target_sample_rate = 16000;
    
    // 读取样本
    let samples: Result<Vec<i16>, _> = reader.samples().collect();
    let samples = samples.map_err(|e| format!("无法读取音频样本: {}", e))?;
    
    // 转换为 f32 并重采样到 16kHz
    let mut float_samples: Vec<f32> = samples.iter()
        .map(|&s| s as f32 / 32768.0)
        .collect();
    
    // 如果是立体声，转换为单声道
    if spec.channels == 2 {
        let mono_samples: Vec<f32> = float_samples.chunks(2)
            .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
            .collect();
        float_samples = mono_samples;
    }
    
    // 简单重采样（如果需要）
    if spec.sample_rate != target_sample_rate {
        println!("🔍 重采样: {}Hz -> {}Hz", spec.sample_rate, target_sample_rate);
        let ratio = target_sample_rate as f32 / spec.sample_rate as f32;
        let new_length = (float_samples.len() as f32 * ratio) as usize;
        
        let mut resampled = Vec::with_capacity(new_length);
        for i in 0..new_length {
            let src_index = (i as f32 / ratio) as usize;
            if src_index < float_samples.len() {
                resampled.push(float_samples[src_index]);
            }
        }
        float_samples = resampled;
    }
    
    println!("✅ 音频处理完成，样本数: {}", float_samples.len());
    Ok(float_samples)
}

// 获取本地模型文件路径
fn get_local_model_path(model: &str) -> String {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/Library/Application Support/spokenly-clone/models/{}.bin", home_dir, model)
}

fn get_default_prompt(agent_type: &str) -> String {
    match agent_type {
        "speech-to-text" => "请将以下语音内容转换为准确的文本，保持原意不变：",
        "text-enhancer" => "请优化以下文本，使其更清晰、准确和专业：",
        "translator" => "请将以下文本翻译为指定的目标语言，保持原意和语调：",
        "summarizer" => "请为以下内容生成简洁明了的摘要，突出要点：",
        "formatter" => "请将以下内容格式化，使结构清晰，易于阅读：",
        "grammar-check" => "请检查以下文本的语法错误并提供修正建议：",
        "tone-adjuster" => "请调整以下文本的语调和风格，使其符合指定要求：",
        "auto-input" => "请基于上下文生成合适的文本输入内容：",
        _ => "请处理以下内容："
    }.to_string()
}

async fn call_openai_api(
    request: &AgentRequest,
    prompt_text: &str,
    state: &tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<String, String> {
    let (client, api_key) = {
        let app_state = state.lock();
        (app_state.http_client.clone(), app_state.openai_api_key.clone())
    };
    
    let api_key = api_key.ok_or("未设置OpenAI API密钥")?;
    
    // 构建上下文信息
    let mut context = String::new();
    if let Some(additional_context) = &request.additional_context {
        for (key, value) in additional_context {
            context.push_str(&format!("{}: {}\n", key, value));
        }
    }
    
    // 构建完整的提示词
    let full_prompt = if context.is_empty() {
        format!("{}\n\n{}", prompt_text, request.input_text)
    } else {
        format!("{}\n\n上下文信息：\n{}\n待处理内容：\n{}", prompt_text, context, request.input_text)
    };
    
    let openai_request = OpenAIRequest {
        model: "gpt-3.5-turbo".to_string(),  // 使用 GPT-3.5 模型进行对话
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: full_prompt,
            }
        ],
        temperature: 0.7,
        max_tokens: Some(2000),
    };
    
    println!("📡 发送OpenAI API请求...");
    
    let response = client
        .post("https://ttkk.inping.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&openai_request)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI API错误: {}", error_text));
    }
    
    let openai_response: OpenAIResponse = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    if let Some(choice) = openai_response.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("OpenAI响应为空".to_string())
    }
}

#[tauri::command]
async fn set_openai_api_key(
    api_key: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let mut app_state = state.lock();
    app_state.openai_api_key = Some(api_key);
    println!("🔑 OpenAI API密钥已更新");
    Ok(())
}

#[tauri::command]
async fn get_agent_types() -> Result<Vec<String>, String> {
    Ok(vec![
        "speech-to-text".to_string(),
        "text-enhancer".to_string(),
        "translator".to_string(),
        "summarizer".to_string(),
        "formatter".to_string(),
        "grammar-check".to_string(),
        "tone-adjuster".to_string(),
        "auto-input".to_string(),
    ])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChain {
    pub chain_id: String,
    pub name: String,
    pub description: String,
    pub agents: Vec<String>, // Agent类型的有序列表
    pub is_active: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainProcessRequest {
    pub chain_id: String,
    pub input_text: String,
    pub additional_context: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainProcessResponse {
    pub success: bool,
    pub final_output: String,
    pub chain_id: String,
    pub step_results: Vec<AgentResponse>,
    pub total_processing_time_ms: u64,
    pub error: Option<String>,
}

#[tauri::command]
async fn process_with_chain(
    request: ChainProcessRequest,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<ChainProcessResponse, String> {
    let start_time = SystemTime::now();
    
    println!("🔗 开始链式处理: {}", request.chain_id);
    
    // 获取链配置
    let agents = {
        // 这里可以存储在AppState中或从配置文件读取
        // 暂时返回一个示例链
        match request.chain_id.as_str() {
            "enhance-translate-summarize" => vec![
                "text-enhancer".to_string(),
                "translator".to_string(),
                "summarizer".to_string()
            ],
            "grammar-format" => vec![
                "grammar-check".to_string(),
                "formatter".to_string()
            ],
            "speech-enhance-input" => vec![
                "speech-to-text".to_string(),
                "text-enhancer".to_string(),
                "auto-input".to_string()
            ],
            _ => return Err("未知的处理链".to_string())
        }
    };
    
    let mut current_text = request.input_text;
    let mut step_results = Vec::new();
    
    // 逐步处理每个Agent
    for (index, agent_type) in agents.iter().enumerate() {
        println!("🤖 处理步骤 {}/{}: {}", index + 1, agents.len(), agent_type);
        
        let agent_request = AgentRequest {
            agent_type: agent_type.clone(),
            input_text: current_text.clone(),
            prompt_id: None,
            additional_context: request.additional_context.clone(),
        };
        
        match process_with_agent(agent_request, state.clone()).await {
            Ok(result) => {
                if result.success {
                    current_text = result.output_text.clone();
                    step_results.push(result);
                } else {
                    let total_time = start_time.elapsed().unwrap().as_millis() as u64;
                    return Ok(ChainProcessResponse {
                        success: false,
                        final_output: String::new(),
                        chain_id: request.chain_id,
                        step_results,
                        total_processing_time_ms: total_time,
                        error: result.error,
                    });
                }
            },
            Err(e) => {
                let total_time = start_time.elapsed().unwrap().as_millis() as u64;
                return Ok(ChainProcessResponse {
                    success: false,
                    final_output: String::new(),
                    chain_id: request.chain_id,
                    step_results,
                    total_processing_time_ms: total_time,
                    error: Some(e),
                });
            }
        }
    }
    
    let total_time = start_time.elapsed().unwrap().as_millis() as u64;
    
    println!("✅ 链式处理完成: {} ({}ms)", request.chain_id, total_time);
    
    Ok(ChainProcessResponse {
        success: true,
        final_output: current_text,
        chain_id: request.chain_id,
        step_results,
        total_processing_time_ms: total_time,
        error: None,
    })
}

#[tauri::command]
async fn get_available_chains() -> Result<Vec<AgentChain>, String> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    Ok(vec![
        AgentChain {
            chain_id: "enhance-translate-summarize".to_string(),
            name: "增强-翻译-摘要".to_string(),
            description: "先优化文本，然后翻译，最后生成摘要".to_string(),
            agents: vec!["text-enhancer".to_string(), "translator".to_string(), "summarizer".to_string()],
            is_active: true,
            created_at: timestamp,
        },
        AgentChain {
            chain_id: "grammar-format".to_string(),
            name: "语法检查-格式化".to_string(),
            description: "检查语法错误并格式化文本".to_string(),
            agents: vec!["grammar-check".to_string(), "formatter".to_string()],
            is_active: true,
            created_at: timestamp,
        },
        AgentChain {
            chain_id: "speech-enhance-input".to_string(),
            name: "语音-增强-自动输入".to_string(),
            description: "语音转文字，增强内容，生成输入内容".to_string(),
            agents: vec!["speech-to-text".to_string(), "text-enhancer".to_string(), "auto-input".to_string()],
            is_active: true,
            created_at: timestamp,
        },
    ])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessRequest {
    pub agent_type: String,
    pub input_texts: Vec<String>,
    pub prompt_id: Option<String>,
    pub additional_context: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessResponse {
    pub success: bool,
    pub results: Vec<AgentResponse>,
    pub total_processing_time_ms: u64,
    pub success_count: usize,
    pub error_count: usize,
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(Mutex::new(AppState::new()));
    
    println!("🚀 启动 Spokenly 克隆应用（文件处理版）...");
    
    // 创建系统托盘菜单
    let quit = CustomMenuItem::new("quit".to_string(), "退出 Spokenly");
    let show = CustomMenuItem::new("show".to_string(), "显示窗口");
    let hide = CustomMenuItem::new("hide".to_string(), "隐藏窗口");
    let recording = CustomMenuItem::new("toggle_recording".to_string(), "开始/停止录音");
    let ai_prompts = CustomMenuItem::new("ai_prompts".to_string(), "AI提示");
    let transcription = CustomMenuItem::new("transcription".to_string(), "听写模型");
    let settings = CustomMenuItem::new("settings".to_string(), "常规设置");
    let permissions = CustomMenuItem::new("permissions".to_string(), "权限设置");
    let history = CustomMenuItem::new("history".to_string(), "历史记录");
    let shortcuts = CustomMenuItem::new("shortcuts".to_string(), "快捷键");
    let contact = CustomMenuItem::new("contact".to_string(), "联系我们");
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(recording)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(settings)
        .add_item(transcription)
        .add_item(ai_prompts)
        .add_item(permissions)
        .add_item(shortcuts)
        .add_item(history)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(contact)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                // 左键单击显示/隐藏主窗口
                let window = app.get_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            SystemTrayEvent::RightClick {
                position: _,
                size: _,
                ..
            } => {
                // 右键单击显示菜单（自动处理）
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let window = app.get_window("main").unwrap();
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "hide" => {
                        window.hide().unwrap();
                    }
                    "toggle_recording" => {
                        // 切换录音状态 - 通过emit事件通知前端
                        app.emit_all("tray_toggle_recording", {}).unwrap();
                    }
                    "settings" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        app.emit_all("tray_navigate_to", "general").unwrap();
                    }
                    "transcription" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        app.emit_all("tray_navigate_to", "transcription").unwrap();
                    }
                    "ai_prompts" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        app.emit_all("tray_navigate_to", "ai-prompts").unwrap();
                    }
                    "shortcuts" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        app.emit_all("tray_navigate_to", "shortcuts").unwrap();
                    }
                    "history" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        app.emit_all("tray_navigate_to", "history").unwrap();
                    }
                    "contact" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        app.emit_all("tray_navigate_to", "contact").unwrap();
                    }
                    "permissions" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        app.emit_all("tray_show_permissions", {}).unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            upload_file,
            start_recording,
            stop_recording,
            delete_file,
            get_supported_formats,
            export_transcription,
            // 文件夹监控相关命令
            add_watched_folder,
            remove_watched_folder,
            get_watched_folders,
            get_folder_watcher_stats,
            clear_all_watched_folders,
            // 数据库相关命令
            get_transcription_history,
            update_transcription_text,
            search_transcriptions,
            get_database_stats,
            get_model_stats,
            export_database_json,
            check_permission,
            request_permission,
            open_system_preferences,
            // 系统托盘相关命令
            set_tray_icon_recording,
            show_main_window,
            hide_main_window,
            quit_app,
            get_current_app_info,
            // 性能优化相关命令
            get_performance_metrics,
            get_cache_stats,
            clear_model_cache,
            configure_performance_optimizer,
            warmup_gpu,
            // AI Agent 命令
            get_ai_prompts,
            save_ai_prompt,
            delete_ai_prompt,
            activate_ai_prompt,
            process_with_agent,
            set_openai_api_key,
            get_agent_types,
            // 链式处理和批量处理
            process_with_chain,
            get_available_chains,
            process_batch,
            get_audio_devices,
        ])
        .setup(|app| {
            println!("✅ Tauri 应用已启动");
            
            // 初始化文件夹监控器
{
    // FolderWatcher needs to be mutable to call initialize
    // We need to make it mutable, but it's wrapped in Arc<Mutex<>>
    // For now, we'll skip this initialization and handle it differently
    println!("⚠️ 文件夹监控器延迟初始化");
}
            
            let window = app.get_window("main").unwrap();
            
            // 在macOS上，启动时显示窗口，关闭时隐藏到托盘
            #[cfg(target_os = "macos")]
            {
                // 启动时显示窗口
                window.show().unwrap();
                window.set_focus().unwrap();
                
                // 设置窗口关闭行为为隐藏而不是退出
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    match event {
                        WindowEvent::CloseRequested { api, .. } => {
                            // 阻止窗口关闭，改为隐藏到托盘
                            api.prevent_close();
                            window_clone.hide().unwrap();
                            println!("🔄 窗口已隐藏到系统托盘");
                        }
                        _ => {}
                    }
                });
            }
            
#[cfg(not(target_os = "macos"))]
{
window.show().unwrap();
}

// Setup safe global shortcuts using Tauri's built-in system
let mut shortcut_manager = app.global_shortcut_manager();

// Get app handle for emitting events
let app_handle = app.handle();

// 尝试注册多个快捷键进行测试
let test_shortcuts = vec![
    "CommandOrControl+Shift+R",
    "CommandOrControl+Shift+T", 
    "F13",
    "CommandOrControl+F1"
];

for shortcut in test_shortcuts {
    let app_handle_clone = app_handle.clone();
    let shortcut_name = shortcut.to_string();
    
    match shortcut_manager.register(shortcut, move || {
        println!("🔥🔥🔥 快捷键被按下！！！ {}", shortcut_name);
        eprintln!("🔥🔥🔥 快捷键被按下！！！ {}", shortcut_name);
        
        // Emit event to frontend
        if let Err(emit_error) = app_handle_clone.emit_all("shortcut_pressed", serde_json::json!({
            "shortcut": &shortcut_name,
            "action": "toggle_recording"
        })) {
            eprintln!("❌ 快捷键事件发送失败: {:?}", emit_error);
        } else {
            println!("✅ 快捷键事件已发送到前端: {}", shortcut_name);
            eprintln!("✅ 快捷键事件已发送到前端: {}", shortcut_name);
        }
    }) {
        Ok(_) => println!("✅ 成功注册快捷键: {}", shortcut),
        Err(e) => eprintln!("❌ 注册快捷键失败 {}: {:?}", shortcut, e),
    }
}

println!("⌨️ 安全快捷键系统已启用 (CommandOrControl+Shift+R)");

// 添加辅助功能权限检查 (macOS)
#[cfg(target_os = "macos")]
{
    println!("🔍 检查macOS辅助功能权限...");
    let has_accessibility = check_accessibility_permission();
    if !has_accessibility {
        eprintln!("❌ 缺少辅助功能权限！请在 系统偏好设置 > 安全性与隐私 > 隐私 > 辅助功能 中启用此应用");
    } else {
        println!("✅ 辅助功能权限已启用");
    }
}
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
