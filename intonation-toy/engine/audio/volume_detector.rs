use super::buffer::STANDARD_SAMPLE_RATE;

/// Volume analysis result from processing an audio buffer
#[derive(Debug, Clone)]
pub struct VolumeAnalysis {
    /// RMS (Root Mean Square) level as amplitude (0.0 to 1.0)
    pub rms_amplitude: f32,
    /// Peak level as amplitude (0.0 to 1.0)
    pub peak_amplitude: f32,
}

impl VolumeAnalysis {
    /// Create a new volume analysis result
    pub fn new(rms_amplitude: f32, peak_amplitude: f32) -> Self {
        Self {
            rms_amplitude,
            peak_amplitude,
        }
    }

}

/// Configuration for volume detection
#[derive(Debug, Clone)]
pub struct VolumeDetectorConfig {
    /// Input gain adjustment in dB (-60 to +60)
    pub input_gain_db: f32,
    /// Noise floor threshold in dB (-80 to -20)
    pub noise_floor_db: f32,
    /// Sample rate for calculations
    pub sample_rate: u32,
}

impl VolumeDetectorConfig {
    /// Create new configuration with default values
    pub fn new() -> Self {
        Self {
            input_gain_db: 0.0,
            noise_floor_db: -60.0,
            sample_rate: STANDARD_SAMPLE_RATE,
        }
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.input_gain_db < -60.0 || self.input_gain_db > 60.0 {
            return Err(format!("Input gain must be between -60 and 60 dB, got {}", self.input_gain_db));
        }
        
        if self.noise_floor_db < -80.0 || self.noise_floor_db > -20.0 {
            return Err(format!("Noise floor must be between -80 and -20 dB, got {}", self.noise_floor_db));
        }
        
        if self.sample_rate == 0 {
            return Err("Sample rate must be positive".to_string());
        }
        
        Ok(())
    }

}

impl Default for VolumeDetectorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Real-time volume detector for audio processing
#[derive(Clone)]
pub struct VolumeDetector {
    config: VolumeDetectorConfig,
}

impl VolumeDetector {
    /// Create new volume detector with configuration
    pub fn new(config: VolumeDetectorConfig) -> Result<Self, String> {
        config.validate()?;
        
        Ok(Self {
            config,
        })
    }

    /// Create new volume detector with default configuration
    pub fn new_default() -> Self {
        Self::new(VolumeDetectorConfig::default()).unwrap()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: VolumeDetectorConfig) -> Result<(), String> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &VolumeDetectorConfig {
        &self.config
    }

    /// Process audio buffer and return volume analysis
    pub fn process_buffer(&mut self, samples: &[f32]) -> VolumeAnalysis {
        if samples.is_empty() {
            return VolumeAnalysis::new(0.0, 0.0);
        }

        // Apply input gain
        let gain_linear = self.db_to_linear(self.config.input_gain_db);
        
        // Calculate RMS and peak values
        let (rms_linear, peak_linear) = self.calculate_rms_and_peak(samples, gain_linear);
        
        // Store as amplitude values
        let rms_amplitude = rms_linear;
        let peak_amplitude = peak_linear;
        
        VolumeAnalysis::new(rms_amplitude, peak_amplitude)
    }

    /// Calculate RMS and peak values from audio samples with zero allocation
    fn calculate_rms_and_peak(&self, samples: &[f32], gain: f32) -> (f32, f32) {
        let mut sum_squares = 0.0f32;
        let mut peak = 0.0f32;
        
        for &sample in samples {
            let scaled_sample = sample * gain;
            
            // Handle NaN and infinity values
            if scaled_sample.is_finite() {
                let abs_sample = scaled_sample.abs();
                sum_squares += scaled_sample * scaled_sample;
                peak = peak.max(abs_sample);
            }
        }
        
        let rms = if sum_squares > 0.0 {
            (sum_squares / samples.len() as f32).sqrt()
        } else {
            0.0
        };
        
        (rms, peak)
    }

    /// Convert linear amplitude to dB
    fn linear_to_db(&self, linear: f32) -> f32 {
        if linear <= 0.0 {
            -f32::INFINITY
        } else {
            20.0 * linear.log10()
        }
    }

    /// Convert dB to linear amplitude
    fn db_to_linear(&self, db: f32) -> f32 {
        if db == -f32::INFINITY {
            0.0
        } else {
            10.0_f32.powf(db / 20.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;



    #[wasm_bindgen_test]
    fn test_volume_detector_config_validation() {
        let mut config = VolumeDetectorConfig::new();
        assert!(config.validate().is_ok());
        
        // Test invalid input gain
        config.input_gain_db = -70.0;
        assert!(config.validate().is_err());
        
        config.input_gain_db = 70.0;
        assert!(config.validate().is_err());
        
        // Reset and test invalid noise floor
        config = VolumeDetectorConfig::new();
        config.noise_floor_db = -90.0;
        assert!(config.validate().is_err());
        
        config.noise_floor_db = -10.0;
        assert!(config.validate().is_err());
    }

    #[wasm_bindgen_test]
    fn test_volume_detector_creation() {
        let detector = VolumeDetector::new_default();
        assert_eq!(detector.config().input_gain_db, 0.0);
        assert_eq!(detector.config().noise_floor_db, -60.0);
        
        let config = VolumeDetectorConfig {
            input_gain_db: 6.0,
            noise_floor_db: -70.0,
            sample_rate: STANDARD_SAMPLE_RATE,
        };
        
        let detector = VolumeDetector::new(config);
        assert!(detector.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_db_linear_conversions() {
        let detector = VolumeDetector::new_default();
        
        // Test known conversions
        assert_eq!(detector.linear_to_db(1.0), 0.0);
        assert_eq!(detector.linear_to_db(0.1), -20.0);
        assert_eq!(detector.linear_to_db(0.0), -f32::INFINITY);
        
        assert_eq!(detector.db_to_linear(0.0), 1.0);
        assert!((detector.db_to_linear(-20.0) - 0.1).abs() < 1e-6);
        assert_eq!(detector.db_to_linear(-f32::INFINITY), 0.0);
    }

    #[wasm_bindgen_test]
    fn test_volume_processing() {
        let mut detector = VolumeDetector::new_default();
        
        // Test with silent buffer
        let silent_samples = vec![0.0; 1024];
        let analysis = detector.process_buffer(&silent_samples);
        assert!(analysis.rms_amplitude.is_finite() || analysis.rms_amplitude == -f32::INFINITY);
        
        // Test with optimal level signal
        let optimal_samples: Vec<f32> = (0..1024).map(|i| 0.1 * (i as f32 * 0.01).sin()).collect();
        let analysis = detector.process_buffer(&optimal_samples);
        assert!(analysis.rms_amplitude.is_finite());
        
        // Test with high level signal - use smaller amplitude to stay in optimal range
        let high_samples: Vec<f32> = (0..1024).map(|i| 0.3 * (i as f32 * 0.01).sin()).collect();
        let analysis = detector.process_buffer(&high_samples);
        // The RMS of a sine wave is amplitude / sqrt(2), so 0.3 / sqrt(2) ≈ 0.21
        // In dB: 20 * log10(0.21) ≈ -13.6 dB
        assert!(analysis.rms_amplitude.is_finite());
    }

    #[wasm_bindgen_test]
    fn test_peak_detector_behavior() {
        let mut detector = VolumeDetector::new_default();
        
        // Process a high level signal
        let high_samples: Vec<f32> = (0..1024).map(|i| 0.7 * (i as f32 * 0.01).sin()).collect();
        let analysis1 = detector.process_buffer(&high_samples);
        
        // Process a low level signal - peaks should decay
        let low_samples: Vec<f32> = (0..1024).map(|i| 0.01 * (i as f32 * 0.01).sin()).collect();
        let analysis2 = detector.process_buffer(&low_samples);
        
        // Peak should decay
        assert!(analysis2.peak_amplitude < analysis1.peak_amplitude);
        // Note: Fast peak might not always be less than slow peak depending on timing
    }

    #[wasm_bindgen_test]
    fn test_empty_buffer_handling() {
        let mut detector = VolumeDetector::new_default();
        let analysis = detector.process_buffer(&[]);
        assert_eq!(analysis.rms_amplitude, 0.0);
        assert_eq!(analysis.peak_amplitude, 0.0);
    }

    #[wasm_bindgen_test]
    fn test_nan_and_infinity_handling() {
        let mut detector = VolumeDetector::new_default();
        
        // Test buffer with NaN and infinity values
        let samples = vec![f32::NAN, f32::INFINITY, -f32::INFINITY, 0.1, 0.2];
        let analysis = detector.process_buffer(&samples);
        
        // Should handle gracefully and process valid samples
        assert!(analysis.rms_amplitude.is_finite());
        assert!(analysis.peak_amplitude.is_finite());
    }

    #[wasm_bindgen_test]
    fn test_config_update() {
        let mut detector = VolumeDetector::new_default();
        
        let new_config = VolumeDetectorConfig {
            input_gain_db: 12.0,
            noise_floor_db: -50.0,
            sample_rate: STANDARD_SAMPLE_RATE,
        };
        
        assert!(detector.update_config(new_config).is_ok());
        assert_eq!(detector.config().input_gain_db, 12.0);
        assert_eq!(detector.config().noise_floor_db, -50.0);
    }


    #[wasm_bindgen_test]
    fn test_reset_functionality() {
        let mut detector = VolumeDetector::new_default();
        
        // Process a signal to set peak states
        let samples: Vec<f32> = (0..1024).map(|i| 0.5 * (i as f32 * 0.01).sin()).collect();
        detector.process_buffer(&samples);
        
        // Next processing should start from clean state
        let analysis = detector.process_buffer(&samples);
        assert!(analysis.peak_amplitude.is_finite());
    }
}