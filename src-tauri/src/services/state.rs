use crate::core::{error::Result, types::*};
use crate::services::database::Database;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;

pub enum RecorderCommand {
    Start,
    Stop(tokio::sync::oneshot::Sender<Result<Vec<f32>>>),
}

pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub database: Arc<Database>,
    pub is_recording: Arc<Mutex<bool>>,
    pub recorder_tx: mpsc::UnboundedSender<RecorderCommand>,
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
            recorder_tx: tx,
        })
    }

    pub fn save_settings(&self, new_settings: AppSettings) -> Result<()> {
        self.database.save_settings(&new_settings)?;
        *self.settings.lock() = new_settings;
        Ok(())
    }

    pub async fn start_recording(&self) -> Result<()> {
        let mut is_recording = self.is_recording.lock();
        if *is_recording {
            return Err(crate::core::error::AppError::Other("Already recording".into()));
        }

        self.recorder_tx.send(RecorderCommand::Start)
            .map_err(|_| crate::core::error::AppError::Other("Recorder died".into()))?;
        *is_recording = true;
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
        self.recorder_tx.send(RecorderCommand::Stop(tx))
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
        }
    }
}
