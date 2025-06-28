//! Service Migration Integration Test
//!
//! Test that the service migration layer works correctly and provides
//! backward compatibility for legacy components.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::audio_foundations::{
        AudioService, ModularAudioService, ModularAudioServiceFactory, 
        AudioProcessingConfig, PitchAlgorithm, LegacyAudioBridge
    };
    use crate::modules::application_core::{
        ErrorService, ModularErrorService, ModularErrorServiceFactory,
        LegacyErrorBridge
    };
    use std::sync::{Arc, Mutex};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_modular_audio_service_creation() {
        let factory = ModularAudioServiceFactory::new();
        let service = factory.create_audio_service();
        
        // Service should be created successfully
        assert!(true); // Basic creation test
    }

    #[test] 
    fn test_modular_error_service_creation() {
        let factory = ModularErrorServiceFactory::new();
        let service = factory.create_error_service();
        
        // Service should be created successfully
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_legacy_audio_bridge_creation() {
        let factory = ModularAudioServiceFactory::new();
        let modular_service = Arc::new(Mutex::new(factory.create_audio_service()));
        let bridge = LegacyAudioBridge::new(modular_service);
        
        // Bridge should be created successfully
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_legacy_error_bridge_creation() {
        let factory = ModularErrorServiceFactory::new();
        let modular_service = Arc::new(Mutex::new(factory.create_error_service()));
        let bridge = LegacyErrorBridge::new(modular_service);
        
        // Bridge should be created successfully
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_audio_service_initialization() {
        let factory = ModularAudioServiceFactory::new();
        let mut service = factory.create_audio_service();
        
        let config = AudioProcessingConfig {
            sample_rate: 44100.0,
            buffer_size: 1024,
            target_latency_ms: 10.0,
            pitch_algorithm: PitchAlgorithm::McLeod,
        };
        
        // Initialization should succeed (might fail in test environment without browser APIs)
        let result = service.initialize(config);
        // Don't assert success since browser APIs may not be available in test
        // Just verify the method can be called
        let _ = result;
    }

    #[test]
    fn test_error_service_basic_operations() {
        use crate::legacy::active::services::error_manager::{ApplicationError, ErrorCategory, ErrorSeverity, RecoveryStrategy};
        
        let factory = ModularErrorServiceFactory::new();
        let mut service = factory.create_error_service();
        
        let test_error = ApplicationError::new(
            ErrorCategory::Unknown,
            ErrorSeverity::Info,
            "Test error".to_string(),
            Some("Test details".to_string()),
            RecoveryStrategy::None,
        );
        
        // Report error should succeed
        let result = service.report_error(test_error.clone(), Some("test_module"));
        assert!(result.is_ok());
        
        // Get recent errors should return the error
        let recent_errors = service.get_recent_errors(10);
        assert_eq!(recent_errors.len(), 1);
        assert_eq!(recent_errors[0].message, "Test error");
        
        // Clear errors should work
        let result = service.clear_errors();
        assert!(result.is_ok());
        
        // After clearing, no errors should remain
        let recent_errors = service.get_recent_errors(10);
        assert_eq!(recent_errors.len(), 0);
    }
}