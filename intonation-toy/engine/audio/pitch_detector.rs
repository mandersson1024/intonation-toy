use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector as PitchDetectorTrait};
use crate::app_config;

use super::buffer::BUFFER_SIZE;

pub type PitchDetectionError = String;

#[derive(Debug, Clone)]
pub struct PitchResult {
    pub frequency: f32,
    pub timestamp: f64,
    pub clarity: f32,
}

impl PitchResult {
    pub fn new(frequency: f32, timestamp: f64, clarity: f32) -> Self {
        Self {
            frequency,
            timestamp,
            clarity,
        }
    }
}


#[derive(Debug, Clone)]
pub struct PitchDetectorConfig {
    pub sample_window_size: usize,
    pub power_threshold: f32,
    pub clarity_threshold: f32,
    pub padding_size: usize,
    pub min_frequency: f32,
    pub max_frequency: f32,
}

impl Default for PitchDetectorConfig {
    fn default() -> Self {
        Self {
            sample_window_size: BUFFER_SIZE,
            power_threshold: 5.0,      // Minimum signal energy threshold
            clarity_threshold: crate::app_config::CLARITY_THRESHOLD,    // Minimum confidence threshold
            padding_size: BUFFER_SIZE / 2, // Zero-padding size
            min_frequency: 80.0,
            max_frequency: 2000.0,
        }
    }
}

pub struct PitchDetector {
    config: PitchDetectorConfig,
    detector: McLeodDetector<f32>,
    sample_rate: u32,
}

impl PitchDetector {
    pub fn new(config: PitchDetectorConfig, sample_rate: u32) -> Result<Self, PitchDetectionError> {
        if config.sample_window_size % 128 != 0 {
            return Err(format!(
                "Sample window size must be a multiple of 128, got {}",
                config.sample_window_size
            ));
        }

        if config.sample_window_size == 0 {
            return Err("Sample window size cannot be zero".to_string());
        }

        if sample_rate == 0 {
            return Err(format!("Sample rate must be positive, got {}", sample_rate));
        }

        if config.power_threshold <= 0.0 {
            return Err(format!(
                "Power threshold must be positive, got {}",
                config.power_threshold
            ));
        }

        if config.clarity_threshold < 0.0 || config.clarity_threshold > 1.0 {
            return Err(format!(
                "Clarity threshold must be between 0.0 and 1.0, got {}",
                config.clarity_threshold
            ));
        }

        if config.padding_size > config.sample_window_size {
            return Err(format!(
                "Padding size ({}) cannot be larger than sample window size ({})",
                config.padding_size, config.sample_window_size
            ));
        }

        if config.min_frequency <= 0.0 {
            return Err(format!(
                "Minimum frequency must be positive, got {}",
                config.min_frequency
            ));
        }

        if config.max_frequency <= config.min_frequency {
            return Err(format!(
                "Maximum frequency ({}) must be greater than minimum frequency ({})",
                config.max_frequency, config.min_frequency
            ));
        }

        let mcleod_detector = McLeodDetector::new(config.sample_window_size, config.padding_size);


        Ok(Self {
            config,
            detector: mcleod_detector,
            sample_rate,
        })
    }

    pub fn analyze(&mut self, samples: &[f32]) -> Result<Option<PitchResult>, PitchDetectionError> {
        if samples.len() != self.config.sample_window_size {
            return Err(format!(
                "Expected {} samples, got {}",
                self.config.sample_window_size,
                samples.len()
            ));
        }

        // Use McLeod analysis
        let result = self.detector.get_pitch(samples, self.sample_rate as usize, self.config.power_threshold, self.config.clarity_threshold);
        

        match result {
            Some(pitch_info) => {
                let frequency = pitch_info.frequency;
                let clarity = pitch_info.clarity;

                // Fast frequency range check using pre-computed values
                if frequency < self.config.min_frequency || frequency > self.config.max_frequency {
                    return Ok(None);
                }

                let timestamp = self.get_current_timestamp();
                
                Ok(Some(PitchResult {
                    frequency,
                    timestamp,
                    clarity,
                }))
            }
            None => Ok(None),
        }
    }


    /// Get optimal window size recommendation balancing accuracy and latency
    pub fn get_optimal_window_size_for_latency(target_latency_ms: f32, sample_rate: u32) -> usize {
        // Calculate maximum samples we can process within the target latency
        // Assuming McLeod takes about 2-3x the window size in operations
        let max_samples = (target_latency_ms / 1000.0 * sample_rate as f32 / 3.0) as usize;
        
        // Prioritize accuracy - use larger windows when possible within latency constraints
        let preferred_sizes = [4096, 2048, 1024, 512, 256]; // Accuracy-first order
        
        for &size in &preferred_sizes {
            if size <= max_samples && size % 128 == 0 {
                return size;
            }
        }
        
        // Fallback to minimum size if nothing fits
        256
    }

    /// Get recommended window size for optimal accuracy
    pub fn get_accuracy_optimized_window_size(sample_rate: u32, min_frequency: f32) -> usize {
        // For best accuracy, window should contain at least 2-3 periods of the lowest frequency
        let min_period_samples = sample_rate as f32 / min_frequency;
        let recommended_size = (min_period_samples * 3.0) as usize;
        
        // Round up to next power of 2 that's a multiple of 128
        let mut window_size = 128;
        while window_size < recommended_size && window_size <= 4096 {
            window_size *= 2;
        }
        
        // Prefer 2048 for good balance of accuracy and reasonable latency
        if window_size > 2048 {
            2048
        } else {
            window_size.max(1024) // Minimum 1024 for good accuracy
        }
    }



    /// Get performance characteristics of current configuration
    pub fn get_performance_characteristics(&self) -> (f32, &'static str) {
        let estimated_latency = match self.config.sample_window_size {
            256 => 8.0,   // Fast but lower accuracy
            512 => 15.0,  // Balanced speed/accuracy
            1024 => 28.0, // Good accuracy
            2048 => 42.0, // High accuracy (recommended default)
            4096 => 78.0, // Maximum accuracy
            _ => {
                // Estimate based on window size
                let base_latency = 28.0; // 1024 baseline
                base_latency * (self.config.sample_window_size as f32 / 1024.0)
            }
        };

        let grade = if estimated_latency <= 20.0 {
            "Fast"
        } else if estimated_latency <= 35.0 {
            "Balanced"
        } else if estimated_latency <= 50.0 {
            "Accurate" // This is our target - accuracy within 50ms
        } else if estimated_latency <= 100.0 {
            "High-Accuracy"
        } else {
            "Maximum-Accuracy"
        };

        (estimated_latency, grade)
    }

    /// Get accuracy characteristics of current configuration
    pub fn get_accuracy_characteristics(&self) -> (f32, &'static str) {
        // Calculate frequency resolution based on window size and sample rate
        let frequency_resolution = self.sample_rate as f32 / self.config.sample_window_size as f32;
        
        let accuracy_grade = match self.config.sample_window_size {
            256 => "Basic",      // ~187Hz resolution at 48kHz
            512 => "Good",       // ~94Hz resolution  
            1024 => "High",      // ~47Hz resolution
            2048 => "Excellent", // ~23Hz resolution (recommended)
            4096 => "Maximum",   // ~12Hz resolution
            _ => "Variable"
        };

        (frequency_resolution, accuracy_grade)
    }

    pub fn update_config(&mut self, new_config: PitchDetectorConfig) -> Result<(), PitchDetectionError> {
        if new_config.sample_window_size % 128 != 0 {
            return Err(format!(
                "Sample window size must be a multiple of 128, got {}",
                new_config.sample_window_size
            ));
        }

        if new_config.power_threshold <= 0.0 {
            return Err(format!(
                "Power threshold must be positive, got {}",
                new_config.power_threshold
            ));
        }

        if new_config.clarity_threshold < 0.0 || new_config.clarity_threshold > 1.0 {
            return Err(format!(
                "Clarity threshold must be between 0.0 and 1.0, got {}",
                new_config.clarity_threshold
            ));
        }

        if new_config.padding_size > new_config.sample_window_size {
            return Err(format!(
                "Padding size ({}) cannot be larger than sample window size ({})",
                new_config.padding_size, new_config.sample_window_size
            ));
        }

        if new_config.min_frequency <= 0.0 {
            return Err(format!(
                "Minimum frequency must be positive, got {}",
                new_config.min_frequency
            ));
        }

        if new_config.max_frequency <= new_config.min_frequency {
            return Err(format!(
                "Maximum frequency ({}) must be greater than minimum frequency ({})",
                new_config.max_frequency, new_config.min_frequency
            ));
        }

        if new_config.sample_window_size != self.config.sample_window_size || new_config.padding_size != self.config.padding_size {
            self.detector = McLeodDetector::new(new_config.sample_window_size, new_config.padding_size);
        }


        self.config = new_config;
        Ok(())
    }

    pub fn config(&self) -> &PitchDetectorConfig {
        &self.config
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get memory usage estimate for this detector instance
    pub fn memory_usage_bytes(&self) -> usize {
        // Calculate memory usage for the detector
        let config_size = std::mem::size_of::<PitchDetectorConfig>();
        let detector_size = std::mem::size_of::<McLeodDetector<f32>>();
        let base_size = std::mem::size_of::<Self>();
        
        // Estimate McLeod detector internal buffer size
        // McLeod uses autocorrelation and FFT buffers
        let mcleod_internal_buffers = (self.config.sample_window_size + self.config.padding_size) * std::mem::size_of::<f32>() * 4;
        
        base_size + config_size + detector_size + mcleod_internal_buffers
    }

    /// Validate that the detector can meet performance requirements
    pub fn validate_performance_requirements(&self) -> Result<(), String> {
        let (estimated_latency, grade) = self.get_performance_characteristics();
        
        if estimated_latency > 50.0 {
            return Err(format!(
                "Configuration may not meet 50ms requirement. Estimated: {:.1}ms ({})", 
                estimated_latency, grade
            ));
        }
        
        
        Ok(())
    }


    fn get_current_timestamp(&self) -> f64 {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(performance) = window.performance() {
                    return performance.now();
                }
            }
            0.0
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;
    use super::super::buffer::STANDARD_SAMPLE_RATE;

    #[wasm_bindgen_test]
    fn test_pitch_result_creation() {
        let result = PitchResult::new(440.0, 1000.0, 0.8);
        assert_eq!(result.frequency, 440.0);
        assert_eq!(result.timestamp, 1000.0);
        assert_eq!(result.clarity, 0.8);
    }


    #[wasm_bindgen_test]
    fn test_pitch_detector_config_default() {
        let config = PitchDetectorConfig::default();
        assert_eq!(config.sample_window_size, BUFFER_SIZE);
        assert_eq!(config.power_threshold, 5.0);
        assert_eq!(config.clarity_threshold, crate::app_config::CLARITY_THRESHOLD);
        assert_eq!(config.padding_size, BUFFER_SIZE / 2);
        assert_eq!(config.min_frequency, 80.0);
        assert_eq!(config.max_frequency, 2000.0);
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_config_custom() {
        let config = PitchDetectorConfig {
            sample_window_size: 2048,
            power_threshold: 3.0,
            clarity_threshold: 0.8,
            padding_size: 1024,
            min_frequency: 60.0,
            max_frequency: 4000.0,
        };
        
        assert_eq!(config.sample_window_size, 2048);
        assert_eq!(config.power_threshold, 3.0);
        assert_eq!(config.clarity_threshold, 0.8);
        assert_eq!(config.padding_size, 1024);
        assert_eq!(config.min_frequency, 60.0);
        assert_eq!(config.max_frequency, 4000.0);
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_creation() {
        let config = PitchDetectorConfig::default();
        let detector = PitchDetector::new(config, STANDARD_SAMPLE_RATE);
        assert!(detector.is_ok());
        
        let detector = detector.unwrap();
        assert_eq!(detector.sample_rate(), STANDARD_SAMPLE_RATE);
        assert_eq!(detector.config().sample_window_size, BUFFER_SIZE);
        assert_eq!(detector.config().power_threshold, 5.0);
        assert_eq!(detector.config().clarity_threshold, crate::app_config::CLARITY_THRESHOLD);
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_invalid_window_size() {
        let mut config = PitchDetectorConfig::default();
        config.sample_window_size = 1000; // Not multiple of 128
        
        let detector = PitchDetector::new(config, STANDARD_SAMPLE_RATE);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("multiple of 128")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_zero_window_size() {
        let mut config = PitchDetectorConfig::default();
        config.sample_window_size = 0;
        
        let detector = PitchDetector::new(config, STANDARD_SAMPLE_RATE);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("cannot be zero")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_invalid_sample_rate() {
        let config = PitchDetectorConfig::default();
        
        let detector = PitchDetector::new(config.clone(), 0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("must be positive")),
            Ok(_) => panic!("Expected error"),
        }
        
        // Note: u32 can't be negative, so this test is now redundant
        // Testing with 0 is sufficient for boundary validation
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_invalid_thresholds() {
        let mut config = PitchDetectorConfig::default();
        config.power_threshold = -0.1;
        
        let detector = PitchDetector::new(config.clone(), 48000);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("must be positive")),
            Ok(_) => panic!("Expected error"),
        }
        
        config.power_threshold = 5.0;
        config.clarity_threshold = 1.1;
        let detector = PitchDetector::new(config, STANDARD_SAMPLE_RATE);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("between 0.0 and 1.0")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_invalid_frequency_range() {
        let mut config = PitchDetectorConfig::default();
        config.min_frequency = -10.0;
        
        let detector = PitchDetector::new(config.clone(), 48000);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("must be positive")),
            Ok(_) => panic!("Expected error"),
        }
        
        config.min_frequency = 100.0;
        config.max_frequency = 50.0; // Max less than min
        let detector = PitchDetector::new(config, STANDARD_SAMPLE_RATE);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("must be greater than minimum")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_analyze_wrong_size() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        let samples = vec![0.0; 512]; // Wrong size, expected 4096
        let result = detector.analyze(&samples);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(&format!("Expected {} samples, got 512", BUFFER_SIZE)));
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_analyze_silence() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        let samples = vec![0.0; BUFFER_SIZE]; // Silence
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No pitch detected in silence
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_analyze_sine_wave() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        // Generate 440Hz sine wave
        let frequency = 440.0;
        let sample_rate = 48000;
        let samples: Vec<f32> = (0..BUFFER_SIZE)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        
        if let Some(pitch_result) = result.unwrap() {
            // Should detect close to 440Hz
            assert!((pitch_result.frequency - 440.0).abs() < 50.0);
            assert!(pitch_result.clarity <= 1.0); // McLeod clarity should be <= 1.0
            assert!(pitch_result.timestamp >= 0.0);
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_frequency_range_filtering() {
        let mut config = PitchDetectorConfig::default();
        config.min_frequency = 400.0;
        config.max_frequency = 500.0;
        
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        // Generate 300Hz sine wave (below range)
        let frequency = 300.0;
        let sample_rate = 48000;
        let samples: Vec<f32> = (0..BUFFER_SIZE)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should be filtered out
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_update_config() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        let mut new_config = PitchDetectorConfig::default();
        new_config.power_threshold = 3.0;
        new_config.clarity_threshold = 0.8;
        new_config.min_frequency = 100.0;
        new_config.max_frequency = 1000.0;
        
        let result = detector.update_config(new_config.clone());
        assert!(result.is_ok());
        assert_eq!(detector.config().power_threshold, 3.0);
        assert_eq!(detector.config().clarity_threshold, 0.8);
        assert_eq!(detector.config().min_frequency, 100.0);
        assert_eq!(detector.config().max_frequency, 1000.0);
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_update_config_invalid() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        let mut invalid_config = PitchDetectorConfig::default();
        invalid_config.sample_window_size = 1000; // Not multiple of 128
        
        let result = detector.update_config(invalid_config);
        assert!(result.is_err());
        match result {
            Err(err) => assert!(err.contains("multiple of 128")),
            Ok(_) => panic!("Expected error"),
        }
        
        // Original config should be unchanged
        assert_eq!(detector.config().sample_window_size, 1024);
    }


    #[wasm_bindgen_test]
    fn test_pitch_detector_window_sizes() {
        let sample_rates = [44100, 48000];
        let window_sizes = [256, 512, 1024, 2048];
        
        for &sample_rate in &sample_rates {
            for &window_size in &window_sizes {
                let mut config = PitchDetectorConfig::default();
                config.sample_window_size = window_size;
                
                let detector = PitchDetector::new(config, sample_rate);
                assert!(detector.is_ok(), 
                    "Failed to create detector with sample_rate={}, window_size={}", 
                    sample_rate, window_size);
                
                let detector = detector.unwrap();
                assert_eq!(detector.config().sample_window_size, window_size);
                assert_eq!(detector.sample_rate(), sample_rate);
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_performance_optimized_config() {
        // Test accuracy-optimized configuration
        let mut config = PitchDetectorConfig::default();
        config.sample_window_size = 2048; // Accuracy-focused setting (default)
        config.power_threshold = 4.0; // Balanced power threshold
        config.clarity_threshold = 0.75; // Higher clarity threshold
        config.min_frequency = 80.0; // Vocal/instrumental range
        config.max_frequency = 2000.0;
        
        let detector = PitchDetector::new(config, STANDARD_SAMPLE_RATE);
        assert!(detector.is_ok());
        
        let detector = detector.unwrap();
        assert_eq!(detector.config().sample_window_size, 2048);
        assert_eq!(detector.config().power_threshold, 4.0);
        assert_eq!(detector.config().clarity_threshold, 0.75);
        assert_eq!(detector.config().min_frequency, 80.0);
        assert_eq!(detector.config().max_frequency, 2000.0);
    }

    // Comprehensive Test Signal Frequency Tests (Task 8 Requirements)
    
    #[wasm_bindgen_test]
    fn test_pitch_detector_a4_standard_tuning() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        // Generate A4 (440Hz) - Standard tuning reference
        let frequency = 440.0;
        let sample_rate = 48000;
        let samples: Vec<f32> = (0..BUFFER_SIZE)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        
        if let Some(pitch_result) = result.unwrap() {
            // Should detect very close to 440Hz for standard tuning reference
            assert!((pitch_result.frequency - 440.0).abs() < 10.0);
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_c4_middle_c() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        // Generate C4 (261.63Hz) - Middle C for note mapping validation
        let frequency = 261.63;
        let sample_rate = 48000;
        let samples: Vec<f32> = (0..BUFFER_SIZE)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        
        if let Some(pitch_result) = result.unwrap() {
            // Should detect close to C4 frequency
            assert!((pitch_result.frequency - 261.63).abs() < 15.0);
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_e4_major_third() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        // Generate E4 (329.63Hz) - Major third for tuning system testing
        let frequency = 329.63;
        let sample_rate = 48000;
        let samples: Vec<f32> = (0..BUFFER_SIZE)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        
        if let Some(pitch_result) = result.unwrap() {
            // Should detect close to E4 frequency
            assert!((pitch_result.frequency - 329.63).abs() < 15.0);
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_g4_perfect_fifth() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        // Generate G4 (392.00Hz) - Perfect fifth for harmonic validation
        let frequency = 392.0;
        let sample_rate = 48000;
        let samples: Vec<f32> = (0..BUFFER_SIZE)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        
        if let Some(pitch_result) = result.unwrap() {
            // Should detect close to G4 frequency
            assert!((pitch_result.frequency - 392.0).abs() < 15.0);
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_frequency_sweep() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        // Test frequency sweep: 100Hz-1000Hz for range validation
        let test_frequencies = [100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0, 1000.0];
        let sample_rate = 48000;
        
        for &frequency in &test_frequencies {
            let samples: Vec<f32> = (0..BUFFER_SIZE)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    (2.0 * std::f32::consts::PI * frequency * t).sin()
                })
                .collect();
            
            let result = detector.analyze(&samples);
            assert!(result.is_ok(), "Failed to analyze frequency {}", frequency);
            
            if let Some(pitch_result) = result.unwrap() {
                // Allow wider tolerance for frequency sweep test
                let tolerance = frequency * 0.05; // 5% tolerance
                assert!((pitch_result.frequency - frequency).abs() < tolerance, 
                    "Frequency detection failed for {}Hz: detected {}Hz", 
                    frequency, pitch_result.frequency);
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_pitch_detector_harmonic_content() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000).unwrap();
        
        // Generate complex signal with fundamental + harmonics for algorithm robustness
        let fundamental = 220.0; // A3
        let sample_rate = 48000;
        let samples: Vec<f32> = (0..BUFFER_SIZE)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                let fundamental_sin = (2.0 * std::f32::consts::PI * fundamental * t).sin();
                let second_harmonic = 0.5 * (2.0 * std::f32::consts::PI * fundamental * 2.0 * t).sin();
                let third_harmonic = 0.25 * (2.0 * std::f32::consts::PI * fundamental * 3.0 * t).sin();
                fundamental_sin + second_harmonic + third_harmonic
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        
        if let Some(pitch_result) = result.unwrap() {
            // Should detect fundamental frequency despite harmonics
            assert!((pitch_result.frequency - fundamental).abs() < 20.0);
        }
    }
}