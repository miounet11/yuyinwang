// Recording King - 重构版本
// 模块化架构，统一错误处理，清晰的关注点分离

use tauri::{Manager, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, WindowEvent, WindowBuilder, WindowUrl};
use std::sync::Arc;
use parking_lot::Mutex;
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
mod shortcuts;

// 保留的遗留模块（待进一步重构）
mod folder_watcher;
mod performance_optimizer;
mod security;

// 使用重构后的模块
use errors::{AppError, AppResult};
use config::AppSettings;
use audio::AudioDeviceManager;
use transcription::{TranscriptionService, TranscriptionEditor};
use ai_agent::AIAgentService;
use database::{DatabaseManager, HistoryManager};

// 安全模块

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


/// 创建悬浮输入窗口
fn create_floating_input_window(app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 检查窗口是否已存在
    if app_handle.get_window("floating-input").is_some() {
        return Ok(());
    }
    
    // 创建悬浮输入窗口
    let window = WindowBuilder::new(
        app_handle,
        "floating-input",
        WindowUrl::App("index.html".into()),
    )
    .title("")
    .decorations(false)
    .always_on_top(true)
    .resizable(false)
    .skip_taskbar(true)
    .inner_size(600.0, 120.0)
    .center()
    .visible(false)  // 初始隐藏，由快捷键触发显示
    .build()?;
    
    Ok(())
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
            // 新的权限管理命令
            system::check_all_permissions,
            system::open_permission_settings,
            system::get_permission_guide,
            system::show_permission_warning_dialog,
            system::check_critical_permissions,
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
            commands::confirm_event_received,
            // 录音状态管理命令
            commands::get_recording_state,
            commands::reset_recording_state,
            // 语音输入快捷键命令
            commands::register_voice_shortcut,
            commands::unregister_all_voice_shortcuts,
            commands::get_cursor_position,
            commands::insert_text_to_app,
            commands::configure_voice_shortcuts,
            commands::load_voice_shortcut_config,
            commands::trigger_voice_input_test,
            commands::show_floating_input,
            commands::debug_shortcut_status,
            // 悬浮助手命令
            commands::show_main_window,
            commands::show_settings,
            commands::open_quick_note,
            commands::show_clipboard_history,
            commands::show_search,
            commands::toggle_floating_assistant,
            commands::get_audio_level,
            commands::stop_recording_and_transcribe,
            // 长按快捷键命令
            commands::start_long_press_monitoring,
            commands::test_long_press_trigger,
            commands::get_long_press_status,
            // macOS 语音输入命令
            commands::get_active_app_info_for_voice,
            commands::start_voice_recording,
            commands::stop_voice_recording,
            commands::inject_text_to_active_app,
        ])
        .setup(|app| {
            let app_handle = app.handle();
            
            // 获取应用状态以便管理历史管理器
            let state = app.state::<AppState>();
            app.manage(state.history_manager.clone());
            app.manage(state.transcription_editor.clone());
            
            // 初始化原有的快捷键管理器
            let shortcut_manager = commands::shortcut_management::ShortcutManager::new();
            app.manage(shortcut_manager);
            
            // 初始化语音输入快捷键管理器
            let voice_shortcut_manager = Arc::new(shortcuts::ShortcutManager::new(app_handle.clone()));
            app.manage(voice_shortcut_manager.clone());
            
            // 创建悬浮输入窗口
            match create_floating_input_window(&app_handle) {
                Ok(_) => println!("✅ 悬浮输入窗口创建成功"),
                Err(e) => eprintln!("❌ 悬浮输入窗口创建失败: {}", e),
            }
            
            // 使用新的全局快捷键管理器
            match shortcuts::EnhancedShortcutManager::new(app_handle.clone()) {
                Ok(global_manager) => {
                    if let Err(e) = global_manager.register_shortcuts() {
                        eprintln!("⚠️ 注册全局快捷键失败: {}", e);
                    }
                    // 保存管理器实例
                    app.manage(Arc::new(global_manager));
                }
                Err(e) => {
                    eprintln!("⚠️ 创建全局快捷键管理器失败: {}", e);
                    // 回退到旧的快捷键系统
                    if let Err(e) = voice_shortcut_manager.register_voice_input_shortcut("Option+Space", "press") {
                        eprintln!("⚠️ 回退快捷键注册也失败: {}", e);
                    }
                }
            }
            
            println!("✅ 历史管理器已注册");
            println!("✅ 转录编辑器已注册");
            println!("✅ 快捷键管理器已注册");
            println!("✅ 语音输入快捷键管理器已注册");
            
            // 移除直接快捷键注册，改由 enhancedShortcutManager 统一管理
            println!("ℹ️ 快捷键注册已委托给 enhancedShortcutManager");
            
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