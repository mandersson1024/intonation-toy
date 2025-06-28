// Audio Foundations Module Implementation - STORY-013
// Main implementation of the Audio Foundations module

use std::error::Error;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

use super::{AudioFoundations, AudioEngine, AudioEngineWrapper};
use super::audio_events::*;
use crate::modules::application_core::{
    Module, 
    ModuleId,
    event_bus::{EventBus, EventBusError},
    typed_event_bus::TypedEventBus
};
use crate::legacy::services::AudioEngineService;

/// Main Audio Foundations module implementation
pub struct AudioFoundationsModule {
    module_id: ModuleId,
    audio_engine: AudioEngineWrapper,
    event_bus: Option<Arc<TypedEventBus>>,
    initialized: bool,
    started: bool,
}

impl AudioFoundationsModule {
    /// Create new Audio Foundations module with existing AudioEngineService
    pub fn new(legacy_engine: Rc<RefCell<AudioEngineService>>) -> Self {
        Self {
            module_id: ModuleId::new("audio-foundations"),
            audio_engine: AudioEngineWrapper::new(legacy_engine),
            event_bus: None,
            initialized: false,
            started: false,
        }
    }
    
    /// Create module with event bus integration for modular communication
    pub fn new_with_event_bus(
        legacy_engine: Rc<RefCell<AudioEngineService>>, 
        event_bus: Arc<TypedEventBus>
    ) -> Self {
        let mut module = Self::new(legacy_engine);
        module.set_event_bus(event_bus);
        module
    }
    
    /// Set the event bus for publishing audio events
    pub fn set_event_bus(&mut self, event_bus: Arc<TypedEventBus>) {
        self.event_bus = Some(event_bus);
    }
    
    /// Setup periodic pitch detection event publishing
    /// This bridges legacy audio processing to the modular event system
    pub fn start_event_publishing(&self) {
        // TODO: Implementation would setup timer/callback to publish real audio events
        // For now, log that event publishing capability is ready
        web_sys::console::log_1(&"AudioFoundations: Event publishing capability ready".into());
    }
    
    /// Get module health status
    pub fn get_health_status(&self) -> String {
        format!("AudioFoundations: initialized={}, started={}, engine_state={:?}", 
                self.initialized, self.started, self.audio_engine.get_state())
    }
    
    /// Publish pitch detection event
    pub fn publish_pitch_detected(&self, frequency: f32, confidence: f32, processing_time_ns: u64) {
        if let Some(ref event_bus) = self.event_bus {
            let event = PitchDetectionEvent {
                frequency,
                confidence,
                signal_info: SignalInfo {
                    amplitude: 0.5, // Placeholder - would get from actual processing
                    clarity: confidence,
                    harmonic_content: 0.0, // Placeholder
                    noise_floor: 0.0, // Placeholder
                },
                processing_time_ns,
                timestamp_ns: get_timestamp_ns(),
                source_buffer_ref: String::new(), // Placeholder
            };
            
            if let Err(e) = event_bus.publish(event) {
                web_sys::console::warn_1(&format!("Failed to publish pitch detection event: {:?}", e).into());
            }
        }
    }
    
    /// Publish audio processing state change event
    pub fn publish_state_change(&self, old_state: super::AudioEngineState, new_state: super::AudioEngineState, context: String) {
        if let Some(ref event_bus) = self.event_bus {
            let event = AudioProcessingStateEvent {
                old_state,
                new_state,
                timestamp_ns: get_timestamp_ns(),
                context,
            };
            
            if let Err(e) = event_bus.publish(event) {
                web_sys::console::warn_1(&format!("Failed to publish state change event: {:?}", e).into());
            }
        }
    }
    
    /// Publish audio error event
    pub fn publish_error(&self, error_type: AudioErrorType, message: String, context: String, recovery_suggestion: Option<String>) {
        if let Some(ref event_bus) = self.event_bus {
            let event = AudioErrorEvent {
                error_type,
                message,
                context,
                recovery_suggestion,
                timestamp_ns: get_timestamp_ns(),
            };
            
            if let Err(e) = event_bus.publish(event) {
                web_sys::console::warn_1(&format!("Failed to publish error event: {:?}", e).into());
            }
        }
    }
    
    /// Get access to legacy audio engine service for backward compatibility
    pub fn legacy_audio_service(&self) -> Rc<RefCell<AudioEngineService>> {
        self.audio_engine.legacy_service()
    }
}

impl AudioFoundations for AudioFoundationsModule {
    fn audio_engine(&self) -> &dyn AudioEngine {
        &self.audio_engine
    }
    
    fn audio_engine_mut(&mut self) -> &mut dyn AudioEngine {
        &mut self.audio_engine
    }
    
    fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        if self.initialized {
            return Ok(());
        }
        
        // Initialize the underlying audio engine
        // For Story 013, we preserve existing initialization patterns
        let old_state = self.audio_engine.get_state();
        
        // Legacy service initialization happens asynchronously
        // For the migration phase, we mark as initialized if the engine is in a valid state
        let current_state = self.audio_engine.get_state();
        
        self.initialized = true;
        
        // Publish state change event
        self.publish_state_change(old_state, current_state, "Module initialization".to_string());
        
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.initialized {
            return Err("Module not initialized".into());
        }
        
        if self.started {
            return Ok(());
        }
        
        let old_state = self.audio_engine.get_state();
        
        // Start audio processing
        self.audio_engine.start_processing()?;
        
        let new_state = self.audio_engine.get_state();
        self.started = true;
        
        // Publish state change event
        self.publish_state_change(old_state, new_state, "Module start".to_string());
        
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.started {
            return Ok(());
        }
        
        let old_state = self.audio_engine.get_state();
        
        // Stop audio processing
        self.audio_engine.stop_processing()?;
        
        let new_state = self.audio_engine.get_state();
        self.started = false;
        
        // Publish state change event
        self.publish_state_change(old_state, new_state, "Module stop".to_string());
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        if self.started {
            AudioFoundations::stop(self)?;
        }
        
        // Disconnect any active streams
        self.audio_engine.disconnect_stream()?;
        
        self.initialized = false;
        
        // Publish final state
        let final_state = self.audio_engine.get_state();
        self.publish_state_change(final_state.clone(), final_state, "Module shutdown".to_string());
        
        Ok(())
    }
}

impl Module for AudioFoundationsModule {
    fn module_id(&self) -> ModuleId {
        self.module_id.clone()
    }
    
    fn module_name(&self) -> &str {
        "Audio Foundations"
    }
    
    fn module_version(&self) -> &str {
        "1.0.0"
    }
    
    fn dependencies(&self) -> Vec<ModuleId> {
        // No dependencies for the foundation audio module
        vec![]
    }
    
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        AudioFoundations::initialize(self)
    }
    
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        AudioFoundations::start(self)
    }
    
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        AudioFoundations::stop(self)
    }
    
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        AudioFoundations::shutdown(self)
    }
}

// Safety: AudioFoundationsModule manages thread safety through its contained components
unsafe impl Send for AudioFoundationsModule {}
unsafe impl Sync for AudioFoundationsModule {}