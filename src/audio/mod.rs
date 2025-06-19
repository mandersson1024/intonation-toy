//! Audio processing and analysis module
//! 
//! This module handles real-time audio input, pitch detection, and musical interval
//! analysis. It's designed to run in a separate high-priority thread to maintain
//! low-latency performance.

use anyhow::Result;

pub mod engine;
pub mod pitch_detector;
pub mod interval_calc;

pub use engine::AudioEngine;
pub use pitch_detector::{PitchDetector, PitchResult};
pub use interval_calc::{IntervalCalculator, Interval};

/// Common audio constants
pub mod constants {
    /// Standard sample rate for audio processing
    pub const SAMPLE_RATE: u32 = 44100;
    
    /// Audio buffer size (samples) - targeting ~3ms latency
    pub const BUFFER_SIZE: u32 = 128;
    
    /// Number of audio channels (mono)
    pub const CHANNELS: u16 = 1;
    
    /// Standard A4 reference frequency
    pub const A4_FREQUENCY: f32 = 440.0;
    
    /// Semitone ratio (12th root of 2)
    pub const SEMITONE_RATIO: f32 = 1.0594630943592953;
    
    /// Number of cents in a semitone
    pub const CENTS_PER_SEMITONE: f32 = 100.0;
    
    /// Minimum confidence threshold for pitch detection
    pub const MIN_CONFIDENCE: f32 = 0.3;
}

/// Audio processing error types
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Audio device not available: {0}")]
    DeviceNotAvailable(String),
    
    #[error("Audio stream error: {0}")]
    StreamError(String),
    
    #[error("Pitch detection failed: {0}")]
    PitchDetectionError(String),
    
    #[error("Invalid audio configuration: {0}")]
    InvalidConfiguration(String),
}

/// Initialize audio subsystem and enumerate available devices
pub fn initialize_audio() -> Result<Vec<String>> {
    use cpal::traits::{DeviceTrait, HostTrait};
    
    let host = cpal::default_host();
    let devices: Result<Vec<String>> = host
        .input_devices()?
        .map(|device| {
            device.name()
                .map_err(|e| anyhow::anyhow!("Failed to get device name: {}", e))
        })
        .collect();
    
    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_initialization() {
        // Test that audio system can be initialized
        let result = initialize_audio();
        assert!(result.is_ok(), "Audio initialization should succeed");
        
        let devices = result.unwrap();
        println!("Available audio devices: {:?}", devices);
    }

    #[test]
    fn test_constants() {
        use constants::*;
        
        // Test that our constants make sense
        assert_eq!(SAMPLE_RATE, 44100);
        assert_eq!(CHANNELS, 1);
        assert!(BUFFER_SIZE > 0 && BUFFER_SIZE <= 1024);
        assert!((SEMITONE_RATIO - 1.0594630943592953).abs() < 1e-10);
    }
} 