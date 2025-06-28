use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, MediaStream};
use std::rc::Rc;
use std::cell::RefCell;
use gloo::console;

use crate::audio::performance_monitor::PerformanceMetrics;
use crate::audio::engine::AudioEngine;
use crate::modules::application_core::{ApplicationError, ErrorCategory, ErrorSeverity, RecoveryStrategy};

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

/// Information about current test signal being processed
#[derive(Debug, Clone, PartialEq)]
pub struct TestSignalInfo {
    pub frequency: f32,
    pub amplitude: f32,
    pub signal_type: String,
    pub is_active: bool,
}

impl Default for TestSignalInfo {
    fn default() -> Self {
        Self {
            frequency: 440.0,
            amplitude: 0.0,
            signal_type: "None".to_string(),
            is_active: false,
        }
    }
}

/// Audio device configuration information
#[derive(Debug, Clone, PartialEq)]
pub struct AudioDeviceInfo {
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub channels: u32,
    pub device_name: String,
    pub latency: f64,
}

/// AudioEngine service for real-time audio processing in web applications
pub struct AudioEngineService {
    audio_context: Option<AudioContext>,
    core_engine: Rc<RefCell<AudioEngine>>,
    state: AudioEngineState,
    target_latency_ms: f32,
    test_signal_info: TestSignalInfo,
    
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
            core_engine: Rc::new(RefCell::new(AudioEngine::new(sample_rate, buffer_size))),
            state: AudioEngineState::Uninitialized,
            target_latency_ms: 10.0,
            test_signal_info: TestSignalInfo::default(),
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
        self.core_engine = Rc::new(RefCell::new(AudioEngine::new(sample_rate, buffer_size)));
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

        // For now, just simulate connection success and start processing simulation
        self.set_state(AudioEngineState::Processing);
        
        // Simulate audio processing to generate real performance metrics
        self.start_audio_processing_simulation();
        
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

    /// Get current performance metrics from underlying engine
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        match self.core_engine.try_borrow() {
            Ok(engine) => engine.get_performance_metrics(),
            Err(_) => PerformanceMetrics::default(),
        }
    }

    /// Get current device information (simplified implementation)
    pub fn get_device_info(&self) -> Option<AudioDeviceInfo> {
        self.audio_context.as_ref().map(|ctx| {
            AudioDeviceInfo {
                sample_rate: ctx.sample_rate(),
                buffer_size: 1024, // Hardcoded for now
                channels: 2,       // Hardcoded for now
                device_name: "Default".to_string(),
                latency: self.target_latency_ms as f64 / 1000.0,
            }
        })
    }

    /// Continuous audio processing loop for generating realistic test data
    fn process_continuous_audio(&self) {
        let mut engine = self.core_engine.borrow_mut();
        
        // Generate realistic test audio data
        let time_ms = js_sys::Date::now();
        let frequency = 440.0 + 20.0 * (time_ms / 1000.0).sin() as f32;
        let confidence = 0.8 + 0.2 * (time_ms / 500.0).cos() as f32;
        
        let audio_data = AudioData {
            pitch_frequency: frequency,
            confidence: confidence,
            processing_time_ms: 2.5 + 1.0 * (time_ms / 750.0).sin() as f32,
            audio_level: 0.5 + 0.3 * (time_ms / 300.0).cos() as f32,
            timestamp: time_ms,
        };
        
        // Notify subscribers
        if let Some(callback) = &self.on_audio_data {
            callback.emit(audio_data);
        }
        
        // Update performance metrics
        if let Some(callback) = &self.on_performance_update {
            callback.emit(engine.get_performance_metrics());
        }
    }

    /// Set target processing latency
    pub fn set_target_latency(&mut self, latency_ms: f32) {
        self.target_latency_ms = latency_ms;
        self.core_engine.borrow_mut().set_target_latency(latency_ms);
    }

    /// Enable or disable audio processing
    pub fn set_enabled(&mut self, enabled: bool) {
        // Implementation depends on specific engine capabilities
        if enabled && matches!(self.state, AudioEngineState::Ready) {
            self.set_state(AudioEngineState::Processing);
        } else if !enabled && matches!(self.state, AudioEngineState::Processing) {
            self.set_state(AudioEngineState::Ready);
        }
    }

    /// Set test signal information
    pub fn set_test_signal_info(&mut self, frequency: f32, amplitude: f32, signal_type: &str, is_active: bool) {
        self.test_signal_info = TestSignalInfo {
            frequency,
            amplitude,
            signal_type: signal_type.to_string(),
            is_active,
        };
    }

    /// Get current test signal information
    pub fn get_test_signal_info(&self) -> &TestSignalInfo {
        &self.test_signal_info
    }

    /// Get simulated audio data for testing
    pub fn get_simulated_audio_data(&self) -> Option<AudioData> {
        if matches!(self.state, AudioEngineState::Processing) {
            let time_ms = js_sys::Date::now();
            Some(AudioData {
                pitch_frequency: 440.0 + 50.0 * (time_ms / 1000.0).sin() as f32,
                confidence: 0.85 + 0.1 * (time_ms / 800.0).cos() as f32,
                processing_time_ms: 2.0 + 0.5 * (time_ms / 600.0).sin() as f32,
                audio_level: 0.6 + 0.2 * (time_ms / 400.0).cos() as f32,
                timestamp: time_ms,
            })
        } else {
            None
        }
    }

    /// Switch to different input device
    pub fn switch_input_device(&mut self, device_id: &str) -> Result<(), ApplicationError> {
        // Simplified implementation - would need actual device switching logic
        console::log!(&format!("Switching to device: {}", device_id));
        
        // For now, just simulate success
        if self.audio_context.is_some() {
            Ok(())
        } else {
            Err(ApplicationError::new(
                ErrorCategory::AudioContextCreation,
                ErrorSeverity::Warning,
                "Cannot switch device: AudioContext not initialized".to_string(),
                None,
                RecoveryStrategy::UserGuidedRetry {
                    instructions: "Initialize audio engine before switching devices".to_string(),
                },
            ))
        }
    }
    
    /// Start audio processing simulation for testing
    fn start_audio_processing_simulation(&mut self) {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        
        let process_callback = {
            let engine = self.core_engine.clone();
            let on_audio_data = self.on_audio_data.clone();
            let on_performance_update = self.on_performance_update.clone();
            
            Closure::wrap(Box::new(move || {
                // Simulate continuous audio processing
                let time_ms = js_sys::Date::now();
                let frequency = 440.0 + 20.0 * (time_ms / 1000.0).sin() as f32;
                let confidence = 0.8 + 0.2 * (time_ms / 500.0).cos() as f32;
                
                let audio_data = AudioData {
                    pitch_frequency: frequency,
                    confidence: confidence,
                    processing_time_ms: 2.5 + 1.0 * (time_ms / 750.0).sin() as f32,
                    audio_level: 0.5 + 0.3 * (time_ms / 300.0).cos() as f32,
                    timestamp: time_ms,
                };
                
                if let Some(callback) = &on_audio_data {
                    callback.emit(audio_data);
                }
                
                if let Some(callback) = &on_performance_update {
                    if let Ok(eng) = engine.try_borrow() {
                        callback.emit(eng.get_performance_metrics());
                    }
                }
            }) as Box<dyn Fn()>)
        };
        
        // Set up periodic callback
        if let Some(window) = web_sys::window() {
            let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
                process_callback.as_ref().unchecked_ref(),
                50, // 50ms interval for 20Hz updates
            );
            process_callback.forget();
        }
    }

    /// Handle errors and notify subscribers
    fn handle_error(&mut self, error: ApplicationError) {
        if let Some(callback) = &self.on_error {
            callback.emit(error.clone());
        }
        
        // Log error to console
        console::error!(&format!("AudioEngine Error: {}", error.message));
        if let Some(details) = &error.details {
            console::error!(&format!("Error Details: {}", details));
        }
        
        // Update state if it's a critical error
        if matches!(error.severity, ErrorSeverity::Critical) {
            self.set_state(AudioEngineState::Error(error.message.clone()));
        }
    }

    /// Update state and notify subscribers
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
        if matches!(self.state, AudioEngineState::Processing) {
            self.disconnect_stream();
        }
    }
}

impl PartialEq for AudioEngineService {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.target_latency_ms == other.target_latency_ms
    }
} 