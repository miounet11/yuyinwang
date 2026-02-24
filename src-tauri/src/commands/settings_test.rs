#[cfg(test)]
mod tests {
    use super::super::settings::*;
    use crate::core::types::AppSettings;
    use crate::services::database::Database;
    use crate::services::state::AppState;
    use tempfile::tempdir;

    fn create_test_state() -> AppState {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        AppState::new(&db_path).unwrap()
    }

    #[test]
    fn test_get_settings_returns_default() {
        let state = create_test_state();
        let result = get_settings(tauri::State::from(&state));
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.selected_model, "luyin-free");
        assert_eq!(settings.auto_inject, false);
        assert_eq!(settings.inject_delay_ms, 100);
    }

    #[test]
    fn test_update_settings_persists() {
        let state = create_test_state();

        let mut new_settings = AppSettings::default();
        new_settings.openai_api_key = Some("test-key-123".to_string());
        new_settings.luyin_token = Some("test-token-456".to_string());
        new_settings.selected_model = "whisper-base".to_string();
        new_settings.auto_inject = true;
        new_settings.inject_delay_ms = 200;
        new_settings.shortcut_key = Some("Cmd+Shift+V".to_string());

        let result = update_settings(tauri::State::from(&state), new_settings.clone());
        assert!(result.is_ok());

        // Verify settings were saved
        let loaded = get_settings(tauri::State::from(&state)).unwrap();
        assert_eq!(loaded.openai_api_key, new_settings.openai_api_key);
        assert_eq!(loaded.selected_model, new_settings.selected_model);
        assert_eq!(loaded.auto_inject, new_settings.auto_inject);
        assert_eq!(loaded.inject_delay_ms, new_settings.inject_delay_ms);
    }

    #[test]
    fn test_update_settings_with_empty_keys() {
        let state = create_test_state();

        let mut settings = AppSettings::default();
        settings.selected_model = "whisper-tiny".to_string();
        settings.inject_delay_ms = 50;

        let result = update_settings(tauri::State::from(&state), settings.clone());
        assert!(result.is_ok());

        let loaded = get_settings(tauri::State::from(&state)).unwrap();
        assert_eq!(loaded.openai_api_key, None);
        assert_eq!(loaded.luyin_token, None);
    }

    #[test]
    fn test_settings_serialization() {
        let mut settings = AppSettings::default();
        settings.openai_api_key = Some("sk-test123".to_string());
        settings.luyin_token = Some("token-abc".to_string());
        settings.selected_model = "gpt-4o-mini-transcribe".to_string();
        settings.auto_inject = true;
        settings.inject_delay_ms = 150;
        settings.shortcut_key = Some("Ctrl+Alt+V".to_string());

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, settings);
    }

    #[cfg(test)]
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_settings_roundtrip(
            model in "[a-z-]{5,20}",
            auto_inject: bool,
            delay in 10u64..1000u64
        ) {
            let state = create_test_state();

            let mut settings = AppSettings::default();
            settings.selected_model = model.clone();
            settings.auto_inject = auto_inject;
            settings.inject_delay_ms = delay;

            let _ = update_settings(tauri::State::from(&state), settings.clone());
            let loaded = get_settings(tauri::State::from(&state)).unwrap();

            prop_assert_eq!(loaded.selected_model, model);
            prop_assert_eq!(loaded.auto_inject, auto_inject);
            prop_assert_eq!(loaded.inject_delay_ms, delay);
        }
    }
}
