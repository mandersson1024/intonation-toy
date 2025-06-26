//! Integration example for AudioFoundations module with main application
//! This demonstrates how Story 013 enables gradual migration to modular architecture

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;

use super::{AudioFoundationsModule, AudioFoundations};
use crate::legacy::services::AudioEngineService;
use crate::modules::application_core::{
    Module,
    typed_event_bus::TypedEventBus,
    event_bus::EventBus
};

/// Example of how to integrate AudioFoundations module with existing application
pub struct IntegratedAudioApplication {
    // Legacy components still work
    legacy_audio_service: Rc<RefCell<AudioEngineService>>,
    
    // New modular architecture
    audio_foundations: AudioFoundationsModule,
    event_bus: Arc<TypedEventBus>,
}

impl IntegratedAudioApplication {
    /// Create application with both legacy and modular architectures
    pub fn new() -> Self {
        // Create legacy service (existing code continues to work)
        let legacy_audio_service = Rc::new(RefCell::new(AudioEngineService::new()));
        
        // Create event bus for modular architecture
        let event_bus = Arc::new(TypedEventBus::new());
        
        // Create AudioFoundations module (Story 013 implementation)
        let mut audio_foundations = AudioFoundationsModule::new(legacy_audio_service.clone());
        audio_foundations.set_event_bus(event_bus.clone());
        
        Self {
            legacy_audio_service,
            audio_foundations,
            event_bus,
        }
    }
    
    /// Initialize the application (demonstrates gradual migration)
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the audio foundations module
        AudioFoundations::initialize(&mut self.audio_foundations)?;
        
        // Legacy components can still be initialized separately if needed
        // (preserving existing initialization code)
        
        println!("Application initialized with both legacy and modular audio systems");
        Ok(())
    }
    
    /// Start audio processing (both approaches work)
    pub fn start_audio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Option 1: Use new modular interface
        AudioFoundations::start(&mut self.audio_foundations)?;
        
        // Option 2: Legacy code still works unchanged
        // self.legacy_audio_service.borrow_mut().set_enabled(true);
        
        println!("Audio processing started");
        Ok(())
    }
    
    /// Get current audio state (demonstrates preserved functionality)
    pub fn get_audio_state(&self) -> String {
        // New modular interface
        format!("{:?}", self.audio_foundations.audio_engine().get_state())
    }
    
    /// Legacy method access (backward compatibility)
    pub fn legacy_audio_operations(&self) {
        // Existing code continues to work without changes
        let service = &self.legacy_audio_service;
        let state = service.borrow().get_state();
        println!("Legacy access - Audio state: {:?}", state);
        
        // Legacy callbacks, settings, etc. all still work
        // service.borrow_mut().set_target_latency(25.0);
    }
    
    /// Demonstrate event publishing (new capability)
    pub fn simulate_pitch_detection(&self) {
        self.audio_foundations.publish_pitch_detected(440.0, 0.95, 1500);
    }
    
    /// Stop audio processing
    pub fn stop_audio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        AudioFoundations::stop(&mut self.audio_foundations)?;
        println!("Audio processing stopped");
        Ok(())
    }
    
    /// Get module information (new modular capabilities)
    pub fn get_module_info(&self) -> String {
        format!(
            "Module: {} v{} (ID: {})",
            self.audio_foundations.module_name(),
            self.audio_foundations.module_version(),
            self.audio_foundations.module_id().as_str()
        )
    }
}

/// Example usage in main application
#[cfg(test)]
mod integration_example_tests {
    use super::*;
    
    #[test]
    fn test_integrated_application() {
        let mut app = IntegratedAudioApplication::new();
        
        // Test initialization
        app.initialize().expect("Failed to initialize application");
        
        // Test module info
        let module_info = app.get_module_info();
        assert!(module_info.contains("Audio Foundations"));
        assert!(module_info.contains("1.0.0"));
        
        // Test audio operations
        app.start_audio().expect("Failed to start audio");
        
        let state = app.get_audio_state();
        println!("Audio state after start: {}", state);
        
        // Test legacy compatibility
        app.legacy_audio_operations();
        
        // Test new event capabilities
        app.simulate_pitch_detection();
        
        // Test stopping
        app.stop_audio().expect("Failed to stop audio");
        
        println!("✅ Integrated application test completed successfully");
    }
}

/// Example of how existing Yew components can gradually migrate
pub fn example_yew_component_migration() {
    println!("📋 Yew Component Migration Example:");
    println!("   Before Story 013:");
    println!("   ```rust");
    println!("   // Old approach - direct legacy service access");
    println!("   let audio_engine = use_state(|| Some(Rc::new(RefCell::new(AudioEngineService::new()))));");
    println!("   ```");
    println!("");
    println!("   After Story 013:");
    println!("   ```rust");
    println!("   // New approach - can use either legacy or modular interface");
    println!("   let legacy_service = Rc::new(RefCell::new(AudioEngineService::new()));");
    println!("   let audio_module = AudioFoundationsModule::new(legacy_service.clone());");
    println!("   ");
    println!("   // Component can access both:");
    println!("   // - audio_module.audio_engine() // New modular interface");
    println!("   // - legacy_service             // Existing legacy interface");
    println!("   ```");
    println!("");
    println!("   ✅ Migration can happen gradually - no breaking changes!");
}

/// Demonstration function
pub fn demonstrate_story_013_benefits() {
    println!("🎯 Story 013 Benefits Demonstration:\n");
    
    let mut app = IntegratedAudioApplication::new();
    app.initialize().expect("Initialization failed");
    
    println!("1. ✅ Backward Compatibility:");
    app.legacy_audio_operations();
    
    println!("\n2. ✅ New Modular Interface:");
    println!("   {}", app.get_module_info());
    
    println!("\n3. ✅ Event-Driven Architecture:");
    app.simulate_pitch_detection();
    
    println!("\n4. ✅ Zero Performance Regression:");
    println!("   - Wrapper pattern provides zero-cost abstraction");
    println!("   - Direct delegation to existing implementation");
    
    println!("\n5. ✅ Gradual Migration Path:");
    example_yew_component_migration();
    
    app.stop_audio().expect("Stop failed");
    
    println!("\n🚀 Story 013 successfully enables gradual migration to modular architecture!");
}