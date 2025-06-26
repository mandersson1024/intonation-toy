//! Simple test to validate AudioFoundations module - Story 013

#[cfg(test)]
mod audio_foundations_tests {
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::sync::Arc;
    
    use crate::legacy::services::AudioEngineService;
    use crate::modules::audio_foundations::{AudioFoundationsModule, AudioFoundations};
    use crate::modules::application_core::{Module, typed_event_bus::TypedEventBus};

    #[test]
    fn test_module_creation_and_basic_operations() {
        // Create legacy audio engine service
        let legacy_engine = Rc::new(RefCell::new(AudioEngineService::new()));
        
        // Create AudioFoundations module
        let mut audio_module = AudioFoundationsModule::new(legacy_engine.clone());
        
        // Test Module trait implementation
        assert_eq!(audio_module.module_id().as_str(), "audio-foundations");
        assert_eq!(audio_module.module_name(), "Audio Foundations");
        assert_eq!(audio_module.module_version(), "1.0.0");
        assert!(audio_module.dependencies().is_empty());
        
        // Test initialization using AudioFoundations trait explicitly
        AudioFoundations::initialize(&mut audio_module).expect("Module initialization failed");
        
        // Test backward compatibility - ensure legacy service is accessible
        let legacy_service = audio_module.legacy_audio_service();
        assert!(Rc::ptr_eq(&legacy_engine, &legacy_service));
    }
    
    #[test]
    fn test_event_bus_integration() {
        // Create legacy audio engine service
        let legacy_engine = Rc::new(RefCell::new(AudioEngineService::new()));
        
        // Create event bus
        let event_bus = Arc::new(TypedEventBus::new());
        
        // Create AudioFoundations module
        let mut audio_module = AudioFoundationsModule::new(legacy_engine);
        audio_module.set_event_bus(event_bus.clone());
        
        // Initialize the module using AudioFoundations trait explicitly
        AudioFoundations::initialize(&mut audio_module).expect("Module initialization failed");
        
        // Test that event publishing doesn't crash (basic validation)
        audio_module.publish_pitch_detected(440.0, 0.95, 1500);
    }
    
    #[test]
    fn test_audio_engine_wrapper() {
        // Create legacy audio engine service
        let legacy_engine = Rc::new(RefCell::new(AudioEngineService::new()));
        
        // Create AudioFoundations module
        let audio_module = AudioFoundationsModule::new(legacy_engine.clone());
        
        // Test audio engine access
        let audio_engine = audio_module.audio_engine();
        let current_state = audio_engine.get_state();
        
        // Should be in uninitialized state initially
        match current_state {
            crate::modules::audio_foundations::AudioEngineState::Uninitialized => {
                // Expected
            }
            _ => panic!("Expected uninitialized state, got: {:?}", current_state),
        }
    }
}