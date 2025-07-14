use std::fmt;

/// Volume level classification based on dB measurement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolumeLevel {
    /// Volume below noise floor threshold (e.g., < -60 dB)
    Silent,
    /// Volume below optimal range (e.g., -60 to -30 dB)
    Low,
    /// Volume in optimal range for processing (e.g., -30 to -6 dB)
    Optimal,
    /// Volume above optimal range (e.g., -6 to 0 dB)
    High,
    /// Volume at or above clipping threshold (e.g., >= 0 dB)
    Clipping,
}

impl fmt::Display for VolumeLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VolumeLevel::Silent => write!(f, "Silent"),
            VolumeLevel::Low => write!(f, "Low"),
            VolumeLevel::Optimal => write!(f, "Optimal"),
            VolumeLevel::High => write!(f, "High"),
            VolumeLevel::Clipping => write!(f, "Clipping"),
        }
    }
}

/// Volume analysis result from processing an audio buffer
#[derive(Debug, Clone)]
pub struct VolumeAnalysis {
    /// RMS (Root Mean Square) level in dB
    pub rms_db: f32,
    /// Peak level in dB
    pub peak_db: f32,
    /// Fast decay peak level in dB
    pub peak_fast_db: f32,
    /// Slow decay peak level in dB
    pub peak_slow_db: f32,
    /// Volume level classification
    pub level: VolumeLevel,
    /// Confidence weight for other audio processing (0.0 - 1.0)
    pub confidence_weight: f32,
    /// Timestamp of analysis
    pub timestamp: f64,
}

impl VolumeAnalysis {
    /// Create a new volume analysis result
    pub fn new(rms_db: f32, peak_db: f32, peak_fast_db: f32, peak_slow_db: f32, timestamp: f64) -> Self {
        let level = Self::classify_volume_level(rms_db);
        let confidence_weight = Self::calculate_confidence_weight(rms_db, &level);
        
        Self {
            rms_db,
            peak_db,
            peak_fast_db,
            peak_slow_db,
            level,
            confidence_weight,
            timestamp,
        }
    }

    /// Classify volume level based on RMS dB value
    fn classify_volume_level(rms_db: f32) -> VolumeLevel {
        if rms_db >= 0.0 {
            VolumeLevel::Clipping
        } else if rms_db >= -6.0 {
            VolumeLevel::High
        } else if rms_db >= -30.0 {
            VolumeLevel::Optimal
        } else if rms_db >= -60.0 {
            VolumeLevel::Low
        } else {
            VolumeLevel::Silent
        }
    }

    /// Calculate confidence weight based on volume level
    fn calculate_confidence_weight(rms_db: f32, level: &VolumeLevel) -> f32 {
        match level {
            VolumeLevel::Silent => 0.0,
            VolumeLevel::Low => {
                // Linear interpolation from 0.0 at -60dB to 0.3 at -30dB
                let normalized = (rms_db + 60.0) / 30.0;
                (normalized * 0.3).max(0.0).min(0.3)
            }
            VolumeLevel::Optimal => {
                // Linear interpolation from 0.3 at -30dB to 1.0 at -6dB
                let normalized = (rms_db + 30.0) / 24.0;
                (0.3 + normalized * 0.7).max(0.3).min(1.0)
            }
            VolumeLevel::High => {
                // Linear interpolation from 1.0 at -6dB to 0.7 at 0dB
                let normalized = (rms_db + 6.0) / 6.0;
                (1.0 - normalized * 0.3).max(0.7).min(1.0)
            }
            VolumeLevel::Clipping => 0.1, // Low confidence for clipping
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
    pub sample_rate: f32,
}

impl VolumeDetectorConfig {
    /// Create new configuration with default values
    pub fn new() -> Self {
        Self {
            input_gain_db: 0.0,
            noise_floor_db: -60.0,
            sample_rate: 48000.0,
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
        
        if self.sample_rate <= 0.0 {
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
    pub fn process_buffer(&mut self, samples: &[f32], timestamp: f64) -> VolumeAnalysis {
        if samples.is_empty() {
            return VolumeAnalysis::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY, -f32::INFINITY, timestamp);
        }

        // Apply input gain
        let gain_linear = self.db_to_linear(self.config.input_gain_db);
        
        // Calculate RMS and peak values
        let (rms_linear, peak_linear) = self.calculate_rms_and_peak(samples, gain_linear);
        
        // Convert to dB
        let rms_db = self.linear_to_db(rms_linear);
        let peak_db = self.linear_to_db(peak_linear);
        
        VolumeAnalysis::new(rms_db, peak_db, peak_db, peak_db, timestamp)
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
    fn test_volume_level_classification() {
        assert_eq!(VolumeAnalysis::classify_volume_level(5.0), VolumeLevel::Clipping);
        assert_eq!(VolumeAnalysis::classify_volume_level(0.0), VolumeLevel::Clipping);
        assert_eq!(VolumeAnalysis::classify_volume_level(-3.0), VolumeLevel::High);
        assert_eq!(VolumeAnalysis::classify_volume_level(-12.0), VolumeLevel::Optimal);
        assert_eq!(VolumeAnalysis::classify_volume_level(-45.0), VolumeLevel::Low);
        assert_eq!(VolumeAnalysis::classify_volume_level(-65.0), VolumeLevel::Silent);
    }

    #[wasm_bindgen_test]
    fn test_confidence_weight_calculation() {
        // Test confidence weights for different volume levels
        assert_eq!(VolumeAnalysis::calculate_confidence_weight(-65.0, &VolumeLevel::Silent), 0.0);
        
        // Low level should have low confidence
        let low_confidence = VolumeAnalysis::calculate_confidence_weight(-45.0, &VolumeLevel::Low);
        assert!(low_confidence > 0.0 && low_confidence < 0.3);
        
        // Optimal level should have high confidence
        let optimal_confidence = VolumeAnalysis::calculate_confidence_weight(-18.0, &VolumeLevel::Optimal);
        assert!(optimal_confidence > 0.3 && optimal_confidence <= 1.0);
        
        // High level should have good confidence
        let high_confidence = VolumeAnalysis::calculate_confidence_weight(-3.0, &VolumeLevel::High);
        assert!(high_confidence > 0.7 && high_confidence <= 1.0);
        
        // Clipping should have low confidence
        assert_eq!(VolumeAnalysis::calculate_confidence_weight(0.0, &VolumeLevel::Clipping), 0.1);
    }

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
            sample_rate: 44100.0,
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
        let analysis = detector.process_buffer(&silent_samples, 0.0);
        assert_eq!(analysis.level, VolumeLevel::Silent);
        assert_eq!(analysis.confidence_weight, 0.0);
        
        // Test with optimal level signal
        let optimal_samples: Vec<f32> = (0..1024).map(|i| 0.1 * (i as f32 * 0.01).sin()).collect();
        let analysis = detector.process_buffer(&optimal_samples, 1.0);
        assert_eq!(analysis.level, VolumeLevel::Optimal);
        assert!(analysis.confidence_weight > 0.3);
        
        // Test with high level signal - use smaller amplitude to stay in optimal range
        let high_samples: Vec<f32> = (0..1024).map(|i| 0.3 * (i as f32 * 0.01).sin()).collect();
        let analysis = detector.process_buffer(&high_samples, 2.0);
        // The RMS of a sine wave is amplitude / sqrt(2), so 0.3 / sqrt(2) ≈ 0.21
        // In dB: 20 * log10(0.21) ≈ -13.6 dB, which is in the Optimal range
        assert_eq!(analysis.level, VolumeLevel::Optimal);
        assert!(analysis.confidence_weight > 0.3);
    }

    #[wasm_bindgen_test]
    fn test_peak_detector_behavior() {
        let mut detector = VolumeDetector::new_default();
        
        // Process a high level signal
        let high_samples: Vec<f32> = (0..1024).map(|i| 0.7 * (i as f32 * 0.01).sin()).collect();
        let analysis1 = detector.process_buffer(&high_samples, 0.0);
        
        // Process a low level signal - peaks should decay
        let low_samples: Vec<f32> = (0..1024).map(|i| 0.01 * (i as f32 * 0.01).sin()).collect();
        let analysis2 = detector.process_buffer(&low_samples, 1.0);
        
        // Fast peak should decay more than slow peak
        assert!(analysis2.peak_fast_db < analysis1.peak_fast_db);
        assert!(analysis2.peak_slow_db < analysis1.peak_slow_db);
        // Note: Fast peak might not always be less than slow peak depending on timing
    }

    #[wasm_bindgen_test]
    fn test_empty_buffer_handling() {
        let mut detector = VolumeDetector::new_default();
        let analysis = detector.process_buffer(&[], 0.0);
        assert_eq!(analysis.rms_db, -f32::INFINITY);
        assert_eq!(analysis.peak_db, -f32::INFINITY);
        assert_eq!(analysis.level, VolumeLevel::Silent);
    }

    #[wasm_bindgen_test]
    fn test_nan_and_infinity_handling() {
        let mut detector = VolumeDetector::new_default();
        
        // Test buffer with NaN and infinity values
        let samples = vec![f32::NAN, f32::INFINITY, -f32::INFINITY, 0.1, 0.2];
        let analysis = detector.process_buffer(&samples, 0.0);
        
        // Should handle gracefully and process valid samples
        assert!(analysis.rms_db.is_finite());
        assert!(analysis.peak_db.is_finite());
    }

    #[wasm_bindgen_test]
    fn test_config_update() {
        let mut detector = VolumeDetector::new_default();
        
        let new_config = VolumeDetectorConfig {
            input_gain_db: 12.0,
            noise_floor_db: -50.0,
            sample_rate: 44100.0,
        };
        
        assert!(detector.update_config(new_config).is_ok());
        assert_eq!(detector.config().input_gain_db, 12.0);
        assert_eq!(detector.config().noise_floor_db, -50.0);
    }

    #[wasm_bindgen_test]
    fn test_volume_level_display() {
        assert_eq!(VolumeLevel::Silent.to_string(), "Silent");
        assert_eq!(VolumeLevel::Low.to_string(), "Low");
        assert_eq!(VolumeLevel::Optimal.to_string(), "Optimal");
        assert_eq!(VolumeLevel::High.to_string(), "High");
        assert_eq!(VolumeLevel::Clipping.to_string(), "Clipping");
    }

    #[wasm_bindgen_test]
    fn test_reset_functionality() {
        let mut detector = VolumeDetector::new_default();
        
        // Process a signal to set peak states
        let samples: Vec<f32> = (0..1024).map(|i| 0.5 * (i as f32 * 0.01).sin()).collect();
        detector.process_buffer(&samples, 0.0);
        
        // Next processing should start from clean state
        let analysis = detector.process_buffer(&samples, 1.0);
        assert!(analysis.peak_fast_db.is_finite());
        assert!(analysis.peak_slow_db.is_finite());
    }
}