use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use crate::api::ApiConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub hotkey: String,
    pub selected_device: Option<String>,
    pub selected_model: String,
    pub api_config: ApiConfig,
    pub auto_start: bool,
    pub show_in_dock: bool,
    pub show_in_status_bar: bool,
    pub sound_effects: bool,
    pub mute_on_recording: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            hotkey: "CommandOrControl+Shift+Space".to_string(),
            selected_device: None,
            selected_model: "gpt-4o-mini".to_string(),
            api_config: ApiConfig {
                openai_api_key: None,
                deepgram_api_key: None,
                mistral_api_key: None,
                elevenlabs_api_key: None,
            },
            auto_start: false,
            show_in_dock: false,
            show_in_status_bar: true,
            sound_effects: true,
            mute_on_recording: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub timestamp: u64,
    pub duration: u64,
    pub model: String,
    pub confidence: f32,
    pub audio_file_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct StorageManager {
    app_dir: PathBuf,
    settings_file: PathBuf,
    history_file: PathBuf,
    audio_dir: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let project_dirs = ProjectDirs::from("com", "spokenly", "spokenly-clone")
            .ok_or("Failed to get project directories")?;
        
        let app_dir = project_dirs.data_dir().to_path_buf();
        let settings_file = app_dir.join("settings.json");
        let history_file = app_dir.join("transcription_history.json");
        let audio_dir = app_dir.join("audio_recordings");

        Ok(Self {
            app_dir,
            settings_file,
            history_file,
            audio_dir,
        })
    }

    pub async fn ensure_directories(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.app_dir).await?;
        fs::create_dir_all(&self.audio_dir).await?;
        Ok(())
    }

    // 设置管理
    pub async fn save_settings(&self, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_directories().await?;
        let json = serde_json::to_string_pretty(settings)?;
        fs::write(&self.settings_file, json).await?;
        println!("⚙️ 设置已保存到: {:?}", self.settings_file);
        Ok(())
    }

    pub async fn load_settings(&self) -> Result<AppSettings, Box<dyn std::error::Error>> {
        if self.settings_file.exists() {
            let json = fs::read_to_string(&self.settings_file).await?;
            let settings: AppSettings = serde_json::from_str(&json)?;
            println!("📂 设置已加载从: {:?}", self.settings_file);
            Ok(settings)
        } else {
            println!("⚙️ 使用默认设置");
            Ok(AppSettings::default())
        }
    }

    // 转录历史管理
    pub async fn save_transcription_entry(&self, entry: &TranscriptionEntry) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_directories().await?;
        
        let mut history = self.load_transcription_history().await.unwrap_or_default();
        history.insert(0, entry.clone()); // 插入到开头
        
        // 限制历史记录数量（最多1000条）
        if history.len() > 1000 {
            history.truncate(1000);
        }
        
        let json = serde_json::to_string_pretty(&history)?;
        fs::write(&self.history_file, json).await?;
        println!("📝 转录记录已保存");
        Ok(())
    }

    pub async fn load_transcription_history(&self) -> Result<Vec<TranscriptionEntry>, Box<dyn std::error::Error>> {
        if self.history_file.exists() {
            let json = fs::read_to_string(&self.history_file).await?;
            let history: Vec<TranscriptionEntry> = serde_json::from_str(&json)?;
            println!("📂 加载了 {} 条转录历史记录", history.len());
            Ok(history)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn clear_transcription_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.history_file.exists() {
            fs::remove_file(&self.history_file).await?;
        }
        println!("🗑️ 转录历史已清空");
        Ok(())
    }

    // 音频文件管理
    pub fn get_audio_file_path(&self, entry_id: &str) -> PathBuf {
        self.audio_dir.join(format!("{}.wav", entry_id))
    }

    pub async fn save_audio_file(&self, entry_id: &str, audio_data: &[u8]) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.ensure_directories().await?;
        let file_path = self.get_audio_file_path(entry_id);
        fs::write(&file_path, audio_data).await?;
        println!("🎵 音频文件已保存到: {:?}", file_path);
        Ok(file_path)
    }

    pub async fn cleanup_old_audio_files(&self, days: u64) -> Result<(), Box<dyn std::error::Error>> {
        use std::time::{Duration, SystemTime};
        
        if !self.audio_dir.exists() {
            return Ok(());
        }

        let cutoff_time = SystemTime::now() - Duration::from_secs(days * 24 * 60 * 60);
        let mut entries = fs::read_dir(&self.audio_dir).await?;
        let mut deleted_count = 0;

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_time {
                        if let Err(e) = fs::remove_file(entry.path()).await {
                            eprintln!("删除音频文件失败 {:?}: {}", entry.path(), e);
                        } else {
                            deleted_count += 1;
                        }
                    }
                }
            }
        }

        if deleted_count > 0 {
            println!("🗑️ 已清理 {} 个旧音频文件", deleted_count);
        }
        Ok(())
    }

    // 导出功能
    pub async fn export_history_to_json(&self, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let history = self.load_transcription_history().await?;
        let json = serde_json::to_string_pretty(&history)?;
        fs::write(file_path, json).await?;
        println!("📤 转录历史已导出到: {:?}", file_path);
        Ok(())
    }

    pub async fn export_history_to_csv(&self, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let history = self.load_transcription_history().await?;
        let mut csv_content = "ID,Text,Timestamp,Duration,Model,Confidence\n".to_string();
        
        for entry in history {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{}\n",
                entry.id,
                entry.text.replace(',', ";").replace('\n', " "),
                entry.timestamp,
                entry.duration,
                entry.model,
                entry.confidence
            ));
        }
        
        fs::write(file_path, csv_content).await?;
        println!("📤 转录历史已导出为CSV到: {:?}", file_path);
        Ok(())
    }
}