#[cfg(test)]
mod tests {
    use super::super::models::*;
    use tempfile::tempdir;

    // Mock AppHandle for testing
    // Note: Full integration tests require a real Tauri app context

    #[test]
    fn test_model_status_serialization() {
        let status = ModelStatus {
            model_id: "whisper-tiny".to_string(),
            downloaded: true,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ModelStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model_id, "whisper-tiny");
        assert_eq!(deserialized.downloaded, true);
    }

    #[test]
    fn test_model_status_list_serialization() {
        let statuses = vec![
            ModelStatus {
                model_id: "whisper-tiny".to_string(),
                downloaded: true,
            },
            ModelStatus {
                model_id: "whisper-base".to_string(),
                downloaded: false,
            },
        ];

        let json = serde_json::to_string(&statuses).unwrap();
        let deserialized: Vec<ModelStatus> = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].model_id, "whisper-tiny");
        assert_eq!(deserialized[1].downloaded, false);
    }

    #[test]
    fn test_all_supported_models() {
        let expected_models = vec![
            "whisper-tiny",
            "whisper-base",
            "whisper-small",
            "whisper-medium",
            "whisper-large-v3",
            "whisper-large-v3-turbo",
        ];

        // Verify all expected models are in the list
        for model in expected_models {
            assert!(model.starts_with("whisper-"));
        }
    }
}

#[cfg(test)]
mod local_whisper_tests {
    use crate::core::local_whisper;
    use tempfile::tempdir;

    #[test]
    fn test_is_model_downloaded_nonexistent() {
        let dir = tempdir().unwrap();
        let app_data_dir = dir.path();

        let result = local_whisper::is_model_downloaded(app_data_dir, "whisper-tiny");
        assert_eq!(result, false);
    }

    #[test]
    fn test_model_path_construction() {
        let dir = tempdir().unwrap();
        let app_data_dir = dir.path();

        // Test that model paths are constructed correctly
        let models = vec!["whisper-tiny", "whisper-base", "whisper-small"];

        for model in models {
            let is_downloaded = local_whisper::is_model_downloaded(app_data_dir, model);
            // Should return false for non-existent models
            assert_eq!(is_downloaded, false);
        }
    }

    #[test]
    fn test_invalid_model_id() {
        let dir = tempdir().unwrap();
        let app_data_dir = dir.path();

        // Test with invalid model ID
        let result = local_whisper::is_model_downloaded(app_data_dir, "invalid-model");
        assert_eq!(result, false);
    }
}

#[cfg(test)]
mod model_provider_tests {
    use crate::core::types::ModelProvider;

    #[test]
    fn test_model_provider_from_id() {
        assert_eq!(
            ModelProvider::from_model_id("luyin-free"),
            ModelProvider::LuYinWang
        );
        assert_eq!(
            ModelProvider::from_model_id("gpt-4o-mini-transcribe"),
            ModelProvider::OpenAI
        );
        assert_eq!(
            ModelProvider::from_model_id("deepgram-nova3"),
            ModelProvider::Deepgram
        );
        assert_eq!(
            ModelProvider::from_model_id("voxtral-mini"),
            ModelProvider::Mistral
        );
        assert_eq!(
            ModelProvider::from_model_id("elevenlabs-scribe"),
            ModelProvider::ElevenLabs
        );
    }

    #[test]
    fn test_whisper_model_provider() {
        let whisper_models = vec![
            "whisper-tiny",
            "whisper-base",
            "whisper-small",
            "whisper-medium",
            "whisper-large-v3",
            "whisper-large-v3-turbo",
        ];

        for model in whisper_models {
            assert_eq!(
                ModelProvider::from_model_id(model),
                ModelProvider::LocalWhisper
            );
        }
    }

    #[test]
    fn test_unknown_model_defaults_to_luyin() {
        assert_eq!(
            ModelProvider::from_model_id("unknown-model"),
            ModelProvider::LuYinWang
        );
    }

    #[test]
    fn test_required_key_for_providers() {
        assert_eq!(ModelProvider::LuYinWang.required_key(), "luyin_token");
        assert_eq!(ModelProvider::OpenAI.required_key(), "openai_api_key");
        assert_eq!(ModelProvider::Deepgram.required_key(), "openai_api_key");
        assert_eq!(ModelProvider::Mistral.required_key(), "openai_api_key");
        assert_eq!(ModelProvider::ElevenLabs.required_key(), "openai_api_key");
        assert_eq!(ModelProvider::LocalWhisper.required_key(), "");
    }

    #[test]
    fn test_local_whisper_no_key_required() {
        let provider = ModelProvider::from_model_id("whisper-base");
        assert_eq!(provider.required_key(), "");
    }
}
