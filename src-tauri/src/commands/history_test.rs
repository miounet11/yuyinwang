#[cfg(test)]
mod tests {
    use super::super::history::*;
    use crate::core::types::TranscriptionEntry;
    use crate::services::state::AppState;
    use tempfile::tempdir;

    fn create_test_state() -> AppState {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        AppState::new(&db_path).unwrap()
    }

    fn create_test_entry(id: &str, text: &str, timestamp: i64) -> TranscriptionEntry {
        TranscriptionEntry {
            id: id.to_string(),
            text: text.to_string(),
            timestamp,
            duration: 5.0,
            model: "whisper-base".to_string(),
            confidence: 0.95,
            audio_file_path: None,
        }
    }

    #[test]
    fn test_get_history_empty() {
        let state = create_test_state();
        let result = get_history(tauri::State::from(&state), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_get_history_with_entries() {
        let state = create_test_state();

        // Add test entries
        let entry1 = create_test_entry("1", "First transcription", 1000);
        let entry2 = create_test_entry("2", "Second transcription", 2000);
        let entry3 = create_test_entry("3", "Third transcription", 3000);

        state.database.save_transcription(&entry1).unwrap();
        state.database.save_transcription(&entry2).unwrap();
        state.database.save_transcription(&entry3).unwrap();

        let result = get_history(tauri::State::from(&state), None).unwrap();
        assert_eq!(result.len(), 3);

        // Should be ordered by timestamp DESC
        assert_eq!(result[0].id, "3");
        assert_eq!(result[1].id, "2");
        assert_eq!(result[2].id, "1");
    }

    #[test]
    fn test_get_history_with_limit() {
        let state = create_test_state();

        for i in 0..10 {
            let entry = create_test_entry(&i.to_string(), &format!("Entry {}", i), i as i64);
            state.database.save_transcription(&entry).unwrap();
        }

        let result = get_history(tauri::State::from(&state), Some(5)).unwrap();
        assert_eq!(result.len(), 5);

        // Should get the 5 most recent
        assert_eq!(result[0].id, "9");
        assert_eq!(result[4].id, "5");
    }

    #[test]
    fn test_search_history_finds_matches() {
        let state = create_test_state();

        let entry1 = create_test_entry("1", "Hello world", 1000);
        let entry2 = create_test_entry("2", "Goodbye world", 2000);
        let entry3 = create_test_entry("3", "Testing audio", 3000);

        state.database.save_transcription(&entry1).unwrap();
        state.database.save_transcription(&entry2).unwrap();
        state.database.save_transcription(&entry3).unwrap();

        let result = search_history(
            tauri::State::from(&state),
            "world".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|e| e.text.contains("Hello")));
        assert!(result.iter().any(|e| e.text.contains("Goodbye")));
    }

    #[test]
    fn test_search_history_case_insensitive() {
        let state = create_test_state();

        let entry = create_test_entry("1", "Hello World", 1000);
        state.database.save_transcription(&entry).unwrap();

        let result = search_history(
            tauri::State::from(&state),
            "hello".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "Hello World");
    }

    #[test]
    fn test_search_history_no_matches() {
        let state = create_test_state();

        let entry = create_test_entry("1", "Hello world", 1000);
        state.database.save_transcription(&entry).unwrap();

        let result = search_history(
            tauri::State::from(&state),
            "nonexistent".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_search_history_with_limit() {
        let state = create_test_state();

        for i in 0..10 {
            let entry = create_test_entry(&i.to_string(), "test entry", i as i64);
            state.database.save_transcription(&entry).unwrap();
        }

        let result = search_history(
            tauri::State::from(&state),
            "test".to_string(),
            Some(3),
        )
        .unwrap();

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_delete_entry_removes_from_history() {
        let state = create_test_state();

        let entry1 = create_test_entry("1", "First", 1000);
        let entry2 = create_test_entry("2", "Second", 2000);

        state.database.save_transcription(&entry1).unwrap();
        state.database.save_transcription(&entry2).unwrap();

        let result = delete_entry(tauri::State::from(&state), "1".to_string());
        assert!(result.is_ok());

        let history = get_history(tauri::State::from(&state), None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, "2");
    }

    #[test]
    fn test_delete_nonexistent_entry() {
        let state = create_test_state();

        // Deleting non-existent entry should not error
        let result = delete_entry(tauri::State::from(&state), "nonexistent".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_all_entries() {
        let state = create_test_state();

        let entry1 = create_test_entry("1", "First", 1000);
        let entry2 = create_test_entry("2", "Second", 2000);

        state.database.save_transcription(&entry1).unwrap();
        state.database.save_transcription(&entry2).unwrap();

        delete_entry(tauri::State::from(&state), "1".to_string()).unwrap();
        delete_entry(tauri::State::from(&state), "2".to_string()).unwrap();

        let history = get_history(tauri::State::from(&state), None).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[cfg(test)]
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_history_preserves_data(
            text in "[\\w\\s]{10,100}",
            timestamp in 1000i64..9999999i64,
            duration in 0.5f64..30.0f64
        ) {
            let state = create_test_state();
            let entry = TranscriptionEntry {
                id: uuid::Uuid::new_v4().to_string(),
                text: text.clone(),
                timestamp,
                duration,
                model: "test-model".to_string(),
                confidence: 0.95,
                audio_file_path: None,
            };

            state.database.save_transcription(&entry).unwrap();
            let history = get_history(tauri::State::from(&state), None).unwrap();

            prop_assert_eq!(history.len(), 1);
            prop_assert_eq!(history[0].text, text);
            prop_assert_eq!(history[0].timestamp, timestamp);
        }
    }
}
