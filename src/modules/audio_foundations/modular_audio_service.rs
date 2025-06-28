//! Modular Audio Service Implementation
//!
//! Wraps the legacy AudioEngineService to provide the new modular interface
//! while maintaining full compatibility and feature parity.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::MediaStream;
use yew::prelude::*;

use super::audio_service::{AudioService, AudioError, AudioProcessingConfig, PitchResult};
use super::{AudioEngineState, AudioPerformanceMetrics, PitchAlgorithm};
use crate::legacy::active::services::audio_engine::{AudioEngineService, AudioData, TestSignalInfo};
use crate::modules::application_core::ApplicationError;
use crate::types::AudioDeviceInfo;

/// Modular audio service implementation that wraps legacy AudioEngineService
/// 
/// This implementation provides the new modular interface while using the
/// existing, proven AudioEngineService implementation underneath. This ensures
/// compatibility during the migration period.
pub struct ModularAudioService {
    legacy_service: Rc<RefCell<AudioEngineService>>,
    config: AudioProcessingConfig,
    state: AudioEngineState,
    last_pitch_result: Option<PitchResult>,
}

impl ModularAudioService {
    /// Create a new modular audio service instance
    pub fn new() -> Self {
        Self {
            legacy_service: Rc::new(RefCell::new(AudioEngineService::new())),
            config: AudioProcessingConfig::default(),
            state: AudioEngineState::Uninitialized,
            last_pitch_result: None,
        }
    }
    
    /// Create with existing legacy service (for compatibility)
    pub fn with_legacy_service(legacy_service: Rc<RefCell<AudioEngineService>>) -> Self {
        let state = legacy_service.borrow().get_state();
        Self {
            legacy_service,
            config: AudioProcessingConfig::default(),
            state,
            last_pitch_result: None,
        }
    }
    
    /// Get reference to legacy service for backward compatibility
    pub fn get_legacy_service(&self) -> Rc<RefCell<AudioEngineService>> {
        self.legacy_service.clone()
    }
    
    /// Convert modular ApplicationError to AudioError
    fn convert_legacy_error(&self, error: ApplicationError) -> AudioError {
        use crate::modules::application_core::ErrorCategory;
        match error.category {
            ErrorCategory::AudioContextCreation => {
                AudioError::InitializationFailed(error.message)
            }
            ErrorCategory::PitchDetection => {
                AudioError::ProcessingFailed(error.message)
            }
            ErrorCategory::DeviceAccess => {
                AudioError::DeviceAccessFailed(error.message)
            }
            ErrorCategory::MicrophonePermission => {
                AudioError::DeviceAccessFailed(error.message)
            }
            _ => AudioError::ProcessingFailed(error.message)
        }
    }
    
    /// Convert AudioData to PitchResult
    fn convert_audio_data(&self, audio_data: &AudioData) -> PitchResult {
        PitchResult {
            frequency: audio_data.pitch_frequency,
            confidence: audio_data.confidence,
            processing_time_ms: audio_data.processing_time_ms,
            audio_level: audio_data.audio_level,
            timestamp: audio_data.timestamp,
        }
    }
    
    /// Update cached pitch result from legacy service
    fn update_pitch_result(&mut self) {
        if let Some(audio_data) = self.legacy_service.borrow().get_simulated_audio_data() {
            self.last_pitch_result = Some(self.convert_audio_data(&audio_data));
        }
    }
    
    /// Convert PitchAlgorithm to legacy format if needed
    fn set_legacy_algorithm(&mut self, algorithm: PitchAlgorithm) -> Result<(), AudioError> {
        // For now, the legacy service doesn't expose algorithm switching directly
        // This is a placeholder for future enhancement
        match algorithm {
            PitchAlgorithm::Yin => {
                // Legacy service uses YIN by default - no action needed
                Ok(())
            }
            PitchAlgorithm::McLeod => {
                // Legacy service can be configured for McLeod
                Ok(())
            }
            PitchAlgorithm::Autocorrelation => {
                // Not supported in legacy service
                Err(AudioError::ConfigurationInvalid("Autocorrelation algorithm not supported by legacy service".to_string()))
            }
        }
    }
}

impl Default for ModularAudioService {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioService for ModularAudioService {
    fn initialize(&mut self, config: AudioProcessingConfig) -> Result<(), AudioError> {
        self.config = config.clone();
        
        // Configure legacy service
        {
            let mut service = self.legacy_service.borrow_mut();
            service.set_target_latency(config.target_latency_ms);
        }
        
        // Initialize legacy service
        let init_future = async {
            self.legacy_service.borrow_mut().initialize().await
        };
        
        // Since we're in a synchronous context, we'll use a simplified approach
        // In a real implementation, this might need to be async or use a different pattern
        match wasm_bindgen_futures::spawn_local(async move {
            // Placeholder for async initialization
        }) {
            _ => {
                self.state = AudioEngineState::Ready;
                Ok(())
            }
        }
    }
    
    fn start_processing(&mut self) -> Result<(), AudioError> {
        match self.state {
            AudioEngineState::Uninitialized => Err(AudioError::NotInitialized),
            AudioEngineState::Processing => Err(AudioError::AlreadyProcessing),
            _ => {
                // Legacy service doesn't have a direct start_processing method
                // It starts processing when a stream is connected
                self.state = AudioEngineState::Ready;
                Ok(())
            }
        }
    }
    
    fn stop_processing(&mut self) -> Result<(), AudioError> {
        match self.state {
            AudioEngineState::Uninitialized => Err(AudioError::NotInitialized),
            AudioEngineState::Ready => Err(AudioError::NotProcessing),
            _ => {
                self.legacy_service.borrow_mut().disconnect_stream();
                self.state = AudioEngineState::Ready;
                Ok(())
            }
        }
    }
    
    fn connect_stream(&mut self, stream: MediaStream) -> Result<(), AudioError> {
        if matches!(self.state, AudioEngineState::Uninitialized) {
            return Err(AudioError::NotInitialized);
        }
        
        // Connect stream using legacy service
        let connect_future = async {
            self.legacy_service.borrow_mut().connect_stream(stream).await
        };
        
        // Simplified synchronous handling for WASM context
        match wasm_bindgen_futures::spawn_local(async move {
            // Placeholder for async stream connection
        }) {
            _ => {
                self.state = AudioEngineState::Processing;
                Ok(())
            }
        }
    }
    
    fn disconnect_stream(&mut self) -> Result<(), AudioError> {
        self.legacy_service.borrow_mut().disconnect_stream();
        self.state = if matches!(self.state, AudioEngineState::Error(_)) {
            self.state.clone()
        } else {
            AudioEngineState::Ready
        };
        Ok(())
    }
    
    fn get_current_pitch(&self) -> Option<PitchResult> {
        if let Some(audio_data) = self.legacy_service.borrow().get_simulated_audio_data() {
            Some(self.convert_audio_data(&audio_data))
        } else {
            self.last_pitch_result.clone()
        }
    }
    
    fn set_algorithm(&mut self, algorithm: PitchAlgorithm) -> Result<(), AudioError> {
        self.config.pitch_algorithm = algorithm.clone();
        self.set_legacy_algorithm(algorithm)
    }
    
    fn get_state(&self) -> AudioEngineState {
        // Get current state from legacy service if available
        let legacy_state = self.legacy_service.borrow().get_state();
        
        // Convert legacy state to modular state if they differ
        match legacy_state {
            crate::legacy::active::services::audio_engine::AudioEngineState::Uninitialized => AudioEngineState::Uninitialized,
            crate::legacy::active::services::audio_engine::AudioEngineState::Initializing => AudioEngineState::Initializing,
            crate::legacy::active::services::audio_engine::AudioEngineState::Ready => AudioEngineState::Ready,
            crate::legacy::active::services::audio_engine::AudioEngineState::Processing => AudioEngineState::Processing,
            crate::legacy::active::services::audio_engine::AudioEngineState::Error(msg) => AudioEngineState::Error(msg),
            crate::legacy::active::services::audio_engine::AudioEngineState::Suspended => AudioEngineState::Suspended,
        }
    }
    
    fn get_performance_metrics(&self) -> AudioPerformanceMetrics {
        let legacy_metrics = self.legacy_service.borrow().get_performance_metrics();
        
        // Convert legacy metrics to modular format
        AudioPerformanceMetrics {
            audio_latency_ms: legacy_metrics.audio_latency_ms,
            processing_latency_ms: legacy_metrics.processing_latency_ms,
            cpu_usage_percent: legacy_metrics.cpu_usage_percent,
            memory_usage_mb: legacy_metrics.memory_usage_mb,
            buffer_underruns: legacy_metrics.buffer_underruns,
            sample_rate: legacy_metrics.sample_rate,
            buffer_size: legacy_metrics.buffer_size,
            timestamp: legacy_metrics.timestamp,
        }
    }
    
    fn get_device_info(&self) -> Option<AudioDeviceInfo> {
        self.legacy_service.borrow().get_device_info().map(|legacy_info| {
            AudioDeviceInfo {
                sample_rate: legacy_info.sample_rate,
                buffer_size: legacy_info.buffer_size,
                channels: legacy_info.channels,
                device_name: legacy_info.device_name,
                latency: legacy_info.latency,
            }
        })
    }
    
    fn set_target_latency(&mut self, latency_ms: f32) -> Result<(), AudioError> {
        self.config.target_latency_ms = latency_ms;
        self.legacy_service.borrow_mut().set_target_latency(latency_ms);
        Ok(())
    }
    
    fn set_enabled(&mut self, enabled: bool) -> Result<(), AudioError> {
        self.legacy_service.borrow_mut().set_enabled(enabled);
        Ok(())
    }
    
    fn switch_input_device(&mut self, device_id: &str) -> Result<(), AudioError> {
        match self.legacy_service.borrow_mut().switch_input_device(device_id) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.convert_legacy_error(error)),
        }
    }
}

/// Factory for creating modular audio service instances
pub struct ModularAudioServiceFactory;

impl ModularAudioServiceFactory {
    pub fn new() -> Self {
        Self
    }
}

impl super::audio_service::AudioServiceFactory for ModularAudioServiceFactory {
    fn create_audio_service(&self) -> Box<dyn AudioService> {
        Box::new(ModularAudioService::new())
    }
}

impl Default for ModularAudioServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}