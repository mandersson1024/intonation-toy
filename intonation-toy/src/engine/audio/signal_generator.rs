
use crate::app_config::STANDARD_SAMPLE_RATE;

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

/// Configuration for tuning fork audio generation
#[derive(Debug, Clone, PartialEq)]
pub struct TuningForkConfig {
    /// Tuning fork frequency in Hz
    pub frequency: f32,
    /// Volume amplitude (0.0-1.0)
    pub volume: f32,
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

impl Default for TuningForkConfig {
    fn default() -> Self {
        Self {
            frequency: 220.0,
            volume: 0.0,
        }
    }
}