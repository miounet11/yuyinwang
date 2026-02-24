//! Integration tests for Recording King
//!
//! These tests verify end-to-end functionality across multiple components.

use recording_king::core::types::*;
use recording_king::services::database::Database;
use recording_king::services::state::AppState;
use tempfile::tempdir;

#[test]
fn test_app_state_initialization() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let state = AppState::new(&db_path);
    assert!(state.is_ok());

    let state = state.unwrap();
    assert_eq!(state.is_recording(), false);
}

#[test]
fn test_database_initialization() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Database::new(&db_path);
    assert!(db.is_ok());
}

#[test]
fn test_settings_persistence_across_sessions() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Session 1: Save settings
    {
        let db = Database::new(&db_path).unwrap();
        let mut settings = AppSettings::default();
        settings.openai_api_key = Some("test-key".to_string());
        settings.luyin_token = Some("test-token".to_string());
        settings.selected_model = "whisper-base".to_string();
        settings.auto_inject = true;
        settings.inject_delay_ms = 150;
        settings.shortcut_key = Some("Cmd+Shift+V".to_string());
        db.save_settings(&settings).unwrap();
    }

    // Session 2: Load settings
    {
        let db = Database::new(&db_path).unwrap();
        let loaded = db.load_settings().unwrap();

        assert_eq!(loaded.openai_api_key, Some("test-key".to_string()));
        assert_eq!(loaded.selected_model, "whisper-base");
        assert_eq!(loaded.auto_inject, true);
        assert_eq!(loaded.inject_delay_ms, 150);
    }
}

#[test]
fn test_transcription_history_persistence() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Session 1: Save transcriptions
    {
        let db = Database::new(&db_path).unwrap();

        for i in 0..5 {
            let entry = TranscriptionEntry {
                id: format!("entry-{}", i),
                text: format!("Transcription {}", i),
                timestamp: 1000 + i as i64,
                duration: 5.0,
                model: "whisper-base".to_string(),
                confidence: 0.95,
                audio_file_path: None,
            };
            db.save_transcription(&entry).unwrap();
        }
    }

    // Session 2: Load transcriptions
    {
        let db = Database::new(&db_path).unwrap();
        let history = db.get_history(10).unwrap();

        assert_eq!(history.len(), 5);
        // Should be in reverse chronological order
        assert_eq!(history[0].id, "entry-4");
        assert_eq!(history[4].id, "entry-0");
    }
}

#[test]
fn test_search_functionality() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new(&db_path).unwrap();

    // Add diverse transcriptions
    let entries = vec![
        ("1", "Hello world, this is a test"),
        ("2", "Testing audio transcription"),
        ("3", "Another test entry"),
        ("4", "Completely different content"),
        ("5", "Final test message"),
    ];

    for (id, text) in entries {
        let entry = TranscriptionEntry {
            id: id.to_string(),
            text: text.to_string(),
            timestamp: 1000,
            duration: 5.0,
            model: "whisper-base".to_string(),
            confidence: 0.95,
            audio_file_path: None,
        };
        db.save_transcription(&entry).unwrap();
    }

    // Search for "test"
    let results = db.search("test", 10).unwrap();
    assert_eq!(results.len(), 4);

    // Search for "audio"
    let results = db.search("audio", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "2");

    // Search for non-existent term
    let results = db.search("nonexistent", 10).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_delete_functionality() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new(&db_path).unwrap();

    // Add entries
    for i in 0..3 {
        let entry = TranscriptionEntry {
            id: format!("entry-{}", i),
            text: format!("Text {}", i),
            timestamp: 1000 + i as i64,
            duration: 5.0,
            model: "whisper-base".to_string(),
            confidence: 0.95,
            audio_file_path: None,
        };
        db.save_transcription(&entry).unwrap();
    }

    // Delete one entry
    db.delete("entry-1").unwrap();

    let history = db.get_history(10).unwrap();
    assert_eq!(history.len(), 2);
    assert!(history.iter().all(|e| e.id != "entry-1"));
}

#[tokio::test]
async fn test_recording_state_management() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let state = AppState::new(&db_path).unwrap();

    // Initial state
    assert_eq!(state.is_recording(), false);

    // Start recording
    let result = state.start_recording().await;
    assert!(result.is_ok());
    assert_eq!(state.is_recording(), true);

    // Cannot start again
    let result = state.start_recording().await;
    assert!(result.is_err());

    // Stop recording
    let result = state.stop_recording().await;
    // May fail due to audio device issues in test environment
    if result.is_ok() {
        assert_eq!(state.is_recording(), false);
    }
}

#[test]
fn test_model_provider_detection() {
    let test_cases = vec![
        ("luyin-free", ModelProvider::LuYinWang, "luyin_token"),
        ("gpt-4o-mini-transcribe", ModelProvider::OpenAI, "openai_api_key"),
        ("whisper-tiny", ModelProvider::LocalWhisper, ""),
        ("whisper-base", ModelProvider::LocalWhisper, ""),
        ("deepgram-nova3", ModelProvider::Deepgram, "openai_api_key"),
    ];

    for (model_id, expected_provider, expected_key) in test_cases {
        let provider = ModelProvider::from_model_id(model_id);
        assert_eq!(provider, expected_provider);
        assert_eq!(provider.required_key(), expected_key);
    }
}

#[test]
fn test_settings_with_different_models() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new(&db_path).unwrap();

    let models = vec![
        "luyin-free",
        "whisper-tiny",
        "whisper-base",
        "gpt-4o-mini-transcribe",
        "deepgram-nova3",
    ];

    for model in models {
        let mut settings = AppSettings::default();
        settings.openai_api_key = Some("test-key".to_string());
        settings.luyin_token = Some("test-token".to_string());
        settings.selected_model = model.to_string();

        db.save_settings(&settings).unwrap();
        let loaded = db.load_settings().unwrap();

        assert_eq!(loaded.selected_model, model);
    }
}

#[test]
fn test_api_key_encryption() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new(&db_path).unwrap();

    let mut settings = AppSettings::default();
    settings.openai_api_key = Some("sk-secret-key-12345".to_string());
    settings.luyin_token = Some("secret-token-67890".to_string());
    settings.selected_model = "whisper-base".to_string();

    db.save_settings(&settings).unwrap();

    // Verify keys are not stored in plaintext
    use rusqlite::Connection;
    let conn = Connection::open(&db_path).unwrap();
    let stored_value: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'settings'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    // Stored value should not contain plaintext keys
    assert!(!stored_value.contains("sk-secret-key-12345"));
    assert!(!stored_value.contains("secret-token-67890"));

    // But should be recoverable
    let loaded = db.load_settings().unwrap();
    assert_eq!(loaded.openai_api_key, Some("sk-secret-key-12345".to_string()));
    assert_eq!(loaded.luyin_token, Some("secret-token-67890".to_string()));
}

#[test]
fn test_transcription_with_audio_file_path() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new(&db_path).unwrap();

    let entry = TranscriptionEntry {
        id: "file-entry".to_string(),
        text: "Transcription from file".to_string(),
        timestamp: 1000,
        duration: 15.5,
        model: "whisper-small".to_string(),
        confidence: 0.98,
        audio_file_path: Some("/path/to/audio.wav".to_string()),
    };

    db.save_transcription(&entry).unwrap();

    let history = db.get_history(10).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].audio_file_path, Some("/path/to/audio.wav".to_string()));
}

#[test]
fn test_concurrent_database_access() {
    use std::sync::Arc;
    use std::thread;

    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Arc::new(Database::new(&db_path).unwrap());

    let mut handles = vec![];

    // Spawn multiple threads writing to database
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let entry = TranscriptionEntry {
                id: format!("thread-{}", i),
                text: format!("Text from thread {}", i),
                timestamp: 1000 + i as i64,
                duration: 5.0,
                model: "whisper-base".to_string(),
                confidence: 0.95,
                audio_file_path: None,
            };
            db_clone.save_transcription(&entry).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all entries were saved
    let history = db.get_history(10).unwrap();
    assert_eq!(history.len(), 5);
}
