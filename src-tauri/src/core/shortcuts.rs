use crate::core::error::Result;
use parking_lot::Mutex;
use rdev::{listen, Event, EventType, Key};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// 激活模式
#[derive(Debug, Clone, PartialEq)]
pub enum ActivationMode {
    Hold,
    Toggle,
    DoubleClick,
    HoldOrToggle,
}

impl ActivationMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "hold" => Self::Hold,
            "toggle" => Self::Toggle,
            "double-click" => Self::DoubleClick,
            "hold-or-toggle" => Self::HoldOrToggle,
            _ => Self::HoldOrToggle,
        }
    }
}

/// 解析快捷键字符串为 rdev Key 集合
fn parse_shortcut(shortcut: &str) -> Vec<Key> {
    shortcut
        .split('+')
        .filter_map(|part| {
            match part.trim() {
                "RightCommand" => Some(Key::MetaRight),
                "RightOption" => Some(Key::AltGr),
                "RightShift" => Some(Key::ShiftRight),
                "RightControl" => Some(Key::ControlRight),
                "Command" | "Cmd" | "Meta" => Some(Key::MetaLeft),
                "Option" | "Alt" => Some(Key::Alt),
                "Shift" => Some(Key::ShiftLeft),
                "Control" | "Ctrl" => Some(Key::ControlLeft),
                "Fn" => Some(Key::Function),
                "CommandOrControl" | "Super" => Some(Key::MetaLeft),
                "F1" | "f1" => Some(Key::F1),
                "F2" | "f2" => Some(Key::F2),
                "F3" | "f3" => Some(Key::F3),
                "F4" | "f4" => Some(Key::F4),
                "F5" | "f5" => Some(Key::F5),
                "F6" | "f6" => Some(Key::F6),
                "F7" | "f7" => Some(Key::F7),
                "F8" | "f8" => Some(Key::F8),
                "F9" | "f9" => Some(Key::F9),
                "F10" | "f10" => Some(Key::F10),
                "F11" | "f11" => Some(Key::F11),
                "F12" | "f12" => Some(Key::F12),
                "Space" | "space" => Some(Key::Space),
                "Tab" | "tab" => Some(Key::Tab),
                "Escape" | "Esc" | "escape" | "esc" => Some(Key::Escape),
                "Return" | "Enter" | "return" | "enter" => Some(Key::Return),
                "Backspace" | "backspace" => Some(Key::Backspace),
                s if s.len() == 1 => {
                    let c = s.to_lowercase().chars().next().unwrap();
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
                        '0' => Some(Key::Num0), '1' => Some(Key::Num1),
                        '2' => Some(Key::Num2), '3' => Some(Key::Num3),
                        '4' => Some(Key::Num4), '5' => Some(Key::Num5),
                        '6' => Some(Key::Num6), '7' => Some(Key::Num7),
                        '8' => Some(Key::Num8), '9' => Some(Key::Num9),
                        _ => None,
                    }
                }
                _ => None,
            }
        })
        .collect()
}

fn key_matches(pressed: &Key, target: &Key) -> bool {
    if pressed == target {
        return true;
    }
    match target {
        Key::MetaRight => matches!(pressed, Key::MetaRight),
        Key::AltGr => matches!(pressed, Key::AltGr),
        Key::ShiftRight => matches!(pressed, Key::ShiftRight),
        Key::ControlRight => matches!(pressed, Key::ControlRight),
        Key::MetaLeft => matches!(pressed, Key::MetaLeft | Key::MetaRight),
        Key::ShiftLeft => matches!(pressed, Key::ShiftLeft | Key::ShiftRight),
        Key::Alt => matches!(pressed, Key::Alt | Key::AltGr),
        Key::ControlLeft => matches!(pressed, Key::ControlLeft | Key::ControlRight),
        _ => false,
    }
}

fn key_matches_str(pressed_str: &str, target: &Key) -> bool {
    match target {
        Key::MetaRight => pressed_str == "MetaRight",
        Key::AltGr => pressed_str == "AltGr",
        Key::ShiftRight => pressed_str == "ShiftRight",
        Key::ControlRight => pressed_str == "ControlRight",
        Key::MetaLeft => pressed_str == "MetaLeft" || pressed_str == "MetaRight",
        Key::ShiftLeft => pressed_str == "ShiftLeft" || pressed_str == "ShiftRight",
        Key::Alt => pressed_str == "Alt" || pressed_str == "AltGr",
        Key::ControlLeft => pressed_str == "ControlLeft" || pressed_str == "ControlRight",
        Key::Function => pressed_str == "Function",
        _ => false,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 全局快捷键监听器
/// 关键设计：rdev::listen 只启动一次，通过共享状态切换快捷键和模式
/// 避免多次调用 rdev::listen 导致 macOS SIGTRAP 崩溃
pub struct HoldToTalkListener {
    /// 是否启用监听（false = 忽略所有事件，但线程不退出）
    enabled: Arc<AtomicBool>,
    /// rdev 监听线程是否已启动（只启动一次）
    listener_spawned: Arc<AtomicBool>,
    /// 当前快捷键
    shortcut_keys: Arc<Mutex<Vec<Key>>>,
    /// 当前激活模式
    activation_mode: Arc<Mutex<ActivationMode>>,
    /// 回调函数
    on_press: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
    on_release: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
    /// 内部状态（需要在切换快捷键时重置）
    is_recording: Arc<AtomicBool>,
    pressed_keys: Arc<Mutex<HashSet<String>>>,
    keys_down_time: Arc<AtomicU64>,
    last_click_time: Arc<AtomicU64>,
}

impl HoldToTalkListener {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            listener_spawned: Arc::new(AtomicBool::new(false)),
            shortcut_keys: Arc::new(Mutex::new(Vec::new())),
            activation_mode: Arc::new(Mutex::new(ActivationMode::HoldOrToggle)),
            on_press: Arc::new(Mutex::new(None)),
            on_release: Arc::new(Mutex::new(None)),
            is_recording: Arc::new(AtomicBool::new(false)),
            pressed_keys: Arc::new(Mutex::new(HashSet::new())),
            keys_down_time: Arc::new(AtomicU64::new(0)),
            last_click_time: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn set_shortcut(&self, shortcut: &str) {
        let keys = parse_shortcut(shortcut);
        println!("⌨️ 快捷键解析: {} -> {:?}", shortcut, keys);
        *self.shortcut_keys.lock() = keys;
        // 重置内部状态
        self.is_recording.store(false, Ordering::SeqCst);
        self.pressed_keys.lock().clear();
        self.keys_down_time.store(0, Ordering::SeqCst);
        self.last_click_time.store(0, Ordering::SeqCst);
    }

    pub fn set_activation_mode(&self, mode: &str) {
        let mode = ActivationMode::from_str(mode);
        println!("⌨️ 激活模式: {:?}", mode);
        *self.activation_mode.lock() = mode;
    }

    /// 启动监听。rdev::listen 只会被调用一次。
    /// 后续调用只更新回调函数并启用监听。
    pub fn start<P, R>(&self, on_press: P, on_release: R) -> Result<()>
    where
        P: Fn() + Send + Sync + 'static,
        R: Fn() + Send + Sync + 'static,
    {
        // 更新回调
        *self.on_press.lock() = Some(Arc::new(on_press));
        *self.on_release.lock() = Some(Arc::new(on_release));

        // 启用
        self.enabled.store(true, Ordering::SeqCst);

        // 只启动一次 rdev 监听线程
        if self.listener_spawned.compare_exchange(
            false, true, Ordering::SeqCst, Ordering::SeqCst
        ).is_ok() {
            self.spawn_listener();
        }

        Ok(())
    }

    fn spawn_listener(&self) {
        let enabled = self.enabled.clone();
        let shortcut_keys = self.shortcut_keys.clone();
        let activation_mode = self.activation_mode.clone();
        let on_press_holder = self.on_press.clone();
        let on_release_holder = self.on_release.clone();
        let is_recording = self.is_recording.clone();
        let pressed_keys = self.pressed_keys.clone();
        let keys_down_time = self.keys_down_time.clone();
        let last_click_time = self.last_click_time.clone();

        const HOLD_THRESHOLD_MS: u64 = 300;
        const DOUBLE_CLICK_MS: u64 = 400;

        thread::spawn(move || {
            let _ = listen(move |event: Event| {
                if !enabled.load(Ordering::SeqCst) {
                    return;
                }

                let target_keys = shortcut_keys.lock();
                if target_keys.is_empty() {
                    return;
                }

                let mode = activation_mode.lock().clone();

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let key_str = format!("{:?}", key);
                        let was_already_pressed = pressed_keys.lock().contains(&key_str);
                        pressed_keys.lock().insert(key_str);

                        let all_pressed = !target_keys.is_empty() && target_keys.iter().all(|target| {
                            let pressed = pressed_keys.lock();
                            pressed.iter().any(|p_str| {
                                let p_key = format!("{:?}", target);
                                if p_str == &p_key {
                                    return true;
                                }
                                key_matches_str(p_str, target)
                            })
                        });

                        if !all_pressed {
                            return;
                        }

                        if was_already_pressed && target_keys.len() == 1 {
                            return;
                        }

                        let recording = is_recording.load(Ordering::SeqCst);

                        match mode {
                            ActivationMode::Hold => {
                                if !recording {
                                    keys_down_time.store(now_ms(), Ordering::SeqCst);
                                    is_recording.store(true, Ordering::SeqCst);
                                    if let Some(cb) = on_press_holder.lock().as_ref() {
                                        cb();
                                    }
                                }
                            }
                            ActivationMode::Toggle => {
                                if recording {
                                    is_recording.store(false, Ordering::SeqCst);
                                    if let Some(cb) = on_release_holder.lock().as_ref() {
                                        cb();
                                    }
                                } else {
                                    is_recording.store(true, Ordering::SeqCst);
                                    if let Some(cb) = on_press_holder.lock().as_ref() {
                                        cb();
                                    }
                                }
                            }
                            ActivationMode::DoubleClick => {
                                let now = now_ms();
                                let last = last_click_time.load(Ordering::SeqCst);

                                if recording {
                                    is_recording.store(false, Ordering::SeqCst);
                                    last_click_time.store(0, Ordering::SeqCst);
                                    if let Some(cb) = on_release_holder.lock().as_ref() {
                                        cb();
                                    }
                                } else if last > 0 && now - last < DOUBLE_CLICK_MS {
                                    is_recording.store(true, Ordering::SeqCst);
                                    last_click_time.store(0, Ordering::SeqCst);
                                    if let Some(cb) = on_press_holder.lock().as_ref() {
                                        cb();
                                    }
                                } else {
                                    last_click_time.store(now, Ordering::SeqCst);
                                }
                            }
                            ActivationMode::HoldOrToggle => {
                                if recording {
                                    is_recording.store(false, Ordering::SeqCst);
                                    if let Some(cb) = on_release_holder.lock().as_ref() {
                                        cb();
                                    }
                                } else {
                                    keys_down_time.store(now_ms(), Ordering::SeqCst);
                                    is_recording.store(true, Ordering::SeqCst);
                                    if let Some(cb) = on_press_holder.lock().as_ref() {
                                        cb();
                                    }
                                }
                            }
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let key_str = format!("{:?}", key);
                        pressed_keys.lock().remove(&key_str);

                        let recording = is_recording.load(Ordering::SeqCst);
                        if !recording {
                            return;
                        }

                        let released_is_part = target_keys.is_empty()
                            || target_keys.iter().any(|t| key_matches(&key, t));

                        if !released_is_part {
                            return;
                        }

                        match mode {
                            ActivationMode::Hold => {
                                is_recording.store(false, Ordering::SeqCst);
                                if let Some(cb) = on_release_holder.lock().as_ref() {
                                    cb();
                                }
                            }
                            ActivationMode::Toggle | ActivationMode::DoubleClick => {
                                // release 不做任何事
                            }
                            ActivationMode::HoldOrToggle => {
                                let down_time = keys_down_time.load(Ordering::SeqCst);
                                let held_duration = now_ms() - down_time;

                                if held_duration >= HOLD_THRESHOLD_MS {
                                    is_recording.store(false, Ordering::SeqCst);
                                    if let Some(cb) = on_release_holder.lock().as_ref() {
                                        cb();
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            });
        });
    }

    pub fn stop(&self) {
        self.enabled.store(false, Ordering::SeqCst);
        // 重置录音状态
        self.is_recording.store(false, Ordering::SeqCst);
        self.pressed_keys.lock().clear();
    }

    pub fn is_running(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

impl Drop for HoldToTalkListener {
    fn drop(&mut self) {
        self.stop();
    }
}
