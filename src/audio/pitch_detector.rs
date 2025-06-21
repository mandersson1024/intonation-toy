use pitch_detection::detector::{mcleod::McLeodDetector, yin::YINDetector, PitchDetector as PitchDetectorTrait};
use wasm_bindgen::prelude::*;

/// Pitch detection algorithm options
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PitchAlgorithm {
    YIN,
    McLeod,
}

/// Configuration for pitch detection
#[wasm_bindgen]
#[derive(Clone)]
pub struct PitchConfig {
    algorithm: PitchAlgorithm,
    sample_rate: f32,
    min_frequency: f32,
    max_frequency: f32,
    tolerance: f32, // ±5 cents tolerance
}

#[wasm_bindgen]
impl PitchConfig {
    /// Create new pitch detection configuration
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32) -> PitchConfig {
        PitchConfig {
            algorithm: PitchAlgorithm::YIN,
            sample_rate,
            min_frequency: 80.0,  // Practical musical range lower bound
            max_frequency: 2000.0, // Practical musical range upper bound
            tolerance: 5.0, // ±5 cents
        }
    }

    /// Set the pitch detection algorithm
    #[wasm_bindgen]
    pub fn set_algorithm(&mut self, algorithm: PitchAlgorithm) {
        self.algorithm = algorithm;
    }

    /// Set frequency range for validation
    #[wasm_bindgen]
    pub fn set_frequency_range(&mut self, min_freq: f32, max_freq: f32) {
        self.min_frequency = min_freq;
        self.max_frequency = max_freq;
    }

    /// Set tolerance in cents
    #[wasm_bindgen]
    pub fn set_tolerance(&mut self, tolerance: f32) {
        self.tolerance = tolerance;
    }
}

/// Pitch detection result
#[wasm_bindgen]
pub struct PitchResult {
    frequency: f32,
    clarity: f32,
    is_valid: bool,
}

#[wasm_bindgen]
impl PitchResult {
    /// Get detected frequency in Hz
    #[wasm_bindgen]
    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    /// Get clarity/confidence of detection (0.0-1.0)
    #[wasm_bindgen]
    pub fn clarity(&self) -> f32 {
        self.clarity
    }

    /// Check if result is within valid range
    #[wasm_bindgen]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
}

/// Core pitch detector using multiple algorithms
#[wasm_bindgen]
pub struct PitchDetector {
    config: PitchConfig,
    yin_detector: Option<YINDetector<f32>>,
    mcleod_detector: Option<McLeodDetector<f32>>,
}

#[wasm_bindgen]
impl PitchDetector {
    /// Create new pitch detector
    #[wasm_bindgen(constructor)]
    pub fn new(config: PitchConfig) -> PitchDetector {
        PitchDetector {
            config,
            yin_detector: None,
            mcleod_detector: None,
        }
    }

    /// Detect pitch from audio buffer
    #[wasm_bindgen]
    pub fn detect_pitch(&mut self, audio_buffer: &[f32]) -> Option<PitchResult> {
        if audio_buffer.is_empty() {
            return None;
        }

        let result = match self.config.algorithm {
            PitchAlgorithm::YIN => self.detect_with_yin(audio_buffer),
            PitchAlgorithm::McLeod => self.detect_with_mcleod(audio_buffer),
        };

        if let Some(mut result) = result {
            // Validate frequency range
            result.is_valid = self.validate_frequency(result.frequency);
            Some(result)
        } else {
            None
        }
    }

    /// Set algorithm configuration
    #[wasm_bindgen]
    pub fn set_config(&mut self, config: PitchConfig) {
        self.config = config;
        // Reset detectors to apply new configuration
        self.yin_detector = None;
        self.mcleod_detector = None;
    }
}

impl PitchDetector {
    /// Detect pitch using YIN algorithm
    fn detect_with_yin(&mut self, audio_buffer: &[f32]) -> Option<PitchResult> {
        if self.yin_detector.is_none() {
            self.yin_detector = Some(YINDetector::new(audio_buffer.len(), audio_buffer.len() / 2));
        }

        if let Some(ref mut detector) = self.yin_detector {
            if let Some(pitch) = detector.get_pitch(audio_buffer, self.config.sample_rate as usize, 5.0, 0.2) {
                Some(PitchResult {
                    frequency: pitch.frequency,
                    clarity: pitch.clarity,
                    is_valid: false, // Will be set in detect_pitch
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Detect pitch using McLeod algorithm
    fn detect_with_mcleod(&mut self, audio_buffer: &[f32]) -> Option<PitchResult> {
        if self.mcleod_detector.is_none() {
            self.mcleod_detector = Some(McLeodDetector::new(audio_buffer.len(), audio_buffer.len() / 2));
        }

        if let Some(ref mut detector) = self.mcleod_detector {
            if let Some(pitch) = detector.get_pitch(audio_buffer, self.config.sample_rate as usize, 5.0, 0.7) {
                Some(PitchResult {
                    frequency: pitch.frequency,
                    clarity: pitch.clarity,
                    is_valid: false, // Will be set in detect_pitch
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Validate frequency is within acceptable range
    fn validate_frequency(&self, frequency: f32) -> bool {
        frequency >= self.config.min_frequency && frequency <= self.config.max_frequency
    }

    /// Calculate cents difference between two frequencies
    #[allow(dead_code)]
    fn cents_difference(freq1: f32, freq2: f32) -> f32 {
        1200.0 * (freq2 / freq1).log2()
    }

    /// Check if frequency is within tolerance
    #[allow(dead_code)]
    fn is_within_tolerance(&self, detected: f32, reference: f32) -> bool {
        let cents_diff = Self::cents_difference(reference, detected).abs();
        cents_diff <= self.config.tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_config_creation() {
        let config = PitchConfig::new(44100.0);
        assert_eq!(config.sample_rate, 44100.0);
        assert_eq!(config.min_frequency, 80.0);
        assert_eq!(config.max_frequency, 2000.0);
        assert_eq!(config.tolerance, 5.0);
        assert_eq!(config.algorithm, PitchAlgorithm::YIN);
    }

    #[test]
    fn test_pitch_config_modifications() {
        let mut config = PitchConfig::new(48000.0);
        
        config.set_algorithm(PitchAlgorithm::McLeod);
        assert_eq!(config.algorithm, PitchAlgorithm::McLeod);
        
        config.set_frequency_range(100.0, 1500.0);
        assert_eq!(config.min_frequency, 100.0);
        assert_eq!(config.max_frequency, 1500.0);
        
        config.set_tolerance(10.0);
        assert_eq!(config.tolerance, 10.0);
    }

    #[test]
    fn test_frequency_validation() {
        let config = PitchConfig::new(44100.0);
        let detector = PitchDetector::new(config);

        assert!(detector.validate_frequency(440.0)); // A4 - valid
        assert!(detector.validate_frequency(80.0));  // Lower bound
        assert!(detector.validate_frequency(2000.0)); // Upper bound
        assert!(!detector.validate_frequency(50.0)); // Below range
        assert!(!detector.validate_frequency(3000.0)); // Above range
    }

    #[test]
    fn test_cents_calculation() {
        // A4 (440 Hz) to A5 (880 Hz) should be 1200 cents (1 octave)
        let cents = PitchDetector::cents_difference(440.0, 880.0);
        assert!((cents - 1200.0).abs() < 0.1);

        // A4 (440 Hz) to A#4 (466.16 Hz) should be ~100 cents (1 semitone)
        let cents = PitchDetector::cents_difference(440.0, 466.16);
        assert!((cents - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_tolerance_checking() {
        let config = PitchConfig::new(44100.0);
        let detector = PitchDetector::new(config);

        // Within 5 cents tolerance
        assert!(detector.is_within_tolerance(440.0, 441.25)); // ~4.9 cents
        assert!(detector.is_within_tolerance(440.0, 438.75)); // ~-4.9 cents
        
        // Outside tolerance
        assert!(!detector.is_within_tolerance(440.0, 445.0)); // ~20 cents
        assert!(!detector.is_within_tolerance(440.0, 435.0)); // ~-20 cents
    }

    #[test]
    fn test_pitch_result_accessors() {
        let result = PitchResult {
            frequency: 440.0,
            clarity: 0.8,
            is_valid: true,
        };

        assert_eq!(result.frequency(), 440.0);
        assert_eq!(result.clarity(), 0.8);
        assert!(result.is_valid());
    }

    #[test]
    fn test_empty_buffer_handling() {
        let config = PitchConfig::new(44100.0);
        let mut detector = PitchDetector::new(config);
        
        let empty_buffer: Vec<f32> = vec![];
        let result = detector.detect_pitch(&empty_buffer);
        assert!(result.is_none());
    }

    #[test]
    fn test_detector_algorithm_switching() {
        let mut config = PitchConfig::new(44100.0);
        let mut detector = PitchDetector::new(config.clone());

        // Test with YIN
        config.set_algorithm(PitchAlgorithm::YIN);
        detector.set_config(config.clone());

        // Test with McLeod  
        config.set_algorithm(PitchAlgorithm::McLeod);
        detector.set_config(config);

        // Should not panic or crash
        let test_buffer = vec![0.1; 1024];
        let _result = detector.detect_pitch(&test_buffer);
    }

    #[test]
    fn test_actual_pitch_detection_with_sine_wave() {
        let config = PitchConfig::new(44100.0);
        let mut detector = PitchDetector::new(config);
        
        // Generate 440Hz sine wave (2048 samples for good resolution)
        let buffer_size = 2048;
        let frequency = 440.0;
        let sample_rate = 44100.0;
        let mut test_buffer = vec![0.0; buffer_size];
        
        for i in 0..buffer_size {
            test_buffer[i] = 0.8 * (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate).sin();
        }
        
        // Debug: Check buffer characteristics
        let max_val = test_buffer.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        let rms = (test_buffer.iter().map(|&x| x * x).sum::<f32>() / buffer_size as f32).sqrt();
        println!("Buffer stats: max={:.3}, rms={:.3}, size={}", max_val, rms, buffer_size);
        
        println!("Testing YIN with {}Hz sine wave ({} samples)", frequency, buffer_size);
        
        // Test YIN directly with different parameters
        let mut yin_detector = YINDetector::new(buffer_size, buffer_size / 2);
        println!("YIN detector created: buffer_size={}, tau_max={}", buffer_size, buffer_size / 2);
        
        // Try different thresholds
        let thresholds = [0.1, 0.2, 0.3, 0.5];
        for &threshold in &thresholds {
            if let Some(pitch) = yin_detector.get_pitch(&test_buffer, buffer_size / 2, threshold, sample_rate) {
                println!("YIN (threshold={:.1}): {:.1}Hz, clarity={:.3}", threshold, pitch.frequency, pitch.clarity);
            } else {
                println!("YIN (threshold={:.1}): No detection", threshold);
            }
        }
        
        // Test McLeod directly
        let mut mcleod_detector = McLeodDetector::new(buffer_size, buffer_size / 2);
        println!("McLeod detector created: buffer_size={}, tau_max={}", buffer_size, buffer_size / 2);
        
        for &threshold in &thresholds {
            if let Some(pitch) = mcleod_detector.get_pitch(&test_buffer, buffer_size / 2, threshold, sample_rate) {
                println!("McLeod (threshold={:.1}): {:.1}Hz, clarity={:.3}", threshold, pitch.frequency, pitch.clarity);
            } else {
                println!("McLeod (threshold={:.1}): No detection", threshold);
            }
        }
        
        // Test with our wrapper
        let result = detector.detect_pitch(&test_buffer);
        
        if let Some(pitch_result) = result {
            println!("Wrapper YIN detected: {:.1}Hz, clarity: {:.3}, valid: {}", 
                pitch_result.frequency(), pitch_result.clarity(), pitch_result.is_valid());
        } else {
            println!("Wrapper YIN: No pitch detected!");
        }
    }
} 