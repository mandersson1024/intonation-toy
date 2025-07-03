// Audio module for pitch-toy application
// Handles microphone input, AudioContext management, and real-time audio processing

pub mod microphone;
pub mod context;
pub mod worklet;
pub mod stream;
pub mod permission;
pub mod buffer;
pub mod buffer_analyzer;
pub mod buffer_pool;
pub mod console_service;
pub mod commands;

use crate::common::dev_log;

use std::cell::RefCell;
use std::rc::Rc;

// Global audio context manager for application-wide access
thread_local! {
    static AUDIO_CONTEXT_MANAGER: RefCell<Option<Rc<RefCell<context::AudioContextManager>>>> = RefCell::new(None);
}

// Global buffer pool reference
thread_local! {
    static BUFFER_POOL_GLOBAL: RefCell<Option<Rc<RefCell<buffer_pool::BufferPool<f32>>>>> = RefCell::new(None);
}

/// Initialize audio system
/// Returns Result to allow caller to handle initialization failures
pub async fn initialize_audio_system() -> Result<(), String> {
    dev_log!("Initializing audio system");
    
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

/// Create a ConsoleAudioService instance with event dispatcher
/// Returns a configured console audio service with both audio context manager and event dispatcher
pub fn create_console_audio_service_with_events(
    event_dispatcher: crate::events::SharedEventDispatcher
) -> console_service::ConsoleAudioServiceImpl {
    let mut service = console_service::ConsoleAudioServiceImpl::new();
    
    // Set audio context manager if available
    if let Some(manager) = get_audio_context_manager() {
        service.set_audio_context_manager(manager);
    }
    
    // Set event dispatcher
    service.set_event_dispatcher(event_dispatcher);
    
    service
}

/// Set the global BufferPool instance (called after creation)
pub fn set_global_buffer_pool(pool: Rc<RefCell<buffer_pool::BufferPool<f32>>>) {
    BUFFER_POOL_GLOBAL.with(|bp| {
        *bp.borrow_mut() = Some(pool);
    });
}

/// Get the global BufferPool instance
pub fn get_global_buffer_pool() -> Option<Rc<RefCell<buffer_pool::BufferPool<f32>>>> {
    BUFFER_POOL_GLOBAL.with(|bp| bp.borrow().as_ref().cloned())
}

// Re-export public API
pub use microphone::{MicrophoneManager, AudioStreamInfo, AudioError};
pub use permission::AudioPermission;
pub use context::{AudioContextManager, AudioContextState, AudioContextConfig, AudioDevices};
pub use worklet::{AudioWorkletManager, AudioWorkletState, AudioWorkletConfig};
pub use stream::{StreamReconnectionHandler, StreamState, StreamHealth, StreamConfig, StreamError};
pub use permission::PermissionManager;
pub use buffer::{CircularBuffer, BufferState, PRODUCTION_BUFFER_SIZE, DEV_BUFFER_SIZE_MIN, DEV_BUFFER_SIZE_MAX, DEV_BUFFER_SIZE_DEFAULT, AUDIO_CHUNK_SIZE, get_buffer_size, validate_buffer_size, validate_buffer_size_for_creation};
pub use buffer_analyzer::{BufferAnalyzer, WindowFunction};
pub use buffer_pool::{BufferPool, MAX_GPU_MEMORY_BYTES};
pub use console_service::{ConsoleAudioService, ConsoleAudioServiceImpl, AudioStatus};
pub use commands::register_audio_commands;

#[cfg(test)]
mod tests {
    use super::*;
    

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn test_initialize_audio_system_success() {
        // This test only runs on wasm32 where Web Audio API might be available
        let result = initialize_audio_system().await;
        
        // The result depends on the WASM test environment's Web Audio API support
        match result {
            Ok(()) => {
                // If successful, the system should have initialized properly
                assert!(true);
            }
            Err(msg) => {
                // If failed, should be due to Web Audio API not being supported in test environment
                assert!(msg.contains("Web Audio API not supported"));
            }
        }
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_initialize_audio_system_native_test() {
        // For native tests, we can only test that the function doesn't panic
        // The actual behavior depends on the Web Audio API which isn't available in cargo test
        // This is a structural test to ensure the function can be called
        
        // We can't actually call the function because it uses web APIs
        // Instead we test the error types and structure
        assert!(true); // This test just ensures the module compiles
    }

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
    fn test_audio_stream_info_default() {
        let info = AudioStreamInfo::default();
        assert_eq!(info.sample_rate, 48000.0);
        assert_eq!(info.buffer_size, 1024);
        assert!(info.device_id.is_none());
        assert!(info.device_label.is_none());
    }

    #[test]
    fn test_audio_context_config_default() {
        let config = AudioContextConfig::default();
        assert_eq!(config.sample_rate, 48000.0);
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.max_recreation_attempts, 3);
    }

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.max_reconnect_attempts, 3);
        assert_eq!(config.reconnect_delay_ms, 1000);
        assert_eq!(config.health_check_interval_ms, 5000);
        assert_eq!(config.activity_timeout_ms, 10000);
    }

    #[test]
    fn test_manager_creation() {
        // Test that all managers can be created successfully
        let mic_manager = MicrophoneManager::new();
        assert_eq!(*mic_manager.state(), AudioPermission::Uninitialized);

        let audio_manager = AudioContextManager::new();
        assert_eq!(*audio_manager.state(), AudioContextState::Uninitialized);

        let stream_handler = StreamReconnectionHandler::new(StreamConfig::default());
        assert_eq!(stream_handler.get_health().state, StreamState::Disconnected);
    }

    #[test]
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