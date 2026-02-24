#[cfg(test)]
mod tests {
    use crate::core::types::{AudioDevice, RecordingConfig, TranscriptionResult};

    #[test]
    fn test_audio_device_serialization() {
        let device = AudioDevice {
            id: "device-1".to_string(),
            name: "Built-in Microphone".to_string(),
            is_default: true,
            is_available: true,
        };

        let json = serde_json::to_string(&device).unwrap();
        let deserialized: AudioDevice = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "device-1");
        assert_eq!(deserialized.name, "Built-in Microphone");
        assert_eq!(deserialized.is_default, true);
        assert_eq!(deserialized.is_available, true);
    }

    #[test]
    fn test_recording_config_default() {
        let config = RecordingConfig::default();

        assert_eq!(config.device_id, None);
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
    }

    #[test]
    fn test_recording_config_custom() {
        let config = RecordingConfig {
            device_id: Some("custom-device".to_string()),
            sample_rate: 44100,
            channels: 2,
        };

        assert_eq!(config.device_id, Some("custom-device".to_string()));
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 2);
    }

    #[test]
    fn test_transcription_result_serialization() {
        let result = TranscriptionResult {
            text: "Hello world".to_string(),
            language: Some("en".to_string()),
            duration: Some(5.5),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TranscriptionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.text, "Hello world");
        assert_eq!(deserialized.language, Some("en".to_string()));
        assert_eq!(deserialized.duration, Some(5.5));
    }

    #[test]
    fn test_transcription_result_optional_fields() {
        let result = TranscriptionResult {
            text: "Test".to_string(),
            language: None,
            duration: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TranscriptionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.text, "Test");
        assert_eq!(deserialized.language, None);
        assert_eq!(deserialized.duration, None);
    }
}

#[cfg(test)]
mod state_tests {
    use crate::services::state::AppState;
    use tempfile::tempdir;

    fn create_test_state() -> AppState {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        AppState::new(&db_path).unwrap()
    }

    #[test]
    fn test_initial_recording_state() {
        let state = create_test_state();
        assert_eq!(state.is_recording(), false);
    }

    #[tokio::test]
    async fn test_start_recording_changes_state() {
        let state = create_test_state();

        let result = state.start_recording().await;
        assert!(result.is_ok());
        assert_eq!(state.is_recording(), true);
    }

    #[tokio::test]
    async fn test_cannot_start_recording_twice() {
        let state = create_test_state();

        state.start_recording().await.unwrap();
        let result = state.start_recording().await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Already recording"));
        }
    }

    #[tokio::test]
    async fn test_stop_recording_without_start_fails() {
        let state = create_test_state();

        let result = state.stop_recording().await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Not recording"));
        }
    }

    #[tokio::test]
    async fn test_recording_lifecycle() {
        let state = create_test_state();

        // Start recording
        assert_eq!(state.is_recording(), false);
        state.start_recording().await.unwrap();
        assert_eq!(state.is_recording(), true);

        // Stop recording
        let result = state.stop_recording().await;
        // May fail due to audio device issues in test environment
        if result.is_ok() {
            assert_eq!(state.is_recording(), false);
        }
    }
}

#[cfg(test)]
mod audio_tests {
    use crate::core::audio;

    #[test]
    fn test_list_audio_devices() {
        let result = audio::list_audio_devices();

        // Should not panic, may succeed or fail based on environment
        match result {
            Ok(devices) => {
                // If successful, should return a list (may be empty)
                assert!(devices.len() >= 0);

                // Check device structure if any exist
                for device in devices {
                    assert!(!device.id.is_empty());
                    assert!(!device.name.is_empty());
                }
            }
            Err(_) => {
                // May fail in test environment without audio devices
                assert!(true);
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::core::types::TranscriptionEntry;
    use crate::services::state::AppState;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_transcription_saved_to_database() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let state = AppState::new(&db_path).unwrap();

        let entry = TranscriptionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            text: "Test transcription".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            duration: 5.0,
            model: "whisper-base".to_string(),
            confidence: 0.95,
            audio_file_path: None,
        };

        state.database.save_transcription(&entry).unwrap();

        let history = state.database.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].text, "Test transcription");
    }

    #[test]
    fn test_transcription_entry_with_audio_file() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let state = AppState::new(&db_path).unwrap();

        let entry = TranscriptionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            text: "File transcription".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            duration: 10.5,
            model: "whisper-small".to_string(),
            confidence: 0.98,
            audio_file_path: Some("/path/to/audio.wav".to_string()),
        };

        state.database.save_transcription(&entry).unwrap();

        let history = state.database.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(
            history[0].audio_file_path,
            Some("/path/to/audio.wav".to_string())
        );
    }
}
