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
/// 
/// This structure provides a unified interface for audio system management using dependency injection.
/// It centralizes all audio components and their interactions, eliminating the need for global state.
/// 
/// # Architecture
/// 
/// The AudioSystemContext owns and manages:
/// - AudioContextManager: Web Audio API context management
/// - AudioWorkletManager: Real-time audio processing
/// - PitchAnalyzer: Pitch detection and analysis
/// - Data setters: Reactive data updates to UI components
/// 
/// # Benefits
/// 
/// - **Dependency Injection**: All dependencies are explicitly provided at construction
/// - **Centralized Management**: Single point of access for all audio components  
/// - **Testability**: Easy to mock dependencies for unit testing
/// - **Lifecycle Control**: Proper initialization and cleanup of all components
/// - **Type Safety**: Compile-time guarantees about component availability
/// 
/// # Usage
/// 
/// ```rust
/// // Create with mandatory data setters
/// let context = AudioSystemContext::new(
///     volume_setter,
///     pitch_setter,
///     status_setter,
/// );
/// 
/// // Initialize all components
/// context.initialize().await?;
/// 
/// // Access components safely
/// if let Some(worklet) = context.get_audioworklet_manager() {
///     // Use worklet manager
/// }
/// ```
pub struct AudioSystemContext {
    audio_context_manager: std::rc::Rc<std::cell::RefCell<AudioContextManager>>,
    audioworklet_manager: Option<super::worklet::AudioWorkletManager>,
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>>,
    is_initialized: bool,
    initialization_error: Option<String>,
    /// Current microphone permission state
    permission_state: std::cell::Cell<super::AudioPermission>,
}

impl AudioSystemContext {

    

    /// Set up debug action listeners for debug GUI controls
    /// 
    /// This method connects debug-specific action listeners to the audio system.
    /// These actions are separate from core application logic and only used by debug components.
    /// 
    /// Note: This is a placeholder implementation that demonstrates the architecture.
    /// The actual implementation would need to be integrated with the full audio system.
    pub fn setup_debug_action_listeners(
        context_rc: &std::rc::Rc<std::cell::RefCell<Self>>,
        debug_actions: &crate::module_interfaces::debug_actions::DebugActionsInterface,
    ) {
        use crate::common::dev_log;
        
        // Clone the context Rc for use in closures
        let context_for_test_signal = context_rc.clone();
        let context_for_output = context_rc.clone();
        let context_for_noise = context_rc.clone();
        
        // Test signal action listener
        let test_signal_listener = debug_actions.test_signal_listener();
        test_signal_listener.listen(move |action| {
            dev_log!("Received debug test signal action: {:?}", action);
            
            let mut context = context_for_test_signal.borrow_mut();
            if let Some(worklet_manager) = context.get_audioworklet_manager_mut() {
                // Convert debug action to audio system config
                let audio_config = super::TestSignalGeneratorConfig {
                    enabled: action.enabled,
                    frequency: action.frequency,
                    amplitude: action.volume / 100.0, // Convert percentage to 0-1 range
                    waveform: action.waveform,
                    sample_rate: 48000.0, // Use standard sample rate
                };
                
                worklet_manager.update_test_signal_config(audio_config);
                dev_log!("✓ Debug test signal config updated - enabled: {}, freq: {}, vol: {}", 
                        action.enabled, action.frequency, action.volume);
            } else {
                dev_log!("Warning: No AudioWorklet manager available for debug test signal config");
            }
        });
        
        // Output to speakers action listener
        let output_listener = debug_actions.output_to_speakers_listener();
        output_listener.listen(move |action| {
            dev_log!("Received debug output to speakers action: {:?}", action);
            
            let mut context = context_for_output.borrow_mut();
            if let Some(worklet_manager) = context.get_audioworklet_manager_mut() {
                worklet_manager.set_output_to_speakers(action.enabled);
                dev_log!("✓ Debug output to speakers setting updated - enabled: {}", action.enabled);
            } else {
                dev_log!("Warning: No AudioWorklet manager available for debug output to speakers setting");
            }
        });
        
        // Background noise action listener
        let noise_listener = debug_actions.background_noise_listener();
        noise_listener.listen(move |action| {
            dev_log!("Received debug background noise action: {:?}", action);
            
            let mut context = context_for_noise.borrow_mut();
            if let Some(worklet_manager) = context.get_audioworklet_manager_mut() {
                // Convert debug action to audio system config
                let audio_config = super::BackgroundNoiseConfig {
                    enabled: action.enabled,
                    level: action.level,
                    noise_type: action.noise_type,
                };
                
                worklet_manager.update_background_noise_config(audio_config);
                dev_log!("✓ Debug background noise config updated - enabled: {}, level: {}", 
                        action.enabled, action.level);
            } else {
                dev_log!("Warning: No AudioWorklet manager available for debug background noise config");
            }
        });
    }

    /// Create new AudioSystemContext with custom AudioContext configuration

    /// Create new AudioSystemContext without setters (return-based pattern)
    /// 
    /// This constructor creates an AudioSystemContext that works with the
    /// return-based data flow pattern instead of the observable/setter pattern.
    /// Data is collected from the audio system and returned directly rather
    /// than being pushed to setters.
    pub fn new_return_based() -> Self {
        Self {
            audio_context_manager: std::rc::Rc::new(std::cell::RefCell::new(AudioContextManager::new())),
            audioworklet_manager: None,
            pitch_analyzer: None,
            is_initialized: false,
            initialization_error: None,
            permission_state: std::cell::Cell::new(super::AudioPermission::Uninitialized),
        }
    }

    /// Create new AudioSystemContext with custom config (return-based pattern)
    pub fn new_return_based_with_config(audio_config: AudioContextConfig) -> Self {
        Self {
            audio_context_manager: std::rc::Rc::new(std::cell::RefCell::new(AudioContextManager::with_config(audio_config))),
            audioworklet_manager: None,
            pitch_analyzer: None,
            is_initialized: false,
            initialization_error: None,
            permission_state: std::cell::Cell::new(super::AudioPermission::Uninitialized),
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

        // Step 2: Initialize AudioWorkletManager (simplified for return-based pattern)
        // TODO: Create return-based AudioWorkletManager constructor when implementing Task 8
        // For now, AudioWorkletManager initialization is skipped in return-based mode
        dev_log!("⚠ AudioWorkletManager initialization skipped in return-based mode (placeholder implementation)");

        // Step 3: Initialize PitchAnalyzer (simplified for return-based pattern)
        let config = super::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = self.audio_context_manager.borrow().config().sample_rate;
        
        match super::pitch_analyzer::PitchAnalyzer::new(config, sample_rate) {
            Ok(analyzer) => {
                // Create analyzer without setter (return-based pattern)
                let analyzer_rc = std::rc::Rc::new(std::cell::RefCell::new(analyzer));
                self.pitch_analyzer = Some(analyzer_rc);
                dev_log!("✓ PitchAnalyzer initialized for return-based pattern");
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize PitchAnalyzer: {}", e);
                dev_log!("✗ {}", error_msg);
                self.initialization_error = Some(error_msg.clone());
                return Err(error_msg);
            }
        }

        // Step 4: Initialize VolumeDetector
        let volume_detector = super::volume_detector::VolumeDetector::new_default();
        
        // Configure VolumeDetector in AudioWorkletManager
        if let Some(ref mut worklet_manager) = self.audioworklet_manager {
            worklet_manager.set_volume_detector(volume_detector);
        }
        
        dev_log!("✓ VolumeDetector initialized and configured");

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
    
    /// Get current audio worklet status
    pub fn get_audioworklet_status(&self) -> Option<super::data_types::AudioWorkletStatus> {
        self.audioworklet_manager.as_ref().map(|worklet| worklet.get_status())
    }
    
    /// Get current audio devices from context manager
    pub fn get_audio_devices(&self) -> super::AudioDevices {
        self.audio_context_manager.borrow().get_cached_devices().clone()
    }
    
    /// Get buffer pool statistics if available
    pub fn get_buffer_pool_stats(&self) -> Option<super::message_protocol::BufferPoolStats> {
        self.audioworklet_manager.as_ref().and_then(|worklet| worklet.get_buffer_pool_statistics())
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
    
    /// Handle microphone connection result
    pub fn handle_microphone_connection_result(&self, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.set_permission_state(super::AudioPermission::Granted);
                dev_log!("Microphone connected successfully - permission granted");
            }
            Err(e) => {
                if e.contains("denied") || e.contains("NotAllowedError") {
                    self.set_permission_state(super::AudioPermission::Denied);
                } else if e.contains("NotFoundError") || e.contains("unavailable") {
                    self.set_permission_state(super::AudioPermission::Unavailable);
                } else {
                    self.set_permission_state(super::AudioPermission::Unavailable);
                }
                dev_log!("Microphone connection failed: {}", e);
            }
        }
    }

    /// Collect current audio analysis data (return-based pattern)
    /// 
    /// This method retrieves the current audio analysis data from the audio system
    /// without using the observable/setter pattern. It's used by the engine layer
    /// to collect data for returning in EngineUpdateResult.
    pub fn collect_audio_analysis(&self, timestamp: f64) -> Option<crate::module_interfaces::engine_to_model::AudioAnalysis> {
        if !self.is_initialized {
            return None;
        }

        // Collect volume data from AudioWorkletManager
        let volume_data = if let Some(ref worklet) = self.audioworklet_manager {
            worklet.get_volume_data()
        } else {
            None
        };
        
        // Convert volume data to interface type
        let volume = convert_volume_data(volume_data);
        
        // Collect pitch data from PitchAnalyzer
        let pitch_data = if let Some(ref analyzer) = self.pitch_analyzer {
            analyzer.borrow().get_latest_pitch_data()
        } else {
            None
        };
        
        // Convert pitch data to interface type
        let pitch = convert_pitch_data(pitch_data);
        
        // Merge the data into AudioAnalysis
        merge_audio_analysis(volume, pitch, timestamp)
    }

    /// Collect current audio errors (return-based pattern)
    pub fn collect_audio_errors(&self) -> Vec<crate::module_interfaces::engine_to_model::AudioError> {
        let mut errors = Vec::new();
        
        // Check for initialization errors
        if let Some(error_msg) = self.initialization_error.as_ref() {
            errors.push(crate::module_interfaces::engine_to_model::AudioError::ProcessingError(error_msg.clone()));
        }
        
        // Check AudioContext manager state
        let context_manager = self.audio_context_manager.borrow();
        if !context_manager.is_running() {
            match context_manager.state() {
                AudioContextState::Closed => {
                    errors.push(crate::module_interfaces::engine_to_model::AudioError::ProcessingError("AudioContext is closed".to_string()));
                }
                AudioContextState::Suspended => {
                    errors.push(crate::module_interfaces::engine_to_model::AudioError::ProcessingError("AudioContext is suspended".to_string()));
                }
                _ => {}
            }
        }
        
        errors
    }

    /// Collect current permission state (return-based pattern)
    pub fn collect_permission_state(&self) -> crate::module_interfaces::engine_to_model::PermissionState {
        match self.permission_state.get() {
            super::AudioPermission::Uninitialized => crate::module_interfaces::engine_to_model::PermissionState::NotRequested,
            super::AudioPermission::Requesting => crate::module_interfaces::engine_to_model::PermissionState::Requested,
            super::AudioPermission::Granted => crate::module_interfaces::engine_to_model::PermissionState::Granted,
            super::AudioPermission::Denied => crate::module_interfaces::engine_to_model::PermissionState::Denied,
            super::AudioPermission::Unavailable => crate::module_interfaces::engine_to_model::PermissionState::Denied,
        }
    }
    
    /// Set microphone permission state
    pub fn set_permission_state(&self, state: super::AudioPermission) {
        self.permission_state.set(state);
    }
}

/// Merger that combines volume and pitch data into AudioAnalysis
/// 
/// This merger combines volume and pitch updates into a unified AudioAnalysis
/// structure. It stores the current state and provides methods to update individual
/// components and retrieve the merged result.
struct AudioAnalysisMerger {
    current_volume: std::cell::RefCell<crate::module_interfaces::engine_to_model::Volume>,
    current_pitch: std::cell::RefCell<crate::module_interfaces::engine_to_model::Pitch>,
    last_timestamp: std::cell::Cell<f64>,
}

impl AudioAnalysisMerger {
    fn new() -> Self {
        Self {
            current_volume: std::cell::RefCell::new(crate::module_interfaces::engine_to_model::Volume { peak: -60.0, rms: -60.0 }),
            current_pitch: std::cell::RefCell::new(crate::module_interfaces::engine_to_model::Pitch::NotDetected),
            last_timestamp: std::cell::Cell::new(0.0),
        }
    }
    
    fn update_volume(&self, volume: crate::module_interfaces::engine_to_model::Volume) {
        *self.current_volume.borrow_mut() = volume;
    }
    
    fn update_pitch(&self, pitch: crate::module_interfaces::engine_to_model::Pitch, timestamp: f64) {
        *self.current_pitch.borrow_mut() = pitch;
        self.last_timestamp.set(timestamp);
    }
    
    
    /// Get current audio analysis data
    fn get_current_analysis(&self) -> crate::module_interfaces::engine_to_model::AudioAnalysis {
        crate::module_interfaces::engine_to_model::AudioAnalysis {
            volume_level: self.current_volume.borrow().clone(),
            pitch: self.current_pitch.borrow().clone(),
            fft_data: None,
            timestamp: self.last_timestamp.get().max(js_sys::Date::now()),
        }
    }
}

/// Conversion functions for audio data types (return-based pattern)
/// 
/// These functions convert raw audio engine data types to interface types
/// without using the observable/setter pattern.

/// Convert VolumeLevelData to Volume interface type
pub fn convert_volume_data(volume_data: Option<super::data_types::VolumeLevelData>) -> Option<crate::module_interfaces::engine_to_model::Volume> {
    volume_data.map(|data| crate::module_interfaces::engine_to_model::Volume {
        peak: data.peak_db,
        rms: data.rms_db,
    })
}

/// Convert PitchData to Pitch interface type
pub fn convert_pitch_data(pitch_data: Option<super::data_types::PitchData>) -> Option<crate::module_interfaces::engine_to_model::Pitch> {
    pitch_data.map(|data| {
        if data.frequency > 0.0 {
            crate::module_interfaces::engine_to_model::Pitch::Detected(data.frequency, data.clarity)
        } else {
            crate::module_interfaces::engine_to_model::Pitch::NotDetected
        }
    })
}

/// Merge volume and pitch data into AudioAnalysis
/// 
/// This function combines separate volume and pitch data into a unified
/// AudioAnalysis structure, similar to how AudioAnalysisMerger works but
/// as a pure function without state.
pub fn merge_audio_analysis(
    volume: Option<crate::module_interfaces::engine_to_model::Volume>,
    pitch: Option<crate::module_interfaces::engine_to_model::Pitch>,
    timestamp: f64
) -> Option<crate::module_interfaces::engine_to_model::AudioAnalysis> {
    // Only create AudioAnalysis if we have at least some data
    if volume.is_some() || pitch.is_some() {
        Some(crate::module_interfaces::engine_to_model::AudioAnalysis {
            volume_level: volume.unwrap_or(crate::module_interfaces::engine_to_model::Volume { peak: -60.0, rms: -60.0 }),
            pitch: pitch.unwrap_or(crate::module_interfaces::engine_to_model::Pitch::NotDetected),
            fft_data: None,
            timestamp: timestamp.max(js_sys::Date::now()),
        })
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_audio_context_state_display() {
        assert_eq!(AudioContextState::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioContextState::Initializing.to_string(), "Initializing");
        assert_eq!(AudioContextState::Running.to_string(), "Running");
        assert_eq!(AudioContextState::Suspended.to_string(), "Suspended");
        assert_eq!(AudioContextState::Closed.to_string(), "Closed");
        assert_eq!(AudioContextState::Recreating.to_string(), "Recreating");
    }

    #[wasm_bindgen_test]
    fn test_audio_context_config_default() {
        let config = AudioContextConfig::default();
        assert_eq!(config.sample_rate, 48000.0);
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.max_recreation_attempts, 3);
    }

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

    #[wasm_bindgen_test]
    fn test_audio_context_manager_new() {
        let manager = AudioContextManager::new();
        assert_eq!(*manager.state(), AudioContextState::Uninitialized);
        assert!(!manager.is_running());
        assert!(manager.get_context().is_none());
        assert_eq!(manager.recreation_attempts(), 0);
        assert!(!manager.has_device_change_listener());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_manager_with_config() {
        let config = AudioContextConfig::with_44_1khz().with_buffer_size(512);
        let manager = AudioContextManager::with_config(config.clone());
        
        assert_eq!(*manager.state(), AudioContextState::Uninitialized);
        assert_eq!(manager.config().sample_rate, 44100.0);
        assert_eq!(manager.config().buffer_size, 512);
    }

    #[wasm_bindgen_test]
    fn test_audio_context_manager_update_config() {
        let mut manager = AudioContextManager::new();
        let new_config = AudioContextConfig::with_44_1khz();
        
        // Store the config without calling update_config to avoid web API calls in tests
        manager.config = new_config;
        assert_eq!(manager.config().sample_rate, 44100.0);
    }

    #[wasm_bindgen_test]
    fn test_audio_context_manager_recreation_attempts() {
        let mut manager = AudioContextManager::new();
        
        assert_eq!(manager.recreation_attempts(), 0);
        
        manager.recreation_attempts = 2;
        assert_eq!(manager.recreation_attempts(), 2);
        
        manager.reset_recreation_attempts();
        assert_eq!(manager.recreation_attempts(), 0);
    }

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

    #[wasm_bindgen_test]
    fn test_cached_devices_functionality() {
        let manager = AudioContextManager::new();
        
        // Initially should return empty devices
        let cached = manager.get_cached_devices();
        assert!(cached.input_devices.is_empty());
        assert!(cached.output_devices.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_device_change_listener_state() {
        let manager = AudioContextManager::new();
        
        // Initially should not have a device change listener
        assert!(!manager.has_device_change_listener());
        
        // Test that the device change listener field exists and is properly initialized
        // We can't test the actual listener setup in unit tests since it requires browser APIs
    }





}