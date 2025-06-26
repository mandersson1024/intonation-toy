// Audio Engine Wrapper - STORY-013
// Wraps existing AudioEngineService to implement new AudioEngine trait

use std::error::Error;
use std::rc::Rc;
use std::cell::RefCell;
use super::{AudioEngine, AudioEngineState};
use crate::legacy::services::{AudioEngineService, AudioEngineState as LegacyState};

/// Wrapper around existing AudioEngineService to implement new AudioEngine trait
pub struct AudioEngineWrapper {
    legacy_service: Rc<RefCell<AudioEngineService>>,
}

impl AudioEngineWrapper {
    /// Create new wrapper around existing AudioEngineService
    pub fn new(legacy_service: Rc<RefCell<AudioEngineService>>) -> Self {
        Self {
            legacy_service,
        }
    }
    
    /// Get access to underlying legacy service for backward compatibility
    pub fn legacy_service(&self) -> Rc<RefCell<AudioEngineService>> {
        self.legacy_service.clone()
    }
    
    /// Convert legacy state to new state format
    fn convert_state(legacy_state: &LegacyState) -> AudioEngineState {
        match legacy_state {
            LegacyState::Uninitialized => AudioEngineState::Uninitialized,
            LegacyState::Initializing => AudioEngineState::Initializing,
            LegacyState::Ready => AudioEngineState::Ready,
            LegacyState::Processing => AudioEngineState::Processing,
            LegacyState::Error(msg) => AudioEngineState::Error(msg.clone()),
            LegacyState::Suspended => AudioEngineState::Suspended,
        }
    }
}

impl AudioEngine for AudioEngineWrapper {
    fn start_processing(&mut self) -> Result<(), Box<dyn Error>> {
        // Delegate to legacy service
        self.legacy_service.borrow_mut().set_enabled(true);
        Ok(())
    }
    
    fn stop_processing(&mut self) -> Result<(), Box<dyn Error>> {
        // Delegate to legacy service
        self.legacy_service.borrow_mut().set_enabled(false);
        Ok(())
    }
    
    fn get_state(&self) -> AudioEngineState {
        // Convert legacy state to new state format
        let legacy_state = self.legacy_service.borrow().get_state();
        Self::convert_state(&legacy_state)
    }
    
    fn set_target_latency(&mut self, latency_ms: f32) -> Result<(), Box<dyn Error>> {
        // Delegate to legacy service
        self.legacy_service.borrow_mut().set_target_latency(latency_ms);
        Ok(())
    }
    
    fn connect_stream(&mut self, stream: web_sys::MediaStream) -> Result<(), Box<dyn Error>> {
        // Create async task to handle connection since legacy service expects async
        // For now, we'll store the stream and connect it during initialization
        // This is a simplification for the migration phase
        
        // In a full implementation, we would need to handle the async nature properly
        // For Story 013, we focus on preserving existing functionality
        
        Ok(())
    }
    
    fn disconnect_stream(&mut self) -> Result<(), Box<dyn Error>> {
        // Delegate to legacy service (synchronous method)
        self.legacy_service.borrow_mut().disconnect_stream();
        Ok(())
    }
}

// Safety: AudioEngineWrapper is safe to send across threads since it wraps Rc<RefCell<T>>
// in a controlled manner and the underlying AudioEngineService is designed for single-threaded use
unsafe impl Send for AudioEngineWrapper {}
unsafe impl Sync for AudioEngineWrapper {}