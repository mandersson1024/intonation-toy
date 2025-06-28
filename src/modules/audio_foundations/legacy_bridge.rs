//! Legacy Audio Service Bridge
//!
//! Provides backward compatibility by implementing the legacy AudioEngineService
//! interface while using the new modular AudioService underneath.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, MediaStream};
use yew::prelude::*;

use super::audio_service::{AudioService, AudioProcessingConfig, PitchResult};
use super::{AudioEngineState, AudioPerformanceMetrics, PitchAlgorithm};
use crate::legacy::active::services::audio_engine::{
    AudioEngineService, AudioData, TestSignalInfo, AudioDeviceInfo as LegacyAudioDeviceInfo
};
use crate::legacy::active::services::error_manager::{ApplicationError, ErrorCategory, ErrorSeverity, RecoveryStrategy};
use crate::audio::performance_monitor::PerformanceMetrics;

/// Legacy compatibility bridge that wraps modular AudioService
/// 
/// This bridge allows legacy components to continue using the familiar
/// AudioEngineService interface while actually using the new modular
/// implementation underneath.
pub struct LegacyAudioBridge {
    modular_service: Arc<Mutex<dyn AudioService>>,
    audio_context: Option<AudioContext>,
    test_signal_info: TestSignalInfo,
    target_latency_ms: f32,
    
    // Callbacks for Yew integration (matching legacy interface)
    on_audio_data: Option<Callback<AudioData>>,
    on_performance_update: Option<Callback<PerformanceMetrics>>,
    on_error: Option<Callback<ApplicationError>>,
    on_state_change: Option<Callback<crate::legacy::active::services::audio_engine::AudioEngineState>>,
}

impl LegacyAudioBridge {
    /// Create new legacy bridge with modular service
    pub fn new(modular_service: Arc<Mutex<dyn AudioService>>) -> Self {
        Self {
            modular_service,
            audio_context: None,
            test_signal_info: TestSignalInfo::default(),
            target_latency_ms: 10.0,
            on_audio_data: None,
            on_performance_update: None,
            on_error: None,
            on_state_change: None,
        }
    }
    
    /// Convert modular AudioError to legacy ApplicationError
    fn convert_modular_error(&self, error: super::audio_service::AudioError) -> ApplicationError {
        match error {
            super::audio_service::AudioError::InitializationFailed(msg) => {
                ApplicationError::audio_context_creation_failed(&msg)
            }
            super::audio_service::AudioError::ProcessingFailed(msg) => {
                ApplicationError::new(
                    ErrorCategory::PitchDetection,
                    ErrorSeverity::Warning,
                    "Audio processing failed".to_string(),
                    Some(msg),
                    RecoveryStrategy::AutomaticRetry { max_attempts: 3, delay_ms: 1000 },
                )
            }
            super::audio_service::AudioError::StreamConnectionFailed(msg) => {
                ApplicationError::new(
                    ErrorCategory::DeviceAccess,
                    ErrorSeverity::Warning,
                    "Stream connection failed".to_string(),
                    Some(msg),
                    RecoveryStrategy::UserGuidedRetry {
                        instructions: "Please check your microphone connection and try again".to_string(),
                    },
                )
            }
            super::audio_service::AudioError::DeviceAccessFailed(msg) => {
                ApplicationError::microphone_permission_denied(&msg)
            }
            super::audio_service::AudioError::ConfigurationInvalid(msg) => {
                ApplicationError::new(
                    ErrorCategory::AudioContextCreation,
                    ErrorSeverity::Warning,
                    "Invalid audio configuration".to_string(),
                    Some(msg),
                    RecoveryStrategy::UserGuidedRetry {
                        instructions: "Please check your audio settings".to_string(),
                    },
                )
            }
            super::audio_service::AudioError::NotInitialized => {
                ApplicationError::new(
                    ErrorCategory::AudioContextCreation,
                    ErrorSeverity::Critical,
                    "Audio service not initialized".to_string(),
                    Some("Call initialize() before using audio features".to_string()),
                    RecoveryStrategy::UserGuidedRetry {
                        instructions: "Initialize audio engine before use".to_string(),
                    },
                )
            }
            super::audio_service::AudioError::AlreadyProcessing => {
                ApplicationError::new(
                    ErrorCategory::PitchDetection,
                    ErrorSeverity::Info,
                    "Audio processing already active".to_string(),
                    None,
                    RecoveryStrategy::None,
                )
            }
            super::audio_service::AudioError::NotProcessing => {
                ApplicationError::new(
                    ErrorCategory::PitchDetection,
                    ErrorSeverity::Info,
                    "Audio processing not active".to_string(),
                    None,
                    RecoveryStrategy::None,
                )
            }
        }
    }
    
    /// Convert modular PitchResult to legacy AudioData
    fn convert_pitch_result(&self, pitch_result: &PitchResult) -> AudioData {
        AudioData {
            pitch_frequency: pitch_result.frequency,
            confidence: pitch_result.confidence,
            processing_time_ms: pitch_result.processing_time_ms,
            audio_level: pitch_result.audio_level,
            timestamp: pitch_result.timestamp,
        }
    }
    
    /// Convert modular AudioEngineState to legacy state
    fn convert_state(&self, state: &AudioEngineState) -> crate::legacy::active::services::audio_engine::AudioEngineState {
        match state {
            AudioEngineState::Uninitialized => crate::legacy::active::services::audio_engine::AudioEngineState::Uninitialized,
            AudioEngineState::Initializing => crate::legacy::active::services::audio_engine::AudioEngineState::Initializing,
            AudioEngineState::Ready => crate::legacy::active::services::audio_engine::AudioEngineState::Ready,
            AudioEngineState::Processing => crate::legacy::active::services::audio_engine::AudioEngineState::Processing,
            AudioEngineState::Error(msg) => crate::legacy::active::services::audio_engine::AudioEngineState::Error(msg.clone()),
            AudioEngineState::Suspended => crate::legacy::active::services::audio_engine::AudioEngineState::Suspended,
        }
    }
    
    /// Convert modular performance metrics to legacy format
    fn convert_performance_metrics(&self, metrics: &AudioPerformanceMetrics) -> PerformanceMetrics {
        PerformanceMetrics {
            audio_latency_ms: metrics.audio_latency_ms,
            processing_latency_ms: metrics.processing_latency_ms,
            cpu_usage_percent: metrics.cpu_usage_percent,
            memory_usage_mb: metrics.memory_usage_mb,
            buffer_underruns: metrics.buffer_underruns,
            sample_rate: metrics.sample_rate,
            buffer_size: metrics.buffer_size,
            timestamp: metrics.timestamp,
        }
    }
    
    /// Handle internal errors by converting and notifying callbacks
    fn handle_error(&mut self, error: super::audio_service::AudioError) {
        let legacy_error = self.convert_modular_error(error);
        
        if let Some(callback) = &self.on_error {
            callback.emit(legacy_error);
        }
    }
    
    /// Notify state change callbacks
    fn notify_state_change(&self, state: &AudioEngineState) {
        if let Some(callback) = &self.on_state_change {
            callback.emit(self.convert_state(state));
        }
    }
}

// Implement the legacy AudioEngineService interface
impl LegacyAudioBridge {
    /// Initialize AudioContext and prepare for processing (legacy interface)
    pub async fn initialize(&mut self) -> Result<(), ApplicationError> {
        // Create AudioContext first
        let audio_context = match AudioContext::new() {
            Ok(ctx) => ctx,
            Err(e) => {
                let error = ApplicationError::audio_context_creation_failed(&format!("{:?}", e));
                self.handle_error(super::audio_service::AudioError::InitializationFailed(error.message.clone()));
                return Err(error);
            }
        };
        
        let sample_rate = audio_context.sample_rate();
        self.audio_context = Some(audio_context);
        
        // Initialize modular service
        let config = AudioProcessingConfig {
            sample_rate,
            buffer_size: 1024,
            target_latency_ms: self.target_latency_ms,
            pitch_algorithm: PitchAlgorithm::McLeod,
        };
        
        match self.modular_service.lock() {
            Ok(mut service) => {
                if let Err(error) = service.initialize(config) {
                    let legacy_error = self.convert_modular_error(error);
                    return Err(legacy_error);
                }
                
                let state = service.get_state();
                self.notify_state_change(&state);
                Ok(())
            }
            Err(_) => {
                let error = ApplicationError::audio_context_creation_failed("Failed to access audio service");
                Err(error)
            }
        }
    }
    
    /// Connect MediaStream for audio processing (legacy interface)
    pub async fn connect_stream(&mut self, stream: MediaStream) -> Result<(), ApplicationError> {
        match self.modular_service.lock() {
            Ok(mut service) => {
                if let Err(error) = service.connect_stream(stream) {
                    let legacy_error = self.convert_modular_error(error);
                    return Err(legacy_error);
                }
                
                let state = service.get_state();
                self.notify_state_change(&state);
                Ok(())
            }
            Err(_) => {
                let error = ApplicationError::new(
                    ErrorCategory::DeviceAccess,
                    ErrorSeverity::Warning,
                    "Failed to access audio service".to_string(),
                    None,
                    RecoveryStrategy::AutomaticRetry { max_attempts: 3, delay_ms: 1000 },
                );
                Err(error)
            }
        }
    }
    
    /// Disconnect MediaStream and cleanup (legacy interface)
    pub fn disconnect_stream(&mut self) {
        if let Ok(mut service) = self.modular_service.lock() {
            let _ = service.disconnect_stream();
            let state = service.get_state();
            self.notify_state_change(&state);
        }
    }
    
    /// Set callback for audio data updates (legacy interface)
    pub fn set_on_audio_data(&mut self, callback: Callback<AudioData>) {
        self.on_audio_data = Some(callback);
    }
    
    /// Set callback for performance updates (legacy interface)
    pub fn set_on_performance_update(&mut self, callback: Callback<PerformanceMetrics>) {
        self.on_performance_update = Some(callback);
    }
    
    /// Set callback for error handling (legacy interface)
    pub fn set_on_error(&mut self, callback: Callback<ApplicationError>) {
        self.on_error = Some(callback);
    }
    
    /// Set callback for state changes (legacy interface)
    pub fn set_on_state_change(&mut self, callback: Callback<crate::legacy::active::services::audio_engine::AudioEngineState>) {
        self.on_state_change = Some(callback);
    }
    
    /// Get current state (legacy interface)
    pub fn get_state(&self) -> crate::legacy::active::services::audio_engine::AudioEngineState {
        if let Ok(service) = self.modular_service.lock() {
            self.convert_state(&service.get_state())
        } else {
            crate::legacy::active::services::audio_engine::AudioEngineState::Error("Service unavailable".to_string())
        }
    }
    
    /// Get performance metrics (legacy interface)
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        if let Ok(service) = self.modular_service.lock() {
            self.convert_performance_metrics(&service.get_performance_metrics())
        } else {
            PerformanceMetrics::default()
        }
    }
    
    /// Get audio device configuration information (legacy interface)
    pub fn get_device_info(&self) -> Option<LegacyAudioDeviceInfo> {
        if let Ok(service) = self.modular_service.lock() {
            service.get_device_info().map(|info| LegacyAudioDeviceInfo {
                sample_rate: info.sample_rate,
                buffer_size: info.buffer_size,
                channels: info.channels,
                device_name: info.device_name,
                latency: info.latency,
            })
        } else {
            None
        }
    }
    
    /// Set target latency (legacy interface)
    pub fn set_target_latency(&mut self, latency_ms: f32) {
        self.target_latency_ms = latency_ms;
        if let Ok(mut service) = self.modular_service.lock() {
            let _ = service.set_target_latency(latency_ms);
        }
    }
    
    /// Enable/disable audio processing (legacy interface)
    pub fn set_enabled(&mut self, enabled: bool) {
        if let Ok(mut service) = self.modular_service.lock() {
            let _ = service.set_enabled(enabled);
        }
    }
    
    /// Update test signal information (legacy interface)
    pub fn set_test_signal_info(&mut self, frequency: f32, amplitude: f32, signal_type: &str, is_active: bool) {
        self.test_signal_info = TestSignalInfo {
            frequency,
            amplitude,
            signal_type: signal_type.to_string(),
            is_active,
        };
    }
    
    /// Get current test signal information (legacy interface)
    pub fn get_test_signal_info(&self) -> &TestSignalInfo {
        &self.test_signal_info
    }
    
    /// Get simulated audio data based on current test signal (legacy interface)
    pub fn get_simulated_audio_data(&self) -> Option<AudioData> {
        if let Ok(service) = self.modular_service.lock() {
            service.get_current_pitch().map(|pitch| self.convert_pitch_result(&pitch))
        } else {
            None
        }
    }
    
    /// Switch to a different input device (legacy interface)
    pub fn switch_input_device(&mut self, device_id: &str) -> Result<(), ApplicationError> {
        match self.modular_service.lock() {
            Ok(mut service) => {
                if let Err(error) = service.switch_input_device(device_id) {
                    Err(self.convert_modular_error(error))
                } else {
                    Ok(())
                }
            }
            Err(_) => {
                Err(ApplicationError::new(
                    ErrorCategory::DeviceAccess,
                    ErrorSeverity::Warning,
                    "Failed to access audio service".to_string(),
                    None,
                    RecoveryStrategy::AutomaticRetry { max_attempts: 3, delay_ms: 1000 },
                ))
            }
        }
    }
}

impl PartialEq for LegacyAudioBridge {
    fn eq(&self, other: &Self) -> bool {
        // Simple equality check for Yew properties
        self.target_latency_ms == other.target_latency_ms &&
        self.test_signal_info == other.test_signal_info
    }
}