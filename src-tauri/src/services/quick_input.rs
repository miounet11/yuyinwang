use crate::core::{error::Result, shortcuts::HoldToTalkListener, transcription::TranscriptionService, types::*};
use crate::services::state::AppState;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

pub struct QuickInputService {
    listener: Arc<HoldToTalkListener>,
    is_active: Arc<Mutex<bool>>,
    original_app: Arc<Mutex<Option<String>>>,
}

impl QuickInputService {
    pub fn new() -> Self {
        Self {
            listener: Arc::new(HoldToTalkListener::new()),
            is_active: Arc::new(Mutex::new(false)),
            original_app: Arc::new(Mutex::new(None)),
        }
    }

    pub fn register_shortcut(&self, key: &str, app_handle: AppHandle) -> Result<()> {
        self.listener.stop();
        self.listener.set_shortcut(key);

        let is_active = self.is_active.clone();
        let original_app = self.original_app.clone();
        let app_handle_press = app_handle.clone();
        let app_handle_release = app_handle;

        let on_press = move || {
            let is_active = is_active.clone();
            let original_app = original_app.clone();
            let app = app_handle_press.clone();

            tauri::async_runtime::spawn(async move {
                if *is_active.lock().await {
                    return;
                }

                #[cfg(target_os = "macos")]
                {
                    if let Ok(bundle_id) = crate::core::injection::get_frontmost_app_bundle_id() {
                        *original_app.lock().await = Some(bundle_id);
                    }
                }

                if let Some(window) = app.get_window("quick-input") {
                    let _ = window.show();
                }

                let state = app.state::<AppState>();
                if let Err(e) = state.start_recording().await {
                    println!("❌ 录音启动失败: {}", e);
                    let _ = app.emit_all("quick-input-error", e.to_string());
                    return;
                }

                *is_active.lock().await = true;
                let _ = app.emit_all("quick-input-started", ());
            });
        };

        let is_active_release = self.is_active.clone();
        let original_app_release = self.original_app.clone();

        let on_release = move || {
            let is_active = is_active_release.clone();
            let original_app = original_app_release.clone();
            let app = app_handle_release.clone();

            tauri::async_runtime::spawn(async move {
                if !*is_active.lock().await {
                    return;
                }
                *is_active.lock().await = false;

                let state = app.state::<AppState>();

                let samples = match state.stop_recording().await {
                    Ok(s) => s,
                    Err(e) => {
                        if let Some(w) = app.get_window("quick-input") { let _ = w.hide(); }
                        let _ = app.emit_all("quick-input-error", e.to_string());
                        return;
                    }
                };

                let _ = app.emit_all("quick-input-transcribing", ());
                if let Some(w) = app.get_window("quick-input") { let _ = w.hide(); }

                #[cfg(target_os = "macos")]
                let saved_app = original_app.lock().await.take();
                #[cfg(not(target_os = "macos"))]
                let saved_app: Option<String> = None;

                #[cfg(target_os = "macos")]
                if let Some(ref bundle_id) = saved_app {
                    let _ = crate::core::injection::activate_app(bundle_id);
                }

                // 从设置读取 token + model
                let settings = state.settings.lock().clone();
                let mut service = TranscriptionService::new(settings.clone());
                if let Some(dir) = app.path_resolver().app_data_dir() {
                    service = service.with_app_data_dir(dir);
                }
                let result = service.transcribe_samples(&samples, 16000).await;

                match result {
                    Ok(transcription) => {
                        let entry = TranscriptionEntry {
                            id: uuid::Uuid::new_v4().to_string(),
                            text: transcription.text.clone(),
                            timestamp: chrono::Utc::now().timestamp(),
                            duration: transcription.duration.unwrap_or(0.0),
                            model: settings.selected_model.clone(),
                            confidence: 1.0,
                            audio_file_path: None,
                        };
                        let _ = state.database.save_transcription(&entry);
                        let _ = app.emit_all("quick-input-result", &transcription.text);

                        if settings.auto_inject && !transcription.text.is_empty() {
                            let text = transcription.text.clone();
                            let delay = settings.inject_delay_ms;
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_millis(100));
                                let _ = crate::core::injection::inject_text(&text, delay);
                            });
                        }
                    }
                    Err(e) => {
                        let _ = app.emit_all("quick-input-error", e.to_string());
                    }
                }
            });
        };

        self.listener.start(on_press, on_release)?;
        println!("⌨️ 按住说话快捷键已注册: {}", key);
        Ok(())
    }

    pub fn unregister_shortcut(&self) {
        self.listener.stop();
    }

    pub fn trigger_quick_input(&self, app_handle: AppHandle) -> Result<()> {
        let is_active = self.is_active.clone();
        let original_app = self.original_app.clone();

        tauri::async_runtime::spawn(async move {
            let currently_active = *is_active.lock().await;
            if currently_active {
                *is_active.lock().await = false;
                let state = app_handle.state::<AppState>();

                let samples = match state.stop_recording().await {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = app_handle.emit_all("quick-input-error", e.to_string());
                        if let Some(w) = app_handle.get_window("quick-input") { let _ = w.hide(); }
                        return;
                    }
                };

                let _ = app_handle.emit_all("quick-input-transcribing", ());
                if let Some(w) = app_handle.get_window("quick-input") { let _ = w.hide(); }

                #[cfg(target_os = "macos")]
                let saved_app = original_app.lock().await.take();
                #[cfg(not(target_os = "macos"))]
                let saved_app: Option<String> = None;

                #[cfg(target_os = "macos")]
                if let Some(ref bundle_id) = saved_app {
                    let _ = crate::core::injection::activate_app(bundle_id);
                }

                let settings = state.settings.lock().clone();
                let mut service = TranscriptionService::new(settings.clone());
                if let Some(dir) = app_handle.path_resolver().app_data_dir() {
                    service = service.with_app_data_dir(dir);
                }
                match service.transcribe_samples(&samples, 16000).await {
                    Ok(transcription) => {
                        let entry = TranscriptionEntry {
                            id: uuid::Uuid::new_v4().to_string(),
                            text: transcription.text.clone(),
                            timestamp: chrono::Utc::now().timestamp(),
                            duration: transcription.duration.unwrap_or(0.0),
                            model: settings.selected_model.clone(),
                            confidence: 1.0,
                            audio_file_path: None,
                        };
                        let _ = state.database.save_transcription(&entry);
                        let _ = app_handle.emit_all("quick-input-result", &transcription.text);

                        if settings.auto_inject && !transcription.text.is_empty() {
                            let text = transcription.text.clone();
                            let delay = settings.inject_delay_ms;
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_millis(100));
                                let _ = crate::core::injection::inject_text(&text, delay);
                            });
                        }
                    }
                    Err(e) => {
                        let _ = app_handle.emit_all("quick-input-error", e.to_string());
                    }
                }
            } else {
                #[cfg(target_os = "macos")]
                {
                    if let Ok(bundle_id) = crate::core::injection::get_frontmost_app_bundle_id() {
                        *original_app.lock().await = Some(bundle_id);
                    }
                }

                if let Some(w) = app_handle.get_window("quick-input") { let _ = w.show(); }

                let state = app_handle.state::<AppState>();
                if let Err(e) = state.start_recording().await {
                    let _ = app_handle.emit_all("quick-input-error", e.to_string());
                    return;
                }

                *is_active.lock().await = true;
                let _ = app_handle.emit_all("quick-input-started", ());
            }
        });

        Ok(())
    }

    pub async fn is_active(&self) -> bool {
        *self.is_active.lock().await
    }
}
