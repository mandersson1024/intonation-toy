//! Integration test for AudioFoundations module - Story 013

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;

use super::{AudioFoundationsModule, AudioFoundations, AudioEngine};
use crate::legacy::services::AudioEngineService;
use crate::modules::application_core::{
    Module,
    typed_event_bus::TypedEventBus,
    event_bus::{EventBus, EventBusState}
};

/// Integration test for Story 013 - AudioFoundations module wrapper
#[cfg(test)]
pub async fn test_audio_foundations_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Create legacy audio engine service
    let legacy_engine = Rc::new(RefCell::new(AudioEngineService::new()));
    
    // Create event bus for integration
    let event_bus = Arc::new(TypedEventBus::new()?);
    
    // Create AudioFoundations module
    let mut audio_module = AudioFoundationsModule::new(legacy_engine.clone());
    audio_module.set_event_bus(event_bus.clone());
    
    // Test Module trait implementation
    assert_eq!(audio_module.module_id().as_str(), "audio-foundations");
    assert_eq!(audio_module.module_name(), "Audio Foundations");
    assert_eq!(audio_module.module_version(), "1.0.0");
    assert!(audio_module.dependencies().is_empty());
    
    // Test AudioFoundations trait implementation
    
    // 1. Initialize module
    audio_module.initialize()?;
    
    // 2. Start module
    audio_module.start()?;
    
    // 3. Test audio engine access
    let audio_engine = audio_module.audio_engine();
    let current_state = audio_engine.get_state();
    println!("Current audio engine state: {:?}", current_state);
    
    // 4. Test legacy service backward compatibility
    let legacy_service = audio_module.legacy_audio_service();
    let legacy_state = legacy_service.borrow().get_state();
    println!("Legacy service state: {:?}", legacy_state);
    
    // 5. Test event publishing (basic validation)
    audio_module.publish_pitch_detected(440.0, 0.9, 1000);
    
    // 6. Stop module
    audio_module.stop()?;
    
    // 7. Shutdown module
    audio_module.shutdown()?;
    
    println!("✅ AudioFoundations integration test completed successfully");
    Ok(())
}

/// Test backward compatibility with existing AudioEngineService
#[cfg(test)]
pub fn test_backward_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    // Create legacy audio engine service
    let legacy_engine = Rc::new(RefCell::new(AudioEngineService::new()));
    
    // Create AudioFoundations module
    let audio_module = AudioFoundationsModule::new(legacy_engine.clone());
    
    // Verify that we can still access the legacy service
    let legacy_service = audio_module.legacy_audio_service();
    
    // Verify that it's the same instance
    assert!(Rc::ptr_eq(&legacy_engine, &legacy_service));
    
    // Test that legacy methods still work
    legacy_service.borrow_mut().set_enabled(true);
    legacy_service.borrow_mut().set_target_latency(20.0);
    
    let state = legacy_service.borrow().get_state();
    println!("Legacy compatibility test - state: {:?}", state);
    
    println!("✅ Backward compatibility test completed successfully");
    Ok(())
}

/// Test event publishing functionality
#[cfg(test)]
pub async fn test_event_publishing() -> Result<(), Box<dyn std::error::Error>> {
    // Create legacy audio engine service
    let legacy_engine = Rc::new(RefCell::new(AudioEngineService::new()));
    
    // Create event bus
    let event_bus = Arc::new(TypedEventBus::new()?);
    
    // Create AudioFoundations module
    let mut audio_module = AudioFoundationsModule::new(legacy_engine);
    audio_module.set_event_bus(event_bus.clone());
    
    // Initialize the module
    audio_module.initialize()?;
    
    // Test various event publishing scenarios
    
    // 1. Pitch detection event
    audio_module.publish_pitch_detected(440.0, 0.95, 1500);
    
    // 2. State change event
    audio_module.publish_state_change(
        super::AudioEngineState::Uninitialized,
        super::AudioEngineState::Ready,
        "Test state change".to_string()
    );
    
    // 3. Error event
    audio_module.publish_error(
        super::audio_events::AudioErrorType::Warning,
        "Test error".to_string(),
        "Test context".to_string(),
        Some("Restart audio engine".to_string())
    );
    
    println!("✅ Event publishing test completed successfully");
    Ok(())
}

/// Performance test to ensure no regression
#[cfg(test)]
pub fn test_performance_regression() -> Result<(), Box<dyn std::error::Error>> {
    // Create legacy audio engine service
    let legacy_engine = Rc::new(RefCell::new(AudioEngineService::new()));
    
    // Create AudioFoundations module
    let audio_module = AudioFoundationsModule::new(legacy_engine.clone());
    
    // Time operations to ensure no significant performance regression
    let start = std::time::Instant::now();
    
    // Perform typical operations
    for _i in 0..1000 {
        let _state = audio_module.audio_engine().get_state();
        let _legacy_state = legacy_engine.borrow().get_state();
    }
    
    let duration = start.elapsed();
    println!("Performance test: 1000 state queries took {:?}", duration);
    
    // Ensure operations complete in reasonable time (< 10ms for 1000 operations)
    assert!(duration.as_millis() < 10, "Performance regression detected: operations took {:?}", duration);
    
    println!("✅ Performance regression test completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_creation() {
        let legacy_engine = Rc::new(RefCell::new(AudioEngineService::new()));
        let audio_module = AudioFoundationsModule::new(legacy_engine);
        
        assert_eq!(audio_module.module_id().as_str(), "audio-foundations");
        assert_eq!(audio_module.module_name(), "Audio Foundations");
        assert_eq!(audio_module.module_version(), "1.0.0");
    }
    
    #[test]
    fn test_backward_compatibility_sync() {
        test_backward_compatibility().expect("Backward compatibility test failed");
    }
    
    #[test]
    fn test_performance_regression_sync() {
        test_performance_regression().expect("Performance regression test failed");
    }
}