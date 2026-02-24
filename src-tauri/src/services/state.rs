use crate::core::{error::Result, types::*};
use crate::services::database::Database;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;

pub enum RecorderCommand {
    Start,
    Stop(tokio::sync::oneshot::Sender<Result<Vec<f32>>>),
    HealthCheck(tokio::sync::oneshot::Sender<bool>),
}

pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub database: Arc<Database>,
    pub is_recording: Arc<Mutex<bool>>,
    pub recorder_tx: Arc<Mutex<mpsc::UnboundedSender<RecorderCommand>>>,
}

impl AppState {
    pub fn new(db_path: &std::path::Path) -> Result<Self> {
        let database = Database::new(db_path)?;
        let settings = database.load_settings()?;

        let (tx, rx) = mpsc::unbounded_channel();
        std::thread::spawn(move || {
            recorder_thread(rx);
        });

        Ok(Self {
            settings: Arc::new(Mutex::new(settings)),
            database: Arc::new(database),
            is_recording: Arc::new(Mutex::new(false)),
            recorder_tx: Arc::new(Mutex::new(tx)),
        })
    }

    /// 检查录音线程是否健康，如果死亡则重启
    async fn ensure_recorder_alive(&self) -> Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let send_result = self.recorder_tx.lock().send(RecorderCommand::HealthCheck(tx));

        if send_result.is_err() || tokio::time::timeout(
            std::time::Duration::from_millis(100),
            rx
        ).await.is_err() {
            eprintln!("⚠️ 录音线程已死亡，正在重启...");
            self.restart_recorder_thread();
        }
        Ok(())
    }

    fn restart_recorder_thread(&self) {
        let (tx, rx) = mpsc::unbounded_channel();
        std::thread::spawn(move || {
            recorder_thread(rx);
        });
        *self.recorder_tx.lock() = tx;
        println!("✅ 录音线程已重启");
    }

    pub fn save_settings(&self, new_settings: AppSettings) -> Result<()> {
        // 先写数据库，成功后再更新内存状态（原子性保证）
        self.database.save_settings(&new_settings)?;
        *self.settings.lock() = new_settings;
        Ok(())
    }

    pub async fn start_recording(&self) -> Result<()> {
        {
            let is_recording = self.is_recording.lock();
            if *is_recording {
                return Err(crate::core::error::AppError::Other("Already recording".into()));
            }
        }

        // 确保录音线程存活
        self.ensure_recorder_alive().await?;

        self.recorder_tx.lock().send(RecorderCommand::Start)
            .map_err(|_| crate::core::error::AppError::Other("Recorder died".into()))?;
        *self.is_recording.lock() = true;
        Ok(())
    }

    pub async fn stop_recording(&self) -> Result<Vec<f32>> {
        {
            let is_recording = self.is_recording.lock();
            if !*is_recording {
                return Err(crate::core::error::AppError::Other("Not recording".into()));
            }
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.recorder_tx.lock().send(RecorderCommand::Stop(tx))
            .map_err(|_| crate::core::error::AppError::Other("Recorder died".into()))?;
        let samples = rx.await
            .map_err(|_| crate::core::error::AppError::Other("Recorder died".into()))??;

        *self.is_recording.lock() = false;
        Ok(samples)
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock()
    }
}

fn recorder_thread(mut rx: mpsc::UnboundedReceiver<RecorderCommand>) {
    use crate::core::audio::AudioRecorder;

    let recorder = AudioRecorder::new(RecordingConfig::default());

    while let Some(cmd) = rx.blocking_recv() {
        match cmd {
            RecorderCommand::Start => {
                let _ = recorder.start();
            }
            RecorderCommand::Stop(tx) => {
                let result = recorder.stop();
                let _ = tx.send(result);
            }
            RecorderCommand::HealthCheck(tx) => {
                let _ = tx.send(true);
            }
        }
    }
}
