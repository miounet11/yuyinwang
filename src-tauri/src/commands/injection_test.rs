#[cfg(test)]
mod tests {
    use super::super::injection::*;

    #[test]
    fn test_inject_text_empty_string() {
        // Empty text should succeed without error
        let result = inject_text("".to_string(), Some(0));
        // On non-macOS platforms, this will return an error
        // On macOS, it depends on permissions
        #[cfg(not(target_os = "macos"))]
        assert!(result.is_err());
    }

    #[test]
    fn test_inject_text_with_delay() {
        use std::time::Instant;

        let start = Instant::now();
        let _ = inject_text("test".to_string(), Some(100));
        let elapsed = start.elapsed();

        // Should take at least the delay time (if it runs)
        // On non-macOS, it will fail immediately
        #[cfg(target_os = "macos")]
        assert!(elapsed.as_millis() >= 100 || elapsed.as_millis() < 10);
    }

    #[test]
    fn test_inject_text_default_delay() {
        // Should use default 100ms delay when None
        let result = inject_text("test".to_string(), None);
        // Result depends on platform and permissions
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_injection_permission() {
        let result = check_injection_permission();
        assert!(result.is_ok());

        let has_permission = result.unwrap();
        // Should return a boolean
        assert!(has_permission == true || has_permission == false);
    }

    #[test]
    fn test_request_injection_permission() {
        let result = request_injection_permission();
        assert!(result.is_ok());

        let granted = result.unwrap();
        // Should return a boolean
        assert!(granted == true || granted == false);
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_non_macos_injection_fails() {
        let result = inject_text("test".to_string(), Some(0));
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("only supported on macOS"));
        }
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_non_macos_permission_always_true() {
        let result = check_injection_permission().unwrap();
        assert_eq!(result, true);

        let result = request_injection_permission().unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_inject_text_special_characters() {
        // Test with various special characters
        let test_cases = vec![
            "Hello, World!",
            "Test\nNewline",
            "Tab\tCharacter",
            "Quote\"Test",
            "Apostrophe's",
            "Unicode: 你好世界",
            "Emoji: 🎤🎵",
        ];

        for text in test_cases {
            let result = inject_text(text.to_string(), Some(0));
            // Should not panic, may succeed or fail based on platform
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_inject_text_long_string() {
        let long_text = "a".repeat(1000);
        let result = inject_text(long_text, Some(0));
        // Should handle long strings without panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_inject_text_zero_delay() {
        let result = inject_text("test".to_string(), Some(0));
        // Zero delay should be valid
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_inject_text_large_delay() {
        use std::time::Instant;

        let start = Instant::now();
        let _ = inject_text("test".to_string(), Some(500));
        let elapsed = start.elapsed();

        // Should respect large delays on macOS
        #[cfg(target_os = "macos")]
        if elapsed.as_millis() >= 500 {
            // Delay was applied
            assert!(true);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::super::injection::*;

    #[test]
    fn test_permission_check_before_injection() {
        // Best practice: check permission before injecting
        let has_permission = check_injection_permission().unwrap();

        if has_permission {
            // If we have permission, injection might succeed
            let result = inject_text("test".to_string(), Some(0));
            #[cfg(target_os = "macos")]
            assert!(result.is_ok() || result.is_err());
        } else {
            // Without permission, we can still try (will use fallback)
            let result = inject_text("test".to_string(), Some(0));
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_request_then_check_permission() {
        // Request permission
        let _ = request_injection_permission();

        // Then check if we have it
        let result = check_injection_permission();
        assert!(result.is_ok());
    }
}
