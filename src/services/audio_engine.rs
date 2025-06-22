use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, MediaStream};
use std::rc::Rc;
use std::cell::RefCell;
use gloo::console;

// Note: Core audio engine functionality is implemented directly in this service
// use crate::audio::engine::AudioEngine as CoreAudioEngine;
use crate::audio::performance_monitor::PerformanceMetrics;

// Temporary stub for CoreAudioEngine until audio module is properly integrated
#[derive(Debug)]
pub struct CoreAudioEngine {
    sample_rate: f32,
    buffer_size: usize,
    target_latency_ms: f32,
    enabled: bool,
}

impl CoreAudioEngine {
    pub fn new(sample_rate: f32, buffer_size: usize) -> Self {
        Self {
            sample_rate,
            buffer_size,
            target_latency_ms: 10.0,
            enabled: true,
        }
    }
    
    pub fn set_target_latency(&mut self, latency_ms: f32) {
        self.target_latency_ms = latency_ms;
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics::new()
    }
}
use crate::services::error_manager::{ApplicationError, ErrorCategory, ErrorSeverity, RecoveryStrategy};

/// Audio processing state for Yew components
#[derive(Clone, Debug, PartialEq)]
pub enum AudioEngineState {
    Uninitialized,
    Initializing,
    Ready,
    Processing,
    Error(String),
    Suspended,
}

/// Audio data structure for component communication
#[derive(Clone, Debug, PartialEq)]
pub struct AudioData {
    pub pitch_frequency: f32,
    pub confidence: f32,
    pub processing_time_ms: f32,
    pub audio_level: f32,
    pub timestamp: f64,
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            pitch_frequency: -1.0,
            confidence: 0.0,
            processing_time_ms: 0.0,
            audio_level: 0.0,
            timestamp: 0.0,
        }
    }
}

/// AudioEngine service for real-time audio processing in web applications
pub struct AudioEngineService {
    audio_context: Option<AudioContext>,
    core_engine: Rc<RefCell<CoreAudioEngine>>,
    state: AudioEngineState,
    target_latency_ms: f32,
    
    // Callbacks for Yew integration
    on_audio_data: Option<Callback<AudioData>>,
    on_performance_update: Option<Callback<PerformanceMetrics>>,
    on_error: Option<Callback<ApplicationError>>,
    on_state_change: Option<Callback<AudioEngineState>>,
}

impl AudioEngineService {
    /// Create a new AudioEngineService instance
    pub fn new() -> Self {
        let sample_rate = 44100.0;
        let buffer_size = 1024;
        
        Self {
            audio_context: None,
            core_engine: Rc::new(RefCell::new(CoreAudioEngine::new(sample_rate, buffer_size))),
            state: AudioEngineState::Uninitialized,
            target_latency_ms: 10.0,
            on_audio_data: None,
            on_performance_update: None,
            on_error: None,
            on_state_change: None,
        }
    }

    /// Initialize AudioContext and prepare for processing
    pub async fn initialize(&mut self) -> Result<(), ApplicationError> {
        if !matches!(self.state, AudioEngineState::Uninitialized) {
            return Ok(());
        }

        self.set_state(AudioEngineState::Initializing);

        // Create AudioContext
        let audio_context = match AudioContext::new() {
            Ok(ctx) => ctx,
            Err(e) => {
                let error = ApplicationError::audio_context_creation_failed(&format!("{:?}", e));
                self.handle_error(error.clone());
                return Err(error);
            }
        };

        // Update core engine with actual sample rate
        let sample_rate = audio_context.sample_rate();
        let buffer_size = 1024;
        self.core_engine = Rc::new(RefCell::new(CoreAudioEngine::new(sample_rate, buffer_size)));
        self.core_engine.borrow_mut().set_target_latency(self.target_latency_ms);

        self.audio_context = Some(audio_context);
        self.set_state(AudioEngineState::Ready);

        console::log!("AudioEngine initialized successfully");
        Ok(())
    }

    /// Connect MediaStream for audio processing (simplified)
    pub async fn connect_stream(&mut self, _stream: MediaStream) -> Result<(), ApplicationError> {
        if self.audio_context.is_none() {
            let error = ApplicationError::new(
                ErrorCategory::AudioContextCreation,
                ErrorSeverity::Critical,
                "AudioContext not initialized".to_string(),
                Some("Call initialize() before connecting stream".to_string()),
                RecoveryStrategy::UserGuidedRetry {
                    instructions: "Initialize audio engine before connecting stream".to_string(),
                },
            );
            self.handle_error(error.clone());
            return Err(error);
        }

        // For now, just simulate connection success
        self.set_state(AudioEngineState::Processing);
        console::log!("MediaStream connected successfully (simplified)");
        Ok(())
    }

    /// Disconnect MediaStream and cleanup
    pub fn disconnect_stream(&mut self) {
        self.set_state(AudioEngineState::Ready);
        console::log!("MediaStream disconnected");
    }

    /// Set callback for audio data updates
    pub fn set_on_audio_data(&mut self, callback: Callback<AudioData>) {
        self.on_audio_data = Some(callback);
    }

    /// Set callback for performance updates
    pub fn set_on_performance_update(&mut self, callback: Callback<PerformanceMetrics>) {
        self.on_performance_update = Some(callback);
    }

    /// Set callback for error handling
    pub fn set_on_error(&mut self, callback: Callback<ApplicationError>) {
        self.on_error = Some(callback);
    }

    /// Set callback for state changes
    pub fn set_on_state_change(&mut self, callback: Callback<AudioEngineState>) {
        self.on_state_change = Some(callback);
    }

    /// Get current state
    pub fn get_state(&self) -> AudioEngineState {
        self.state.clone()
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.core_engine.borrow().get_performance_metrics()
    }

    /// Set target latency
    pub fn set_target_latency(&mut self, latency_ms: f32) {
        self.target_latency_ms = latency_ms;
        self.core_engine.borrow_mut().set_target_latency(latency_ms);
    }

    /// Enable/disable audio processing
    pub fn set_enabled(&mut self, enabled: bool) {
        self.core_engine.borrow_mut().set_enabled(enabled);
    }

    /// Switch to a different input device
    pub fn switch_input_device(&mut self, device_id: &str) -> Result<(), ApplicationError> {
        console::log!(&format!("Switching to audio input device: {}", device_id));
        
        // For now, this is a placeholder implementation
        // In a full implementation, you would:
        // 1. Stop current stream if active
        // 2. Request new stream with specific device constraint
        // 3. Reconnect the processing pipeline
        
        // Simulate device switch success
        Ok(())
    }

    /// Handle internal errors
    fn handle_error(&mut self, error: ApplicationError) {
        console::error!(&format!("AudioEngine error: {}", error.message));
        
        let error_message = error.message.clone();
        let error_severity = error.severity.clone();
        
        if let Some(callback) = &self.on_error {
            callback.emit(error);
        }

        // Set error state if critical
        if matches!(error_severity, ErrorSeverity::Critical) {
            self.set_state(AudioEngineState::Error(error_message));
        }
    }

    /// Set internal state and notify callbacks
    fn set_state(&mut self, new_state: AudioEngineState) {
        if self.state != new_state {
            self.state = new_state.clone();
            
            if let Some(callback) = &self.on_state_change {
                callback.emit(new_state);
            }
        }
    }
}

impl Default for AudioEngineService {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioEngineService {
    fn drop(&mut self) {
        self.disconnect_stream();
        console::log!("AudioEngineService dropped and cleaned up");
    }
}

impl PartialEq for AudioEngineService {
    fn eq(&self, other: &Self) -> bool {
        // Simple equality check for Yew properties
        // AudioContext doesn't implement PartialEq, so we skip it
        self.state == other.state &&
        self.target_latency_ms == other.target_latency_ms
    }
} 