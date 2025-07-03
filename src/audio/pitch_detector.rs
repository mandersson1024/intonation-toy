use std::fmt;
use pitch_detection::detector::{yin::YINDetector, PitchDetector as PitchDetectorTrait};

pub type PitchDetectionError = String;

#[derive(Debug, Clone)]
pub struct PitchResult {
    pub frequency: f32,
    pub confidence: f32,
    pub timestamp: f64,
    pub clarity: f32,
}

impl PitchResult {
    pub fn new(frequency: f32, confidence: f32, timestamp: f64, clarity: f32) -> Self {
        Self {
            frequency,
            confidence,
            timestamp,
            clarity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MusicalNote {
    pub note: NoteName,
    pub octave: i32,
    pub cents: f32,
    pub frequency: f32,
}

impl MusicalNote {
    pub fn new(note: NoteName, octave: i32, cents: f32, frequency: f32) -> Self {
        Self {
            note,
            octave,
            cents,
            frequency,
        }
    }
}

impl fmt::Display for MusicalNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.note, self.octave)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteName {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl fmt::Display for NoteName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteName::C => write!(f, "C"),
            NoteName::CSharp => write!(f, "C#"),
            NoteName::D => write!(f, "D"),
            NoteName::DSharp => write!(f, "D#"),
            NoteName::E => write!(f, "E"),
            NoteName::F => write!(f, "F"),
            NoteName::FSharp => write!(f, "F#"),
            NoteName::G => write!(f, "G"),
            NoteName::GSharp => write!(f, "G#"),
            NoteName::A => write!(f, "A"),
            NoteName::ASharp => write!(f, "A#"),
            NoteName::B => write!(f, "B"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TuningSystem {
    EqualTemperament { reference_pitch: f32 },
    JustIntonation { reference_pitch: f32 },
    Custom { frequency_ratios: Vec<f32> },
}

impl Default for TuningSystem {
    fn default() -> Self {
        TuningSystem::EqualTemperament {
            reference_pitch: 440.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PitchDetectorConfig {
    pub sample_window_size: usize,
    pub threshold: f32,
    pub tuning_system: TuningSystem,
    pub min_frequency: f32,
    pub max_frequency: f32,
}

impl Default for PitchDetectorConfig {
    fn default() -> Self {
        Self {
            sample_window_size: 1024,
            threshold: 0.15,
            tuning_system: TuningSystem::default(),
            min_frequency: 80.0,
            max_frequency: 2000.0,
        }
    }
}

pub struct PitchDetector {
    config: PitchDetectorConfig,
    yin_detector: YINDetector<f32>,
    sample_rate: f32,
}

impl PitchDetector {
    pub fn new(config: PitchDetectorConfig, sample_rate: f32) -> Result<Self, PitchDetectionError> {
        if config.sample_window_size % 128 != 0 {
            return Err(format!(
                "Sample window size must be a multiple of 128, got {}",
                config.sample_window_size
            ));
        }

        if config.sample_window_size == 0 {
            return Err("Sample window size cannot be zero".to_string());
        }

        if sample_rate <= 0.0 {
            return Err(format!("Sample rate must be positive, got {}", sample_rate));
        }

        if config.threshold < 0.0 || config.threshold > 1.0 {
            return Err(format!(
                "Threshold must be between 0.0 and 1.0, got {}",
                config.threshold
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

        let yin_detector = YINDetector::new(config.sample_window_size, 0);

        Ok(Self {
            config,
            yin_detector,
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

        let result = self.yin_detector.get_pitch(samples, self.sample_rate as usize, 0.0, self.config.threshold);

        match result {
            Some(pitch_info) => {
                let frequency = pitch_info.frequency;
                let clarity = pitch_info.clarity;

                if frequency < self.config.min_frequency || frequency > self.config.max_frequency {
                    return Ok(None);
                }

                let confidence = self.normalize_confidence(clarity);

                if confidence < 0.5 {
                    return Ok(None);
                }

                let timestamp = self.get_current_timestamp();
                
                Ok(Some(PitchResult {
                    frequency,
                    confidence,
                    timestamp,
                    clarity,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn update_config(&mut self, new_config: PitchDetectorConfig) -> Result<(), PitchDetectionError> {
        if new_config.sample_window_size % 128 != 0 {
            return Err(format!(
                "Sample window size must be a multiple of 128, got {}",
                new_config.sample_window_size
            ));
        }

        if new_config.threshold < 0.0 || new_config.threshold > 1.0 {
            return Err(format!(
                "Threshold must be between 0.0 and 1.0, got {}",
                new_config.threshold
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

        if new_config.sample_window_size != self.config.sample_window_size {
            self.yin_detector = YINDetector::new(new_config.sample_window_size, 0);
        }

        self.config = new_config;
        Ok(())
    }

    pub fn config(&self) -> &PitchDetectorConfig {
        &self.config
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn normalize_confidence(&self, clarity: f32) -> f32 {
        let normalized = 1.0 - clarity.min(1.0).max(0.0);
        normalized.min(1.0).max(0.0)
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

    #[test]
    fn test_pitch_result_creation() {
        let result = PitchResult::new(440.0, 0.9, 1000.0, 0.8);
        assert_eq!(result.frequency, 440.0);
        assert_eq!(result.confidence, 0.9);
        assert_eq!(result.timestamp, 1000.0);
        assert_eq!(result.clarity, 0.8);
    }

    #[test]
    fn test_musical_note_creation() {
        let note = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        assert_eq!(note.note, NoteName::A);
        assert_eq!(note.octave, 4);
        assert_eq!(note.cents, 0.0);
        assert_eq!(note.frequency, 440.0);
    }

    #[test]
    fn test_musical_note_display() {
        let note = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        assert_eq!(format!("{}", note), "A4");
        
        let sharp_note = MusicalNote::new(NoteName::CSharp, 5, 0.0, 554.37);
        assert_eq!(format!("{}", sharp_note), "C#5");
    }

    #[test]
    fn test_note_name_display() {
        assert_eq!(format!("{}", NoteName::C), "C");
        assert_eq!(format!("{}", NoteName::CSharp), "C#");
        assert_eq!(format!("{}", NoteName::D), "D");
        assert_eq!(format!("{}", NoteName::DSharp), "D#");
        assert_eq!(format!("{}", NoteName::E), "E");
        assert_eq!(format!("{}", NoteName::F), "F");
        assert_eq!(format!("{}", NoteName::FSharp), "F#");
        assert_eq!(format!("{}", NoteName::G), "G");
        assert_eq!(format!("{}", NoteName::GSharp), "G#");
        assert_eq!(format!("{}", NoteName::A), "A");
        assert_eq!(format!("{}", NoteName::ASharp), "A#");
        assert_eq!(format!("{}", NoteName::B), "B");
    }

    #[test]
    fn test_tuning_system_equal_temperament() {
        let tuning = TuningSystem::EqualTemperament {
            reference_pitch: 440.0,
        };
        match tuning {
            TuningSystem::EqualTemperament { reference_pitch } => {
                assert_eq!(reference_pitch, 440.0);
            }
            _ => panic!("Expected EqualTemperament"),
        }
    }

    #[test]
    fn test_tuning_system_just_intonation() {
        let tuning = TuningSystem::JustIntonation {
            reference_pitch: 440.0,
        };
        match tuning {
            TuningSystem::JustIntonation { reference_pitch } => {
                assert_eq!(reference_pitch, 440.0);
            }
            _ => panic!("Expected JustIntonation"),
        }
    }

    #[test]
    fn test_tuning_system_custom() {
        let ratios = vec![1.0, 1.125, 1.25, 1.333, 1.5, 1.667, 1.875, 2.0];
        let tuning = TuningSystem::Custom {
            frequency_ratios: ratios.clone(),
        };
        match tuning {
            TuningSystem::Custom { frequency_ratios } => {
                assert_eq!(frequency_ratios, ratios);
            }
            _ => panic!("Expected Custom"),
        }
    }

    #[test]
    fn test_tuning_system_default() {
        let tuning = TuningSystem::default();
        match tuning {
            TuningSystem::EqualTemperament { reference_pitch } => {
                assert_eq!(reference_pitch, 440.0);
            }
            _ => panic!("Expected EqualTemperament as default"),
        }
    }

    #[test]
    fn test_pitch_detector_config_default() {
        let config = PitchDetectorConfig::default();
        assert_eq!(config.sample_window_size, 1024);
        assert_eq!(config.threshold, 0.15);
        assert_eq!(config.min_frequency, 80.0);
        assert_eq!(config.max_frequency, 2000.0);
        
        match config.tuning_system {
            TuningSystem::EqualTemperament { reference_pitch } => {
                assert_eq!(reference_pitch, 440.0);
            }
            _ => panic!("Expected EqualTemperament as default tuning system"),
        }
    }

    #[test]
    fn test_pitch_detector_config_custom() {
        let custom_tuning = TuningSystem::JustIntonation {
            reference_pitch: 432.0,
        };
        let config = PitchDetectorConfig {
            sample_window_size: 2048,
            threshold: 0.2,
            tuning_system: custom_tuning,
            min_frequency: 60.0,
            max_frequency: 4000.0,
        };
        
        assert_eq!(config.sample_window_size, 2048);
        assert_eq!(config.threshold, 0.2);
        assert_eq!(config.min_frequency, 60.0);
        assert_eq!(config.max_frequency, 4000.0);
        
        match config.tuning_system {
            TuningSystem::JustIntonation { reference_pitch } => {
                assert_eq!(reference_pitch, 432.0);
            }
            _ => panic!("Expected JustIntonation"),
        }
    }

    #[test]
    fn test_note_name_equality() {
        assert_eq!(NoteName::A, NoteName::A);
        assert_ne!(NoteName::A, NoteName::B);
        assert_ne!(NoteName::A, NoteName::ASharp);
    }

    #[test]
    fn test_musical_note_equality() {
        let note1 = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        let note2 = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        let note3 = MusicalNote::new(NoteName::A, 5, 0.0, 880.0);
        
        assert_eq!(note1, note2);
        assert_ne!(note1, note3);
    }

    #[test]
    fn test_pitch_detector_creation() {
        let config = PitchDetectorConfig::default();
        let detector = PitchDetector::new(config, 48000.0);
        assert!(detector.is_ok());
        
        let detector = detector.unwrap();
        assert_eq!(detector.sample_rate(), 48000.0);
        assert_eq!(detector.config().sample_window_size, 1024);
        assert_eq!(detector.config().threshold, 0.15);
    }

    #[test]
    fn test_pitch_detector_invalid_window_size() {
        let mut config = PitchDetectorConfig::default();
        config.sample_window_size = 1000; // Not multiple of 128
        
        let detector = PitchDetector::new(config, 48000.0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("multiple of 128")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_pitch_detector_zero_window_size() {
        let mut config = PitchDetectorConfig::default();
        config.sample_window_size = 0;
        
        let detector = PitchDetector::new(config, 48000.0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("cannot be zero")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_pitch_detector_invalid_sample_rate() {
        let config = PitchDetectorConfig::default();
        
        let detector = PitchDetector::new(config.clone(), 0.0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("must be positive")),
            Ok(_) => panic!("Expected error"),
        }
        
        let detector = PitchDetector::new(config, -1000.0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("must be positive")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_pitch_detector_invalid_threshold() {
        let mut config = PitchDetectorConfig::default();
        config.threshold = -0.1;
        
        let detector = PitchDetector::new(config.clone(), 48000.0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("between 0.0 and 1.0")),
            Ok(_) => panic!("Expected error"),
        }
        
        config.threshold = 1.1;
        let detector = PitchDetector::new(config, 48000.0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("between 0.0 and 1.0")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_pitch_detector_invalid_frequency_range() {
        let mut config = PitchDetectorConfig::default();
        config.min_frequency = -10.0;
        
        let detector = PitchDetector::new(config.clone(), 48000.0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("must be positive")),
            Ok(_) => panic!("Expected error"),
        }
        
        config.min_frequency = 100.0;
        config.max_frequency = 50.0; // Max less than min
        let detector = PitchDetector::new(config, 48000.0);
        assert!(detector.is_err());
        match detector {
            Err(err) => assert!(err.contains("must be greater than minimum")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_pitch_detector_analyze_wrong_size() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000.0).unwrap();
        
        let samples = vec![0.0; 512]; // Wrong size, expected 1024
        let result = detector.analyze(&samples);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 1024 samples, got 512"));
    }

    #[test]
    fn test_pitch_detector_analyze_silence() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000.0).unwrap();
        
        let samples = vec![0.0; 1024]; // Silence
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No pitch detected in silence
    }

    #[test]
    fn test_pitch_detector_analyze_sine_wave() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000.0).unwrap();
        
        // Generate 440Hz sine wave
        let frequency = 440.0;
        let sample_rate = 48000.0;
        let samples: Vec<f32> = (0..1024)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        
        if let Some(pitch_result) = result.unwrap() {
            // Should detect close to 440Hz
            assert!((pitch_result.frequency - 440.0).abs() < 50.0);
            assert!(pitch_result.confidence > 0.5);
            assert!(pitch_result.clarity <= 1.0); // YIN clarity should be <= 1.0
            assert!(pitch_result.timestamp >= 0.0);
        }
    }

    #[test]
    fn test_pitch_detector_frequency_range_filtering() {
        let mut config = PitchDetectorConfig::default();
        config.min_frequency = 400.0;
        config.max_frequency = 500.0;
        
        let mut detector = PitchDetector::new(config, 48000.0).unwrap();
        
        // Generate 300Hz sine wave (below range)
        let frequency = 300.0;
        let sample_rate = 48000.0;
        let samples: Vec<f32> = (0..1024)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should be filtered out
    }

    #[test]
    fn test_pitch_detector_update_config() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000.0).unwrap();
        
        let mut new_config = PitchDetectorConfig::default();
        new_config.threshold = 0.2;
        new_config.min_frequency = 100.0;
        new_config.max_frequency = 1000.0;
        
        let result = detector.update_config(new_config.clone());
        assert!(result.is_ok());
        assert_eq!(detector.config().threshold, 0.2);
        assert_eq!(detector.config().min_frequency, 100.0);
        assert_eq!(detector.config().max_frequency, 1000.0);
    }

    #[test]
    fn test_pitch_detector_update_config_invalid() {
        let config = PitchDetectorConfig::default();
        let mut detector = PitchDetector::new(config, 48000.0).unwrap();
        
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

    #[test]
    fn test_confidence_normalization() {
        let config = PitchDetectorConfig::default();
        let detector = PitchDetector::new(config, 48000.0).unwrap();
        
        // Test confidence normalization (1.0 - clarity)
        assert_eq!(detector.normalize_confidence(0.0), 1.0);
        assert_eq!(detector.normalize_confidence(1.0), 0.0);
        assert_eq!(detector.normalize_confidence(0.5), 0.5);
        
        // Test bounds
        assert_eq!(detector.normalize_confidence(-1.0), 1.0);
        assert_eq!(detector.normalize_confidence(2.0), 0.0);
    }

    #[test]
    fn test_pitch_detector_window_sizes() {
        let sample_rates = [44100.0, 48000.0];
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

    #[test]
    fn test_pitch_detector_performance_optimized_config() {
        // Test production-optimized configuration
        let mut config = PitchDetectorConfig::default();
        config.sample_window_size = 1024; // Production setting
        config.threshold = 0.15; // Production threshold
        config.min_frequency = 80.0; // Vocal/instrumental range
        config.max_frequency = 2000.0;
        
        let detector = PitchDetector::new(config, 48000.0);
        assert!(detector.is_ok());
        
        let detector = detector.unwrap();
        assert_eq!(detector.config().sample_window_size, 1024);
        assert_eq!(detector.config().threshold, 0.15);
        assert_eq!(detector.config().min_frequency, 80.0);
        assert_eq!(detector.config().max_frequency, 2000.0);
    }
}