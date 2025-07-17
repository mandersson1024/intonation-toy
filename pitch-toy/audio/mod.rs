// Audio module for pitch-toy application
// Handles microphone input, AudioContext management, and real-time audio processing

pub mod microphone;
pub mod context;
pub mod worklet;
pub mod stream;
pub mod permission;
pub mod buffer;
pub mod buffer_analyzer;
pub mod console_service;
pub mod commands;
pub mod pitch_detector;
pub mod note_mapper;
pub mod pitch_analyzer;
pub mod volume_detector;
pub mod test_signal_generator;
pub mod message_protocol;
pub mod data_types;

use crate::common::dev_log;

use std::cell::RefCell;
use std::rc::Rc;

// Global audio context manager for application-wide access
thread_local! {
    static AUDIO_CONTEXT_MANAGER: RefCell<Option<Rc<RefCell<context::AudioContextManager>>>> = RefCell::new(None);
}

// Note: Buffer pool global state removed - using direct processing with transferable buffers

// Global AudioWorklet manager reference
thread_local! {
    static AUDIOWORKLET_MANAGER_GLOBAL: RefCell<Option<Rc<RefCell<worklet::AudioWorkletManager>>>> = RefCell::new(None);
}

// Temporary global AudioSystemContext instance for bridge pattern
thread_local! {
    static GLOBAL_AUDIO_SYSTEM_CONTEXT: RefCell<Option<Rc<RefCell<context::AudioSystemContext>>>> = RefCell::new(None);
}

/// Initialize audio system with dependency injection
/// Returns AudioSystemContext for centralized audio management
pub async fn initialize_audio_system_with_context(
    volume_level_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<VolumeLevelData>>>,
    pitch_data_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<PitchData>>>,
    audioworklet_status_setter: std::rc::Rc<dyn observable_data::DataSetter<AudioWorkletStatus>>,
) -> Result<context::AudioSystemContext, String> {
    dev_log!("Initializing audio system with dependency injection");
    
    // Check AudioContext support
    if !context::AudioContextManager::is_supported() {
        return Err("Web Audio API not supported".to_string());
    }
    
    // Create AudioSystemContext with setters passed at construction
    let mut context = context::AudioSystemContext::new(
        volume_level_setter,
        pitch_data_setter,
        audioworklet_status_setter,
    );
    
    // Initialize the context (this handles all component initialization)
    context.initialize().await
        .map_err(|e| format!("AudioSystemContext initialization failed: {}", e))?;
    
    dev_log!("✓ Audio system initialization completed with dependency injection");
    Ok(context)
}

/// Initialize audio system (legacy function for backward compatibility)
/// DEPRECATED: Use initialize_audio_system_with_context instead
#[deprecated(note = "Use initialize_audio_system_with_context instead")]
pub async fn initialize_audio_system() -> Result<(), String> {
    dev_log!("Initializing audio system (legacy)");
    
    // Check AudioContext support
    if !context::AudioContextManager::is_supported() {
        return Err("Web Audio API not supported".to_string());
    }
    
    // Create and initialize AudioContext with default configuration (48kHz, 1024 buffer)
    let mut audio_manager = context::AudioContextManager::new();
    
    match audio_manager.initialize().await {
        Ok(_) => {
            dev_log!("✓ AudioContext created successfully");
            dev_log!("  Sample rate: {:.1} kHz", audio_manager.config().sample_rate / 1000.0);
            dev_log!("  Buffer size: {} samples", audio_manager.config().buffer_size);
            
            // Store the initialized manager globally for application use
            AUDIO_CONTEXT_MANAGER.with(|manager| {
                *manager.borrow_mut() = Some(Rc::new(RefCell::new(audio_manager)));
            });
        }
        Err(e) => {
            return Err(format!("Failed to initialize AudioContext: {}", e));
        }
    }
    
    // AudioWorklet initialization is now available via worklet::AudioWorkletManager
    // Stream management is now available via stream::StreamReconnectionHandler
    
    dev_log!("✓ Audio system initialization completed");
    Ok(())
}

/// Store AudioContextManager globally for backward compatibility
pub fn set_global_audio_context_manager(manager: Rc<RefCell<context::AudioContextManager>>) {
    AUDIO_CONTEXT_MANAGER.with(|global_manager| {
        *global_manager.borrow_mut() = Some(manager);
    });
}

/// Get the global AudioContext manager
/// Returns None if audio system hasn't been initialized
pub fn get_audio_context_manager() -> Option<Rc<RefCell<context::AudioContextManager>>> {
    AUDIO_CONTEXT_MANAGER.with(|manager| {
        manager.borrow().as_ref().cloned()
    })
}

/// Check if audio system is initialized and running
pub fn is_audio_system_ready() -> bool {
    AUDIO_CONTEXT_MANAGER.with(|manager| {
        if let Some(ref audio_manager_rc) = *manager.borrow() {
            audio_manager_rc.borrow().is_running()
        } else {
            false
        }
    })
}

/// Create a ConsoleAudioService instance
/// Returns a configured console audio service with audio context manager if available
pub fn create_console_audio_service() -> console_service::ConsoleAudioServiceImpl {
    let mut service = console_service::ConsoleAudioServiceImpl::new();
    
    // Set audio context manager if available
    if let Some(manager) = get_audio_context_manager() {
        service.set_audio_context_manager(manager);
    }
    
    service
}




// Note: Buffer pool global functions removed - using direct processing with transferable buffers

/// Set the global AudioWorklet manager instance (called after creation)
/// DEPRECATED: Use AudioSystemContext constructor instead
#[deprecated(note = "Use AudioSystemContext constructor instead of global storage")]
pub fn set_global_audioworklet_manager(manager: Rc<RefCell<worklet::AudioWorkletManager>>) {
    AUDIOWORKLET_MANAGER_GLOBAL.with(|awm| {
        *awm.borrow_mut() = Some(manager);
    });
}

/// Get the global AudioWorklet manager instance
/// DEPRECATED: Use AudioSystemContext directly instead
#[deprecated(note = "Use AudioSystemContext directly instead of global access")]
pub fn get_global_audioworklet_manager() -> Option<Rc<RefCell<worklet::AudioWorkletManager>>> {
    AUDIOWORKLET_MANAGER_GLOBAL.with(|awm| awm.borrow().as_ref().cloned())
}

/// Set the global AudioSystemContext instance (bridge pattern)
/// TEMPORARY: This is for migration purposes only
pub fn set_global_audio_system_context(context: Rc<RefCell<context::AudioSystemContext>>) {
    GLOBAL_AUDIO_SYSTEM_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(context);
    });
}

/// Get the global AudioSystemContext instance (bridge pattern)
/// TEMPORARY: This is for migration purposes only
pub fn get_global_audio_system_context() -> Option<Rc<RefCell<context::AudioSystemContext>>> {
    GLOBAL_AUDIO_SYSTEM_CONTEXT.with(|ctx| ctx.borrow().as_ref().cloned())
}

// Bridge pattern wrapper functions that delegate to context methods
// TEMPORARY: These functions provide backwards compatibility during migration

/// Get AudioWorklet manager from global context
/// TEMPORARY: Use AudioSystemContext directly instead
pub fn get_global_audioworklet_manager_from_context() -> Option<Rc<RefCell<worklet::AudioWorkletManager>>> {
    // Note: The context method returns Option<&AudioWorkletManager>, but we need Option<Rc<RefCell<AudioWorkletManager>>>
    // For now, fall back to the global access - this is a temporary bridge function anyway
    get_global_audioworklet_manager()
}

/// Connect microphone using global context
/// TEMPORARY: Use context-aware functions instead
pub fn connect_microphone_to_audioworklet_from_context() -> impl std::future::Future<Output = Result<(), String>> {
    async move {
        if let Some(ctx) = get_global_audio_system_context() {
            microphone::connect_microphone_to_audioworklet_with_context(&ctx).await
        } else {
            Err("AudioSystemContext not available".to_string())
        }
    }
}

/// Initialize pitch analyzer using global context
/// TEMPORARY: Use context-aware functions instead
pub fn initialize_pitch_analyzer_from_context() -> impl std::future::Future<Output = Result<(), String>> {
    async move {
        if let Some(ctx) = get_global_audio_system_context() {
            // For now, use the existing function - can be improved later
            initialize_pitch_analyzer().await
        } else {
            Err("AudioSystemContext not available".to_string())
        }
    }
}

/// Set the AudioWorklet status setter on the global AudioWorkletManager
/// DEPRECATED: Use AudioSystemContext constructor instead
#[deprecated(note = "Use AudioSystemContext constructor instead - setters should be passed during initialization")]
pub fn set_audioworklet_status_setter(
    setter: std::rc::Rc<dyn observable_data::DataSetter<AudioWorkletStatus>>
) {
    if let Some(manager_rc) = get_global_audioworklet_manager() {
        {
            let mut manager = manager_rc.borrow_mut();
            manager.set_audioworklet_status_setter(setter);
        }
        // Immediately publish current status after setting the setter
        manager_rc.borrow().publish_audioworklet_status();
        dev_log!("AudioWorklet status setter configured on global manager and status published");
    } else {
        dev_log!("Warning: Cannot set AudioWorklet status setter - manager not initialized");
    }
}

/// Set the pitch data setter on the global PitchAnalyzer
/// DEPRECATED: Use AudioSystemContext constructor instead
#[deprecated(note = "Use AudioSystemContext constructor instead - setters should be passed during initialization")]
pub fn set_pitch_data_setter(
    setter: std::rc::Rc<dyn observable_data::DataSetter<Option<PitchData>>>
) {
    if let Some(analyzer_rc) = commands::get_global_pitch_analyzer() {
        let mut analyzer = analyzer_rc.borrow_mut();
        analyzer.set_pitch_data_setter(setter.clone());
        dev_log!("Pitch data setter configured on global pitch analyzer");
    } else {
        dev_log!("Warning: Cannot set pitch data setter - pitch analyzer not initialized");
    }
    
    // Also set it on the AudioWorkletManager for direct processing
    if let Some(manager_rc) = get_global_audioworklet_manager() {
        let mut manager = manager_rc.borrow_mut();
        manager.set_pitch_data_setter(setter);
        dev_log!("Pitch data setter configured on AudioWorkletManager");
    } else {
        dev_log!("Warning: Cannot set pitch data setter - AudioWorkletManager not initialized");
    }
}

/// Set the volume level setter on the global AudioWorkletManager
/// DEPRECATED: Use AudioSystemContext constructor instead
#[deprecated(note = "Use AudioSystemContext constructor instead - setters should be passed during initialization")]
pub fn set_volume_level_setter(
    setter: std::rc::Rc<dyn observable_data::DataSetter<Option<VolumeLevelData>>>
) {
    if let Some(manager_rc) = get_global_audioworklet_manager() {
        let mut manager = manager_rc.borrow_mut();
        manager.set_volume_level_setter(setter);
        dev_log!("Volume level setter configured on global AudioWorklet manager");
    } else {
        dev_log!("Warning: Cannot set volume level setter - AudioWorklet manager not initialized");
    }
}


// Note: initialize_buffer_pool removed - using direct processing with transferable buffers

/// Initialize global pitch analyzer with default configuration
/// DEPRECATED: Use AudioSystemContext::initialize() instead
#[deprecated(note = "Use AudioSystemContext::initialize() instead - pitch analyzer is now initialized automatically")]
pub async fn initialize_pitch_analyzer() -> Result<(), String> {
    dev_log!("Initializing pitch analyzer");
    
    // Use default configuration optimized for strong harmonic instruments
    let config = pitch_detector::PitchDetectorConfig::default();
    
    // Standard sample rate for audio processing
    let sample_rate = 48000.0;
    
    // Create pitch analyzer instance
    match pitch_analyzer::PitchAnalyzer::new(config.clone(), sample_rate) {
        Ok(analyzer) => {
            let analyzer_rc = Rc::new(RefCell::new(analyzer));
            
            
            // Log configuration details
            dev_log!("✓ Pitch analyzer created with configuration:");
            dev_log!("  Window size: {} samples", config.sample_window_size);
            dev_log!("  Threshold: {:.2} (optimized for strong harmonic instruments)", config.threshold);
            dev_log!("  Frequency range: {:.1} - {:.1} Hz", config.min_frequency, config.max_frequency);
            dev_log!("  Sample rate: {:.1} kHz", sample_rate / 1000.0);
            
            // Register globally for console commands access
            commands::set_global_pitch_analyzer(analyzer_rc.clone());
            
            // Configure the pitch analyzer for direct processing via AudioWorklet
            if let Some(manager) = get_global_audioworklet_manager() {
                let mut manager_borrowed = manager.borrow_mut();
                manager_borrowed.set_pitch_analyzer(analyzer_rc.clone());
                dev_log!("✓ Pitch analyzer configured for direct processing via AudioWorklet");
            } else {
                dev_log!("Warning: No AudioWorklet manager available for pitch analyzer setup");
            }
            
            // Note: Event-based processing is replaced with direct processing from AudioWorklet messages
            // The pitch analyzer is now called directly from handle_audio_data_batch() in the worklet manager
            
            Ok(())
        }
        Err(e) => {
            Err(format!("Failed to create pitch analyzer: {}", e))
        }
    }
}

// Re-export public API
pub use microphone::{MicrophoneManager, AudioStreamInfo, AudioError, connect_microphone_to_audioworklet, connect_microphone_to_audioworklet_with_context};
pub use permission::{AudioPermission, connect_microphone_with_context};
pub use context::{AudioContextManager, AudioContextState, AudioContextConfig, AudioDevices, AudioSystemContext};
pub use worklet::{AudioWorkletManager, AudioWorkletState, AudioWorkletConfig};
pub use stream::{StreamReconnectionHandler, StreamState, StreamHealth, StreamConfig, StreamError};
pub use permission::PermissionManager;
pub use buffer::{CircularBuffer, BufferState, PRODUCTION_BUFFER_SIZE, DEV_BUFFER_SIZE_MIN, DEV_BUFFER_SIZE_MAX, DEV_BUFFER_SIZE_DEFAULT, AUDIO_CHUNK_SIZE, get_buffer_size, validate_buffer_size, validate_buffer_size_for_creation};
pub use buffer_analyzer::{BufferAnalyzer, WindowFunction};
// Note: BufferPool re-export removed - using direct processing with transferable buffers
pub use console_service::{ConsoleAudioService, ConsoleAudioServiceImpl, AudioStatus};
pub use commands::{register_audio_commands, set_global_pitch_analyzer, get_global_pitch_analyzer};
pub use pitch_detector::{PitchResult, PitchDetectorConfig, MusicalNote, NoteName, TuningSystem, PitchDetector, PitchDetectionError};
pub use note_mapper::NoteMapper;
pub use pitch_analyzer::{PitchAnalyzer, PitchPerformanceMetrics, PitchAnalysisError};
pub use volume_detector::{VolumeDetector, VolumeDetectorConfig, VolumeLevel, VolumeAnalysis};
pub use test_signal_generator::{TestSignalGenerator, TestSignalGeneratorConfig, TestWaveform, BackgroundNoiseConfig};
pub use data_types::{VolumeLevelData, PitchData, AudioWorkletStatus};
pub use message_protocol::{
    ToWorkletMessage, FromWorkletMessage, ToWorkletEnvelope, FromWorkletEnvelope,
    AudioDataBatch, ProcessorStatus, BatchConfig, WorkletError, WorkletErrorCode,
    ErrorContext, MemoryUsage, MessageEnvelope, 
    SerializationResult, SerializationError, ToJsMessage, FromJsMessage, MessageValidator,
    MessageSerializer, MessageDeserializer,
    MessageConstructionResult, MessageConstructionError, MessageIdGenerator,
    MessageBuilder, AudioWorkletMessageFactory, generate_unique_message_id,
    get_high_resolution_timestamp,
    // Enhanced error handling types
    MessageProtocolError, ValidationError, TransferError, MessageProtocolResult, ValidationResult, TransferResult,
    MessageContext, MessageDirection, SystemState
};

/// Setup UI action listeners for audio module
/// New version with AudioSystemContext parameter - currently using global fallback
/// TODO: Implement proper thread-safe context sharing for closures
pub fn setup_ui_action_listeners_with_context(
    listeners: crate::UIControlListeners,
    microphone_permission_setter: impl observable_data::DataSetter<AudioPermission> + Clone + 'static,
    _audio_context: std::rc::Rc<std::cell::RefCell<AudioSystemContext>>,
) {
    // For now, we fall back to global access due to thread safety requirements
    // The ActionListener requires Send + Sync closures, but Rc<RefCell<T>> is not Send/Sync
    // Future improvement: Consider using Arc<RwLock<T>> or a different approach
    
    // Test signal action listener
    listeners.test_signal.listen(|action| {
        dev_log!("Received test signal action: {:?}", action);
        
        if let Some(worklet_rc) = get_global_audioworklet_manager() {
            let mut worklet = worklet_rc.borrow_mut();
            
            // Convert UI action to audio system config
            let audio_config = TestSignalGeneratorConfig {
                enabled: action.enabled,
                frequency: action.frequency,
                amplitude: action.volume / 100.0, // Convert percentage to 0-1 range
                waveform: action.waveform,
                sample_rate: 48000.0, // Use standard sample rate
            };
            
            worklet.update_test_signal_config(audio_config);
            dev_log!("✓ Test signal config updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for test signal config");
        }
    });
    
    // Background noise action listener
    listeners.background_noise.listen(|action| {
        dev_log!("Received background noise action: {:?}", action);
        
        if let Some(worklet_rc) = get_global_audioworklet_manager() {
            let mut worklet = worklet_rc.borrow_mut();
            
            // Convert UI action to audio system config
            let audio_config = BackgroundNoiseConfig {
                enabled: action.enabled,
                level: action.level,
                noise_type: action.noise_type,
            };
            
            worklet.update_background_noise_config(audio_config);
            dev_log!("✓ Background noise config updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for background noise config");
        }
    });
    
    // Output to speakers action listener
    listeners.output_to_speakers.listen(|action| {
        dev_log!("Received output to speakers action: {:?}", action);
        
        if let Some(worklet_rc) = get_global_audioworklet_manager() {
            let mut worklet = worklet_rc.borrow_mut();
            worklet.set_output_to_speakers(action.enabled);
            dev_log!("✓ Output to speakers setting updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for output to speakers setting");
        }
    });
    
    // Microphone permission action listener  
    listeners.microphone_permission.listen(move |action| {
        dev_log!("Received microphone permission action: {:?}", action);
        
        if action.request_permission {
            // TODO: Use context-aware microphone connection when available
            // For now, falling back to global access due to thread safety constraints
            permission::connect_microphone(microphone_permission_setter.clone());
            dev_log!("✓ Microphone permission request triggered via action");
        }
    });
}

/// Setup UI action listeners for audio module
/// DEPRECATED: Use setup_ui_action_listeners_with_context instead
#[deprecated(note = "Use setup_ui_action_listeners_with_context instead - pass AudioSystemContext for better dependency management")]
pub fn setup_ui_action_listeners(
    listeners: crate::UIControlListeners,
    microphone_permission_setter: impl observable_data::DataSetter<AudioPermission> + Clone + 'static,
) {
    // Test signal action listener
    listeners.test_signal.listen(|action| {
        dev_log!("Received test signal action: {:?}", action);
        
        if let Some(worklet_rc) = get_global_audioworklet_manager() {
            let mut worklet = worklet_rc.borrow_mut();
            
            // Convert UI action to audio system config
            let audio_config = TestSignalGeneratorConfig {
                enabled: action.enabled,
                frequency: action.frequency,
                amplitude: action.volume / 100.0, // Convert percentage to 0-1 range
                waveform: action.waveform,
                sample_rate: 48000.0, // Use standard sample rate
            };
            
            worklet.update_test_signal_config(audio_config);
            dev_log!("✓ Test signal config updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for test signal config");
        }
    });
    
    // Background noise action listener
    listeners.background_noise.listen(|action| {
        dev_log!("Received background noise action: {:?}", action);
        
        if let Some(worklet_rc) = get_global_audioworklet_manager() {
            let mut worklet = worklet_rc.borrow_mut();
            
            // Convert UI action to audio system config
            let audio_config = BackgroundNoiseConfig {
                enabled: action.enabled,
                level: action.level,
                noise_type: action.noise_type,
            };
            
            worklet.update_background_noise_config(audio_config);
            dev_log!("✓ Background noise config updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for background noise config");
        }
    });
    
    // Output to speakers action listener
    listeners.output_to_speakers.listen(|action| {
        dev_log!("Received output to speakers action: {:?}", action);
        
        if let Some(worklet_rc) = get_global_audioworklet_manager() {
            let mut worklet = worklet_rc.borrow_mut();
            worklet.set_output_to_speakers(action.enabled);
            dev_log!("✓ Output to speakers setting updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for output to speakers setting");
        }
    });
    
    // Microphone permission action listener
    listeners.microphone_permission.listen(move |action| {
        dev_log!("Received microphone permission action: {:?}", action);
        
        if action.request_permission {
            // Trigger microphone permission request using the same function
            // that was previously called directly by the button
            permission::connect_microphone(microphone_permission_setter.clone());
            dev_log!("✓ Microphone permission request triggered via action");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // No wasm_bindgen_test_configure! needed for Node.js
   


    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_error_types() {
        let permission_error = AudioError::PermissionDenied("Test permission denied".to_string());
        assert!(permission_error.to_string().contains("Permission denied"));
        assert!(permission_error.to_string().contains("Test permission denied"));

        let device_error = AudioError::DeviceUnavailable("Test device unavailable".to_string());
        assert!(device_error.to_string().contains("Device unavailable"));
        assert!(device_error.to_string().contains("Test device unavailable"));

        let not_supported_error = AudioError::NotSupported("Test not supported".to_string());
        assert!(not_supported_error.to_string().contains("Not supported"));
        assert!(not_supported_error.to_string().contains("Test not supported"));

        let stream_error = AudioError::StreamInitFailed("Test stream init failed".to_string());
        assert!(stream_error.to_string().contains("Stream initialization failed"));
        assert!(stream_error.to_string().contains("Test stream init failed"));

        let generic_error = AudioError::Generic("Test generic error".to_string());
        assert!(generic_error.to_string().contains("Audio error"));
        assert!(generic_error.to_string().contains("Test generic error"));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_microphone_state_enum() {
        // Test all microphone states
        assert_eq!(AudioPermission::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioPermission::Requesting.to_string(), "Requesting");
        assert_eq!(AudioPermission::Granted.to_string(), "Granted");
        assert_eq!(AudioPermission::Denied.to_string(), "Denied");
        assert_eq!(AudioPermission::Unavailable.to_string(), "Unavailable");

        // Test PartialEq implementation
        assert_eq!(AudioPermission::Granted, AudioPermission::Granted);
        assert_ne!(AudioPermission::Granted, AudioPermission::Denied);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_context_state_enum() {
        // Test all audio context states
        assert_eq!(AudioContextState::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioContextState::Initializing.to_string(), "Initializing");
        assert_eq!(AudioContextState::Running.to_string(), "Running");
        assert_eq!(AudioContextState::Suspended.to_string(), "Suspended");
        assert_eq!(AudioContextState::Closed.to_string(), "Closed");
        assert_eq!(AudioContextState::Recreating.to_string(), "Recreating");

        // Test PartialEq implementation
        assert_eq!(AudioContextState::Running, AudioContextState::Running);
        assert_ne!(AudioContextState::Running, AudioContextState::Suspended);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_stream_state_enum() {
        // Test all stream states
        assert_eq!(StreamState::Disconnected, StreamState::Disconnected);
        assert_eq!(StreamState::Connecting, StreamState::Connecting);
        assert_eq!(StreamState::Connected, StreamState::Connected);
        assert_eq!(StreamState::Reconnecting, StreamState::Reconnecting);
        assert_eq!(StreamState::Failed, StreamState::Failed);

        // Test different states are not equal
        assert_ne!(StreamState::Connected, StreamState::Disconnected);
        assert_ne!(StreamState::Connecting, StreamState::Reconnecting);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_stream_error_types() {
        let device_disconnected = StreamError::DeviceDisconnected;
        assert_eq!(device_disconnected.to_string(), "Audio device disconnected");

        let permission_revoked = StreamError::PermissionRevoked;
        assert_eq!(permission_revoked.to_string(), "Microphone permission revoked");

        let unknown_device = StreamError::UnknownDevice;
        assert_eq!(unknown_device.to_string(), "Unknown audio device");

        let reconnection_failed = StreamError::ReconnectionFailed;
        assert_eq!(reconnection_failed.to_string(), "Failed to reconnect audio stream");

        let stream_ended = StreamError::StreamEnded;
        assert_eq!(stream_ended.to_string(), "Audio stream ended unexpectedly");

        let config_error = StreamError::ConfigurationError("Test config error".to_string());
        assert!(config_error.to_string().contains("Stream configuration error"));
        assert!(config_error.to_string().contains("Test config error"));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_stream_info_default() {
        let info = AudioStreamInfo::default();
        assert_eq!(info.sample_rate, 48000.0);
        assert_eq!(info.buffer_size, 1024);
        assert!(info.device_id.is_none());
        assert!(info.device_label.is_none());
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
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.max_reconnect_attempts, 3);
        assert_eq!(config.reconnect_delay_ms, 1000);
        assert_eq!(config.health_check_interval_ms, 5000);
        assert_eq!(config.activity_timeout_ms, 10000);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_manager_creation() {
        // Test that all managers can be created successfully
        let mic_manager = MicrophoneManager::new();
        assert_eq!(*mic_manager.state(), AudioPermission::Uninitialized);

        let audio_manager = AudioContextManager::new();
        assert_eq!(*audio_manager.state(), AudioContextState::Uninitialized);

        let stream_handler = StreamReconnectionHandler::new(StreamConfig::default());
        assert_eq!(stream_handler.get_health().state, StreamState::Disconnected);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_error_handling_integration() {
        // Test that error types can be properly used together
        let audio_error = AudioError::Generic("Integration test".to_string());
        let stream_error = StreamError::ConfigurationError("Integration test".to_string());
        
        // Both should format correctly
        assert!(audio_error.to_string().contains("Integration test"));
        assert!(stream_error.to_string().contains("Integration test"));
        
        // Both should be Debug-formatted correctly
        assert!(format!("{:?}", audio_error).contains("Generic"));
        assert!(format!("{:?}", stream_error).contains("ConfigurationError"));
    }
}