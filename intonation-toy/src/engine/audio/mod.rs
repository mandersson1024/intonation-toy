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
pub mod permission;
pub mod buffer;
pub mod commands;
pub mod pitch_detector;
pub mod pitch_analyzer;
pub mod volume_detector;
pub mod signal_generator;
pub mod message_protocol;
pub mod data_types;
pub mod tuning_fork_node;
pub mod test_signal_node;

use crate::common::dev_log;

use std::cell::RefCell;
use std::rc::Rc;

// Global audio context manager for application-wide access
// TODO: FUTURE REFACTORING - Remove this global variable and replace with dependency injection through AudioSystemContext.
// This is a planned future task. Do NOT refactor this during unrelated work.
// See docs/global_variables_refactoring_guide.md for refactoring strategy.
thread_local! {
    static AUDIO_CONTEXT_MANAGER: RefCell<Option<Rc<RefCell<context::AudioContextManager>>>> = const { RefCell::new(None) };
}

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
/// let context = initialize_audio_system_with_interfaces().await?;
/// ```
pub async fn initialize_audio_system_with_interfaces() -> Result<context::AudioSystemContext, String> {
    dev_log!("Initializing audio system with three-layer architecture interfaces");
    
    // Check AudioContext support
    if !context::AudioContextManager::is_supported() {
        return Err("Web Audio API not supported".to_string());
    }
    
    // Create AudioSystemContext using return-based pattern
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
    
    // Create AudioSystemContext using return-based pattern
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

// Re-export public API for external usage
pub use microphone::MicrophoneManager;
pub use context::{AudioSystemContext, convert_volume_data, convert_pitch_data, merge_audio_analysis, AudioDevices};
pub use worklet::AudioWorkletState;
pub(crate) use commands::register_audio_commands;
pub use signal_generator::{SignalGeneratorConfig, TuningForkConfig};
pub use data_types::{VolumeLevelData, PitchData, AudioWorkletStatus};
pub use permission::AudioPermission;
pub use tuning_fork_node::TuningForkAudioNode;
pub use test_signal_node::TestSignalAudioNode;

// Private re-exports for internal module use only
use microphone::{AudioError};
use context::{AudioContextManager, AudioContextState};
use volume_detector::{VolumeDetector, VolumeAnalysis};

