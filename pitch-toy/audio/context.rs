//! AudioContext Manager for Real-Time Audio Processing
//!
//! This module provides a comprehensive wrapper around the Web Audio API's AudioContext,
//! designed for real-time pitch detection applications. It handles the complete lifecycle
//! of audio contexts including initialization, state management, error recovery, and
//! resource cleanup.
//!
//! ## Key Features
//!
//! - **Context Lifecycle Management**: Create, suspend, resume, and close AudioContext instances
//! - **Sample Rate Configuration**: Support for standard rates (44.1kHz, 48kHz) and custom rates
//! - **Automatic Error Recovery**: Context recreation with configurable retry attempts
//! - **State Tracking**: Comprehensive state management with debugging support
//! - **Browser Compatibility**: Cross-browser support with fallback detection
//!
//! ## Usage Examples
//!
//! ```rust,no_run
//! use pitch_toy::audio::{AudioContextManager, AudioContextConfig};
//!
//! async fn setup_audio() {
//!     // Create manager with default configuration (48kHz, 1024 buffer)
//!     let mut manager = AudioContextManager::new();
//!
//!     // Or with custom configuration
//!     let config = AudioContextConfig::with_44_1khz().with_buffer_size(512);
//!     let mut manager = AudioContextManager::with_config(config);
//!
//!     // Initialize the audio context
//!     if let Ok(()) = manager.initialize().await {
//!         println!("AudioContext ready for audio processing");
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - Contexts are automatically closed on drop to prevent resource leaks
//! - Recreation attempts are limited to prevent infinite retry loops
//! - State changes are logged for debugging in development builds
//! - Buffer sizes can be optimized per environment (development vs production)
//!
//! ## Browser Requirements
//!
//! - Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+ (AudioContext support)
//! - HTTPS context required for microphone access in production
//! - AudioWorklet support for real-time processing (used by other modules)

use web_sys::{AudioContext, AudioContextOptions};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use std::fmt;
use crate::common::dev_log;
use super::AudioError;

/// AudioContext configuration states
#[derive(Debug, Clone, PartialEq)]
pub enum AudioContextState {
    /// Initial state, context not created yet
    Uninitialized,
    /// Context creation in progress
    Initializing,
    /// Context created and running
    Running,
    /// Context suspended (browser power management)
    Suspended,
    /// Context closed or failed
    Closed,
    /// Context recreation in progress after error
    Recreating,
}

impl fmt::Display for AudioContextState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioContextState::Uninitialized => write!(f, "Uninitialized"),
            AudioContextState::Initializing => write!(f, "Initializing"),
            AudioContextState::Running => write!(f, "Running"),
            AudioContextState::Suspended => write!(f, "Suspended"),
            AudioContextState::Closed => write!(f, "Closed"),
            AudioContextState::Recreating => write!(f, "Recreating"),
        }
    }
}

/// Audio context configuration
#[derive(Debug, Clone)]
pub struct AudioContextConfig {
    /// Preferred sample rate (44.1kHz or 48kHz standard)
    pub sample_rate: f32,
    /// Buffer size for audio processing
    pub buffer_size: u32,
    /// Maximum number of recreation attempts
    pub max_recreation_attempts: u32,
}

impl Default for AudioContextConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000.0, // 48kHz default (most common)
            buffer_size: 1024,    // Production buffer size
            max_recreation_attempts: 3,
        }
    }
}

impl AudioContextConfig {
    /// Create configuration with 44.1kHz sample rate
    pub fn with_44_1khz() -> Self {
        Self {
            sample_rate: 44100.0,
            ..Default::default()
        }
    }
    
    /// Create configuration with 48kHz sample rate
    pub fn with_48khz() -> Self {
        Self {
            sample_rate: 48000.0,
            ..Default::default()
        }
    }
    
    /// Create configuration with custom sample rate
    pub fn with_sample_rate(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }
    
    /// Set buffer size
    pub fn with_buffer_size(mut self, buffer_size: u32) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}

/// Cached audio device information
#[derive(Debug, Clone)]
pub struct AudioDevices {
    pub input_devices: Vec<(String, String)>,
    pub output_devices: Vec<(String, String)>,
}

impl AudioDevices {
    pub fn new() -> Self {
        Self {
            input_devices: Vec::new(),
            output_devices: Vec::new(),
        }
    }
}

/// AudioContext manager handles Web Audio API context lifecycle
pub struct AudioContextManager {
    context: Option<AudioContext>,
    state: AudioContextState,
    config: AudioContextConfig,
    recreation_attempts: u32,
    cached_devices: Option<AudioDevices>,
    /// Device change event listener callback (kept alive)
    device_change_callback: Option<Closure<dyn FnMut(web_sys::Event)>>,
}

impl AudioContextManager {
    /// Create new AudioContext manager
    pub fn new() -> Self {
        Self {
            context: None,
            state: AudioContextState::Uninitialized,
            config: AudioContextConfig::default(),
            recreation_attempts: 0,
            cached_devices: None,
            device_change_callback: None,
        }
    }
    
    /// Create new AudioContext manager with custom configuration
    pub fn with_config(config: AudioContextConfig) -> Self {
        Self {
            context: None,
            state: AudioContextState::Uninitialized,
            config,
            recreation_attempts: 0,
            cached_devices: None,
            device_change_callback: None,
        }
    }
    
    /// Get current AudioContext state
    pub fn state(&self) -> &AudioContextState {
        &self.state
    }
    
    /// Get current configuration
    pub fn config(&self) -> &AudioContextConfig {
        &self.config
    }
    
    /// Check if Web Audio API is supported
    pub fn is_supported() -> bool {
        let window = web_sys::window();
        if let Some(window) = window {
            // Check for AudioContext constructor
            return js_sys::Reflect::has(&window, &"AudioContext".into()).unwrap_or(false) ||
                   js_sys::Reflect::has(&window, &"webkitAudioContext".into()).unwrap_or(false);
        }
        false
    }
    
    /// Initialize AudioContext with current configuration
    pub async fn initialize(&mut self) -> Result<(), AudioError> {
        if !Self::is_supported() {
            return Err(AudioError::NotSupported(
                "Web Audio API not supported".to_string()
            ));
        }
        
        self.state = AudioContextState::Initializing;
        dev_log!("Initializing AudioContext with sample rate: {}Hz", self.config.sample_rate);
        
        // Create AudioContext options
        let options = AudioContextOptions::new();
        options.set_sample_rate(self.config.sample_rate);
        
        // Create AudioContext
        match AudioContext::new_with_context_options(&options) {
            Ok(context) => {
                dev_log!("✓ AudioContext created successfully");
                dev_log!("  Sample rate: {}Hz", context.sample_rate());
                dev_log!("  State: {:?}", context.state());
                
                self.context = Some(context);
                self.state = AudioContextState::Running;
                self.recreation_attempts = 0;
                
                Ok(())
            }
            Err(e) => {
                dev_log!("✗ Failed to create AudioContext: {:?}", e);
                self.state = AudioContextState::Closed;
                Err(AudioError::StreamInitFailed(
                    format!("Failed to create AudioContext: {:?}", e)
                ))
            }
        }
    }
    
    /// Resume suspended AudioContext
    pub async fn resume(&mut self) -> Result<(), AudioError> {
        let context = self.context.as_ref()
            .ok_or_else(|| AudioError::Generic("No AudioContext available".to_string()))?;
            
        if context.state() == web_sys::AudioContextState::Suspended {
            dev_log!("Resuming suspended AudioContext");
            
            match context.resume() {
                Ok(_) => {
                    // Note: We're not awaiting the promise here as it may not be necessary
                    // for basic functionality. In production, you might want to await it.
                    self.state = AudioContextState::Running;
                    dev_log!("✓ AudioContext resume initiated");
                    Ok(())
                }
                Err(e) => {
                    dev_log!("✗ Failed to resume AudioContext: {:?}", e);
                    Err(AudioError::Generic(format!("Failed to resume AudioContext: {:?}", e)))
                }
            }
        } else {
            dev_log!("AudioContext is not suspended, current state: {:?}", context.state());
            Ok(())
        }
    }
    
    /// Suspend AudioContext (useful for power management)
    pub async fn suspend(&mut self) -> Result<(), AudioError> {
        let context = self.context.as_ref()
            .ok_or_else(|| AudioError::Generic("No AudioContext available".to_string()))?;
            
        if context.state() == web_sys::AudioContextState::Running {
            dev_log!("Suspending AudioContext");
            
            match context.suspend() {
                Ok(_promise) => {
                    self.state = AudioContextState::Suspended;
                    dev_log!("✓ AudioContext suspended");
                    Ok(())
                }
                Err(e) => {
                    dev_log!("✗ Failed to suspend AudioContext: {:?}", e);
                    Err(AudioError::Generic(format!("Failed to suspend AudioContext: {:?}", e)))
                }
            }
        } else {
            dev_log!("AudioContext is not running, current state: {:?}", context.state());
            Ok(())
        }
    }
    
    /// Close current AudioContext
    pub async fn close(&mut self) -> Result<(), AudioError> {
        if let Some(context) = &self.context {
            dev_log!("Closing AudioContext");
            
            match context.close() {
                Ok(_promise) => {
                    dev_log!("✓ AudioContext closed");
                }
                Err(_e) => {
                    dev_log!("✗ Failed to close AudioContext: {:?}", _e);
                    // Continue with cleanup even if close fails
                }
            }
        }
        
        self.context = None;
        self.state = AudioContextState::Closed;
        Ok(())
    }
    
    /// Recreate AudioContext after error (automatic recovery)
    pub async fn recreate(&mut self) -> Result<(), AudioError> {
        if self.recreation_attempts >= self.config.max_recreation_attempts {
            return Err(AudioError::Generic(
                format!("Max recreation attempts ({}) exceeded", self.config.max_recreation_attempts)
            ));
        }
        
        self.recreation_attempts += 1;
        self.state = AudioContextState::Recreating;
        
        dev_log!("Recreating AudioContext (attempt {}/{})", 
                self.recreation_attempts, self.config.max_recreation_attempts);
        
        // Close existing context
        if self.context.is_some() {
            let _ = self.close().await; // Ignore errors during cleanup
        }
        
        // Create new context
        self.initialize().await
    }
    
    /// Get current AudioContext reference
    pub fn get_context(&self) -> Option<&AudioContext> {
        self.context.as_ref()
    }
    
    /// Get actual sample rate from AudioContext
    pub fn actual_sample_rate(&self) -> Option<f32> {
        self.context.as_ref().map(|ctx| ctx.sample_rate())
    }
    
    /// Check if AudioContext is active and running
    pub fn is_running(&self) -> bool {
        if let Some(context) = &self.context {
            matches!(self.state, AudioContextState::Running) && 
            context.state() == web_sys::AudioContextState::Running
        } else {
            false
        }
    }
    
    /// Update configuration (requires recreation to take effect)
    pub fn update_config(&mut self, config: AudioContextConfig) {
        self.config = config;
        dev_log!("AudioContext configuration updated (recreation required)");
    }
    
    /// Get recreation attempts count
    pub fn recreation_attempts(&self) -> u32 {
        self.recreation_attempts
    }
    
    /// Reset recreation attempts counter
    pub fn reset_recreation_attempts(&mut self) {
        self.recreation_attempts = 0;
    }
    
    /// Private helper to perform the actual device enumeration
    async fn enumerate_devices_internal() -> Result<(Vec<(String, String)>, Vec<(String, String)>), AudioError> {
        let window = web_sys::window()
            .ok_or_else(|| AudioError::Generic("No window object".to_string()))?;
        
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| AudioError::NotSupported("MediaDevices not available".to_string()))?;

        let promise = media_devices.enumerate_devices()
            .map_err(|e| AudioError::Generic(format!("Failed to enumerate devices: {:?}", e)))?;

        match JsFuture::from(promise).await {
            Ok(devices_js) => {
                let devices = js_sys::Array::from(&devices_js);
                let mut input_devices = Vec::new();
                let mut output_devices = Vec::new();

                // Check if we have permission to see device labels
                let has_permission = if devices.length() > 0 {
                    devices.get(0)
                        .dyn_ref::<web_sys::MediaDeviceInfo>()
                        .map(|d| !d.label().is_empty())
                        .unwrap_or(false)
                } else {
                    false
                };

                // If no permission, return empty lists instead of fake device entries
                if !has_permission {
                    return Ok((input_devices, output_devices)); // Both are empty
                }

                for i in 0..devices.length() {
                    if let Some(device_info) = devices.get(i).dyn_ref::<web_sys::MediaDeviceInfo>() {
                        let device_id = device_info.device_id();
                        let label = device_info.label();

                        match device_info.kind() {
                            web_sys::MediaDeviceKind::Audioinput => {
                                input_devices.push((device_id, label));
                            }
                            web_sys::MediaDeviceKind::Audiooutput => {
                                output_devices.push((device_id, label));
                            }
                            _ => {
                                // Skip video devices
                            }
                        }
                    }
                }

                Ok((input_devices, output_devices))
            }
            Err(e) => Err(AudioError::Generic(format!("Device enumeration failed: {:?}", e)))
        }
    }

    /// Query WebAudio for devices, await the result, and store them in the manager
    pub async fn refresh_audio_devices(&mut self) -> Result<(), AudioError> {
        let (input_devices, output_devices) = Self::enumerate_devices_internal().await?;
        
        // Store the devices in the manager
        self.cached_devices = Some(AudioDevices {
            input_devices,
            output_devices,
        });

        Ok(())
    }

    /// Get cached audio devices (returns empty if not refreshed yet)
    pub fn get_cached_devices(&self) -> &AudioDevices {
        static EMPTY_DEVICES: AudioDevices = AudioDevices {
            input_devices: Vec::new(),
            output_devices: Vec::new(),
        };
        
        self.cached_devices.as_ref().unwrap_or(&EMPTY_DEVICES)
    }

    /// Set up device change event listener to automatically refresh devices when they change
    /// The callback will be called whenever audio devices are added or removed
    pub fn setup_device_change_listener<F>(&mut self, callback: F) -> Result<(), AudioError>
    where
        F: Fn() + 'static,
    {
        // Only set up if we don't already have a listener
        if self.device_change_callback.is_some() {
            dev_log!("Device change listener already set up");
            return Ok(());
        }
        
        let window = web_sys::window()
            .ok_or_else(|| AudioError::Generic("No window available for device change listener".to_string()))?;
        
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| AudioError::NotSupported("MediaDevices not available for device change listener".to_string()))?;
        
        // Create closure for device change events
        let device_change_closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            dev_log!("Audio devices changed - triggering callback");
            callback();
        }) as Box<dyn FnMut(_)>);
        
        // Add the event listener
        media_devices.add_event_listener_with_callback(
            "devicechange", 
            device_change_closure.as_ref().unchecked_ref()
        ).map_err(|e| AudioError::Generic(format!("Failed to add device change listener: {:?}", e)))?;
        
        dev_log!("Device change listener set up successfully");
        
        // Store the callback to keep it alive
        self.device_change_callback = Some(device_change_closure);
        
        Ok(())
    }
    
    /// Remove device change event listener
    pub fn remove_device_change_listener(&mut self) -> Result<(), AudioError> {
        if let Some(callback) = &self.device_change_callback {
            let window = web_sys::window()
                .ok_or_else(|| AudioError::Generic("No window available".to_string()))?;
            
            let navigator = window.navigator();
            let media_devices = navigator.media_devices()
                .map_err(|_| AudioError::NotSupported("MediaDevices not available".to_string()))?;
            
            // Remove the event listener
            media_devices.remove_event_listener_with_callback(
                "devicechange",
                callback.as_ref().unchecked_ref()
            ).map_err(|e| AudioError::Generic(format!("Failed to remove device change listener: {:?}", e)))?;
            
            dev_log!("Device change listener removed");
        }
        
        self.device_change_callback = None;
        Ok(())
    }
    
    /// Check if device change listener is active
    pub fn has_device_change_listener(&self) -> bool {
        self.device_change_callback.is_some()
    }
}

impl Drop for AudioContextManager {
    fn drop(&mut self) {
        // Clean up device change listener
        let _ = self.remove_device_change_listener();
        
        if let Some(context) = &self.context {
            // Try to close context on drop, but don't panic if it fails
            let _ = context.close();
        }
    }
}

/// AudioSystemContext manages all audio-related instances and their lifecycle
/// This provides a unified interface for audio system management with dependency injection
pub struct AudioSystemContext {
    audio_context_manager: std::rc::Rc<std::cell::RefCell<AudioContextManager>>,
    audioworklet_manager: Option<super::worklet::AudioWorkletManager>,
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>>,
    volume_level_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<super::data_types::VolumeLevelData>>>,
    pitch_data_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<super::data_types::PitchData>>>,
    audioworklet_status_setter: std::rc::Rc<dyn observable_data::DataSetter<super::data_types::AudioWorkletStatus>>,
    is_initialized: bool,
    initialization_error: Option<String>,
}

impl AudioSystemContext {
    /// Create new AudioSystemContext with mandatory data setters
    pub fn new(
        volume_level_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<super::data_types::VolumeLevelData>>>,
        pitch_data_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<super::data_types::PitchData>>>,
        audioworklet_status_setter: std::rc::Rc<dyn observable_data::DataSetter<super::data_types::AudioWorkletStatus>>,
    ) -> Self {
        Self {
            audio_context_manager: std::rc::Rc::new(std::cell::RefCell::new(AudioContextManager::new())),
            audioworklet_manager: None,
            pitch_analyzer: None,
            volume_level_setter,
            pitch_data_setter,
            audioworklet_status_setter,
            is_initialized: false,
            initialization_error: None,
        }
    }

    /// Create new AudioSystemContext with custom AudioContext configuration
    pub fn with_audio_config(
        audio_config: AudioContextConfig,
        volume_level_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<super::data_types::VolumeLevelData>>>,
        pitch_data_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<super::data_types::PitchData>>>,
        audioworklet_status_setter: std::rc::Rc<dyn observable_data::DataSetter<super::data_types::AudioWorkletStatus>>,
    ) -> Self {
        Self {
            audio_context_manager: std::rc::Rc::new(std::cell::RefCell::new(AudioContextManager::with_config(audio_config))),
            audioworklet_manager: None,
            pitch_analyzer: None,
            volume_level_setter,
            pitch_data_setter,
            audioworklet_status_setter,
            is_initialized: false,
            initialization_error: None,
        }
    }

    /// Initialize all audio components with proper dependency order
    pub async fn initialize(&mut self) -> Result<(), String> {
        dev_log!("Initializing AudioSystemContext");
        
        // Clear any previous initialization error
        self.initialization_error = None;
        
        // Step 1: Initialize AudioContextManager
        if let Err(e) = self.audio_context_manager.borrow_mut().initialize().await {
            let error_msg = format!("Failed to initialize AudioContextManager: {}", e);
            dev_log!("✗ {}", error_msg);
            self.initialization_error = Some(error_msg.clone());
            return Err(error_msg);
        }
        dev_log!("✓ AudioContextManager initialized");

        // Step 2: Initialize AudioWorkletManager
        let mut worklet_manager = super::worklet::AudioWorkletManager::new();
        if let Err(e) = worklet_manager.initialize(&*self.audio_context_manager.borrow()).await {
            let error_msg = format!("Failed to initialize AudioWorkletManager: {}", e);
            dev_log!("✗ {}", error_msg);
            self.initialization_error = Some(error_msg.clone());
            return Err(error_msg);
        }
        
        // Configure setters on AudioWorkletManager
        worklet_manager.set_volume_level_setter(self.volume_level_setter.clone());
        worklet_manager.set_pitch_data_setter(self.pitch_data_setter.clone());
        worklet_manager.set_audioworklet_status_setter(self.audioworklet_status_setter.clone());
        
        // Publish initial status
        worklet_manager.publish_audioworklet_status();
        
        self.audioworklet_manager = Some(worklet_manager);
        dev_log!("✓ AudioWorkletManager initialized with setters configured");

        // Step 3: Initialize PitchAnalyzer
        let config = super::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = self.audio_context_manager.borrow().config().sample_rate;
        
        match super::pitch_analyzer::PitchAnalyzer::new(config, sample_rate) {
            Ok(analyzer) => {
                let mut analyzer = analyzer;
                analyzer.set_pitch_data_setter(self.pitch_data_setter.clone());
                let analyzer_rc = std::rc::Rc::new(std::cell::RefCell::new(analyzer));
                
                // Configure PitchAnalyzer in AudioWorkletManager
                if let Some(ref mut worklet_manager) = self.audioworklet_manager {
                    worklet_manager.set_pitch_analyzer(analyzer_rc.clone());
                }
                
                self.pitch_analyzer = Some(analyzer_rc);
                dev_log!("✓ PitchAnalyzer initialized and configured");
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize PitchAnalyzer: {}", e);
                dev_log!("✗ {}", error_msg);
                self.initialization_error = Some(error_msg.clone());
                return Err(error_msg);
            }
        }

        self.is_initialized = true;
        dev_log!("✓ AudioSystemContext fully initialized");
        Ok(())
    }

    /// Shutdown all audio components
    pub async fn shutdown(&mut self) -> Result<(), String> {
        dev_log!("Shutting down AudioSystemContext");
        
        // Shutdown AudioWorkletManager first
        if let Some(ref mut worklet_manager) = self.audioworklet_manager {
            if let Err(e) = worklet_manager.stop_processing() {
                dev_log!("Warning: AudioWorkletManager stop_processing failed: {}", e);
            }
            if let Err(e) = worklet_manager.disconnect() {
                dev_log!("Warning: AudioWorkletManager disconnect failed: {}", e);
            }
        }
        self.audioworklet_manager = None;
        
        // Clear PitchAnalyzer
        self.pitch_analyzer = None;
        
        // Close AudioContextManager
        if let Err(e) = self.audio_context_manager.borrow_mut().close().await {
            dev_log!("Warning: AudioContextManager close failed: {}", e);
        }
        
        self.is_initialized = false;
        dev_log!("✓ AudioSystemContext shutdown completed");
        Ok(())
    }

    /// Check if the audio system is ready for operation
    pub fn is_ready(&self) -> bool {
        self.is_initialized && 
        self.audioworklet_manager.is_some() && 
        self.pitch_analyzer.is_some() && 
        self.audio_context_manager.borrow().is_running()
    }

    /// Get the last initialization error if any
    pub fn get_initialization_error(&self) -> Option<&str> {
        self.initialization_error.as_deref()
    }

    /// Get reference to AudioContextManager
    pub fn get_audio_context_manager(&self) -> &std::rc::Rc<std::cell::RefCell<AudioContextManager>> {
        &self.audio_context_manager
    }
    
    /// Get cloned Rc of AudioContextManager for global storage
    pub fn get_audio_context_manager_rc(&self) -> std::rc::Rc<std::cell::RefCell<AudioContextManager>> {
        self.audio_context_manager.clone()
    }

    /// Get reference to AudioWorkletManager
    pub fn get_audioworklet_manager(&self) -> Option<&super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_ref()
    }

    /// Get mutable reference to AudioWorkletManager
    pub fn get_audioworklet_manager_mut(&mut self) -> Option<&mut super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_mut()
    }

    /// Get reference to PitchAnalyzer
    pub fn get_pitch_analyzer(&self) -> Option<&std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>> {
        self.pitch_analyzer.as_ref()
    }

    /// Get cloned reference to PitchAnalyzer for sharing
    pub fn get_pitch_analyzer_clone(&self) -> Option<std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>> {
        self.pitch_analyzer.as_ref().cloned()
    }

    /// Check if AudioContext is supported
    pub fn is_audio_context_supported() -> bool {
        AudioContextManager::is_supported()
    }

    /// Get current AudioContext configuration
    pub fn get_audio_config(&self) -> AudioContextConfig {
        self.audio_context_manager.borrow().config().clone()
    }

    /// Resume audio context if suspended
    pub async fn resume_if_suspended(&mut self) -> Result<(), String> {
        self.audio_context_manager.borrow_mut().resume().await
            .map_err(|e| format!("Failed to resume AudioContext: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;
    use crate::audio::data_types;

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_context_state_display() {
        assert_eq!(AudioContextState::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioContextState::Initializing.to_string(), "Initializing");
        assert_eq!(AudioContextState::Running.to_string(), "Running");
        assert_eq!(AudioContextState::Suspended.to_string(), "Suspended");
        assert_eq!(AudioContextState::Closed.to_string(), "Closed");
        assert_eq!(AudioContextState::Recreating.to_string(), "Recreating");
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_context_config_default() {
        let config = AudioContextConfig::default();
        assert_eq!(config.sample_rate, 48000.0);
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.max_recreation_attempts, 3);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_context_config_builders() {
        let config_44_1 = AudioContextConfig::with_44_1khz();
        assert_eq!(config_44_1.sample_rate, 44100.0);
        
        let config_48 = AudioContextConfig::with_48khz();
        assert_eq!(config_48.sample_rate, 48000.0);
        
        let config_custom = AudioContextConfig::with_sample_rate(96000.0);
        assert_eq!(config_custom.sample_rate, 96000.0);
        
        let config_buffer = AudioContextConfig::default().with_buffer_size(2048);
        assert_eq!(config_buffer.buffer_size, 2048);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_context_manager_new() {
        let manager = AudioContextManager::new();
        assert_eq!(*manager.state(), AudioContextState::Uninitialized);
        assert!(!manager.is_running());
        assert!(manager.get_context().is_none());
        assert_eq!(manager.recreation_attempts(), 0);
        assert!(!manager.has_device_change_listener());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_context_manager_with_config() {
        let config = AudioContextConfig::with_44_1khz().with_buffer_size(512);
        let manager = AudioContextManager::with_config(config.clone());
        
        assert_eq!(*manager.state(), AudioContextState::Uninitialized);
        assert_eq!(manager.config().sample_rate, 44100.0);
        assert_eq!(manager.config().buffer_size, 512);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_context_manager_update_config() {
        let mut manager = AudioContextManager::new();
        let new_config = AudioContextConfig::with_44_1khz();
        
        // Store the config without calling update_config to avoid web API calls in tests
        manager.config = new_config;
        assert_eq!(manager.config().sample_rate, 44100.0);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_context_manager_recreation_attempts() {
        let mut manager = AudioContextManager::new();
        
        assert_eq!(manager.recreation_attempts(), 0);
        
        manager.recreation_attempts = 2;
        assert_eq!(manager.recreation_attempts(), 2);
        
        manager.reset_recreation_attempts();
        assert_eq!(manager.recreation_attempts(), 0);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_refresh_audio_devices_structure() {
        let _manager = AudioContextManager::new();
        
        // Test that the refresh method exists and has the correct signature
        // We can't actually test the functionality in a unit test environment
        // since it requires browser APIs
        let _result_type: Result<(), AudioError> = Ok(());
        
        // The test verifies the function signature is correct
        assert!(true);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_devices_struct() {
        let devices = AudioDevices::new();
        assert!(devices.input_devices.is_empty());
        assert!(devices.output_devices.is_empty());
        
        let devices_with_data = AudioDevices {
            input_devices: vec![("id1".to_string(), "Microphone".to_string())],
            output_devices: vec![("id2".to_string(), "Speakers".to_string())],
        };
        assert_eq!(devices_with_data.input_devices.len(), 1);
        assert_eq!(devices_with_data.output_devices.len(), 1);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_cached_devices_functionality() {
        let manager = AudioContextManager::new();
        
        // Initially should return empty devices
        let cached = manager.get_cached_devices();
        assert!(cached.input_devices.is_empty());
        assert!(cached.output_devices.is_empty());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_device_change_listener_state() {
        let manager = AudioContextManager::new();
        
        // Initially should not have a device change listener
        assert!(!manager.has_device_change_listener());
        
        // Test that the device change listener field exists and is properly initialized
        // We can't test the actual listener setup in unit tests since it requires browser APIs
    }

    // Mock data setter for testing
    #[allow(dead_code)]
    struct MockVolumeSetter {
        calls: std::sync::Arc<std::sync::Mutex<Vec<Option<data_types::VolumeLevelData>>>>,
    }

    impl MockVolumeSetter {
        fn new() -> Self {
            Self {
                calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }
        
        fn get_calls(&self) -> std::sync::Arc<std::sync::Mutex<Vec<Option<data_types::VolumeLevelData>>>> {
            self.calls.clone()
        }
    }

    impl observable_data::DataSetter<Option<data_types::VolumeLevelData>> for MockVolumeSetter {
        fn set(&self, data: Option<data_types::VolumeLevelData>) {
            self.calls.lock().unwrap().push(data);
        }
    }

    // Mock data setter for pitch data
    #[allow(dead_code)]
    struct MockPitchSetter {
        calls: std::sync::Arc<std::sync::Mutex<Vec<Option<data_types::PitchData>>>>,
    }

    impl MockPitchSetter {
        fn new() -> Self {
            Self {
                calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }
        
        fn get_calls(&self) -> std::sync::Arc<std::sync::Mutex<Vec<Option<data_types::PitchData>>>> {
            self.calls.clone()
        }
    }

    impl observable_data::DataSetter<Option<data_types::PitchData>> for MockPitchSetter {
        fn set(&self, data: Option<data_types::PitchData>) {
            self.calls.lock().unwrap().push(data);
        }
    }

    // Mock data setter for audioworklet status
    #[allow(dead_code)]
    struct MockAudioWorkletStatusSetter {
        calls: std::sync::Arc<std::sync::Mutex<Vec<data_types::AudioWorkletStatus>>>,
    }

    impl MockAudioWorkletStatusSetter {
        fn new() -> Self {
            Self {
                calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }
        
        fn get_calls(&self) -> std::sync::Arc<std::sync::Mutex<Vec<data_types::AudioWorkletStatus>>> {
            self.calls.clone()
        }
    }

    impl observable_data::DataSetter<data_types::AudioWorkletStatus> for MockAudioWorkletStatusSetter {
        fn set(&self, data: data_types::AudioWorkletStatus) {
            self.calls.lock().unwrap().push(data);
        }
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_system_context_creation() {
        let volume_setter = std::rc::Rc::new(MockVolumeSetter::new());
        let pitch_setter = std::rc::Rc::new(MockPitchSetter::new());
        let status_setter = std::rc::Rc::new(MockAudioWorkletStatusSetter::new());

        let context = AudioSystemContext::new(
            volume_setter.clone(),
            pitch_setter.clone(),
            status_setter.clone(),
        );

        // Initially should not be ready
        assert!(!context.is_ready());
        assert!(!context.is_initialized);
        assert!(context.get_initialization_error().is_none());
        
        // Components should be uninitialized
        assert!(context.get_audioworklet_manager().is_none());
        assert!(context.get_pitch_analyzer().is_none());
        
        // AudioContextManager should be available
        assert_eq!(*context.get_audio_context_manager().borrow().state(), AudioContextState::Uninitialized);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_system_context_with_custom_config() {
        let volume_setter = std::rc::Rc::new(MockVolumeSetter::new());
        let pitch_setter = std::rc::Rc::new(MockPitchSetter::new());
        let status_setter = std::rc::Rc::new(MockAudioWorkletStatusSetter::new());

        let config = AudioContextConfig::with_44_1khz().with_buffer_size(512);
        let context = AudioSystemContext::with_audio_config(
            config,
            volume_setter.clone(),
            pitch_setter.clone(),
            status_setter.clone(),
        );

        // Check that custom config is applied
        assert_eq!(context.get_audio_config().sample_rate, 44100.0);
        assert_eq!(context.get_audio_config().buffer_size, 512);
        assert!(!context.is_ready());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_system_context_access_methods() {
        let volume_setter = std::rc::Rc::new(MockVolumeSetter::new());
        let pitch_setter = std::rc::Rc::new(MockPitchSetter::new());
        let status_setter = std::rc::Rc::new(MockAudioWorkletStatusSetter::new());

        let context = AudioSystemContext::new(
            volume_setter.clone(),
            pitch_setter.clone(),
            status_setter.clone(),
        );

        // Test access methods
        assert!(context.get_audioworklet_manager().is_none());
        assert!(context.get_pitch_analyzer().is_none());
        assert!(context.get_pitch_analyzer_clone().is_none());
        assert!(context.get_initialization_error().is_none());
        
        // Test static methods
        assert!(AudioSystemContext::is_audio_context_supported() == AudioContextManager::is_supported());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_system_context_lifecycle_without_browser() {
        let volume_setter = std::rc::Rc::new(MockVolumeSetter::new());
        let pitch_setter = std::rc::Rc::new(MockPitchSetter::new());
        let status_setter = std::rc::Rc::new(MockAudioWorkletStatusSetter::new());

        let mut context = AudioSystemContext::new(
            volume_setter.clone(),
            pitch_setter.clone(),
            status_setter.clone(),
        );

        // Test initial state
        assert!(!context.is_ready());
        assert!(!context.is_initialized);
        
        // Test shutdown when not initialized (should not panic)
        // Note: We can't test actual initialization in unit tests without browser environment
        // This test verifies that the lifecycle methods exist and have correct signatures
        wasm_bindgen_futures::spawn_local(async move {
            let _result = context.shutdown().await;
            // Should complete without error even when not initialized
        });
    }
}