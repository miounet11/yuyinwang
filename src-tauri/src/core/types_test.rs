#[cfg(test)]
mod tests {
    use crate::core::types::*;

    #[test]
    fn test_audio_device_default_values() {
        let device = AudioDevice {
            id: "device-1".to_string(),
            name: "Test Device".to_string(),
            is_default: true,
            is_available: true,
        };

        assert_eq!(device.id, "device-1");
        assert_eq!(device.name, "Test Device");
        assert_eq!(device.is_default, true);
        assert_eq!(device.is_available, true);
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
    fn test_transcription_entry_serialization() {
        let entry = TranscriptionEntry {
            id: "test-id".to_string(),
            text: "Hello world".to_string(),
            timestamp: 1234567890,
            duration: 5.5,
            model: "whisper-base".to_string(),
            confidence: 0.95,
            audio_file_path: Some("/path/to/audio.wav".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: TranscriptionEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.text, entry.text);
        assert_eq!(deserialized.timestamp, entry.timestamp);
        assert_eq!(deserialized.duration, entry.duration);
        assert_eq!(deserialized.model, entry.model);
        assert_eq!(deserialized.confidence, entry.confidence);
        assert_eq!(deserialized.audio_file_path, entry.audio_file_path);
    }

    #[test]
    fn test_transcription_result_with_all_fields() {
        let result = TranscriptionResult {
            text: "Test transcription".to_string(),
            language: Some("en".to_string()),
            duration: Some(10.5),
        };

        assert_eq!(result.text, "Test transcription");
        assert_eq!(result.language, Some("en".to_string()));
        assert_eq!(result.duration, Some(10.5));
    }

    #[test]
    fn test_transcription_result_minimal() {
        let result = TranscriptionResult {
            text: "Test".to_string(),
            language: None,
            duration: None,
        };

        assert_eq!(result.text, "Test");
        assert_eq!(result.language, None);
        assert_eq!(result.duration, None);
    }

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();

        assert_eq!(settings.openai_api_key, None);
        assert_eq!(settings.luyin_token, None);
        assert_eq!(settings.selected_model, "luyin-free");
        assert_eq!(settings.auto_inject, false);
        assert_eq!(settings.inject_delay_ms, 100);
        assert_eq!(settings.shortcut_key, None);
    }

    #[test]
    fn test_app_settings_equality() {
        let mut settings1 = AppSettings::default();
        settings1.openai_api_key = Some("key".to_string());
        settings1.luyin_token = Some("token".to_string());
        settings1.selected_model = "whisper-base".to_string();
        settings1.auto_inject = true;
        settings1.inject_delay_ms = 150;
        settings1.shortcut_key = Some("Cmd+V".to_string());

        let settings2 = settings1.clone();

        assert_eq!(settings1, settings2);
    }

    #[test]
    fn test_app_settings_inequality() {
        let settings1 = AppSettings::default();
        let mut settings2 = AppSettings::default();
        settings2.auto_inject = true;

        assert_ne!(settings1, settings2);
    }

    #[test]
    fn test_model_provider_luyin() {
        let provider = ModelProvider::from_model_id("luyin-free");
        assert_eq!(provider, ModelProvider::LuYinWang);
        assert_eq!(provider.required_key(), "luyin_token");
    }

    #[test]
    fn test_model_provider_openai() {
        let provider = ModelProvider::from_model_id("gpt-4o-mini-transcribe");
        assert_eq!(provider, ModelProvider::OpenAI);
        assert_eq!(provider.required_key(), "openai_api_key");
    }

    #[test]
    fn test_model_provider_deepgram() {
        let provider = ModelProvider::from_model_id("deepgram-nova3");
        assert_eq!(provider, ModelProvider::Deepgram);
        assert_eq!(provider.required_key(), "openai_api_key");
    }

    #[test]
    fn test_model_provider_mistral() {
        let provider = ModelProvider::from_model_id("voxtral-mini");
        assert_eq!(provider, ModelProvider::Mistral);
        assert_eq!(provider.required_key(), "openai_api_key");
    }

    #[test]
    fn test_model_provider_elevenlabs() {
        let provider = ModelProvider::from_model_id("elevenlabs-scribe");
        assert_eq!(provider, ModelProvider::ElevenLabs);
        assert_eq!(provider.required_key(), "openai_api_key");
    }

    #[test]
    fn test_model_provider_local_whisper() {
        let whisper_models = vec![
            "whisper-tiny",
            "whisper-base",
            "whisper-small",
            "whisper-medium",
            "whisper-large-v3",
            "whisper-large-v3-turbo",
        ];

        for model in whisper_models {
            let provider = ModelProvider::from_model_id(model);
            assert_eq!(provider, ModelProvider::LocalWhisper);
            assert_eq!(provider.required_key(), "");
        }
    }

    #[test]
    fn test_model_provider_unknown_defaults_to_luyin() {
        let provider = ModelProvider::from_model_id("unknown-model-xyz");
        assert_eq!(provider, ModelProvider::LuYinWang);
    }

    #[test]
    fn test_model_provider_clone() {
        let provider1 = ModelProvider::LocalWhisper;
        let provider2 = provider1.clone();
        assert_eq!(provider1, provider2);
    }

    #[cfg(test)]
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_transcription_entry_roundtrip(
            text in "[\\w\\s]{1,100}",
            timestamp in 0i64..9999999i64,
            duration in 0.1f64..100.0f64,
            confidence in 0.0f32..1.0f32
        ) {
            let entry = TranscriptionEntry {
                id: uuid::Uuid::new_v4().to_string(),
                text: text.clone(),
                timestamp,
                duration,
                model: "test-model".to_string(),
                confidence,
                audio_file_path: None,
            };

            let json = serde_json::to_string(&entry).unwrap();
            let deserialized: TranscriptionEntry = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(deserialized.text, text);
            prop_assert_eq!(deserialized.timestamp, timestamp);
            prop_assert_eq!(deserialized.duration, duration);
            prop_assert_eq!(deserialized.confidence, confidence);
        }

        #[test]
        fn prop_app_settings_roundtrip(
            model in "[a-z-]{5,20}",
            auto_inject: bool,
            delay in 10u64..1000u64
        ) {
            let mut settings = AppSettings::default();
            settings.selected_model = model.clone();
            settings.auto_inject = auto_inject;
            settings.inject_delay_ms = delay;

            let json = serde_json::to_string(&settings).unwrap();
            let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(deserialized.selected_model, model);
            prop_assert_eq!(deserialized.auto_inject, auto_inject);
            prop_assert_eq!(deserialized.inject_delay_ms, delay);
        }

        #[test]
        fn prop_recording_config_valid_sample_rates(
            sample_rate in 8000u32..96000u32,
            channels in 1u16..8u16
        ) {
            let config = RecordingConfig {
                device_id: None,
                sample_rate,
                channels,
            };

            prop_assert!(config.sample_rate >= 8000);
            prop_assert!(config.channels >= 1);
        }
    }
}
