use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser's Web API
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to provide `println!`-style syntax for `console.log` logging.
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        log(&format!( $( $t )* ))
    }
}

// Import our audio module
pub mod audio;

// Import browser compatibility and error management
pub mod browser_compat;
pub mod error_manager;
pub mod performance_monitor;

// Legacy modules (active during transition)
pub mod legacy;

// Re-export the audio engine for direct access from JavaScript
pub use audio::engine::AudioEngine;

// Re-export pitch detection components for JavaScript integration
pub use audio::pitch_detector::{PitchAlgorithm, PitchConfig, PitchDetector, PitchResult};

// Backward compatibility exports for existing legacy code
pub mod services {
    pub use crate::legacy::services::*;
}

pub mod components {
    pub use crate::legacy::components::*;
}

pub mod hooks {
    pub use crate::legacy::hooks::*;
}

// Re-export services for Yew integration
pub use legacy::services::{ErrorManager as NewErrorManager, ApplicationError, ErrorSeverity, RecoveryStrategy};

// Re-export components for easy access
pub use legacy::components::{
    ErrorDisplayComponent, 
    FallbackUIComponent, ErrorToastComponent, ErrorToastContainer
};

// Re-export hooks for easy access
pub use legacy::hooks::{use_error_handler, use_microphone_permission, PermissionState};

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM Audio Processing Module Initialized");
}

/// JavaScript-callable pitch detection function
/// Returns frequency in Hz or -1 if no pitch detected
#[wasm_bindgen]
pub fn detect_pitch(audio_buffer: &[f32], sample_rate: f32, algorithm: PitchAlgorithm) -> f32 {
    let config = PitchConfig::new(sample_rate);
    let mut config = config;
    config.set_algorithm(algorithm);
    
    let mut detector = PitchDetector::new(config);
    
    match detector.detect_pitch(audio_buffer) {
        Some(result) if result.is_valid() => result.frequency(),
        _ => -1.0, // Invalid or no pitch detected
    }
}

/// JavaScript-callable pitch detection with full result information
#[wasm_bindgen]
pub fn detect_pitch_detailed(audio_buffer: &[f32], sample_rate: f32, algorithm: PitchAlgorithm) -> Option<PitchResult> {
    let config = PitchConfig::new(sample_rate);
    let mut config = config;
    config.set_algorithm(algorithm);
    
    let mut detector = PitchDetector::new(config);
    detector.detect_pitch(audio_buffer)
}

// Core modules
pub mod modules;
pub mod types;
pub mod themes;
