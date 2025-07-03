// Audio module for pitch-toy application
// Handles microphone input, AudioContext management, and real-time audio processing

pub mod microphone;
pub mod context;
pub mod worklet;
pub mod stream;

use crate::modules::common::dev_log;

/// Initialize audio system
/// Returns Result to allow caller to handle initialization failures
pub fn initialize_audio_system() -> Result<(), String> {
    dev_log!("Initializing audio system");
    
    // This function assumes all critical APIs are available
    
    // Check AudioContext support
    if !context::AudioContextManager::is_supported() {
        return Err("Web Audio API not supported".to_string());
    }
    
    // AudioWorklet initialization is now available via worklet::AudioWorkletManager
    // Stream management is now available via stream::StreamReconnectionHandler
    
    dev_log!("✓ Audio system initialization completed");
    Ok(())
}

// Re-export public API
pub use microphone::{MicrophoneManager, MicrophoneState, AudioStreamInfo, AudioError};
pub use context::{AudioContextManager, AudioContextState, AudioContextConfig};
pub use worklet::{AudioWorkletManager, AudioWorkletState, AudioWorkletConfig};
pub use stream::{StreamReconnectionHandler, StreamState, StreamHealth, StreamConfig, StreamError};