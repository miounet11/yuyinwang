//! Test utilities and mock data for Recording King tests

use recording_king::core::types::*;
use recording_king::services::database::Database;
use recording_king::services::state::AppState;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture that provides a temporary database and state
pub struct TestFixture {
    pub temp_dir: TempDir,
    pub db_path: PathBuf,
    pub state: AppState,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = AppState::new(&db_path).unwrap();

        Self {
            temp_dir,
            db_path,
            state,
        }
    }

    pub fn database(&self) -> &Database {
        &self.state.database
    }
}

/// Mock data generators
pub mod mock {
    use super::*;

    pub fn transcription_entry(id: &str, text: &str) -> TranscriptionEntry {
        TranscriptionEntry {
            id: id.to_string(),
            text: text.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            duration: 5.0,
            model: "whisper-base".to_string(),
            confidence: 0.95,
            audio_file_path: None,
        }
    }

    pub fn transcription_entry_with_file(
        id: &str,
        text: &str,
        file_path: &str,
    ) -> TranscriptionEntry {
        TranscriptionEntry {
            id: id.to_string(),
            text: text.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            duration: 10.5,
            model: "whisper-small".to_string(),
            confidence: 0.98,
            audio_file_path: Some(file_path.to_string()),
        }
    }

    pub fn transcription_entry_custom(
        id: &str,
        text: &str,
        timestamp: i64,
        duration: f64,
        model: &str,
        confidence: f32,
    ) -> TranscriptionEntry {
        TranscriptionEntry {
            id: id.to_string(),
            text: text.to_string(),
            timestamp,
            duration,
            model: model.to_string(),
            confidence,
            audio_file_path: None,
        }
    }

    pub fn app_settings_default() -> AppSettings {
        AppSettings::default()
    }

    pub fn app_settings_with_keys(
        openai_key: Option<&str>,
        luyin_token: Option<&str>,
    ) -> AppSettings {
        AppSettings {
            openai_api_key: openai_key.map(|s| s.to_string()),
            luyin_token: luyin_token.map(|s| s.to_string()),
            selected_model: "whisper-base".to_string(),
            auto_inject: false,
            inject_delay_ms: 100,
            shortcut_key: None,
        }
    }

    pub fn app_settings_full(
        openai_key: &str,
        luyin_token: &str,
        model: &str,
        auto_inject: bool,
        delay: u64,
        shortcut: Option<&str>,
    ) -> AppSettings {
        AppSettings {
            openai_api_key: Some(openai_key.to_string()),
            luyin_token: Some(luyin_token.to_string()),
            selected_model: model.to_string(),
            auto_inject,
            inject_delay_ms: delay,
            shortcut_key: shortcut.map(|s| s.to_string()),
        }
    }

    pub fn audio_device(id: &str, name: &str, is_default: bool) -> AudioDevice {
        AudioDevice {
            id: id.to_string(),
            name: name.to_string(),
            is_default,
            is_available: true,
        }
    }

    pub fn recording_config_default() -> RecordingConfig {
        RecordingConfig::default()
    }

    pub fn recording_config_custom(
        device_id: Option<&str>,
        sample_rate: u32,
        channels: u16,
    ) -> RecordingConfig {
        RecordingConfig {
            device_id: device_id.map(|s| s.to_string()),
            sample_rate,
            channels,
        }
    }

    pub fn transcription_result(text: &str, language: Option<&str>) -> TranscriptionResult {
        TranscriptionResult {
            text: text.to_string(),
            language: language.map(|s| s.to_string()),
            duration: Some(5.0),
        }
    }

    /// Generate multiple test transcription entries
    pub fn multiple_entries(count: usize) -> Vec<TranscriptionEntry> {
        (0..count)
            .map(|i| transcription_entry(&format!("entry-{}", i), &format!("Text {}", i)))
            .collect()
    }

    /// Generate entries with specific timestamps for testing ordering
    pub fn entries_with_timestamps(timestamps: Vec<i64>) -> Vec<TranscriptionEntry> {
        timestamps
            .into_iter()
            .enumerate()
            .map(|(i, ts)| {
                transcription_entry_custom(
                    &format!("entry-{}", i),
                    &format!("Text {}", i),
                    ts,
                    5.0,
                    "whisper-base",
                    0.95,
                )
            })
            .collect()
    }
}

/// Assertion helpers
pub mod assert {
    use super::*;

    pub fn settings_equal(a: &AppSettings, b: &AppSettings) {
        assert_eq!(a.openai_api_key, b.openai_api_key);
        assert_eq!(a.luyin_token, b.luyin_token);
        assert_eq!(a.selected_model, b.selected_model);
        assert_eq!(a.auto_inject, b.auto_inject);
        assert_eq!(a.inject_delay_ms, b.inject_delay_ms);
        assert_eq!(a.shortcut_key, b.shortcut_key);
    }

    pub fn entry_equal(a: &TranscriptionEntry, b: &TranscriptionEntry) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.text, b.text);
        assert_eq!(a.timestamp, b.timestamp);
        assert_eq!(a.duration, b.duration);
        assert_eq!(a.model, b.model);
        assert_eq!(a.confidence, b.confidence);
        assert_eq!(a.audio_file_path, b.audio_file_path);
    }

    pub fn entries_ordered_by_timestamp_desc(entries: &[TranscriptionEntry]) {
        for i in 0..entries.len().saturating_sub(1) {
            assert!(
                entries[i].timestamp >= entries[i + 1].timestamp,
                "Entries not in descending timestamp order"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creation() {
        let fixture = TestFixture::new();
        assert!(fixture.db_path.exists());
        assert_eq!(fixture.state.is_recording(), false);
    }

    #[test]
    fn test_mock_transcription_entry() {
        let entry = mock::transcription_entry("test-id", "test text");
        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.text, "test text");
        assert_eq!(entry.model, "whisper-base");
    }

    #[test]
    fn test_mock_multiple_entries() {
        let entries = mock::multiple_entries(5);
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].id, "entry-0");
        assert_eq!(entries[4].id, "entry-4");
    }

    #[test]
    fn test_mock_settings() {
        let settings = mock::app_settings_with_keys(Some("key"), Some("token"));
        assert_eq!(settings.openai_api_key, Some("key".to_string()));
        assert_eq!(settings.luyin_token, Some("token".to_string()));
    }

    #[test]
    fn test_assert_settings_equal() {
        let settings1 = mock::app_settings_default();
        let settings2 = mock::app_settings_default();
        assert::settings_equal(&settings1, &settings2);
    }

    #[test]
    fn test_assert_entries_ordered() {
        let entries = mock::entries_with_timestamps(vec![1000, 900, 800, 700]);
        assert::entries_ordered_by_timestamp_desc(&entries);
    }
}
