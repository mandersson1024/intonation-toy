/// Test signal waveform types
#[derive(Debug, Clone, PartialEq)]
pub enum TestWaveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    WhiteNoise,
    PinkNoise,
}

/// Configuration for test signal generation
#[derive(Debug, Clone, PartialEq)]
pub struct TestSignalGeneratorConfig {
    /// Whether test signal is enabled
    pub enabled: bool,
    /// Signal frequency in Hz (for tonal signals)
    pub frequency: f32,
    /// Signal amplitude (0.0 - 1.0)
    pub amplitude: f32,
    /// Waveform type
    pub waveform: TestWaveform,
    /// Sample rate for generation
    pub sample_rate: f32,
}

/// Configuration for background noise generation
#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundNoiseConfig {
    /// Whether background noise is enabled
    pub enabled: bool,
    /// Noise level (0.0 - 1.0)
    pub level: f32,
    /// Type of noise to generate
    pub noise_type: TestWaveform, // Reuse TestWaveform for WhiteNoise, PinkNoise
}

impl Default for TestSignalGeneratorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 440.0,
            amplitude: 0.3,
            waveform: TestWaveform::Sine,
            sample_rate: 48000.0,
        }
    }
}

impl Default for BackgroundNoiseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: 0.0,
            noise_type: TestWaveform::WhiteNoise,
        }
    }
}
