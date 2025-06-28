//! Audio Service Abstraction
//!
//! Provides the core audio service interface for the modular architecture.
//! This abstraction layer enables different audio engine implementations 
//! and facilitates legacy compatibility.

use std::error::Error;
use std::fmt::{self, Display};
use wasm_bindgen::prelude::*;
use web_sys::MediaStream;
use crate::modules::audio_foundations::{AudioEngineState, AudioPerformanceMetrics, PitchAlgorithm};
use crate::types::AudioDeviceInfo;

/// Audio processing configuration
#[derive(Debug, Clone, PartialEq)]
pub struct AudioProcessingConfig {
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub target_latency_ms: f32,
    pub pitch_algorithm: PitchAlgorithm,
}

impl Default for AudioProcessingConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100.0,
            buffer_size: 1024,
            target_latency_ms: 10.0,
            pitch_algorithm: PitchAlgorithm::McLeod,
        }
    }
}

/// Pitch detection result from audio processing
#[derive(Debug, Clone, PartialEq)]
pub struct PitchResult {
    pub frequency: f32,
    pub confidence: f32,
    pub processing_time_ms: f32,
    pub audio_level: f32,
    pub timestamp: f64,
}

impl Default for PitchResult {
    fn default() -> Self {
        Self {
            frequency: -1.0,
            confidence: 0.0,
            processing_time_ms: 0.0,
            audio_level: 0.0,
            timestamp: 0.0,
        }
    }
}

/// Audio service error types
#[derive(Debug, Clone, PartialEq)]
pub enum AudioError {
    InitializationFailed(String),
    ProcessingFailed(String),
    StreamConnectionFailed(String),
    DeviceAccessFailed(String),
    ConfigurationInvalid(String),
    NotInitialized,
    AlreadyProcessing,
    NotProcessing,
}

impl Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::InitializationFailed(msg) => write!(f, "Audio initialization failed: {}", msg),
            AudioError::ProcessingFailed(msg) => write!(f, "Audio processing failed: {}", msg),
            AudioError::StreamConnectionFailed(msg) => write!(f, "Stream connection failed: {}", msg),
            AudioError::DeviceAccessFailed(msg) => write!(f, "Device access failed: {}", msg),
            AudioError::ConfigurationInvalid(msg) => write!(f, "Invalid configuration: {}", msg),
            AudioError::NotInitialized => write!(f, "Audio service not initialized"),
            AudioError::AlreadyProcessing => write!(f, "Audio processing already active"),
            AudioError::NotProcessing => write!(f, "Audio processing not active"),
        }
    }
}

impl Error for AudioError {}

/// Core audio service interface for modular architecture
/// 
/// This trait abstracts audio processing functionality to enable different
/// implementations and facilitate legacy compatibility during migration.
pub trait AudioService: Send + Sync {
    /// Initialize the audio service with configuration
    /// 
    /// # Arguments
    /// * `config` - Audio processing configuration
    /// 
    /// # Returns
    /// * `Ok(())` - Service initialized successfully
    /// * `Err(AudioError)` - Initialization failed
    fn initialize(&mut self, config: AudioProcessingConfig) -> Result<(), AudioError>;
    
    /// Start audio processing
    /// 
    /// Begins real-time audio processing and pitch detection.
    /// Service must be initialized before calling this method.
    /// 
    /// # Returns
    /// * `Ok(())` - Processing started successfully
    /// * `Err(AudioError)` - Failed to start processing
    fn start_processing(&mut self) -> Result<(), AudioError>;
    
    /// Stop audio processing
    /// 
    /// Stops real-time audio processing while maintaining initialization.
    /// 
    /// # Returns
    /// * `Ok(())` - Processing stopped successfully
    /// * `Err(AudioError)` - Failed to stop processing
    fn stop_processing(&mut self) -> Result<(), AudioError>;
    
    /// Connect audio stream for processing
    /// 
    /// # Arguments
    /// * `stream` - MediaStream from getUserMedia
    /// 
    /// # Returns
    /// * `Ok(())` - Stream connected successfully
    /// * `Err(AudioError)` - Failed to connect stream
    fn connect_stream(&mut self, stream: MediaStream) -> Result<(), AudioError>;
    
    /// Disconnect current audio stream
    /// 
    /// # Returns
    /// * `Ok(())` - Stream disconnected successfully
    /// * `Err(AudioError)` - Failed to disconnect stream
    fn disconnect_stream(&mut self) -> Result<(), AudioError>;
    
    /// Get current pitch detection result
    /// 
    /// Returns the most recent pitch detection result if processing is active.
    /// 
    /// # Returns
    /// * `Some(PitchResult)` - Current pitch data
    /// * `None` - No current pitch data available
    fn get_current_pitch(&self) -> Option<PitchResult>;
    
    /// Set pitch detection algorithm
    /// 
    /// # Arguments
    /// * `algorithm` - Pitch detection algorithm to use
    /// 
    /// # Returns
    /// * `Ok(())` - Algorithm set successfully
    /// * `Err(AudioError)` - Failed to set algorithm
    fn set_algorithm(&mut self, algorithm: PitchAlgorithm) -> Result<(), AudioError>;
    
    /// Get current audio service state
    /// 
    /// # Returns
    /// Current state of the audio service
    fn get_state(&self) -> AudioEngineState;
    
    /// Get performance metrics
    /// 
    /// Returns current performance metrics including latency, CPU usage, etc.
    /// 
    /// # Returns
    /// Current performance metrics
    fn get_performance_metrics(&self) -> AudioPerformanceMetrics;
    
    /// Get audio device information
    /// 
    /// Returns information about the currently connected audio device.
    /// 
    /// # Returns
    /// * `Some(AudioDeviceInfo)` - Device information if available
    /// * `None` - No device information available
    fn get_device_info(&self) -> Option<AudioDeviceInfo>;
    
    /// Set target processing latency
    /// 
    /// # Arguments
    /// * `latency_ms` - Target latency in milliseconds
    /// 
    /// # Returns
    /// * `Ok(())` - Target latency set successfully
    /// * `Err(AudioError)` - Failed to set target latency
    fn set_target_latency(&mut self, latency_ms: f32) -> Result<(), AudioError>;
    
    /// Enable or disable audio processing
    /// 
    /// # Arguments
    /// * `enabled` - Whether processing should be enabled
    /// 
    /// # Returns
    /// * `Ok(())` - Processing state set successfully
    /// * `Err(AudioError)` - Failed to set processing state
    fn set_enabled(&mut self, enabled: bool) -> Result<(), AudioError>;
    
    /// Switch to a different input device
    /// 
    /// # Arguments
    /// * `device_id` - Device identifier to switch to
    /// 
    /// # Returns
    /// * `Ok(())` - Device switched successfully
    /// * `Err(AudioError)` - Failed to switch device
    fn switch_input_device(&mut self, device_id: &str) -> Result<(), AudioError>;
}

/// Audio service factory trait for creating service instances
pub trait AudioServiceFactory {
    /// Create a new audio service instance
    /// 
    /// # Returns
    /// New audio service instance
    fn create_audio_service(&self) -> Box<dyn AudioService>;
}