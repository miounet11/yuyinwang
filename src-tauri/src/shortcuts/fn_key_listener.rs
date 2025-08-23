use rdev::{listen, Event, EventType, Key};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::Manager;
use crate::errors::AppResult;

pub struct FnKeyListener {
    app_handle: tauri::AppHandle,
    is_running: Arc<Mutex<bool>>,
    last_fn_press: Arc<Mutex<Option<Instant>>>,
    // 新增：记录 Fn/F1 是否处于按下状态以支持长按语音输入
    is_fn_down: Arc<Mutex<bool>>,
    // 新增：Alt+Space 组合按键状态与延迟释放
    alt_down: Arc<Mutex<bool>>,
    space_down: Arc<Mutex<bool>>,
    hold_release_deadline: Arc<Mutex<Option<Instant>>>,
}

impl FnKeyListener {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            app_handle,
            is_running: Arc::new(Mutex::new(false)),
            last_fn_press: Arc::new(Mutex::new(None)),
            is_fn_down: Arc::new(Mutex::new(false)),
            alt_down: Arc::new(Mutex::new(false)),
            space_down: Arc::new(Mutex::new(false)),
            hold_release_deadline: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn start(&self) -> AppResult<()> {
        let mut running = self.is_running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);
        
        let app_handle = self.app_handle.clone();
        let is_running = self.is_running.clone();
        let last_fn_press = self.last_fn_press.clone();
        let is_fn_down = self.is_fn_down.clone();
        let alt_down = self.alt_down.clone();
        let space_down = self.space_down.clone();
        let hold_release_deadline = self.hold_release_deadline.clone();
        
        thread::spawn(move || {
            println!("🎮 特殊键监听器已启动");
            
            let start_hold = |app_handle: &tauri::AppHandle| {
                if let Some(window) = app_handle.get_window("floating-input") {
                    let _ = window.emit("voice_input_hold_start", ());
                }
                let _ = app_handle.emit_all("progressive_trigger_activated", serde_json::json!({
                    "trigger": "hold",
                    "shortcut": "Hold",
                    "timestamp": chrono::Utc::now().timestamp_millis(),
                }));
                let app_handle_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::commands::start_progressive_voice_input(
                        None,
                        app_handle_clone,
                        Some(true),
                    ).await;
                });
            };

            let stop_hold = |app_handle: &tauri::AppHandle| {
                if let Some(window) = app_handle.get_window("floating-input") {
                    let _ = window.emit("voice_input_hold_end", ());
                }
                let _ = app_handle.emit_all("quick_voice_key_released", ());
                let app_handle_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::commands::stop_voice_recording(app_handle_clone).await;
                });
            };
            
            let callback = move |event: Event| {
                match event.event_type {
                    // ====== Hold-to-talk: Fn/Globe (macOS) 和 F1 作为备用 ======
                    EventType::KeyPress(Key::Function) | EventType::KeyPress(Key::F1) => {
                        let mut down = is_fn_down.lock().unwrap();
                        if !*down {
                            *down = true;
                            println!("🔴 Fn/F1 按下：启动渐进式语音输入 (hold)");
                            start_hold(&app_handle);
                        }
                    }
                    EventType::KeyRelease(Key::Function) | EventType::KeyRelease(Key::F1) => {
                        let mut down = is_fn_down.lock().unwrap();
                        if *down {
                            *down = false;
                            println!("🟢 Fn/F1 松开：停止语音输入 (hold)");
                            stop_hold(&app_handle);
                        }
                    }

                    // ====== Hold-to-talk: Option(Alt) + Space 组合 ======
                    EventType::KeyPress(Key::Alt) => {
                        *alt_down.lock().unwrap() = true;
                        if *space_down.lock().unwrap() {
                            println!("🔴 Alt+Space 按下：启动语音输入 (hold)");
                            start_hold(&app_handle);
                        }
                        // 保留双击行为
                        check_double_press(&last_fn_press, &app_handle, "Option");
                    }
                    EventType::KeyPress(Key::Space) => {
                        *space_down.lock().unwrap() = true;
                        if *alt_down.lock().unwrap() {
                            println!("🔴 Alt+Space 按下：启动语音输入 (hold)");
                            start_hold(&app_handle);
                        }
                    }
                    EventType::KeyRelease(Key::Alt) => {
                        *alt_down.lock().unwrap() = false;
                        // 若 Space 已松开或即将松开，触发延迟结束
                        let mut deadline = hold_release_deadline.lock().unwrap();
                        *deadline = Some(Instant::now() + Duration::from_millis(150));
                        drop(deadline);
                        schedule_delayed_release(&app_handle, &hold_release_deadline, &alt_down, &space_down, &stop_hold);
                    }
                    EventType::KeyRelease(Key::Space) => {
                        *space_down.lock().unwrap() = false;
                        let mut deadline = hold_release_deadline.lock().unwrap();
                        *deadline = Some(Instant::now() + Duration::from_millis(150));
                        drop(deadline);
                        schedule_delayed_release(&app_handle, &hold_release_deadline, &alt_down, &space_down, &stop_hold);
                    }

                    // ====== 其他双击快捷触发 ======
                    EventType::KeyPress(Key::MetaRight) => {
                        println!("🔑 检测到右 Command 键按下");
                        check_double_press(&last_fn_press, &app_handle, "RightCmd");
                    }
                    EventType::KeyPress(Key::CapsLock) => {
                        println!("🔑 检测到 Caps Lock 键按下");
                        check_double_press(&last_fn_press, &app_handle, "CapsLock");
                    }
                    _ => {}
                }
            };
            
            // 这会阻塞当前线程
            if let Err(error) = listen(callback) {
                eprintln!("❌ 键监听器错误: {:?}", error);
                *is_running.lock().unwrap() = false;
            }
        });
        
        Ok(())
    }
    
    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
        println!("🛑  键监听器已停止");
    }
}

fn schedule_delayed_release<F>(
    app_handle: &tauri::AppHandle,
    deadline: &Arc<Mutex<Option<Instant>>>,
    alt_down: &Arc<Mutex<bool>>,
    space_down: &Arc<Mutex<bool>>,
    stop_hold: &F,
) where F: Fn(&tauri::AppHandle) + Send + Sync + 'static {
    let app = app_handle.clone();
    let dl = deadline.clone();
    let a = alt_down.clone();
    let s = space_down.clone();
    // 简单延迟线程，避免频繁松按导致的抖动
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        let now = Instant::now();
        let can_release = {
            let alt = *a.lock().unwrap();
            let space = *s.lock().unwrap();
            let d = *dl.lock().unwrap();
            !alt && !space && d.map(|t| now >= t).unwrap_or(true)
        };
        if can_release {
            stop_hold(&app);
        }
    });
}

fn check_double_press(
    last_press: &Arc<Mutex<Option<Instant>>>, 
    app_handle: &tauri::AppHandle,
    key_name: &str
) {
    let now = Instant::now();
    let mut last_press_guard = last_press.lock().unwrap();
    
    if let Some(last_time) = *last_press_guard {
        let duration = now.duration_since(last_time);
        if duration < Duration::from_millis(500) {
            // 双击检测到
            println!("⚡ 检测到 {} 双击，触发悬浮输入窗口", key_name);
            trigger_floating_input(app_handle);
            *last_press_guard = None; // 重置以避免连续触发
            return;
        }
    }
    
    *last_press_guard = Some(now);
}

fn trigger_floating_input(app_handle: &tauri::AppHandle) {
    // 显示悬浮输入窗口
    if let Some(window) = app_handle.get_window("floating-input") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("voice_input_triggered", ());
    } else {
        eprintln!("❌ 悬浮输入窗口未找到");
    }
}