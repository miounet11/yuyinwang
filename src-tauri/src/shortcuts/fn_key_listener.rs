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
}

impl FnKeyListener {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            app_handle,
            is_running: Arc::new(Mutex::new(false)),
            last_fn_press: Arc::new(Mutex::new(None)),
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
        
        thread::spawn(move || {
            println!("🎮 特殊键监听器已启动");
            
            let callback = move |event: Event| {
                // 尝试多种键位
                match event.event_type {
                    // Globe/Fn 键 (macOS 特殊处理)
                    EventType::KeyPress(Key::Function) => {
                        println!("🔑 检测到 Function 键按下");
                        check_double_press(&last_fn_press, &app_handle, "Function");
                    }
                    // F键作为备用选项
                    EventType::KeyPress(Key::F1) => {
                        println!("🔑 检测到 F1 键按下");
                        check_double_press(&last_fn_press, &app_handle, "F1");
                    }
                    // Option/Alt 键双击
                    EventType::KeyPress(Key::Alt) => {
                        println!("🔑 检测到 Option/Alt 键按下");
                        check_double_press(&last_fn_press, &app_handle, "Option");
                    }
                    // 右 Command 键
                    EventType::KeyPress(Key::MetaRight) => {
                        println!("🔑 检测到右 Command 键按下");
                        check_double_press(&last_fn_press, &app_handle, "RightCmd");
                    }
                    // Caps Lock 键
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
        let _ = window.emit("floating_input_triggered", ());
    } else {
        eprintln!("❌ 悬浮输入窗口未找到");
    }
}