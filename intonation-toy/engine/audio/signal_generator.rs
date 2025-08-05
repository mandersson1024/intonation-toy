
use super::buffer::STANDARD_SAMPLE_RATE;

/// Configuration for test signal generation
#[derive(Debug, Clone, PartialEq)]
pub struct SignalGeneratorConfig {
    /// Whether test signal is enabled
    pub enabled: bool,
    /// Signal frequency in Hz (for tonal signals)
    pub frequency: f32,
    /// Signal amplitude (0.0 - 1.0)
    pub amplitude: f32,
    /// Sample rate for generation
    pub sample_rate: u32,
}

/// Configuration for root note audio generation
#[derive(Debug, Clone, PartialEq)]
pub struct RootNoteAudioConfig {
    /// Whether root note audio is enabled
    pub enabled: bool,
    /// Root note frequency in Hz
    pub frequency: f32,
}

impl Default for SignalGeneratorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 440.0,
            amplitude: 0.15,
            sample_rate: STANDARD_SAMPLE_RATE,
        }
    }
}

impl Default for RootNoteAudioConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 220.0,
        }
    }
}