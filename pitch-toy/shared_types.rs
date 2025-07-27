//! Shared data types for the pitch-toy application.
//!
//! This module contains all shared data structures used for communication
//! between the three layers of the architecture:
//! - Engine → Model: Raw audio analysis and system state
//! - Model → Presentation: Processed data ready for display
//!
//! The types are organized to facilitate clear data flow and minimize
//! duplication across the application layers.

#[derive(Debug, Clone, PartialEq)]
pub struct Volume {
    pub peak_amplitude: f32,
    pub rms_amplitude: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteName {
    C,
    DFlat,
    D,
    EFlat,
    E,
    F,
    FSharp,
    G,
    AFlat,
    A,
    BFlat,
    B,
}

/// MIDI note number type (0-127).
/// 
/// Standard MIDI note numbers where:
/// - C-1 = 0 (theoretical, below human hearing)
/// - C4 = 60 (middle C)
/// - A4 = 69 (440 Hz concert pitch)
/// - G9 = 127 (highest MIDI note)
pub type MidiNote = u8;

/// Check if a value is a valid MIDI note number (0-127).
pub fn is_valid_midi_note(value: MidiNote) -> bool {
    value <= 127
}

/// Convert a MIDI note number to a NoteName.
/// 
/// Extracts the note class from the MIDI note using modulo 12.
/// 
/// # Examples
/// ```
/// use pitch_toy::shared_types::{from_midi_note, NoteName};
/// 
/// assert_eq!(from_midi_note(60), NoteName::C);  // C4
/// assert_eq!(from_midi_note(69), NoteName::A);  // A4
/// assert_eq!(from_midi_note(127), NoteName::G); // G9
/// ```
pub fn from_midi_note(midi_note: MidiNote) -> NoteName {
    match midi_note % 12 {
        0 => NoteName::C,
        1 => NoteName::DFlat,
        2 => NoteName::D,
        3 => NoteName::EFlat,
        4 => NoteName::E,
        5 => NoteName::F,
        6 => NoteName::FSharp,
        7 => NoteName::G,
        8 => NoteName::AFlat,
        9 => NoteName::A,
        10 => NoteName::BFlat,
        11 => NoteName::B,
        _ => unreachable!(),
    }
}

/// Convert a NoteName and octave to a MIDI note number.
/// 
/// Uses MIDI octave numbering where C4 = 60 (middle C).
/// Returns None if the resulting MIDI note would be outside the valid range (0-127).
/// 
/// # Examples
/// ```
/// use pitch_toy::shared_types::{to_midi_note, NoteName};
/// 
/// assert_eq!(to_midi_note(NoteName::C, 4), Some(60));    // C4
/// assert_eq!(to_midi_note(NoteName::A, 4), Some(69));    // A4
/// assert_eq!(to_midi_note(NoteName::B, 4), Some(71));    // B4
/// assert_eq!(to_midi_note(NoteName::C, 10), None);       // C10 is out of range
/// ```
pub fn to_midi_note(note_name: NoteName, octave: i8) -> Option<MidiNote> {
    let note_offset = match note_name {
        NoteName::C => 0,
        NoteName::DFlat => 1,
        NoteName::D => 2,
        NoteName::EFlat => 3,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::FSharp => 6,
        NoteName::G => 7,
        NoteName::AFlat => 8,
        NoteName::A => 9,
        NoteName::BFlat => 10,
        NoteName::B => 11,
    };
    
    // MIDI octave -1 starts at note 0
    let midi_note = (octave + 1) as i16 * 12 + note_offset as i16;
    
    if midi_note >= 0 && midi_note <= 127 {
        Some(midi_note as MidiNote)
    } else {
        None
    }
}

/// Safely increment a MIDI note number.
/// 
/// Returns None if incrementing would exceed the valid MIDI range (127).
/// 
/// # Examples
/// ```
/// use pitch_toy::shared_types::increment_midi_note;
/// 
/// assert_eq!(increment_midi_note(69), Some(70));  // A4 to A#4
/// assert_eq!(increment_midi_note(127), None);     // G9 cannot increment
/// ```
pub fn increment_midi_note(midi_note: MidiNote) -> Option<MidiNote> {
    if midi_note < 127 {
        Some(midi_note + 1)
    } else {
        None
    }
}

/// Safely decrement a MIDI note number.
/// 
/// Returns None if decrementing would go below the valid MIDI range (0).
/// 
/// # Examples
/// ```
/// use pitch_toy::shared_types::decrement_midi_note;
/// 
/// assert_eq!(decrement_midi_note(69), Some(68));  // A4 to G#4
/// assert_eq!(decrement_midi_note(0), None);       // C-1 cannot decrement
/// ```
pub fn decrement_midi_note(midi_note: MidiNote) -> Option<MidiNote> {
    if midi_note > 0 {
        Some(midi_note - 1)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Accuracy {
    pub closest_note: NoteName,
    pub accuracy: f32, // 0.0 = perfect, 1.0 = maximum deviation
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pitch {
    Detected(f32, f32), // frequency, clarity
    NotDetected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioAnalysis {
    pub volume_level: Volume,
    pub pitch: Pitch,
    pub fft_data: Option<Vec<f32>>, // roadmap
    pub timestamp: f64,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    MicrophonePermissionDenied,
    MicrophoneNotAvailable,
    ProcessingError(String),
    BrowserApiNotSupported,
    AudioContextInitFailed,
    AudioContextSuspended,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionState {
    NotRequested,
    Requested,
    Granted,
    Denied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EngineUpdateResult {
    pub audio_analysis: Option<AudioAnalysis>,
    pub audio_errors: Vec<Error>,
    pub permission_state: PermissionState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelUpdateResult {
    pub volume: Volume,
    pub pitch: Pitch,
    pub accuracy: Accuracy,
    pub tuning_system: TuningSystem,
    pub errors: Vec<Error>,
    pub permission_state: PermissionState,
}


#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_engine_update_result_creation() {
        let test_analysis = AudioAnalysis {
            volume_level: Volume { peak_amplitude: 0.5, rms_amplitude: 0.3 },
            pitch: Pitch::Detected(440.0, 0.9),
            fft_data: None,
            timestamp: 123.456,
        };

        let test_errors = vec![Error::MicrophonePermissionDenied];

        let update_result = EngineUpdateResult {
            audio_analysis: Some(test_analysis.clone()),
            audio_errors: test_errors.clone(),
            permission_state: PermissionState::Granted,
        };

        assert_eq!(update_result.audio_analysis, Some(test_analysis));
        assert_eq!(update_result.audio_errors, test_errors);
        assert_eq!(update_result.permission_state, PermissionState::Granted);
    }

    #[wasm_bindgen_test]
    fn test_model_update_result_creation() {
        let test_volume = Volume { peak_amplitude: 0.8, rms_amplitude: 0.6 };
        let test_pitch = Pitch::Detected(440.0, 0.9);
        let test_accuracy = Accuracy { closest_note: NoteName::A, accuracy: 0.1 };
        let test_tuning_system = TuningSystem::EqualTemperament;
        let test_errors = vec![Error::ProcessingError("Test error".to_string())];

        let update_result = ModelUpdateResult {
            volume: test_volume.clone(),
            pitch: test_pitch.clone(),
            accuracy: test_accuracy.clone(),
            tuning_system: test_tuning_system.clone(),
            errors: test_errors.clone(),
            permission_state: PermissionState::Granted,
        };

        assert_eq!(update_result.volume, test_volume);
        assert_eq!(update_result.pitch, test_pitch);
        assert_eq!(update_result.accuracy, test_accuracy);
        assert_eq!(update_result.tuning_system, test_tuning_system);
        assert_eq!(update_result.errors, test_errors);
        assert_eq!(update_result.permission_state, PermissionState::Granted);
    }

    #[wasm_bindgen_test]
    fn test_volume_creation() {
        let volume = Volume { peak_amplitude: 1.0, rms_amplitude: 0.7 };
        assert_eq!(volume.peak_amplitude, 1.0);
        assert_eq!(volume.rms_amplitude, 0.7);
    }

    #[wasm_bindgen_test]
    fn test_pitch_variants() {
        let detected = Pitch::Detected(440.0, 0.9);
        let not_detected = Pitch::NotDetected;
        
        assert_ne!(detected, not_detected);
        
        if let Pitch::Detected(freq, clarity) = detected {
            assert_eq!(freq, 440.0);
            assert_eq!(clarity, 0.9);
        }
    }

    #[wasm_bindgen_test]
    fn test_permission_state_variants() {
        let states = vec![
            PermissionState::NotRequested,
            PermissionState::Requested,
            PermissionState::Granted,
            PermissionState::Denied,
        ];
        
        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i == j {
                    assert_eq!(state1, state2);
                } else {
                    assert_ne!(state1, state2);
                }
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_error_variants() {
        let errors = vec![
            Error::MicrophonePermissionDenied,
            Error::MicrophoneNotAvailable,
            Error::ProcessingError("test".to_string()),
            Error::BrowserApiNotSupported,
            Error::AudioContextInitFailed,
            Error::AudioContextSuspended,
        ];
        
        assert_eq!(errors.len(), 6);
    }


    #[wasm_bindgen_test]
    fn test_audio_analysis_creation() {
        let analysis = AudioAnalysis {
            volume_level: Volume { peak_amplitude: 0.8, rms_amplitude: 0.6 },
            pitch: Pitch::Detected(440.0, 0.9),
            fft_data: Some(vec![0.1, 0.2, 0.3]),
            timestamp: 123.456,
        };
        
        assert_eq!(analysis.volume_level.peak_amplitude, 0.8);
        assert_eq!(analysis.volume_level.rms_amplitude, 0.6);
        assert_eq!(analysis.timestamp, 123.456);
        assert!(analysis.fft_data.is_some());
    }

    #[wasm_bindgen_test]
    fn test_midi_note_validation() {
        assert!(is_valid_midi_note(0));
        assert!(is_valid_midi_note(60));
        assert!(is_valid_midi_note(127));
        assert!(!is_valid_midi_note(128));
        assert!(!is_valid_midi_note(255));
    }

    #[wasm_bindgen_test]
    fn test_from_midi_note() {
        assert_eq!(from_midi_note(60), NoteName::C);    // C4
        assert_eq!(from_midi_note(61), NoteName::DFlat); // C#4
        assert_eq!(from_midi_note(69), NoteName::A);    // A4
        assert_eq!(from_midi_note(71), NoteName::B);    // B4
        assert_eq!(from_midi_note(127), NoteName::G);   // G9
        assert_eq!(from_midi_note(0), NoteName::C);     // C-1
    }

    #[wasm_bindgen_test]
    fn test_to_midi_note() {
        assert_eq!(to_midi_note(NoteName::C, 4), Some(60));    // C4
        assert_eq!(to_midi_note(NoteName::DFlat, 4), Some(61)); // C#4
        assert_eq!(to_midi_note(NoteName::A, 4), Some(69));    // A4
        assert_eq!(to_midi_note(NoteName::B, 4), Some(71));    // B4
        
        // Test boundary cases
        assert_eq!(to_midi_note(NoteName::C, -1), Some(0));    // C-1
        assert_eq!(to_midi_note(NoteName::G, 9), Some(127));   // G9
        
        // Test out of range
        assert_eq!(to_midi_note(NoteName::C, 10), None);       // C10 is out of range
        assert_eq!(to_midi_note(NoteName::A, -2), None);       // A-2 is out of range
    }

    #[wasm_bindgen_test]
    fn test_midi_note_bidirectional_conversion() {
        // Test that converting back and forth preserves the note name
        for midi_note in 0..=127 {
            let note_name = from_midi_note(midi_note);
            let octave = (midi_note as i16 / 12) - 1;
            assert_eq!(to_midi_note(note_name, octave as i8), Some(midi_note));
        }
    }

    #[wasm_bindgen_test]
    fn test_increment_midi_note() {
        assert_eq!(increment_midi_note(0), Some(1));
        assert_eq!(increment_midi_note(69), Some(70));  // A4 to A#4
        assert_eq!(increment_midi_note(126), Some(127));
        assert_eq!(increment_midi_note(127), None);     // Cannot increment max value
    }

    #[wasm_bindgen_test]
    fn test_decrement_midi_note() {
        assert_eq!(decrement_midi_note(127), Some(126));
        assert_eq!(decrement_midi_note(69), Some(68));  // A4 to G#4
        assert_eq!(decrement_midi_note(1), Some(0));
        assert_eq!(decrement_midi_note(0), None);       // Cannot decrement min value
    }

    #[wasm_bindgen_test]
    fn test_midi_note_octave_calculation() {
        // Test specific octave mappings
        assert_eq!(to_midi_note(NoteName::C, 0), Some(12));   // C0
        assert_eq!(to_midi_note(NoteName::C, 1), Some(24));   // C1
        assert_eq!(to_midi_note(NoteName::C, 2), Some(36));   // C2
        assert_eq!(to_midi_note(NoteName::C, 3), Some(48));   // C3
        assert_eq!(to_midi_note(NoteName::C, 4), Some(60));   // C4 (middle C)
        assert_eq!(to_midi_note(NoteName::C, 5), Some(72));   // C5
    }

    #[wasm_bindgen_test]
    fn test_midi_note_all_note_names() {
        // Test all note names in octave 4
        let expected_midi_notes = vec![
            (NoteName::C, 60),
            (NoteName::DFlat, 61),
            (NoteName::D, 62),
            (NoteName::EFlat, 63),
            (NoteName::E, 64),
            (NoteName::F, 65),
            (NoteName::FSharp, 66),
            (NoteName::G, 67),
            (NoteName::AFlat, 68),
            (NoteName::A, 69),
            (NoteName::BFlat, 70),
            (NoteName::B, 71),
        ];

        for (note_name, expected_midi) in expected_midi_notes {
            assert_eq!(to_midi_note(note_name.clone(), 4), Some(expected_midi));
            assert_eq!(from_midi_note(expected_midi), note_name);
        }
    }
}