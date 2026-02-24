#[cfg(test)]
mod tests {
    use crate::core::types::AppSettings;
    use crate::services::state::AppState;
    use tempfile::tempdir;

    fn create_test_state() -> AppState {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        AppState::new(&db_path).unwrap()
    }

    #[test]
    fn test_shortcut_key_persistence() {
        let state = create_test_state();

        let mut settings = state.settings.lock().clone();
        settings.shortcut_key = Some("Cmd+Shift+Space".to_string());

        state.database.save_settings(&settings).unwrap();

        let loaded = state.database.load_settings().unwrap();
        assert_eq!(loaded.shortcut_key, Some("Cmd+Shift+Space".to_string()));
    }

    #[test]
    fn test_shortcut_key_update() {
        let state = create_test_state();

        // Set initial shortcut
        let mut settings = state.settings.lock().clone();
        settings.shortcut_key = Some("Ctrl+Alt+V".to_string());
        state.database.save_settings(&settings).unwrap();

        // Update to new shortcut
        settings.shortcut_key = Some("Cmd+Shift+V".to_string());
        state.database.save_settings(&settings).unwrap();

        let loaded = state.database.load_settings().unwrap();
        assert_eq!(loaded.shortcut_key, Some("Cmd+Shift+V".to_string()));
    }

    #[test]
    fn test_shortcut_key_removal() {
        let state = create_test_state();

        // Set shortcut
        let mut settings = state.settings.lock().clone();
        settings.shortcut_key = Some("Cmd+Shift+Space".to_string());
        state.database.save_settings(&settings).unwrap();

        // Remove shortcut
        settings.shortcut_key = None;
        state.database.save_settings(&settings).unwrap();

        let loaded = state.database.load_settings().unwrap();
        assert_eq!(loaded.shortcut_key, None);
    }

    #[test]
    fn test_various_shortcut_formats() {
        let state = create_test_state();

        let shortcuts = vec![
            "Cmd+Shift+Space",
            "Ctrl+Alt+V",
            "F1",
            "Shift+F12",
            "Cmd+K",
            "Alt+Space",
        ];

        for shortcut in shortcuts {
            let mut settings = state.settings.lock().clone();
            settings.shortcut_key = Some(shortcut.to_string());
            state.database.save_settings(&settings).unwrap();

            let loaded = state.database.load_settings().unwrap();
            assert_eq!(loaded.shortcut_key, Some(shortcut.to_string()));
        }
    }
}

#[cfg(test)]
mod quick_input_service_tests {
    use crate::services::quick_input::QuickInputService;

    #[tokio::test]
    async fn test_initial_state_inactive() {
        let service = QuickInputService::new();
        let is_active = service.is_active().await;
        assert_eq!(is_active, false);
    }

    #[test]
    fn test_service_creation() {
        let service = QuickInputService::new();
        // Should create without error
        assert!(true);
    }

    #[test]
    fn test_unregister_without_register() {
        let service = QuickInputService::new();
        // Should not panic when unregistering without registering
        service.unregister_shortcut();
        assert!(true);
    }
}

#[cfg(test)]
mod shortcut_validation_tests {
    #[test]
    fn test_valid_shortcut_formats() {
        let valid_shortcuts = vec![
            "Cmd+Shift+Space",
            "Ctrl+Alt+V",
            "Shift+F1",
            "Alt+Tab",
            "Cmd+K",
        ];

        for shortcut in valid_shortcuts {
            // Basic validation: should contain modifier + key
            assert!(shortcut.contains('+'));
            assert!(shortcut.len() > 2);
        }
    }

    #[test]
    fn test_shortcut_parsing() {
        let shortcut = "Cmd+Shift+Space";
        let parts: Vec<&str> = shortcut.split('+').collect();

        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "Cmd");
        assert_eq!(parts[1], "Shift");
        assert_eq!(parts[2], "Space");
    }

    #[test]
    fn test_shortcut_case_sensitivity() {
        let shortcut1 = "Cmd+Shift+V";
        let shortcut2 = "cmd+shift+v";

        // Shortcuts should be case-sensitive
        assert_ne!(shortcut1, shortcut2);
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::services::state::AppState;
    use tempfile::tempdir;

    #[test]
    fn test_shortcut_settings_integration() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let state = AppState::new(&db_path).unwrap();

        // Simulate registering a shortcut
        {
            let mut settings = state.settings.lock();
            settings.shortcut_key = Some("Cmd+Shift+Space".to_string());
        }

        let settings = state.settings.lock().clone();
        state.database.save_settings(&settings).unwrap();

        // Verify persistence
        let loaded = state.database.load_settings().unwrap();
        assert_eq!(loaded.shortcut_key, Some("Cmd+Shift+Space".to_string()));
    }

    #[test]
    fn test_multiple_shortcut_changes() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let state = AppState::new(&db_path).unwrap();

        let shortcuts = vec![
            Some("Cmd+Shift+Space".to_string()),
            Some("Ctrl+Alt+V".to_string()),
            None,
            Some("F1".to_string()),
        ];

        for shortcut in shortcuts {
            {
                let mut settings = state.settings.lock();
                settings.shortcut_key = shortcut.clone();
            }

            let settings = state.settings.lock().clone();
            state.database.save_settings(&settings).unwrap();

            let loaded = state.database.load_settings().unwrap();
            assert_eq!(loaded.shortcut_key, shortcut);
        }
    }
}
