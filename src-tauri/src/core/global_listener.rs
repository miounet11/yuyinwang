use parking_lot::Mutex;
use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub type KeyCallback = Arc<dyn Fn(Key, bool) + Send + Sync>;

pub struct GlobalKeyListener {
    running: Arc<AtomicBool>,
    callback: Arc<Mutex<Option<KeyCallback>>>,
}

impl GlobalKeyListener {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            callback: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start<F>(&self, callback: F) -> Result<(), String>
    where
        F: Fn(Key, bool) + Send + Sync + 'static,
    {
        if self.running.load(Ordering::SeqCst) {
            return Err("Listener already running".to_string());
        }

        *self.callback.lock() = Some(Arc::new(callback));
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let callback_ref = self.callback.clone();

        thread::spawn(move || {
            let _ = listen(move |event: Event| {
                if !running.load(Ordering::SeqCst) {
                    return;
                }

                if let Some(cb) = callback_ref.lock().as_ref() {
                    match event.event_type {
                        EventType::KeyPress(key) => cb(key, true),
                        EventType::KeyRelease(key) => cb(key, false),
                        _ => {}
                    }
                }
            });
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        *self.callback.lock() = None;
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for GlobalKeyListener {
    fn drop(&mut self) {
        self.stop();
    }
}
