// Audio module for pitch-toy application
// 
// This module provides comprehensive audio processing capabilities using dependency injection
// with AudioSystemContext for centralized audio component management.
// 
// # Architecture
// 
// The audio system is built around AudioSystemContext which uses dependency injection
// to manage all audio components:
// - AudioContextManager: Web Audio API management
// - AudioWorkletManager: Real-time audio processing
// - PitchAnalyzer: Pitch detection and analysis
// - Data setters: Reactive data updates
// 
// # Usage
// 
// ```rust
// // Initialize audio system with dependency injection
// let context = initialize_audio_system_with_context(
//     volume_setter,
//     pitch_setter,
//     status_setter,
// ).await?;
// 
// // Connect microphone using context
// connect_microphone_to_audioworklet_with_context(&context).await?;
// 
// // Setup UI action listeners with context
// setup_ui_action_listeners_with_context(listeners, permission_setter, context);
// ```
// 
// # Migration from Global State
// 
// This module has been migrated from global state access to dependency injection.
// The AudioSystemContext provides centralized access to all audio components,
// eliminating the need for global state management.

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
// TODO: FUTURE REFACTORING - Remove this global variable and replace with dependency injection through AudioSystemContext.
// This is a planned future task. Do NOT refactor this during unrelated work.
// See docs/global_variables_refactoring_guide.md for refactoring strategy.
thread_local! {
    static AUDIO_CONTEXT_MANAGER: RefCell<Option<Rc<RefCell<context::AudioContextManager>>>> = RefCell::new(None);
}

// Note: Buffer pool global state removed - using direct processing with transferable buffers



/// Initialize audio system with dependency injection
/// 
/// This function creates and initializes a complete AudioSystemContext with all required
/// audio components using proper dependency injection patterns.
/// 
/// # Parameters
/// - `volume_level_setter`: Data setter for volume level updates
/// - `pitch_data_setter`: Data setter for pitch detection data
/// - `audioworklet_status_setter`: Data setter for AudioWorklet status updates
/// 
/// # Returns
/// - `Ok(AudioSystemContext)` with fully initialized audio system
/// - `Err(String)` with error details if initialization failed
/// 
/// # Components Initialized
/// - AudioContextManager for Web Audio API management
/// - AudioWorkletManager for real-time audio processing
/// - PitchAnalyzer for pitch detection and analysis
/// - Data setter integration for reactive updates
/// 
/// # Example
/// ```rust
/// let volume_setter = /* volume data setter */;
/// let pitch_setter = /* pitch data setter */;
/// let status_setter = /* status data setter */;
/// 
/// let context = initialize_audio_system_with_context(
///     volume_setter,
///     pitch_setter,
///     status_setter,
/// ).await?;
/// ```
pub async fn initialize_audio_system_with_context() -> Result<context::AudioSystemContext, String> {
    dev_log!("Initializing audio system with dependency injection");
    
    // Check AudioContext support
    if !context::AudioContextManager::is_supported() {
        return Err("Web Audio API not supported".to_string());
    }
    
    // Create AudioSystemContext using return-based pattern (no setters needed)
    let mut context = context::AudioSystemContext::new_return_based();
    
    // Initialize the context (this handles all component initialization)
    context.initialize().await
        .map_err(|e| format!("AudioSystemContext initialization failed: {}", e))?;
    
    dev_log!("✓ Audio system initialization completed with dependency injection");
    Ok(context)
}

/// Initialize audio system with three-layer architecture interfaces
/// 
/// This function creates and initializes a complete AudioSystemContext using the three-layer
/// architecture interfaces. Data setters are extracted from the interfaces and used internally.
/// 
/// # Parameters
/// - `engine_to_model`: Interface for engine → model data flow
/// - `model_to_engine`: Interface for model → engine action handling
/// 
/// # Returns
/// - `Ok(AudioSystemContext)` with fully initialized audio system
/// - `Err(String)` with error details if initialization failed
/// 
/// # Components Initialized
/// - AudioContextManager for Web Audio API management
/// - AudioWorkletManager for real-time audio processing
/// - PitchAnalyzer for pitch detection and analysis
/// - Interface-based data routing
/// 
/// # Example
/// ```rust
/// let engine_to_model = EngineToModelInterface::new();
/// let model_to_engine = ModelToEngineInterface::new();
/// 
/// let context = initialize_audio_system_with_interfaces(
///     &engine_to_model,
///     &model_to_engine,
/// ).await?;
/// ```
pub async fn initialize_audio_system_with_interfaces() -> Result<context::AudioSystemContext, String> {
    dev_log!("Initializing audio system with three-layer architecture interfaces");
    
    // Check AudioContext support
    if !context::AudioContextManager::is_supported() {
        return Err("Web Audio API not supported".to_string());
    }
    
    // Create AudioSystemContext using return-based pattern (interfaces no longer needed)
    let mut context = context::AudioSystemContext::new_return_based();
    
    // Initialize the context (this handles all component initialization)
    context.initialize().await
        .map_err(|e| format!("AudioSystemContext initialization failed: {}", e))?;
    
    dev_log!("✓ Audio system initialization completed with interface-based architecture");
    Ok(context)
}

/// Initialize audio system with interfaces and debug actions support
pub async fn initialize_audio_system_with_interfaces_and_debug() -> Result<context::AudioSystemContext, String> {
    dev_log!("Initializing audio system with three-layer architecture interfaces and debug actions");
    
    // Check AudioContext support
    if !context::AudioContextManager::is_supported() {
        return Err("Web Audio API not supported".to_string());
    }
    
    // Create AudioSystemContext using return-based pattern (interfaces no longer needed)
    let mut context = context::AudioSystemContext::new_return_based();
    
    // Initialize the context (this handles all component initialization)
    context.initialize().await
        .map_err(|e| format!("AudioSystemContext initialization failed: {}", e))?;
    
    dev_log!("✓ Audio system initialization completed with interface-based architecture and debug actions");
    Ok(context)
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






// Note: initialize_buffer_pool removed - using direct processing with transferable buffers


// Re-export public API
pub use microphone::{MicrophoneManager, AudioStreamInfo, AudioError, connect_microphone_to_audioworklet_with_context};
pub use permission::{AudioPermission, connect_microphone_with_context};
pub use context::{AudioContextManager, AudioContextState, AudioContextConfig, AudioDevices, AudioSystemContext, convert_volume_data, convert_pitch_data, merge_audio_analysis};
pub use worklet::{AudioWorkletManager, AudioWorkletState, AudioWorkletConfig};
pub use stream::{StreamReconnectionHandler, StreamState, StreamHealth, StreamConfig, StreamError};
pub use permission::PermissionManager;
pub use buffer::{CircularBuffer, BufferState, PRODUCTION_BUFFER_SIZE, DEV_BUFFER_SIZE_MIN, DEV_BUFFER_SIZE_MAX, DEV_BUFFER_SIZE_DEFAULT, AUDIO_CHUNK_SIZE, get_buffer_size, validate_buffer_size, validate_buffer_size_for_creation};
pub use buffer_analyzer::{BufferAnalyzer, WindowFunction};
// Note: BufferPool re-export removed - using direct processing with transferable buffers
pub use console_service::{ConsoleAudioService, ConsoleAudioServiceImpl, AudioStatus};
pub use commands::register_audio_commands;
pub use pitch_detector::{PitchResult, PitchDetectorConfig, MusicalNote, NoteName, TuningSystem, PitchDetector, PitchDetectionError};
pub use note_mapper::NoteMapper;
pub use pitch_analyzer::{PitchAnalyzer, PitchPerformanceMetrics, PitchAnalysisError};
pub use volume_detector::{VolumeDetector, VolumeDetectorConfig, VolumeAnalysis};
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

/// Setup UI action listeners for audio module with AudioSystemContext
/// 
/// This function sets up action listeners for audio-related UI controls using dependency injection.
/// The AudioSystemContext parameter provides access to all audio components needed for configuration.
/// 
/// # Parameters
/// - `listeners`: UI control listeners for audio actions
/// - `audio_context`: AudioSystemContext instance containing all audio components
/// 
pub fn setup_ui_action_listeners_with_context(
    listeners: crate::UIControlListeners,
    audio_context: std::rc::Rc<std::cell::RefCell<AudioSystemContext>>,
) {
    
    // Test signal action listener
    let audio_context_clone = audio_context.clone();
    listeners.test_signal.listen(move |action| {
        dev_log!("Received test signal action: {:?}", action);
        
        let mut context = audio_context_clone.borrow_mut();
        if let Some(worklet_manager) = context.get_audioworklet_manager_mut() {
            // Convert UI action to audio system config
            let audio_config = TestSignalGeneratorConfig {
                enabled: action.enabled,
                frequency: action.frequency,
                amplitude: action.volume / 100.0, // Convert percentage to 0-1 range
                waveform: action.waveform,
                sample_rate: 48000.0, // Use standard sample rate
            };
            
            worklet_manager.update_test_signal_config(audio_config);
            dev_log!("✓ Test signal config updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for test signal config");
        }
    });
    
    // Background noise action listener
    let audio_context_clone = audio_context.clone();
    listeners.background_noise.listen(move |action| {
        dev_log!("Received background noise action: {:?}", action);
        
        let mut context = audio_context_clone.borrow_mut();
        if let Some(worklet_manager) = context.get_audioworklet_manager_mut() {
            // Convert UI action to audio system config
            let audio_config = BackgroundNoiseConfig {
                enabled: action.enabled,
                level: action.level,
                noise_type: action.noise_type,
            };
            
            worklet_manager.update_background_noise_config(audio_config);
            dev_log!("✓ Background noise config updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for background noise config");
        }
    });
    
    // Output to speakers action listener
    let audio_context_clone = audio_context.clone();
    listeners.output_to_speakers.listen(move |action| {
        dev_log!("Received output to speakers action: {:?}", action);
        
        let mut context = audio_context_clone.borrow_mut();
        if let Some(worklet_manager) = context.get_audioworklet_manager_mut() {
            worklet_manager.set_output_to_speakers(action.enabled);
            dev_log!("✓ Output to speakers setting updated via action");
        } else {
            dev_log!("Warning: No AudioWorklet manager available for output to speakers setting");
        }
    });
    
    // Microphone permission action listener  
    let audio_context_clone = audio_context.clone();
    listeners.microphone_permission.listen(move |action| {
        dev_log!("Received microphone permission action: {:?}", action);
        
        if action.request_permission {
            // Permission state is now tracked through AudioSystemContext
            wasm_bindgen_futures::spawn_local({
                let audio_context = audio_context_clone.clone();
                
                async move {
                    // Set requesting state
                    audio_context.borrow().set_permission_state(AudioPermission::Requesting);
                    
                    let result = microphone::connect_microphone_to_audioworklet_with_context(&audio_context).await;
                    
                    // Handle the result and update permission state
                    audio_context.borrow().handle_microphone_connection_result(
                        result.map_err(|e| e.to_string())
                    );
                }
            });
        }
    });
}


#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // No wasm_bindgen_test_configure! needed for Node.js
   


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

    #[wasm_bindgen_test]
    fn test_audio_stream_info_default() {
        let info = AudioStreamInfo::default();
        assert_eq!(info.sample_rate, 48000.0);
        assert_eq!(info.buffer_size, 1024);
        assert!(info.device_id.is_none());
        assert!(info.device_label.is_none());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_config_default() {
        let config = AudioContextConfig::default();
        assert_eq!(config.sample_rate, 48000.0);
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.max_recreation_attempts, 3);
    }

    #[wasm_bindgen_test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.max_reconnect_attempts, 3);
        assert_eq!(config.reconnect_delay_ms, 1000);
        assert_eq!(config.health_check_interval_ms, 5000);
        assert_eq!(config.activity_timeout_ms, 10000);
    }

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

    #[wasm_bindgen_test]
    fn test_return_based_audio_system_context_creation() {
        // Test that AudioSystemContext can be created with return-based pattern
        let context = context::AudioSystemContext::new_return_based();
        
        // Context should be created but not initialized yet
        assert!(!context.is_ready());
        assert!(context.get_initialization_error().is_none());
        
        // Context should have the required components ready for initialization
        assert!(context.get_audio_context_manager().borrow().state() == &context::AudioContextState::Uninitialized);
    }

    #[wasm_bindgen_test]
    fn test_interface_adapter_volume_data_conversion() {
        // Test volume data conversion using new conversion functions
        let volume_data = data_types::VolumeLevelData {
            peak_db: -10.0,
            rms_db: -20.0,
        };
        
        // Test the conversion function
        let converted = context::convert_volume_data(Some(volume_data));
        assert!(converted.is_some());
        
        let volume = converted.unwrap();
        assert_eq!(volume.peak, -10.0);
        assert_eq!(volume.rms, -20.0);
        
        // Test with None input
        let converted_none = context::convert_volume_data(None);
        assert!(converted_none.is_none());
    }

    #[wasm_bindgen_test]
    fn test_interface_adapter_pitch_data_conversion() {
        // Test pitch data conversion using new conversion functions
        let pitch_data = data_types::PitchData {
            frequency: 440.0,
            confidence: 0.9,
            note: pitch_detector::MusicalNote::new(pitch_detector::NoteName::A, 4, 0.0, 440.0),
            clarity: 0.8,
            timestamp: 12345.0,
        };
        
        // Test the conversion function
        let converted = context::convert_pitch_data(Some(pitch_data));
        assert!(converted.is_some());
        
        let pitch = converted.unwrap();
        match pitch {
            crate::module_interfaces::engine_to_model::Pitch::Detected(freq, clarity) => {
                assert_eq!(freq, 440.0);
                assert_eq!(clarity, 0.8);
            }
            _ => panic!("Expected detected pitch"),
        }
        
        // Test with None input
        let converted_none = context::convert_pitch_data(None);
        assert!(converted_none.is_none());
        
        // Test with zero frequency (should be NotDetected)
        let pitch_data_zero = data_types::PitchData {
            frequency: 0.0,
            confidence: 0.0,
            note: pitch_detector::MusicalNote::new(pitch_detector::NoteName::A, 4, 0.0, 440.0),
            clarity: 0.0,
            timestamp: 12345.0,
        };
        
        let converted_zero = context::convert_pitch_data(Some(pitch_data_zero));
        assert!(converted_zero.is_some());
        
        match converted_zero.unwrap() {
            crate::module_interfaces::engine_to_model::Pitch::NotDetected => {
                // Expected
            }
            _ => panic!("Expected NotDetected pitch for zero frequency"),
        }
    }


    #[wasm_bindgen_test]
    fn test_return_based_audio_context_creation() {
        // Test that return-based AudioSystemContext can be created without interfaces
        let _context = context::AudioSystemContext::new_return_based();
        
        // The return-based pattern doesn't use interface-based action handling
        // so this test simply verifies that creation works without dependencies
        assert!(true, "Return-based AudioSystemContext created successfully");
    }




}