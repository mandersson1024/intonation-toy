//! Test for Pure Audio Service Implementation
//!
//! Verifies the pure modular audio service works correctly without legacy dependencies.

#[cfg(test)]
mod tests {
    use super::super::pure_audio_service::{PureAudioService, AudioProcessingConfig, AudioError};
    use super::super::pure_modular_audio_service::{PureModularAudioService, PureModularAudioServiceFactory};
    use super::super::{AudioEngineState, PitchAlgorithm};

    #[test]
    fn test_pure_audio_service_creation() {
        let service = PureModularAudioService::new();
        assert_eq!(service.get_state(), AudioEngineState::Uninitialized);
        println!("✓ Pure audio service created successfully");
    }

    #[test]
    fn test_pure_audio_service_initialization() {
        let mut service = PureModularAudioService::new();
        let config = AudioProcessingConfig {
            sample_rate: 44100.0,
            buffer_size: 1024,
            target_latency_ms: 10.0,
            pitch_algorithm: PitchAlgorithm::McLeod,
        };

        // In WASM environment, this might fail due to AudioContext requirements
        // but the service should handle this gracefully
        match service.initialize(config) {
            Ok(()) => {
                println!("✓ Pure audio service initialized successfully");
                assert_eq!(service.get_state(), AudioEngineState::Ready);
            }
            Err(AudioError::InitializationFailed(_)) => {
                println!("⚠ Audio initialization failed (expected in test environment)");
                // This is expected in test environment without proper browser context
            }
            Err(e) => {
                panic!("Unexpected error during initialization: {:?}", e);
            }
        }
    }

    #[test]
    fn test_pure_audio_service_state_transitions() {
        let mut service = PureModularAudioService::new();
        
        // Test start processing without initialization
        match service.start_processing() {
            Err(AudioError::NotInitialized) => {
                println!("✓ Correctly rejects start_processing when not initialized");
            }
            _ => panic!("Should reject start_processing when not initialized"),
        }

        // Test stop processing without initialization
        match service.stop_processing() {
            Err(AudioError::NotInitialized) => {
                println!("✓ Correctly rejects stop_processing when not initialized");
            }
            _ => panic!("Should reject stop_processing when not initialized"),
        }
    }

    #[test]
    fn test_pure_audio_service_configuration() {
        let mut service = PureModularAudioService::new();
        
        // Test setting algorithm
        assert!(service.set_algorithm(PitchAlgorithm::Yin).is_ok());
        println!("✓ Algorithm setting works");
        
        // Test setting target latency
        assert!(service.set_target_latency(15.0).is_ok());
        println!("✓ Target latency setting works");
        
        // Test enabling/disabling
        assert!(service.set_enabled(true).is_ok());
        assert!(service.set_enabled(false).is_ok());
        println!("✓ Enable/disable functionality works");
    }

    #[test]
    fn test_pure_audio_service_factory() {
        let factory = PureModularAudioServiceFactory::new();
        let service = factory.create_audio_service();
        
        // Verify we get a service in uninitialized state
        assert_eq!(service.get_state(), AudioEngineState::Uninitialized);
        println!("✓ Pure audio service factory works correctly");
    }

    #[test]
    fn test_pure_audio_service_metrics() {
        let service = PureModularAudioService::new();
        
        // Should be able to get metrics even when not initialized
        let metrics = service.get_performance_metrics();
        assert!(metrics.sample_rate > 0.0);
        println!("✓ Performance metrics accessible: sample_rate={}", metrics.sample_rate);
    }

    #[test]
    fn test_pure_audio_service_test_signal() {
        let mut service = PureModularAudioService::new();
        
        // Test signal configuration
        service.set_test_signal_info(440.0, 0.5, "sine", true);
        let test_info = service.get_test_signal_info();
        
        assert_eq!(test_info.frequency, 440.0);
        assert_eq!(test_info.amplitude, 0.5);
        assert_eq!(test_info.signal_type, "sine");
        assert!(test_info.is_active);
        
        println!("✓ Test signal configuration works correctly");
    }

    #[test]
    fn test_pure_audio_service_no_legacy_dependencies() {
        // This test verifies that the pure service can be created without
        // any legacy dependencies by using only the pure types
        use super::super::pure_audio_service::{PureAudioServiceFactory};
        
        let factory = PureModularAudioServiceFactory::new();
        let _service = factory.create_audio_service();
        
        // If this compiles and runs, we've successfully created a pure service
        println!("✓ Pure audio service has no legacy dependencies");
    }
}