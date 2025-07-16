// Test Signal Generator - Generate test audio signals for volume detection validation
//
// This module provides functionality to generate various test signals including sine waves,
// square waves, noise, and composite signals for testing the audio pipeline.

use std::f32::consts::PI;

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

/// Test signal generator for audio pipeline validation
pub struct TestSignalGenerator {
    config: TestSignalGeneratorConfig,
    phase: f32,
    rng_state: u32,
    pink_noise_state: [f32; 7], // State for pink noise generation
}

impl TestSignalGenerator {
    /// Create new test signal generator
    pub fn new(config: TestSignalGeneratorConfig) -> Self {
        Self {
            config,
            phase: 0.0,
            rng_state: 12345,
            pink_noise_state: [0.0; 7],
        }
    }

    /// Create with default configuration
    pub fn new_default() -> Self {
        Self::new(TestSignalGeneratorConfig::default())
    }

    /// Update generator configuration
    pub fn update_config(&mut self, config: TestSignalGeneratorConfig) {
        // Reset phase if frequency changed
        if (self.config.frequency - config.frequency).abs() > 0.1 {
            self.phase = 0.0;
        }
        
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &TestSignalGeneratorConfig {
        &self.config
    }

    /// Generate a chunk of test signal samples
    pub fn generate_chunk(&mut self, chunk_size: usize) -> Vec<f32> {
        if !self.config.enabled {
            return vec![0.0; chunk_size];
        }

        let mut samples = Vec::with_capacity(chunk_size);
        
        for _ in 0..chunk_size {
            let mut sample = self.generate_waveform_sample();
            
            // Apply amplitude scaling
            sample *= self.config.amplitude;
            
            // Clamp to valid range
            sample = sample.clamp(-1.0, 1.0);
            
            samples.push(sample);
            
            // Update phase for next sample
            self.update_phase();
        }
        
        samples
    }

    /// Generate a single waveform sample at current phase
    fn generate_waveform_sample(&mut self) -> f32 {
        match self.config.waveform {
            TestWaveform::Sine => {
                (2.0 * PI * self.phase).sin()
            }
            TestWaveform::Square => {
                if (2.0 * PI * self.phase).sin() >= 0.0 { 1.0 } else { -1.0 }
            }
            TestWaveform::Sawtooth => {
                2.0 * (self.phase - self.phase.floor()) - 1.0
            }
            TestWaveform::Triangle => {
                let t = self.phase - self.phase.floor();
                if t < 0.5 {
                    4.0 * t - 1.0
                } else {
                    3.0 - 4.0 * t
                }
            }
            TestWaveform::WhiteNoise => {
                self.generate_white_noise()
            }
            TestWaveform::PinkNoise => {
                self.generate_pink_noise()
            }
        }
    }

    /// Update phase accumulator for tonal signals
    fn update_phase(&mut self) {
        if !matches!(self.config.waveform, TestWaveform::WhiteNoise | TestWaveform::PinkNoise) {
            let phase_increment = self.config.frequency / self.config.sample_rate;
            self.phase += phase_increment;
            
            // Wrap phase to [0, 1) to prevent accumulation errors
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }

    /// Generate white noise sample
    fn generate_white_noise(&mut self) -> f32 {
        // Linear Congruential Generator
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        
        // Convert to [-1.0, 1.0] range
        (self.rng_state >> 16) as f32 / 32768.0 - 1.0
    }

    /// Generate pink noise sample (1/f noise)
    fn generate_pink_noise(&mut self) -> f32 {
        // Simplified pink noise using white noise filtered through multiple stages
        let white = self.generate_white_noise();
        
        // This is a placeholder - real pink noise would require stateful filtering
        // For now, just return attenuated white noise
        white * 0.5
    }


    /// Generate a burst of test signal for immediate testing
    pub fn generate_test_burst(&mut self, duration_ms: f32) -> Vec<f32> {
        let samples_needed = (duration_ms * self.config.sample_rate / 1000.0) as usize;
        self.generate_chunk(samples_needed)
    }

    /// Reset generator state
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.rng_state = 12345;
        self.pink_noise_state = [0.0; 7];
    }

    /// Check if current configuration would produce audible output
    pub fn is_audible(&self) -> bool {
        self.config.enabled && 
        self.config.amplitude > 0.001 &&
        self.config.frequency >= 20.0 && self.config.frequency <= 20000.0
    }

    /// Get expected RMS level in dB for current configuration
    pub fn expected_rms_db(&self) -> f32 {
        if !self.config.enabled || self.config.amplitude <= 0.0 {
            return -f32::INFINITY;
        }

        let rms_linear = match self.config.waveform {
            TestWaveform::Sine => self.config.amplitude / 2.0_f32.sqrt(), // RMS of sine wave
            TestWaveform::Square => self.config.amplitude, // RMS of square wave
            TestWaveform::Triangle => self.config.amplitude / 3.0_f32.sqrt(), // RMS of triangle wave
            TestWaveform::Sawtooth => self.config.amplitude / 3.0_f32.sqrt(), // RMS of sawtooth wave
            TestWaveform::WhiteNoise => self.config.amplitude / 3.0_f32.sqrt(), // Approximate RMS of white noise
            TestWaveform::PinkNoise => self.config.amplitude / 4.0_f32.sqrt(), // Approximate RMS of pink noise
        };

        // Convert to dB
        20.0 * rms_linear.log10()
    }

    /// Get human-readable description of current signal
    pub fn signal_description(&self) -> String {
        if !self.config.enabled {
            return "Disabled".to_string();
        }

        let waveform_name = match self.config.waveform {
            TestWaveform::Sine => "Sine",
            TestWaveform::Square => "Square",
            TestWaveform::Sawtooth => "Sawtooth", 
            TestWaveform::Triangle => "Triangle",
            TestWaveform::WhiteNoise => "White Noise",
            TestWaveform::PinkNoise => "Pink Noise",
        };

        if matches!(self.config.waveform, TestWaveform::WhiteNoise | TestWaveform::PinkNoise) {
            format!("{} at {:.1}%", waveform_name, self.config.amplitude * 100.0)
        } else {
            format!("{} {:.1} Hz at {:.1}%", waveform_name, self.config.frequency, self.config.amplitude * 100.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_test_signal_generator_creation() {
        let generator = TestSignalGenerator::new_default();
        assert!(!generator.config().enabled);
        assert_eq!(generator.config().frequency, 440.0);
        assert_eq!(generator.config().amplitude, 0.3);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_sine_wave_generation() {
        let config = TestSignalGeneratorConfig {
            enabled: true,
            frequency: 440.0,
            amplitude: 1.0,
            waveform: TestWaveform::Sine,
            sample_rate: 48000.0,
        };
        
        let mut generator = TestSignalGenerator::new(config);
        let samples = generator.generate_chunk(480); // 10ms at 48kHz
        
        assert_eq!(samples.len(), 480);
        
        // Check that we get a reasonable sine wave (non-zero values)
        let max_sample = samples.iter().fold(0.0f32, |max, &x| max.max(x.abs()));
        assert!(max_sample > 0.5); // Should have significant amplitude
        assert!(max_sample <= 1.0); // Should not clip
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_disabled_generator() {
        let config = TestSignalGeneratorConfig {
            enabled: false,
            frequency: 440.0,
            amplitude: 1.0,
            waveform: TestWaveform::Sine,
            sample_rate: 48000.0,
        };
        
        let mut generator = TestSignalGenerator::new(config);
        let samples = generator.generate_chunk(128);
        
        // Should generate silence when disabled
        assert!(samples.iter().all(|&x| x == 0.0));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_noise_generation() {
        let config = TestSignalGeneratorConfig {
            enabled: true,
            frequency: 440.0,
            amplitude: 0.5,
            waveform: TestWaveform::WhiteNoise,
            sample_rate: 48000.0,
        };
        
        let mut generator = TestSignalGenerator::new(config);
        let samples = generator.generate_chunk(1000);
        
        // Check that noise has reasonable statistical properties
        let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        let variance: f32 = samples.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / samples.len() as f32;
        
        // Mean should be close to zero
        assert!(mean.abs() < 0.1);
        
        // Should have reasonable variance (not all zeros)
        assert!(variance > 0.01);
        
        // All samples should be in valid range
        assert!(samples.iter().all(|&x| x >= -1.0 && x <= 1.0));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_expected_rms_calculation() {
        let config = TestSignalGeneratorConfig {
            enabled: true,
            frequency: 1000.0,
            amplitude: 0.5,
            waveform: TestWaveform::Sine,
            sample_rate: 48000.0,
        };
        
        let generator = TestSignalGenerator::new(config);
        let rms_db = generator.expected_rms_db();
        
        // For sine wave with amplitude 0.5, RMS should be 0.5/sqrt(2) ≈ 0.354
        // In dB: 20 * log10(0.354) ≈ -9.0 dB
        assert!((rms_db + 9.0).abs() < 1.0); // Within 1 dB
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_signal_description() {
        let config = TestSignalGeneratorConfig {
            enabled: true,
            frequency: 440.0,
            amplitude: 0.75,
            waveform: TestWaveform::Sine,
            sample_rate: 48000.0,
        };
        
        let generator = TestSignalGenerator::new(config);
        let description = generator.signal_description();
        
        assert!(description.contains("Sine"));
        assert!(description.contains("440.0"));
        assert!(description.contains("75.0"));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_is_audible() {
        let mut config = TestSignalGeneratorConfig::default();
        config.enabled = true;
        config.amplitude = 0.1;
        config.frequency = 440.0;
        
        let generator = TestSignalGenerator::new(config.clone());
        assert!(generator.is_audible());
        
        // Test disabled
        config.enabled = false;
        let generator = TestSignalGenerator::new(config.clone());
        assert!(!generator.is_audible());
        
        // Test zero amplitude
        config.enabled = true;
        config.amplitude = 0.0;
        let generator = TestSignalGenerator::new(config.clone());
        assert!(!generator.is_audible());
        
        // Test inaudible frequency
        config.amplitude = 0.1;
        config.frequency = 50000.0; // Above human hearing
        let generator = TestSignalGenerator::new(config);
        assert!(!generator.is_audible());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_config_update() {
        let mut generator = TestSignalGenerator::new_default();
        
        let new_config = TestSignalGeneratorConfig {
            enabled: true,
            frequency: 1000.0,
            amplitude: 0.8,
            waveform: TestWaveform::Square,
            sample_rate: 44100.0,
        };
        
        generator.update_config(new_config.clone());
        assert_eq!(generator.config().frequency, 1000.0);
        assert_eq!(generator.config().amplitude, 0.8);
        assert_eq!(generator.config().waveform, TestWaveform::Square);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_phase_continuity() {
        let config = TestSignalGeneratorConfig {
            enabled: true,
            frequency: 1000.0, // 1kHz for easy calculation
            amplitude: 1.0,
            waveform: TestWaveform::Sine,
            sample_rate: 48000.0,
        };
        
        let mut generator = TestSignalGenerator::new(config);
        
        // Generate two chunks and check phase continuity
        let chunk1 = generator.generate_chunk(48); // 1ms worth
        let chunk2 = generator.generate_chunk(48); // another 1ms
        
        // The end of chunk1 and start of chunk2 should be continuous
        // This is hard to test precisely due to floating point, but we can check
        // that we're not getting sudden jumps
        let last_sample = chunk1[chunk1.len() - 1];
        let first_sample = chunk2[0];
        
        // The difference should be reasonable for continuous sine wave
        let max_expected_diff = 2.0 * PI * 1000.0 / 48000.0; // One sample's worth of phase change
        assert!((first_sample - last_sample).abs() < max_expected_diff * 2.0);
    }
}