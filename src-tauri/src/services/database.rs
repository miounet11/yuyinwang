use crate::core::{error::Result, types::*};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                duration REAL NOT NULL,
                model TEXT NOT NULL,
                confidence REAL NOT NULL,
                audio_file_path TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON transcriptions(timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn save_transcription(&self, entry: &TranscriptionEntry) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO transcriptions (id, text, timestamp, duration, model, confidence, audio_file_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                &entry.id,
                &entry.text,
                entry.timestamp,
                entry.duration,
                &entry.model,
                entry.confidence,
                &entry.audio_file_path,
            ],
        )?;
        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<TranscriptionEntry>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, text, timestamp, duration, model, confidence, audio_file_path
             FROM transcriptions
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let entries = stmt
            .query_map([limit], |row| {
                Ok(TranscriptionEntry {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    duration: row.get(3)?,
                    model: row.get(4)?,
                    confidence: row.get(5)?,
                    audio_file_path: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<TranscriptionEntry>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, text, timestamp, duration, model, confidence, audio_file_path
             FROM transcriptions
             WHERE text LIKE ?1 ESCAPE '\\'
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        // 转义 LIKE 特殊字符 % 和 _
        let escaped_query = query.replace('\\', "\\\\")
                                 .replace('%', "\\%")
                                 .replace('_', "\\_");
        let search_pattern = format!("%{}%", escaped_query);
        let entries = stmt
            .query_map(rusqlite::params![search_pattern, limit], |row| {
                Ok(TranscriptionEntry {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    duration: row.get(3)?,
                    model: row.get(4)?,
                    confidence: row.get(5)?,
                    audio_file_path: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM transcriptions WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let conn = self.conn.lock();
        let mut settings_to_save = settings.clone();
        // Encode sensitive tokens
        if let Some(api_key) = &settings.openai_api_key {
            settings_to_save.openai_api_key = Some(BASE64.encode(api_key));
        }
        if let Some(token) = &settings.luyin_token {
            settings_to_save.luyin_token = Some(BASE64.encode(token));
        }
        let encoded_json = serde_json::to_string(&settings_to_save)?;

        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('settings', ?1)",
            [encoded_json],
        )?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<AppSettings> {
        let conn = self.conn.lock();
        let result = conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'settings'",
            [],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(json) => {
                let mut settings: AppSettings = serde_json::from_str(&json)?;
                // Decode sensitive tokens
                if let Some(encoded_key) = &settings.openai_api_key {
                    if let Ok(decoded) = BASE64.decode(encoded_key) {
                        if let Ok(key_str) = String::from_utf8(decoded) {
                            settings.openai_api_key = Some(key_str);
                        }
                    }
                }
                if let Some(encoded_token) = &settings.luyin_token {
                    if let Ok(decoded) = BASE64.decode(encoded_token) {
                        if let Ok(token_str) = String::from_utf8(decoded) {
                            settings.luyin_token = Some(token_str);
                        }
                    }
                }
                Ok(settings)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings::default()),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[cfg(test)]
    use uuid::Uuid;

    #[cfg(test)]
    use chrono::Utc;

    #[test]
    fn test_database_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path);
        assert!(db.is_ok());
    }

    #[test]
    fn test_save_and_load_settings() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        let settings = AppSettings {
            openai_api_key: Some("test-key".to_string()),
            luyin_token: None,
            selected_model: "whisper-1".to_string(),
            auto_inject: true,
            inject_delay_ms: 200,
            shortcut_key: Some("Ctrl+Shift+V".to_string()),
            display_style: "notch".to_string(),
            appearance: "dark".to_string(),
            ui_language: "zh-CN".to_string(),
            launch_at_login: true,
            show_in_dock: false,
            show_in_menu_bar: true,
            esc_to_cancel: true,
            shortcut_preset: "right-cmd".to_string(),
            custom_shortcut: None,
            activation_mode: "toggle".to_string(),
            microphone_priority: vec!["device-1".to_string(), "device-2".to_string()],
            onboarding_complete: true,
            word_replacements: vec![],
        };

        db.save_settings(&settings).unwrap();
        let loaded = db.load_settings().unwrap();

        assert_eq!(loaded, settings);
    }

    #[test]
    fn test_load_settings_default_when_empty() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        let loaded = db.load_settings().unwrap();
        assert_eq!(loaded, AppSettings::default());
    }

    #[test]
    fn test_api_key_not_plaintext() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        let settings = AppSettings {
            openai_api_key: Some("secret-key-123".to_string()),
            luyin_token: None,
            selected_model: "whisper-1".to_string(),
            auto_inject: false,
            inject_delay_ms: 100,
            shortcut_key: None,
            ..Default::default()
        };

        db.save_settings(&settings).unwrap();

        // Check database directly
        let conn = db.conn.lock();
        let stored_value: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // Stored value should not contain plaintext key
        assert!(!stored_value.contains("secret-key-123"));
    }

    #[test]
    fn test_transcription_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        let entry = TranscriptionEntry {
            id: "test-id".to_string(),
            text: "Hello world".to_string(),
            timestamp: 1234567890,
            duration: 5.0,
            model: "whisper-1".to_string(),
            confidence: 0.95,
            audio_file_path: None,
        };

        db.save_transcription(&entry).unwrap();

        let history = db.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].text, "Hello world");

        db.delete(&entry.id).unwrap();
        let history = db.get_history(10).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[cfg(test)]
    use proptest::prelude::*;

    #[cfg(test)]
    proptest! {
        #[test]
        fn prop_settings_database_roundtrip(api_key in proptest::option::of("[a-zA-Z0-9]{20,50}"),
                                            model in "[a-z]{5,15}",
                                            auto_inject: bool,
                                            delay in 50u64..1000u64) {
            let dir = tempdir().unwrap();
            let db_path = dir.path().join("test.db");
            let db = Database::new(&db_path).unwrap();

            let settings = AppSettings {
                openai_api_key: api_key,
                luyin_token: None,
                selected_model: model,
                auto_inject,
                inject_delay_ms: delay,
                shortcut_key: None,
                ..Default::default()
            };

            db.save_settings(&settings).unwrap();
            let loaded = db.load_settings().unwrap();

            prop_assert_eq!(loaded, settings);
        }
    }

    #[test]
    fn test_backward_compatibility_old_settings() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        // Simulate old settings JSON without new fields
        let old_settings_json = r#"{
            "openai_api_key": null,
            "luyin_token": null,
            "selected_model": "luyin-free",
            "auto_inject": false,
            "inject_delay_ms": 100,
            "shortcut_key": null
        }"#;

        let conn = db.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('settings', ?1)",
            [old_settings_json],
        )
        .unwrap();
        drop(conn);

        // Load settings should fill in defaults for missing fields
        let loaded = db.load_settings().unwrap();
        assert_eq!(loaded.display_style, "panel");
        assert_eq!(loaded.appearance, "system");
        assert_eq!(loaded.ui_language, "system");
        assert_eq!(loaded.launch_at_login, false);
        assert_eq!(loaded.show_in_dock, true);
        assert_eq!(loaded.show_in_menu_bar, true);
        assert_eq!(loaded.esc_to_cancel, true);
        assert_eq!(loaded.shortcut_preset, "none");
        assert_eq!(loaded.activation_mode, "hold-or-toggle");
        assert_eq!(loaded.microphone_priority.len(), 0);
        assert_eq!(loaded.onboarding_complete, false);
        assert_eq!(loaded.word_replacements.len(), 0);
    }

    #[test]
    fn test_new_settings_fields() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        use crate::core::types::{CustomShortcut, WordReplacement};

        let settings = AppSettings {
            openai_api_key: Some("test-key".to_string()),
            luyin_token: None,
            selected_model: "whisper-base".to_string(),
            auto_inject: true,
            inject_delay_ms: 150,
            shortcut_key: Some("RightCommand".to_string()),
            display_style: "notch".to_string(),
            appearance: "dark".to_string(),
            ui_language: "zh-CN".to_string(),
            launch_at_login: true,
            show_in_dock: false,
            show_in_menu_bar: true,
            esc_to_cancel: false,
            shortcut_preset: "custom".to_string(),
            custom_shortcut: Some(CustomShortcut {
                r#type: "custom".to_string(),
                modifiers: vec!["cmd".to_string(), "shift".to_string()],
                key: "Space".to_string(),
                display_label: "⌘⇧Space".to_string(),
            }),
            activation_mode: "toggle".to_string(),
            microphone_priority: vec!["mic-1".to_string(), "mic-2".to_string()],
            onboarding_complete: true,
            word_replacements: vec![
                WordReplacement {
                    id: "1".to_string(),
                    from: "AI".to_string(),
                    to: "人工智能".to_string(),
                    enabled: true,
                },
                WordReplacement {
                    id: "2".to_string(),
                    from: "ML".to_string(),
                    to: "机器学习".to_string(),
                    enabled: false,
                },
            ],
        };

        db.save_settings(&settings).unwrap();
        let loaded = db.load_settings().unwrap();

        assert_eq!(loaded, settings);
        assert_eq!(loaded.word_replacements.len(), 2);
        assert_eq!(loaded.word_replacements[0].from, "AI");
        assert_eq!(loaded.custom_shortcut.as_ref().unwrap().key, "Space");
    }

    #[test]
    fn test_search_history() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        let entry1 = TranscriptionEntry {
            id: "1".to_string(),
            text: "Hello world".to_string(),
            timestamp: 1234567890,
            duration: 5.0,
            model: "whisper-1".to_string(),
            confidence: 0.95,
            audio_file_path: None,
        };

        let entry2 = TranscriptionEntry {
            id: "2".to_string(),
            text: "Goodbye world".to_string(),
            timestamp: 1234567891,
            duration: 3.0,
            model: "whisper-1".to_string(),
            confidence: 0.90,
            audio_file_path: None,
        };

        db.save_transcription(&entry1).unwrap();
        db.save_transcription(&entry2).unwrap();

        let results = db.search("Hello", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "Hello world");

        let results = db.search("world", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[cfg(test)]
    proptest! {
        #[test]
        fn prop_quick_input_transcription_persisted(text in "[\\w\\s]{10,100}",
                                                      duration in 0.5f64..30.0f64) {
            let dir = tempdir().unwrap();
            let db_path = dir.path().join("test.db");
            let db = Database::new(&db_path).unwrap();

            let entry = TranscriptionEntry {
                id: uuid::Uuid::new_v4().to_string(),
                text: text.clone(),
                timestamp: chrono::Utc::now().timestamp(),
                duration,
                model: "whisper-1".to_string(),
                confidence: 1.0,
                audio_file_path: None,
            };

            db.save_transcription(&entry).unwrap();
            let history = db.get_history(10).unwrap();

            prop_assert!(history.iter().any(|e| e.text == text));
        }
    }
}
