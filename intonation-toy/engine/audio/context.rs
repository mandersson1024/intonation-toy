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
//!     // Create manager with default configuration (44.1kHz, 1024 buffer)
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
use super::buffer::STANDARD_SAMPLE_RATE;

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
    /// Preferred sample rate (44.1kHz standard, 48kHz for testing)
    pub sample_rate: u32,
    /// Buffer size for audio processing
    pub buffer_size: u32,
    /// Maximum number of recreation attempts
    pub max_recreation_attempts: u32,
}

impl Default for AudioContextConfig {
    fn default() -> Self {
        Self {
            sample_rate: STANDARD_SAMPLE_RATE,
            buffer_size: 1024,    // Production buffer size
            max_recreation_attempts: 3,
        }
    }
}

impl AudioContextConfig {
    
    /// Set buffer size
    pub fn with_buffer_size(mut self, buffer_size: u32) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}

/// Cached audio device information
#[derive(Debug, Clone, Default)]
pub struct AudioDevices {
    pub input_devices: Vec<(String, String)>,
    pub output_devices: Vec<(String, String)>,
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

impl Default for AudioContextManager {
    fn default() -> Self {
        Self::new()
    }
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
        options.set_sample_rate(self.config.sample_rate as f32);
        
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
    
    
    /// Get current AudioContext reference
    pub fn get_context(&self) -> Option<&AudioContext> {
        self.context.as_ref()
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


    /// Initialize all audio components with proper dependency order
    pub async fn initialize(&mut self) -> Result<(), String> {
        use crate::common::dev_log;
        
        
        // Clear any previous initialization error
        self.initialization_error = None;
        
        // Step 1: Initialize AudioContextManager
        let init_result = {
            let mut manager = self.audio_context_manager.borrow_mut();
            manager.initialize().await
        };
        if let Err(e) = init_result {
            let error_msg = format!("Failed to initialize AudioContextManager: {}", e);
            dev_log!("✗ {}", error_msg);
            self.initialization_error = Some(error_msg.clone());
            return Err(error_msg);
        }
        dev_log!("✓ AudioContextManager initialized");

        // Step 2: Initialize AudioWorkletManager (simplified for return-based pattern)
        let mut worklet_manager = super::worklet::AudioWorkletManager::new_return_based();
        
        // Initialize the worklet with the audio context
        let worklet_init_result = {
            let manager = self.audio_context_manager.borrow();
            worklet_manager.initialize(&manager).await
        };
        if let Err(e) = worklet_init_result {
            let error_msg = format!("Failed to initialize AudioWorkletManager: {:?}", e);
            dev_log!("✗ {}", error_msg);
            self.initialization_error = Some(error_msg.clone());
            return Err(error_msg);
        }
        
        self.audioworklet_manager = Some(worklet_manager);
        dev_log!("✓ AudioWorkletManager initialized for return-based pattern");

        // Step 3: Initialize PitchAnalyzer (simplified for return-based pattern)
        let config = super::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = {
            let borrowed = self.audio_context_manager.borrow();
            borrowed.config().sample_rate
        };
        
        match super::pitch_analyzer::PitchAnalyzer::new(config, sample_rate) {
            Ok(analyzer) => {
                // Create analyzer without setter (return-based pattern)
                let analyzer_rc = std::rc::Rc::new(std::cell::RefCell::new(analyzer));
                self.pitch_analyzer = Some(analyzer_rc.clone());
                
                // Connect the pitch analyzer to the AudioWorkletManager so it receives audio data
                if let Some(ref mut worklet_manager) = self.audioworklet_manager {
                    worklet_manager.set_pitch_analyzer(analyzer_rc);
                    dev_log!("✓ PitchAnalyzer connected to AudioWorkletManager");
                }
                
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
            
            // Setup message handling now that volume detector is configured
            if let Err(e) = worklet_manager.setup_message_handling() {
                let error_msg = format!("Failed to setup message handling: {:?}", e);
                dev_log!("✗ {}", error_msg);
                self.initialization_error = Some(error_msg.clone());
                return Err(error_msg);
            }
        }
        
        dev_log!("✓ VolumeDetector initialized and configured");

        // Step 5: Store AudioContextManager globally for device change callbacks
        super::set_global_audio_context_manager(self.audio_context_manager.clone());
        dev_log!("✓ AudioContextManager stored globally for device change callbacks");

        // Step 6: Perform initial device refresh to populate the cache
        {
            let mut manager = self.audio_context_manager.borrow_mut();
            if let Err(_e) = manager.refresh_audio_devices().await {
                dev_log!("Initial device refresh failed: {:?}", _e);
            } else {
                dev_log!("✓ Initial device refresh completed - device cache populated");
            }
        }

        // Step 7: Set up device change listener to automatically refresh device cache
        {
            let manager_rc = self.audio_context_manager.clone();
            let callback = move || {
                dev_log!("Device change detected in AudioSystemContext - refreshing device list");
                
                // Clone for the async closure
                let manager_rc_async = manager_rc.clone();
                
                // Spawn async task to refresh devices
                wasm_bindgen_futures::spawn_local(async move {
                    match manager_rc_async.try_borrow_mut() {
                        Ok(mut manager) => {
                            if let Err(_e) = manager.refresh_audio_devices().await {
                                dev_log!("AudioSystemContext auto device refresh failed: {:?}", _e);
                            } else {
                                dev_log!("AudioSystemContext auto device refresh completed successfully");
                            }
                        }
                        Err(_) => {
                            dev_log!("AudioContextManager busy during AudioSystemContext auto device refresh");
                        }
                    }
                });
            };
            
            // Set up the listener in the AudioContextManager
            match self.audio_context_manager.try_borrow_mut() {
                Ok(mut manager) => {
                    if let Err(_e) = manager.setup_device_change_listener(callback) {
                        dev_log!("Failed to set up AudioSystemContext device change listener: {:?}", _e);
                    } else {
                        dev_log!("✓ AudioSystemContext device change listener set up successfully");
                    }
                }
                Err(_) => {
                    dev_log!("AudioContextManager busy, cannot set up AudioSystemContext device change listener");
                }
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
        let close_result = {
            let mut manager = self.audio_context_manager.borrow_mut();
            manager.close().await
        };
        if let Err(e) = close_result {
            dev_log!("Warning: AudioContextManager close failed: {}", e);
        }
        
        self.is_initialized = false;
        dev_log!("✓ AudioSystemContext shutdown completed");
        Ok(())
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
        match self.audio_context_manager.try_borrow() {
            Ok(borrowed) => borrowed.get_cached_devices().clone(),
            Err(_) => super::AudioDevices { input_devices: Vec::new(), output_devices: Vec::new() }
        }
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



    

    /// Collect current audio analysis data (return-based pattern)
    /// 
    /// This method retrieves the current audio analysis data from the audio system
    /// without using the observable/setter pattern. It's used by the engine layer
    /// to collect data for returning in EngineUpdateResult.
    pub fn collect_audio_analysis(&self, timestamp: f64) -> Option<crate::shared_types::AudioAnalysis> {
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
            match analyzer.try_borrow() {
                Ok(borrowed) => borrowed.get_latest_pitch_data(),
                Err(_) => None
            }
        } else {
            None
        };
        
        // Convert pitch data to interface type
        let pitch = convert_pitch_data(pitch_data);
        
        // Merge the data into AudioAnalysis
        merge_audio_analysis(volume, pitch, timestamp)
    }

    /// Collect current audio errors (return-based pattern)
    pub fn collect_audio_errors(&self) -> Vec<crate::shared_types::Error> {
        let mut errors = Vec::new();
        
        // Check for initialization errors
        if let Some(error_msg) = self.initialization_error.as_ref() {
            errors.push(crate::shared_types::Error::ProcessingError(error_msg.clone()));
        }
        
        // Check AudioContext manager state
        let context_manager = match self.audio_context_manager.try_borrow() {
            Ok(borrowed) => borrowed,
            Err(_) => {
                // Audio context manager busy - skip state check to avoid panic
                return errors;
            }
        };
        if !context_manager.is_running() {
            match context_manager.state() {
                AudioContextState::Closed => {
                    errors.push(crate::shared_types::Error::ProcessingError("AudioContext is closed".to_string()));
                }
                AudioContextState::Suspended => {
                    errors.push(crate::shared_types::Error::ProcessingError("AudioContext is suspended".to_string()));
                }
                _ => {}
            }
        }
        
        errors
    }

    /// Collect current permission state (return-based pattern)
    pub fn collect_permission_state(&self) -> crate::shared_types::PermissionState {
        match self.permission_state.get() {
            super::AudioPermission::Uninitialized => crate::shared_types::PermissionState::NotRequested,
            super::AudioPermission::Requesting => crate::shared_types::PermissionState::Requested,
            super::AudioPermission::Granted => crate::shared_types::PermissionState::Granted,
            super::AudioPermission::Denied => crate::shared_types::PermissionState::Denied,
            super::AudioPermission::Unavailable => crate::shared_types::PermissionState::Denied,
        }
    }
    
    /// Configure root note audio system
    /// 
    /// This method configures the root note audio system by delegating to the AudioWorkletManager.
    /// It's used to enable/disable root note audio and set the frequency.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The root note audio configuration
    pub fn configure_root_note_audio(&mut self, config: super::RootNoteAudioConfig) {
        if let Some(ref mut worklet) = self.audioworklet_manager {
            worklet.update_root_note_audio_config(config);
        }
    }
    
    /// Set microphone permission state
    pub fn set_permission_state(&self, state: super::AudioPermission) {
        self.permission_state.set(state);
    }
    
}

/// Conversion functions for audio data types (return-based pattern)
/// 
/// These functions convert raw audio engine data types to interface types
/// without using the observable/setter pattern.
///
/// Convert VolumeLevelData to Volume interface type
pub fn convert_volume_data(volume_data: Option<super::data_types::VolumeLevelData>) -> Option<crate::shared_types::Volume> {
    volume_data.map(|data| crate::shared_types::Volume {
        peak_amplitude: data.peak_amplitude,
        rms_amplitude: data.rms_amplitude,
    })
}

/// Convert PitchData to Pitch interface type
pub fn convert_pitch_data(pitch_data: Option<super::data_types::PitchData>) -> Option<crate::shared_types::Pitch> {
    pitch_data.map(|data| {
        if data.frequency > 0.0 {
            crate::shared_types::Pitch::Detected(data.frequency, data.clarity)
        } else {
            crate::shared_types::Pitch::NotDetected
        }
    })
}

/// Merge volume and pitch data into AudioAnalysis
/// 
/// This function combines separate volume and pitch data into a unified
/// AudioAnalysis structure, similar to how AudioAnalysisMerger works but
/// as a pure function without state.
pub fn merge_audio_analysis(
    volume: Option<crate::shared_types::Volume>,
    pitch: Option<crate::shared_types::Pitch>,
    timestamp: f64
) -> Option<crate::shared_types::AudioAnalysis> {
    // Only create AudioAnalysis if we have at least some data
    if volume.is_some() || pitch.is_some() {
        Some(crate::shared_types::AudioAnalysis {
            volume_level: volume.unwrap_or(crate::shared_types::Volume { peak_amplitude: -60.0, rms_amplitude: -60.0 }),
            pitch: pitch.unwrap_or(crate::shared_types::Pitch::NotDetected),
            fft_data: None,
            timestamp: timestamp.max(js_sys::Date::now()),
        })
    } else {
        None
    }
}

