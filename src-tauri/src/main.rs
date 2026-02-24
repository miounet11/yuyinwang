#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod services;

use services::{quick_input::QuickInputService, state::AppState};
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowBuilder, WindowUrl};

fn main() {
    let voice_input = CustomMenuItem::new("voice_input".to_string(), "🎤 语音输入");
    let start_recording = CustomMenuItem::new("start_recording".to_string(), "开始录音");
    let show = CustomMenuItem::new("show".to_string(), "显示主窗口");
    let settings = CustomMenuItem::new("settings".to_string(), "设置");
    let shortcut_hint = CustomMenuItem::new("shortcut_hint".to_string(), "快捷键: Cmd+Shift+Space").disabled();
    let quit = CustomMenuItem::new("quit".to_string(), "退出 Recording King");

    let tray_menu = SystemTrayMenu::new()
        .add_item(voice_input)
        .add_item(start_recording)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show)
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(shortcut_hint)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                // 左键点击托盘图标 → 显示主窗口
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    let service = app.state::<QuickInputService>();
                    service.unregister_shortcut();
                    println!("👋 Recording King shutting down");
                    std::process::exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "settings" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.emit("navigate", "settings");
                    }
                }
                "voice_input" | "start_recording" => {
                    // 触发快速语音输入
                    let service = app.state::<QuickInputService>();
                    let app_handle = app.app_handle();
                    if let Err(e) = service.trigger_quick_input(app_handle) {
                        eprintln!("Failed to trigger quick input: {}", e);
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                if event.window().label() == "main" {
                    event.window().hide().unwrap();
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            let app_dir = app
                .path_resolver()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir)?;

            let db_path = app_dir.join("recording-king.db");
            let state = AppState::new(&db_path).expect("Failed to initialize app state");

            // 检查辅助功能权限，未授权时弹出系统引导
            #[cfg(target_os = "macos")]
            {
                if !crate::core::injection::check_accessibility_permission() {
                    println!("⚠️  Requesting accessibility permission...");
                    crate::core::injection::request_accessibility_permission();
                }
            }

            let saved_shortcut = state.settings.lock().shortcut_key.clone();
            app.manage(state);

            let quick_input = QuickInputService::new();
            app.manage(quick_input);

            // 自动恢复之前的按住说话快捷键
            if let Some(shortcut_key) = saved_shortcut {
                let service = app.state::<QuickInputService>();
                let app_handle = app.app_handle();
                if let Err(e) = service.register_shortcut(&shortcut_key, app_handle) {
                    eprintln!("Failed to restore shortcut {}: {}", shortcut_key, e);
                }
            }

            // 创建悬浮输入窗口（不抢焦点）
            let _quick_input_window = WindowBuilder::new(
                app,
                "quick-input",
                WindowUrl::App("quick-input.html".into()),
            )
            .title("Quick Input")
            .decorations(false)
            .always_on_top(true)
            .resizable(false)
            .skip_taskbar(true)
            .focused(false)
            .inner_size(360.0, 80.0)
            .center()
            .visible(false)
            .build()?;

            println!("✅ Recording King v7.0 started");
            println!("🎤 按住说话模式就绪");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::recording::get_recording_state,
            commands::recording::get_audio_devices,
            commands::recording::transcribe_file,
            commands::history::get_history,
            commands::history::search_history,
            commands::history::delete_entry,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::injection::inject_text,
            commands::injection::check_injection_permission,
            commands::injection::request_injection_permission,
            commands::quick_input::quick_input_is_active,
            commands::quick_input::register_global_shortcut,
            commands::quick_input::unregister_global_shortcut,
            commands::models::get_local_model_status,
            commands::models::download_local_model,
            commands::models::delete_local_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
