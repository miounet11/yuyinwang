// Recording King - 重构版本
// 模块化架构，统一错误处理，清晰的关注点分离

use tauri::{Manager, AppHandle, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, WindowEvent, GlobalShortcutManager};
use std::sync::Arc;
use parking_lot::Mutex;
use std::path::PathBuf;
use reqwest::Client;

// 核心模块导入
mod errors;
mod types;
mod config;
mod audio;
mod transcription;
mod ai_agent;
mod database;
mod subtitle;
mod system;
mod commands;

// 保留的遗留模块（待进一步重构）
mod folder_watcher;
mod performance_optimizer;
mod security;

// 使用重构后的模块
use errors::{AppError, AppResult};
use types::*;
use config::AppSettings;
use audio::{AudioRecorder, AudioDeviceManager, AudioProcessor};
use transcription::{TranscriptionService, TranscriptionEditor};
use ai_agent::AIAgentService;
use database::{DatabaseManager, HistoryManager};

// 安全模块
use security::command_executor::SecureCommandExecutor;

// 权限检查相关（macOS）
#[cfg(target_os = "macos")]
fn check_accessibility_permission() -> bool {
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

// 应用状态管理
pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub is_recording: Arc<Mutex<bool>>,
    pub database: Arc<DatabaseManager>,
    pub history_manager: Arc<HistoryManager>,
    pub transcription_service: Arc<TranscriptionService>,
    pub transcription_editor: Arc<TranscriptionEditor>,
    pub ai_agent_service: Arc<Mutex<AIAgentService>>,
    pub audio_device_manager: Arc<AudioDeviceManager>,
    pub audio_recorder: Arc<Mutex<audio::AudioRecorder>>,
    pub folder_watcher: Arc<folder_watcher::FolderWatcher>,
    pub performance_optimizer: Arc<Mutex<performance_optimizer::PerformanceOptimizer>>,
}

impl AppState {
    pub fn new() -> AppResult<Self> {
        // 加载配置
        let settings = AppSettings::load()?;
        settings.ensure_directories()?;
        
        // 创建HTTP客户端
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::NetworkError(format!("创建HTTP客户端失败: {}", e)))?;
        
        // 初始化数据库
        let db_path = settings.storage.data_dir.join("spokenly.db");
        let database = Arc::new(DatabaseManager::new(&db_path)?);
        
        // 初始化历史管理器
        let history_manager = HistoryManager::new(database.clone());
        
        // 初始化服务
        let transcription_service = TranscriptionService::new(
            http_client.clone(),
            settings.ai.openai_api_key.clone(),
        );
        
        let ai_agent_service = AIAgentService::new(
            http_client,
            settings.ai.openai_api_key.clone().unwrap_or_default(),
            ai_agent::AgentConfig::default(),
        );
        
        let audio_device_manager = AudioDeviceManager::new();
        
        // 初始化音频录制器
        let default_config = types::RecordingConfig {
            device_id: None,
            sample_rate: 16000,
            channels: 1,
            duration_seconds: None,
            buffer_duration: Some(3.0),
        };
        let audio_recorder = audio::AudioRecorder::new(default_config);
        
        // 初始化转录编辑器
        let transcription_editor = TranscriptionEditor::new();
        
        Ok(Self {
            settings: Arc::new(Mutex::new(settings)),
            is_recording: Arc::new(Mutex::new(false)),
            database: database.clone(),
            history_manager: Arc::new(history_manager),
            transcription_service: Arc::new(transcription_service),
            transcription_editor: Arc::new(transcription_editor),
            ai_agent_service: Arc::new(Mutex::new(ai_agent_service)),
            audio_device_manager: Arc::new(audio_device_manager),
            audio_recorder: Arc::new(Mutex::new(audio_recorder)),
            folder_watcher: Arc::new(folder_watcher::FolderWatcher::new()),
            performance_optimizer: Arc::new(Mutex::new(performance_optimizer::PerformanceOptimizer::new())),
        })
    }
}


fn main() {
    println!("🎙️ Recording King 启动中...");
    
    // 初始化应用状态
    let app_state = match AppState::new() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("❌ 应用初始化失败: {}", e);
            std::process::exit(1);
        }
    };
    
    // 创建系统托盘
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let show = CustomMenuItem::new("show".to_string(), "显示");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);
    
    tauri::Builder::default()
        .manage(app_state)
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "quit" => {
                            std::process::exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::transcribe_file,
            commands::get_transcription_history,
            commands::process_ai_agent,
            commands::get_audio_devices,
            commands::test_audio_input,
            commands::start_recording,
            commands::stop_recording,
            commands::get_app_settings,
            commands::update_app_settings,
            // 权限管理命令
            commands::check_permission,
            commands::request_permission,
            commands::open_system_preferences,
            // 历史记录管理命令
            commands::advanced_search_entries,
            commands::grouped_search_entries,
            commands::bulk_operation_entries,
            commands::get_history_statistics,
            commands::get_smart_suggestions,
            commands::export_history_entries,
            commands::cleanup_history_entries,
            commands::quick_search_entries,
            commands::search_entries_by_date,
            commands::search_entries_by_model,
            commands::get_recent_entries,
            commands::filter_entries_by_confidence,
            commands::filter_entries_by_duration,
            commands::bulk_delete_entries,
            commands::bulk_add_tag,
            commands::bulk_remove_tag,
            commands::bulk_export_entries,
            commands::get_data_integrity_report,
            commands::build_search_options,
            commands::get_search_suggestions,
            commands::save_search_preset,
            commands::load_search_preset,
            commands::get_search_presets,
            // 转录编辑命令
            commands::create_transcription_document,
            commands::get_transcription_document,
            commands::smart_split_text,
            commands::split_paragraph,
            commands::merge_paragraphs,
            commands::edit_paragraph,
            commands::find_and_replace,
            commands::undo_document_edit,
            commands::redo_document_edit,
            commands::save_transcription_document,
            commands::get_document_edit_history,
            commands::is_document_dirty,
            commands::list_open_documents,
            commands::close_transcription_document,
            commands::set_auto_save_interval,
            commands::get_document_statistics,
            commands::export_document,
            commands::import_document,
            // 字幕生成命令
            commands::generate_subtitle_file,
            commands::batch_generate_subtitles,
            commands::merge_subtitles,
            commands::preview_subtitle,
            commands::get_subtitle_statistics,
            commands::get_supported_subtitle_formats,
            commands::get_default_subtitle_options,
            // 文本注入命令
            commands::inject_text_to_cursor,
            commands::smart_inject_text,
            commands::check_text_injection_permission,
            commands::get_active_app_info,
            commands::test_text_injection,
            commands::batch_inject_text,
            commands::get_default_text_injection_config,
            commands::validate_text_injection_config,
            commands::clear_text_injection_history,
            // 快捷键管理命令
            commands::register_global_shortcut,
            commands::unregister_global_shortcut,
            commands::is_shortcut_registered,
            commands::get_registered_shortcuts,
            commands::register_multiple_shortcuts,
            commands::unregister_all_shortcuts,
            commands::validate_shortcut_format,
            commands::check_shortcut_conflicts,
            commands::test_shortcut,
        ])
        .setup(|app| {
            let app_handle = app.handle();
            
            // 获取应用状态以便管理历史管理器
            let state = app.state::<AppState>();
            app.manage(state.history_manager.clone());
            app.manage(state.transcription_editor.clone());
            
            // 初始化快捷键管理器
            let shortcut_manager = commands::shortcut_management::ShortcutManager::new();
            app.manage(shortcut_manager);
            
            println!("✅ 历史管理器已注册");
            println!("✅ 转录编辑器已注册");
            println!("✅ 快捷键管理器已注册");
            
            // 注册全局快捷键
            let shortcut = "CommandOrControl+Shift+R";
            let app_handle_clone = app_handle.clone();
            
            if let Err(e) = app_handle.global_shortcut_manager().register(shortcut, move || {
                println!("🔥 快捷键被按下: {}", shortcut);
                
                // 发送事件到前端
                if let Err(emit_error) = app_handle_clone.emit_all("shortcut_pressed", serde_json::json!({
                    "shortcut": shortcut,
                    "action": "toggle_recording"
                })) {
                    eprintln!("❌ 快捷键事件发送失败: {:?}", emit_error);
                } else {
                    println!("✅ 快捷键事件已发送到前端");
                }
            }) {
                eprintln!("❌ 注册快捷键失败: {:?}", e);
            } else {
                println!("✅ 成功注册快捷键: {}", shortcut);
            }
            
            println!("⌨️ 快捷键系统已启用 (CommandOrControl+Shift+R)");
            
            // 检查macOS权限
            #[cfg(target_os = "macos")]
            {
                println!("🔍 检查macOS权限...");
                let has_accessibility = check_accessibility_permission();
                if !has_accessibility {
                    eprintln!("❌ 缺少辅助功能权限！");
                } else {
                    println!("✅ 辅助功能权限已启用");
                }
            }
            
            println!("🚀 Recording King 启动完成");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}