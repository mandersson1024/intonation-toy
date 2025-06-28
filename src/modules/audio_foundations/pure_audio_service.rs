//! Pure Modular Audio Service Interface
//!
//! This module provides clean, modular audio service interfaces without any legacy dependencies.
//! These types replace the legacy AudioEngineService with pure implementations designed 
//! for the modular architecture.

use std::error::Error;
use std::fmt::{self, Display};
use wasm_bindgen::prelude::*;
use web_sys::MediaStream;
use crate::modules::application_core::ApplicationError;
use crate::modules::audio_foundations::{AudioEngineState, AudioPerformanceMetrics, PitchAlgorithm};
use crate::types::AudioDeviceInfo;

/// Audio processing configuration for pure modular architecture
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

/// Audio service error types for pure modular architecture
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

/// Convert AudioError to ApplicationError for error service integration
impl From<AudioError> for ApplicationError {
    fn from(audio_error: AudioError) -> Self {
        use crate::modules::application_core::{ErrorCategory, ErrorSeverity, RecoveryStrategy};

        match audio_error {
            AudioError::InitializationFailed(msg) => ApplicationError::new(
                ErrorCategory::AudioContextCreation,
                ErrorSeverity::Critical,
                "Audio service initialization failed".to_string(),
                Some(msg),
                RecoveryStrategy::AutomaticRetry { max_attempts: 3, delay_ms: 1000 },
            ),
            AudioError::ProcessingFailed(msg) => ApplicationError::new(
                ErrorCategory::PitchDetection,
                ErrorSeverity::Warning,
                "Audio processing failed".to_string(),
                Some(msg),
                RecoveryStrategy::AutomaticRetry { max_attempts: 5, delay_ms: 500 },
            ),
            AudioError::StreamConnectionFailed(msg) => ApplicationError::new(
                ErrorCategory::DeviceAccess,
                ErrorSeverity::Warning,
                "Audio stream connection failed".to_string(),
                Some(msg),
                RecoveryStrategy::UserGuidedRetry { 
                    instructions: "Check microphone permissions and try again".to_string() 
                },
            ),
            AudioError::DeviceAccessFailed(msg) => ApplicationError::new(
                ErrorCategory::MicrophonePermission,
                ErrorSeverity::Warning,
                "Audio device access failed".to_string(),
                Some(msg),
                RecoveryStrategy::UserGuidedRetry { 
                    instructions: "Grant microphone permissions in your browser".to_string() 
                },
            ),
            AudioError::ConfigurationInvalid(msg) => ApplicationError::new(
                ErrorCategory::StateManagement,
                ErrorSeverity::Warning,
                "Invalid audio configuration".to_string(),
                Some(msg),
                RecoveryStrategy::GracefulDegradation { 
                    fallback_description: "Using default audio settings".to_string() 
                },
            ),
            AudioError::NotInitialized => ApplicationError::new(
                ErrorCategory::StateManagement,
                ErrorSeverity::Warning,
                "Audio service not initialized".to_string(),
                Some("Initialize the audio service before use".to_string()),
                RecoveryStrategy::UserGuidedRetry { 
                    instructions: "Initialize audio service first".to_string() 
                },
            ),
            AudioError::AlreadyProcessing => ApplicationError::new(
                ErrorCategory::StateManagement,
                ErrorSeverity::Info,
                "Audio processing already active".to_string(),
                None,
                RecoveryStrategy::None,
            ),
            AudioError::NotProcessing => ApplicationError::new(
                ErrorCategory::StateManagement,
                ErrorSeverity::Info,
                "Audio processing not active".to_string(),
                None,
                RecoveryStrategy::None,
            ),
        }
    }
}

/// Pure modular audio service interface
/// 
/// This trait abstracts audio processing functionality with pure modular types,
/// enabling different implementations without legacy dependencies.
pub trait PureAudioService: Send + Sync {
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

/// Pure audio service factory trait for creating service instances
pub trait PureAudioServiceFactory {
    /// Create a new audio service instance
    /// 
    /// # Returns
    /// New audio service instance
    fn create_audio_service(&self) -> Box<dyn PureAudioService>;
}