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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntonationData {
    pub closest_midi_note: MidiNote,
    pub cents_offset: f32, // Distance in cents from the closest note (negative = flat, positive = sharp)
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub accuracy: IntonationData,
    pub tuning_system: TuningSystem,
    pub errors: Vec<Error>,
    pub permission_state: PermissionState,
    // Flattened intonation data fields for easier access
    pub closest_midi_note: MidiNote,
    pub cents_offset: f32,
    pub interval_semitones: i32,
    pub root_note: MidiNote,
}

/// Converts a semitone interval to a musical interval name.
/// 
/// This function takes the number of semitones between two notes and returns
/// a descriptive name for the musical interval. It handles both ascending
/// (positive) and descending (negative) intervals, as well as intervals
/// larger than an octave.
/// 
/// # Musical Theory Background
/// 
/// In Western music theory, intervals are classified by:
/// - **Size**: The number of letter names spanned (unison, 2nd, 3rd, etc.)
/// - **Quality**: Perfect, major, minor, augmented, diminished
/// 
/// The basic intervals within an octave are:
/// - 0 semitones: Perfect Unison
/// - 1 semitone: Minor Second
/// - 2 semitones: Major Second
/// - 3 semitones: Minor Third
/// - 4 semitones: Major Third
/// - 5 semitones: Perfect Fourth
/// - 6 semitones: Tritone (Augmented Fourth/Diminished Fifth)
/// - 7 semitones: Perfect Fifth
/// - 8 semitones: Minor Sixth
/// - 9 semitones: Major Sixth
/// - 10 semitones: Minor Seventh
/// - 11 semitones: Major Seventh
/// - 12 semitones: Perfect Octave
/// 
/// # Arguments
/// 
/// * `semitones` - The number of semitones in the interval. Can be positive
///   (ascending) or negative (descending). Values beyond ±12 are handled
///   by calculating the octave displacement and interval within an octave.
/// 
/// # Returns
/// 
/// A string describing the interval. For intervals larger than an octave,
/// the format is "Interval + N Octave(s)" where N is the number of octaves.
/// 
/// # Examples
/// 
/// ```
/// use pitch_toy::shared_types::interval_name_from_semitones;
/// 
/// // Basic intervals
/// assert_eq!(interval_name_from_semitones(0), "Perfect Unison");
/// assert_eq!(interval_name_from_semitones(4), "Major Third");
/// assert_eq!(interval_name_from_semitones(7), "Perfect Fifth");
/// assert_eq!(interval_name_from_semitones(12), "Perfect Octave");
/// 
/// // Intervals larger than an octave
/// assert_eq!(interval_name_from_semitones(16), "Major Third + Octave");
/// assert_eq!(interval_name_from_semitones(24), "2 Octaves");
/// 
/// // Descending intervals
/// assert_eq!(interval_name_from_semitones(-4), "Major Third (descending)");
/// assert_eq!(interval_name_from_semitones(-16), "Major Third + Octave (descending)");
/// ```
pub fn interval_name_from_semitones(semitones: i32) -> String {
    if semitones == 0 {
        return "Perfect Unison".to_string();
    }
    
    let is_descending = semitones < 0;
    let abs_semitones = semitones.abs();
    
    // Calculate octaves and remaining semitones
    let octaves = abs_semitones / 12;
    let remainder = abs_semitones % 12;
    
    // Get the base interval name
    let base_interval = match remainder {
        0 => "Perfect Unison",
        1 => "Minor Second",
        2 => "Major Second", 
        3 => "Minor Third",
        4 => "Major Third",
        5 => "Perfect Fourth",
        6 => "Tritone",
        7 => "Perfect Fifth",
        8 => "Minor Sixth",
        9 => "Major Sixth",
        10 => "Minor Seventh",
        11 => "Major Seventh",
        _ => unreachable!("Remainder should be 0-11"),
    };
    
    // Handle different cases
    let result = match (octaves, remainder) {
        // Exact octaves
        (n, 0) if n > 0 => {
            if n == 1 {
                "Perfect Octave".to_string()
            } else {
                format!("{} Octaves", n)
            }
        },
        // Intervals within one octave
        (0, _) => base_interval.to_string(),
        // Intervals with octave displacement
        (1, _) => format!("{} + Octave", base_interval),
        (n, _) => format!("{} + {} Octaves", base_interval, n),
    };
    
    // Add descending notation if needed
    if is_descending {
        format!("{} (descending)", result)
    } else {
        result
    }
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
        let test_accuracy = IntonationData { closest_midi_note: 69, cents_offset: -10.0 };
        let test_tuning_system = TuningSystem::EqualTemperament;
        let test_errors = vec![Error::ProcessingError("Test error".to_string())];

        let update_result = ModelUpdateResult {
            volume: test_volume.clone(),
            pitch: test_pitch.clone(),
            accuracy: test_accuracy.clone(),
            tuning_system: test_tuning_system.clone(),
            errors: test_errors.clone(),
            permission_state: PermissionState::Granted,
            closest_midi_note: 69,
            cents_offset: -10.0,
            interval_semitones: 0,
            root_note: 53,
        };

        assert_eq!(update_result.volume, test_volume);
        assert_eq!(update_result.pitch, test_pitch);
        assert_eq!(update_result.accuracy, test_accuracy);
        assert_eq!(update_result.tuning_system, test_tuning_system);
        assert_eq!(update_result.errors, test_errors);
        assert_eq!(update_result.permission_state, PermissionState::Granted);
        assert_eq!(update_result.closest_midi_note, 69);
        assert_eq!(update_result.cents_offset, -10.0);
        assert_eq!(update_result.interval_semitones, 0);
        assert_eq!(update_result.root_note, 53);
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
    fn test_interval_name_from_semitones() {
        // Test basic intervals
        assert_eq!(interval_name_from_semitones(0), "Perfect Unison");
        assert_eq!(interval_name_from_semitones(1), "Minor Second");
        assert_eq!(interval_name_from_semitones(2), "Major Second");
        assert_eq!(interval_name_from_semitones(3), "Minor Third");
        assert_eq!(interval_name_from_semitones(4), "Major Third");
        assert_eq!(interval_name_from_semitones(5), "Perfect Fourth");
        assert_eq!(interval_name_from_semitones(6), "Tritone");
        assert_eq!(interval_name_from_semitones(7), "Perfect Fifth");
        assert_eq!(interval_name_from_semitones(8), "Minor Sixth");
        assert_eq!(interval_name_from_semitones(9), "Major Sixth");
        assert_eq!(interval_name_from_semitones(10), "Minor Seventh");
        assert_eq!(interval_name_from_semitones(11), "Major Seventh");
        assert_eq!(interval_name_from_semitones(12), "Perfect Octave");
        
        // Test intervals larger than an octave
        assert_eq!(interval_name_from_semitones(13), "Minor Second + Octave");
        assert_eq!(interval_name_from_semitones(16), "Major Third + Octave");
        assert_eq!(interval_name_from_semitones(24), "2 Octaves");
        assert_eq!(interval_name_from_semitones(36), "3 Octaves");
        
        // Test descending intervals
        assert_eq!(interval_name_from_semitones(-1), "Minor Second (descending)");
        assert_eq!(interval_name_from_semitones(-4), "Major Third (descending)");
        assert_eq!(interval_name_from_semitones(-12), "Perfect Octave (descending)");
        assert_eq!(interval_name_from_semitones(-16), "Major Third + Octave (descending)");
        assert_eq!(interval_name_from_semitones(-24), "2 Octaves (descending)");
    }


}