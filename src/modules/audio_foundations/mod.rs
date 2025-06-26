// Audio Foundations Module - STORY-013 & STORY-014
// Wraps existing AudioEngineService with new module interface
// Includes device management functionality

pub mod audio_foundations_module;
pub mod audio_engine_wrapper;
pub mod audio_events;

// Device Manager modules - STORY-014
pub mod device_manager;
pub mod permission_manager;
pub mod device_monitor;
pub mod device_capabilities;
pub mod graceful_recovery;
pub mod optimization_settings;

#[cfg(test)]
pub mod integration_test;

#[cfg(test)]
pub mod simple_test;

#[cfg(test)]
pub mod device_manager_tests;

pub mod integration_example;

// Re-exports for clean API
pub use audio_foundations_module::AudioFoundationsModule;
pub use audio_engine_wrapper::AudioEngineWrapper;
pub use audio_events::*;

// Device Manager re-exports
pub use device_manager::{DeviceManager, WebDeviceManager, AudioDevice, AudioDeviceType, DeviceError};
pub use permission_manager::{PermissionManager, WebPermissionManager, PermissionRequestResult, PermissionError, PermissionRecoveryAction};
pub use device_monitor::{DeviceMonitor, WebDeviceMonitor, DeviceMonitorError, DeviceMonitoringState, DeviceRecoveryAction};
pub use device_capabilities::{DeviceCapabilityManager, WebDeviceCapabilityManager, DeviceCapabilities, AudioUseCase, OptimalAudioSettings, CapabilityError};
pub use graceful_recovery::{GracefulRecoveryManager, WebGracefulRecoveryManager, RecoveryResult, RecoveryAction, QualityImpact, RecoveryError};
pub use optimization_settings::{DeviceOptimizationManager, WebDeviceOptimizationManager, DeviceOptimizationSettings, PerformanceRecommendation, OptimizationError};

// Core traits for the Audio Foundations module
use std::error::Error;
use std::sync::Arc;
use crate::modules::application_core::event_bus::{EventBus, Event};

/// Core trait for audio engine functionality
pub trait AudioEngine: Send + Sync {
    /// Start audio processing
    fn start_processing(&mut self) -> Result<(), Box<dyn Error>>;
    
    /// Stop audio processing  
    fn stop_processing(&mut self) -> Result<(), Box<dyn Error>>;
    
    /// Get current audio processing state
    fn get_state(&self) -> AudioEngineState;
    
    /// Set target processing latency in milliseconds
    fn set_target_latency(&mut self, latency_ms: f32) -> Result<(), Box<dyn Error>>;
    
    /// Connect audio stream for processing
    fn connect_stream(&mut self, stream: web_sys::MediaStream) -> Result<(), Box<dyn Error>>;
    
    /// Disconnect current audio stream
    fn disconnect_stream(&mut self) -> Result<(), Box<dyn Error>>;
}

/// Core trait for the Audio Foundations module
pub trait AudioFoundations: Send + Sync {
    /// Get access to the audio engine
    fn audio_engine(&self) -> &dyn AudioEngine;
    
    /// Get mutable access to the audio engine
    fn audio_engine_mut(&mut self) -> &mut dyn AudioEngine;
    
    /// Initialize audio foundations
    fn initialize(&mut self) -> Result<(), Box<dyn Error>>;
    
    /// Start audio processing
    fn start(&mut self) -> Result<(), Box<dyn Error>>;
    
    /// Stop audio processing
    fn stop(&mut self) -> Result<(), Box<dyn Error>>;
    
    /// Shutdown audio foundations
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>>;
}

/// Audio processing states
#[derive(Debug, Clone, PartialEq)]
pub enum AudioEngineState {
    Uninitialized,
    Initializing, 
    Ready,
    Processing,
    Error(String),
    Suspended,
}

impl std::fmt::Display for AudioEngineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioEngineState::Uninitialized => write!(f, "Uninitialized"),
            AudioEngineState::Initializing => write!(f, "Initializing"),
            AudioEngineState::Ready => write!(f, "Ready"),
            AudioEngineState::Processing => write!(f, "Processing"),
            AudioEngineState::Error(msg) => write!(f, "Error: {}", msg),
            AudioEngineState::Suspended => write!(f, "Suspended"),
        }
    }
}