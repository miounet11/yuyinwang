use crate::errors::AppResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub audio: AudioSettings,
    pub transcription: TranscriptionSettings,
    pub ai: AiSettings,
    pub ui: UiSettings,
    pub storage: StorageSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub default_device_id: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub chunk_size: usize,
    pub enable_noise_reduction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSettings {
    pub default_model: String,
    pub language: Option<String>,
    pub temperature: f32,
    pub enable_local_whisper: bool,
    pub api_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub openai_api_key: Option<String>,
    pub default_temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub enable_system_tray: bool,
    pub start_minimized: bool,
    pub global_shortcut: String,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub data_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub max_history_entries: usize,
    pub auto_cleanup_days: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        let home_dir = directories::UserDirs::new()
            .map(|dirs| dirs.home_dir().join("Library/Application Support/spokenly-clone"))
            .unwrap_or_else(|| std::env::temp_dir().join("spokenly-clone"));

        Self {
            audio: AudioSettings {
                default_device_id: None,
                sample_rate: 16000,
                channels: 1,
                chunk_size: 1024,
                enable_noise_reduction: false,
            },
            transcription: TranscriptionSettings {
                default_model: "luyingwang-online".to_string(),  // 默认使用LuYinWang在线转录服务
                language: None,
                temperature: 0.0,
                enable_local_whisper: false,  // 默认不使用本地whisper
                api_timeout_seconds: 30,
            },
            ai: AiSettings {
                openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
                default_temperature: 0.7,
                max_tokens: 1000,
                timeout_seconds: 30,
            },
            ui: UiSettings {
                enable_system_tray: true,
                start_minimized: false,
                global_shortcut: "CommandOrControl+Shift+R".to_string(),
                theme: "system".to_string(),
            },
            storage: StorageSettings {
                data_dir: home_dir.clone(),
                temp_dir: std::env::temp_dir().join("spokenly-clone"),
                max_history_entries: 1000,
                auto_cleanup_days: 30,
            },
        }
    }
}

impl AppSettings {
    pub fn load() -> AppResult<Self> {
        let settings_path = Self::get_settings_path()?;
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(settings_path)?;
            let mut settings: AppSettings = serde_json::from_str(&content)?;
            
            // 迁移旧的模型配置到新的默认设置
            let needs_migration = settings.transcription.default_model == "whisper-1" 
                || settings.transcription.default_model.starts_with("whisper-");
            
            if needs_migration {
                println!("🔄 迁移旧的转录模型配置: {} → luyingwang-online", settings.transcription.default_model);
                settings.transcription.default_model = "luyingwang-online".to_string();
                settings.transcription.enable_local_whisper = false;
                settings.save()?; // 保存迁移后的配置
            }
            
            Ok(settings)
        } else {
            let settings = Self::default();
            settings.save()?;
            Ok(settings)
        }
    }
    
    pub fn save(&self) -> AppResult<()> {
        let settings_path = Self::get_settings_path()?;
        
        // 确保配置目录存在
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(settings_path, content)?;
        Ok(())
    }
    
    fn get_settings_path() -> AppResult<PathBuf> {
        let home_dir = directories::UserDirs::new()
            .ok_or("无法获取用户目录")?
            .home_dir()
            .join("Library/Application Support/spokenly-clone");
        
        Ok(home_dir.join("settings.json"))
    }
    
    pub fn ensure_directories(&self) -> AppResult<()> {
        std::fs::create_dir_all(&self.storage.data_dir)?;
        std::fs::create_dir_all(&self.storage.temp_dir)?;
        Ok(())
    }
}