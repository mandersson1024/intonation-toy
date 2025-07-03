use std::fmt;

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
}