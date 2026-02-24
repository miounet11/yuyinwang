use crate::core::error::Result;
use parking_lot::Mutex;
use rdev::{listen, Event, EventType, Key};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// 解析快捷键字符串为 rdev Key 集合
/// 支持格式: "CommandOrControl+Shift+Space", "Alt+Z" 等
fn parse_shortcut(shortcut: &str) -> Vec<Key> {
    shortcut
        .split('+')
        .filter_map(|part| {
            match part.trim().to_lowercase().as_str() {
                "commandorcontrol" | "cmd" | "command" | "meta" | "super" => Some(Key::MetaLeft),
                "shift" => Some(Key::ShiftLeft),
                "alt" | "option" => Some(Key::Alt),
                "control" | "ctrl" => Some(Key::ControlLeft),
                "space" => Some(Key::Space),
                "tab" => Some(Key::Tab),
                "escape" | "esc" => Some(Key::Escape),
                "return" | "enter" => Some(Key::Return),
                "backspace" => Some(Key::Backspace),
                s if s.len() == 1 => {
                    // 单字符: a-z
                    let c = s.chars().next().unwrap();
                    match c {
                        'a' => Some(Key::KeyA), 'b' => Some(Key::KeyB),
                        'c' => Some(Key::KeyC), 'd' => Some(Key::KeyD),
                        'e' => Some(Key::KeyE), 'f' => Some(Key::KeyF),
                        'g' => Some(Key::KeyG), 'h' => Some(Key::KeyH),
                        'i' => Some(Key::KeyI), 'j' => Some(Key::KeyJ),
                        'k' => Some(Key::KeyK), 'l' => Some(Key::KeyL),
                        'm' => Some(Key::KeyM), 'n' => Some(Key::KeyN),
                        'o' => Some(Key::KeyO), 'p' => Some(Key::KeyP),
                        'q' => Some(Key::KeyQ), 'r' => Some(Key::KeyR),
                        's' => Some(Key::KeyS), 't' => Some(Key::KeyT),
                        'u' => Some(Key::KeyU), 'v' => Some(Key::KeyV),
                        'w' => Some(Key::KeyW), 'x' => Some(Key::KeyX),
                        'y' => Some(Key::KeyY), 'z' => Some(Key::KeyZ),
                        _ => None,
                    }
                }
                _ => None,
            }
        })
        .collect()
}

/// 检查一个 Key 是否匹配目标（左右修饰键都算）
fn key_matches(pressed: &Key, target: &Key) -> bool {
    if pressed == target {
        return true;
    }
    match target {
        Key::MetaLeft => matches!(pressed, Key::MetaLeft | Key::MetaRight),
        Key::ShiftLeft => matches!(pressed, Key::ShiftLeft | Key::ShiftRight),
        Key::Alt => matches!(pressed, Key::Alt | Key::AltGr),
        Key::ControlLeft => matches!(pressed, Key::ControlLeft | Key::ControlRight),
        _ => false,
    }
}

/// 按住说话的全局快捷键监听器
/// 用 rdev 监听全局键盘事件，检测快捷键组合的按下和松开
pub struct HoldToTalkListener {
    running: Arc<AtomicBool>,
    shortcut_keys: Arc<Mutex<Vec<Key>>>,
}

impl HoldToTalkListener {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            shortcut_keys: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 设置快捷键（如 "CommandOrControl+Shift+Space"）
    pub fn set_shortcut(&self, shortcut: &str) {
        let keys = parse_shortcut(shortcut);
        println!("⌨️ 快捷键解析: {} -> {:?}", shortcut, keys);
        *self.shortcut_keys.lock() = keys;
    }

    /// 启动监听
    /// on_press: 所有快捷键都按下时触发
    /// on_release: 任一快捷键松开时触发
    pub fn start<P, R>(&self, on_press: P, on_release: R) -> Result<()>
    where
        P: Fn() + Send + Sync + 'static,
        R: Fn() + Send + Sync + 'static,
    {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let shortcut_keys = self.shortcut_keys.clone();
        let is_active = Arc::new(AtomicBool::new(false));
        let pressed_keys: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        let on_press = Arc::new(on_press);
        let on_release = Arc::new(on_release);

        thread::spawn(move || {
            let _ = listen(move |event: Event| {
                if !running.load(Ordering::SeqCst) {
                    return;
                }

                let target_keys = shortcut_keys.lock();
                if target_keys.is_empty() {
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        // 记录按下的键
                        let key_str = format!("{:?}", key);
                        pressed_keys.lock().insert(key_str);

                        // 检查是否所有快捷键都按下了
                        let all_pressed = target_keys.iter().all(|target| {
                            let pressed = pressed_keys.lock();
                            pressed.iter().any(|p_str| {
                                // 简单匹配：检查按下的键是否包含目标键名
                                let p_key = format!("{:?}", target);
                                // 左右修饰键都算
                                if p_str == &p_key {
                                    return true;
                                }
                                match target {
                                    Key::MetaLeft => p_str == "MetaLeft" || p_str == "MetaRight",
                                    Key::ShiftLeft => p_str == "ShiftLeft" || p_str == "ShiftRight",
                                    Key::Alt => p_str == "Alt" || p_str == "AltGr",
                                    Key::ControlLeft => p_str == "ControlLeft" || p_str == "ControlRight",
                                    _ => false,
                                }
                            })
                        });

                        if all_pressed && !is_active.load(Ordering::SeqCst) {
                            is_active.store(true, Ordering::SeqCst);
                            on_press();
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let key_str = format!("{:?}", key);
                        pressed_keys.lock().remove(&key_str);

                        if is_active.load(Ordering::SeqCst) {
                            // 任一快捷键松开就触发 release
                            let target_keys_ref = &*target_keys;
                            let released_is_part = target_keys_ref.iter().any(|t| key_matches(&key, t));

                            if released_is_part {
                                is_active.store(false, Ordering::SeqCst);
                                on_release();
                            }
                        }
                    }
                    _ => {}
                }
            });
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for HoldToTalkListener {
    fn drop(&mut self) {
        self.stop();
    }
}
