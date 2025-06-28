// Audio Foundations Module - STORY-013 & STORY-014
// Wraps existing AudioEngineService with new module interface
// Includes device management functionality

pub mod audio_foundations_module;
pub mod audio_engine_wrapper;
pub mod audio_events;

// Service Layer Migration - Step 2.1
pub mod audio_service;
pub mod modular_audio_service;

// STORY-015: Multi-Algorithm Pitch Detection
pub mod multi_algorithm_pitch_detector;
pub mod runtime_pitch_switching;

// STORY-016: Signal Generator
pub mod signal_generator;

// Device Manager modules - STORY-014
pub mod device_manager;
pub mod permission_manager;
pub mod device_monitor;
pub mod device_capabilities;
pub mod graceful_recovery;
pub mod optimization_settings;

// STORY-017: Performance Monitoring System
pub mod audio_performance_monitor;

#[cfg(test)]
pub mod integration_test;

#[cfg(test)]
pub mod simple_test;

#[cfg(test)]
pub mod device_manager_tests;

#[cfg(test)]
pub mod multi_algorithm_integration_tests;

#[cfg(test)]
pub mod signal_generator_tests;

#[cfg(test)]
pub mod audio_performance_monitor_tests;

#[cfg(test)]
pub mod comprehensive_pitch_detector_tests;

#[cfg(test)]
pub mod performance_regression_tests;

#[cfg(test)]
pub mod service_migration_test;

#[cfg(test)]
pub mod enhanced_integration_tests;

#[cfg(test)]
pub mod test_signal_validation;

#[cfg(test)]
pub mod cross_browser_tests;

#[cfg(test)]
pub mod error_scenario_tests;

#[cfg(test)]
pub mod memory_leak_detection_tests;

pub mod integration_example;

// Re-exports for clean API
pub use audio_foundations_module::{
    AudioFoundationsModule, AudioFoundationsConfig, PitchAlgorithm, 
    AudioPerformanceMetrics, DeviceCapabilities, LatencyProfile,
    AudioEvent, PerformanceEvent
};
pub use audio_engine_wrapper::AudioEngineWrapper;
pub use audio_events::*;

// Service Layer Migration re-exports
pub use audio_service::{
    AudioService, AudioServiceFactory, AudioProcessingConfig, 
    PitchResult as ServicePitchResult, AudioError
};
pub use modular_audio_service::{ModularAudioService, ModularAudioServiceFactory};

// STORY-015: Multi-Algorithm Pitch Detection re-exports
pub use multi_algorithm_pitch_detector::{
    MultiAlgorithmPitchDetector, PitchDetector, PitchDetectionConfig,
    PitchResult, PitchError, AlgorithmInfo, PerformanceComparison
};
pub use runtime_pitch_switching::{
    RuntimePitchSwitcher, AutoSwitchConfig, RuntimePerformanceStats
};

// STORY-016: Signal Generator re-exports
pub use signal_generator::{
    SignalGenerator, WebSignalGenerator, SignalConfig, WaveformType, SignalError,
    TestSignalLibrary, TuningReference, SignalInjector
};

// Device Manager re-exports
pub use device_manager::{DeviceManager, WebDeviceManager, AudioDevice, AudioDeviceType, DeviceError};
pub use permission_manager::{PermissionManager, WebPermissionManager, PermissionRequestResult, PermissionError, PermissionRecoveryAction};
pub use device_monitor::{DeviceMonitor, WebDeviceMonitor, DeviceMonitorError, DeviceMonitoringState, DeviceRecoveryAction};
pub use device_capabilities::{DeviceCapabilityManager, WebDeviceCapabilityManager, DeviceCapabilities as StoredDeviceCapabilities, AudioUseCase, OptimalAudioSettings, CapabilityError};
pub use graceful_recovery::{GracefulRecoveryManager, WebGracefulRecoveryManager, RecoveryResult, RecoveryAction, QualityImpact, RecoveryError};
pub use optimization_settings::{DeviceOptimizationManager, WebDeviceOptimizationManager, DeviceOptimizationSettings, PerformanceRecommendation, OptimizationError};

// STORY-017: Performance Monitoring re-exports
pub use audio_performance_monitor::{
    PerformanceMonitor, AudioPerformanceMonitor, AudioPerformanceMetrics as LegacyAudioPerformanceMetrics, OperationMetrics,
    ThresholdViolation, ViolationSeverity, PerformanceRegression, PerformanceThresholds,
    MonitoringOverhead, MonitoringConfig, MeasurementId, DropoutType
};

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