// Spokenly Clone - 简化版本专注文件上传功能和AI Agent处理
#[cfg(target_os = "macos")]
use objc::runtime::{BOOL, YES};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl, class};
use tauri::{Manager, AppHandle, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, WindowEvent};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use uuid::Uuid;
use tokio::fs;
use reqwest::Client;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub timestamp: u64,
    pub duration: u64,
    pub model: String,
    pub confidence: f32,
    pub audio_file_path: Option<PathBuf>,
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

#[derive(Debug)]
pub struct AppState {
    pub is_recording: bool,
    pub transcription_history: Vec<TranscriptionEntry>,
    pub temp_dir: PathBuf,
    pub ai_prompts: Vec<AIPrompt>,
    pub http_client: Client,
    pub openai_api_key: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir().join("spokenly-clone");
        std::fs::create_dir_all(&temp_dir).ok();
        
        // 创建HTTP客户端
        let http_client = Client::new();
        
        // 从环境变量读取OpenAI API密钥
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
        
        Self {
            is_recording: false,
            transcription_history: Vec::new(),
            temp_dir,
            ai_prompts,
            http_client,
            openai_api_key,
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
    
    // 模拟转录过程 (在实际应用中这里会调用真实的API)
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    let file_name = file_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    
    let mock_transcription = format!(
        "这是从文件 {} 转录的示例文本。在实际应用中，这里会是真实的语音转录结果。",
        file_name
    );
    
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    let entry = TranscriptionEntry {
        id: entry_id,
        text: mock_transcription,
        timestamp,
        duration: 120, // 假设2分钟
        model: "file-upload-whisper".to_string(),
        confidence: 0.95,
        audio_file_path: Some(file_path.clone()),
    };
    
    // 添加到历史记录
    {
        let mut app_state = state.lock();
        app_state.transcription_history.insert(0, entry.clone());
        
        // 限制历史记录数量
        if app_state.transcription_history.len() > 100 {
            app_state.transcription_history.truncate(100);
        }
    }
    
    println!("✅ 文件转录完成: {}", entry.text);
    Ok(entry)
}

#[tauri::command]
fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    Ok(vec![
        AudioDevice {
            name: "MacBook Pro麦克风".to_string(),
            id: "builtin".to_string(),
            is_default: true,
            is_available: true,
        },
        AudioDevice {
            name: "\"iPhone\"的麦克风".to_string(),
            id: "iphone".to_string(),
            is_default: false,
            is_available: false,
        },
    ])
}

#[tauri::command]
async fn get_transcription_history(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<TranscriptionEntry>, String> {
    let app_state = state.lock();
    Ok(app_state.transcription_history.clone())
}

#[tauri::command]
async fn clear_history(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut app_state = state.lock();
    app_state.transcription_history.clear();
    println!("🗑️ 历史记录已清空");
    Ok(())
}

#[tauri::command]
async fn start_recording(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut app_state = state.lock();
    if !app_state.is_recording {
        app_state.is_recording = true;
        println!("🎤 开始录音...");
        Ok("Recording started".to_string())
    } else {
        Err("Already recording".to_string())
    }
}

#[tauri::command]
async fn stop_recording(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_handle: AppHandle
) -> Result<String, String> {
    let mut app_state = state.lock();
    if app_state.is_recording {
        app_state.is_recording = false;
        println!("⏹️ 停止录音");
        
        // 模拟转录结果
        let entry = TranscriptionEntry {
            id: Uuid::new_v4().to_string(),
            text: "这是一个示例转录结果".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            duration: 5,
            model: "gpt-4o-mini".to_string(),
            confidence: 0.95,
            audio_file_path: None,
        };
        
        app_state.transcription_history.insert(0, entry.clone());
        
        let _ = app_handle.emit_all("transcription_result", &entry);
        Ok("Recording stopped".to_string())
    } else {
        Err("Not recording".to_string())
    }
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
        
        // 删除关联的音频文件
        if let Some(file_path) = entry.audio_file_path {
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
        // 在真实应用中，这里可以使用 Cocoa API 或者执行 AppleScript
        // 例如: osascript -e "tell application \"System Events\" to get name of first application process whose frontmost is true"
        
        // 暂时提供一些模拟数据，在实际应用中会被真实的API调用替换
        let sample_apps = vec![
            ("访达", "com.apple.finder", "📁"),
            ("Safari", "com.apple.Safari", "🌐"),
            ("Chrome", "com.google.Chrome", "🔵"),
            ("Xcode", "com.apple.dt.Xcode", "🔨"),
            ("终端", "com.apple.Terminal", "⬛"),
            ("微信", "com.tencent.xinWeChat", "💬"),
            ("钉钉", "com.alibaba.DingTalk", "📞"),
            ("VS Code", "com.microsoft.VSCode", "📝"),
            ("Finder", "com.apple.finder", "📁"),
        ];
        
        // 随机选择一个应用作为演示
        let app_index = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize % sample_apps.len();
            
        let (app_name, bundle_id, icon) = &sample_apps[app_index];
        
        info.insert("name".to_string(), app_name.to_string());
        info.insert("bundle_id".to_string(), bundle_id.to_string());
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
    match permission.as_str() {
        "accessibility" => {
            let trusted: BOOL = unsafe { msg_send![class!(AXAPI), isProcessTrusted] };
            if trusted == YES {
                Ok("granted".to_string())
            } else {
                Ok("denied".to_string())
            }
        },
        "microphone" => {
            let status: i32 = unsafe {
                msg_send![class!(AVCaptureDevice), authorizationStatusForMediaType: "soun"]
            };
            match status {
                3 => Ok("granted".to_string()),  // AVAuthorizationStatusAuthorized
                2 => Ok("denied".to_string()),    // AVAuthorizationStatusDenied
                _ => Ok("not-determined".to_string()),
            }
        },
        "input-monitoring" => {
            let trusted: BOOL = unsafe { msg_send![class!(IOHIDCheckAccess), accessForType: 1] };  // kIOHIDRequestTypeListenEvent
            if trusted == YES {
                Ok("granted".to_string())
            } else {
                Ok("denied".to_string())
            }
        },
        "file-system" => Ok("granted".to_string()),
        "notifications" => Ok("not-determined".to_string()),
        "screen-recording" => Ok("not-determined".to_string()),
        _ => Ok("not-determined".to_string())
    }
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn check_permission(permission: String) -> Result<String, String> {
    // 其他操作系统的权限检查
    Ok("granted".to_string())
}

// 权限请求命令
#[tauri::command]
async fn request_permission(permission: String, _app_handle: AppHandle) -> Result<String, String> {
    match permission.as_str() {
        "microphone" => {
            // 暂时返回模拟结果
            Ok("granted".to_string())
        },
        "accessibility" => {
            // 引导用户到系统设置
            Ok("pending".to_string())
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

#[tauri::command]
async fn export_transcription(
    entry_id: String,
    export_format: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<String, String> {
    let (export_content, export_path) = {
        let app_state = state.lock();
        
        if let Some(entry) = app_state.transcription_history.iter().find(|e| e.id == entry_id) {
            let content = match export_format.as_str() {
                "txt" => entry.text.clone(),
                "json" => serde_json::to_string_pretty(entry).map_err(|e| e.to_string())?,
                "srt" => format!(
                    "1\n00:00:00,000 --> 00:00:{:02},000\n{}\n",
                    entry.duration.min(59),
                    entry.text
                ),
                _ => return Err("不支持的导出格式".to_string()),
            };
            
            let path = app_state.temp_dir.join(format!("export_{}_{}.{}", 
                entry.id, 
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                export_format
            ));
            
            (content, path)
        } else {
            return Err("未找到指定的转录记录".to_string());
        }
    };
    
    tokio::fs::write(&export_path, export_content).await
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    println!("📤 已导出到: {:?}", export_path);
    Ok(export_path.to_string_lossy().to_string())
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
    
    // 获取对应的提示词
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
    
    // 调用OpenAI API
    match call_openai_api(&request, &prompt_text, &state).await {
        Ok(output_text) => {
            let processing_time = start_time.elapsed().unwrap().as_millis() as u64;
            
            println!("✅ Agent处理完成: {} ({}ms)", request.agent_type, processing_time);
            
            Ok(AgentResponse {
                success: true,
                output_text,
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
        model: "gpt-3.5-turbo".to_string(),
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
        .post("https://api.openai.com/v1/chat/completions")
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
            get_audio_devices,
            get_transcription_history,
            clear_history,
            start_recording,
            stop_recording,
            delete_file,
            get_supported_formats,
            export_transcription,
            check_permission,
            request_permission,
            open_system_preferences,
            // 系统托盘相关命令
            set_tray_icon_recording,
            show_main_window,
            hide_main_window,
            quit_app,
            get_current_app_info,
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
            process_batch
        ])
        .setup(|app| {
            println!("✅ Tauri 应用已启动");
            
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
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
