//! Shared data types for the intonation-toy application.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scale {
    Major,
    Minor,
    Chromatic,
}

impl Scale {
    /// Returns a boolean array indicating which semitones (0-11) from the root are included in the scale.
    /// Index 0 represents the root note (always true), index 1 represents +1 semitone from root, etc.
    pub fn pattern(&self) -> [bool; 12] {
        match self {
            // Major scale: Root + W-W-H-W-W-W-H (semitones: 0,2,4,5,7,9,11)
            Scale::Major => [true, false, true, false, true, true, false, true, false, true, false, true],
            // Minor scale: Root + W-H-W-W-H-W-W (semitones: 0,2,3,5,7,8,10)
            Scale::Minor => [true, false, true, true, false, true, false, true, true, false, true, false],
            // Chromatic scale: all semitones including root
            Scale::Chromatic => [true; 12],
        }
    }
}

/// Check if a semitone offset from the root is included in the given scale.
/// The root (offset 0) is always included in any scale.
pub fn semitone_in_scale(scale: Scale, semitone_offset: i32) -> bool {
    // Use rem_euclid to handle negative offsets and octaves
    let normalized_offset = semitone_offset.rem_euclid(12);
    
    // Check the pattern (index 0 = root, index 1 = semitone 1, etc.)
    scale.pattern()[normalized_offset as usize]
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
    pub root_note_audio_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelUpdateResult {
    pub volume: Volume,
    pub pitch: Pitch,
    pub accuracy: IntonationData,
    pub tuning_system: TuningSystem,
    pub scale: Scale,
    pub errors: Vec<Error>,
    pub permission_state: PermissionState,
    // Flattened intonation data fields for easier access
    pub closest_midi_note: MidiNote,
    pub cents_offset: f32,
    pub interval_semitones: i32,
    pub root_note: MidiNote,
    /// Controls whether root note audio generation is enabled
    pub root_note_audio_enabled: bool,
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
            root_note_audio_enabled: false,
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
            scale: Scale::Major,
            errors: test_errors.clone(),
            permission_state: PermissionState::Granted,
            closest_midi_note: 69,
            cents_offset: -10.0,
            interval_semitones: 0,
            root_note: 53,
            root_note_audio_enabled: false,
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
        ];
        
        assert_eq!(errors.len(), 4);
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

    #[wasm_bindgen_test]
    fn test_scale_patterns() {
        // Test Major scale pattern (Root + W-W-H-W-W-W-H)
        let major = Scale::Major.pattern();
        assert_eq!(major, [true, false, true, false, true, true, false, true, false, true, false, true]);
        
        // Test Minor scale pattern (Root + W-H-W-W-H-W-W)
        let minor = Scale::Minor.pattern();
        assert_eq!(minor, [true, false, true, true, false, true, false, true, true, false, true, false]);
        
        // Test Chromatic scale pattern (all semitones including root)
        let chromatic = Scale::Chromatic.pattern();
        assert_eq!(chromatic, [true; 12]);
    }

    #[wasm_bindgen_test]
    fn test_semitone_in_scale() {
        // Test root is always in scale
        assert!(semitone_in_scale(Scale::Major, 0));
        assert!(semitone_in_scale(Scale::Minor, 0));
        assert!(semitone_in_scale(Scale::Chromatic, 0));
        
        // Test Major scale semitones
        assert!(!semitone_in_scale(Scale::Major, 1));  // Minor 2nd not in major
        assert!(semitone_in_scale(Scale::Major, 2));   // Major 2nd
        assert!(!semitone_in_scale(Scale::Major, 3));  // Minor 3rd not in major
        assert!(semitone_in_scale(Scale::Major, 4));   // Major 3rd
        assert!(semitone_in_scale(Scale::Major, 5));   // Perfect 4th
        assert!(!semitone_in_scale(Scale::Major, 6));  // Tritone not in major
        assert!(semitone_in_scale(Scale::Major, 7));   // Perfect 5th
        assert!(!semitone_in_scale(Scale::Major, 8));  // Minor 6th not in major
        assert!(semitone_in_scale(Scale::Major, 9));   // Major 6th
        assert!(!semitone_in_scale(Scale::Major, 10)); // Minor 7th not in major
        assert!(semitone_in_scale(Scale::Major, 11));  // Major 7th
        
        // Test Minor scale semitones
        assert!(!semitone_in_scale(Scale::Minor, 1));  // Minor 2nd not in minor
        assert!(semitone_in_scale(Scale::Minor, 2));   // Major 2nd
        assert!(semitone_in_scale(Scale::Minor, 3));   // Minor 3rd
        assert!(!semitone_in_scale(Scale::Minor, 4));  // Major 3rd not in minor
        assert!(semitone_in_scale(Scale::Minor, 5));   // Perfect 4th
        assert!(!semitone_in_scale(Scale::Minor, 6));  // Tritone not in minor
        assert!(semitone_in_scale(Scale::Minor, 7));   // Perfect 5th
        assert!(semitone_in_scale(Scale::Minor, 8));   // Minor 6th
        assert!(!semitone_in_scale(Scale::Minor, 9));  // Major 6th not in minor
        assert!(semitone_in_scale(Scale::Minor, 10));  // Minor 7th
        assert!(!semitone_in_scale(Scale::Minor, 11)); // Major 7th not in minor
        
        // Test Chromatic scale (all semitones)
        for i in 0..12 {
            assert!(semitone_in_scale(Scale::Chromatic, i));
        }
        
        // Test octave handling
        assert!(semitone_in_scale(Scale::Major, 12));  // Octave is root
        assert!(semitone_in_scale(Scale::Major, 14));  // Octave + Major 2nd
        assert!(!semitone_in_scale(Scale::Major, 13)); // Octave + Minor 2nd not in major
        
        // Test negative offsets
        assert!(semitone_in_scale(Scale::Major, -12)); // Octave below is root
        assert!(semitone_in_scale(Scale::Major, -10)); // Major 2nd below (wraps to +2)
        assert!(!semitone_in_scale(Scale::Major, -11)); // Minor 2nd below (wraps to +1)
        
        // Test large offsets
        assert!(semitone_in_scale(Scale::Major, 24));  // 2 octaves is root
        assert!(semitone_in_scale(Scale::Major, 26));  // 2 octaves + Major 2nd
        assert!(!semitone_in_scale(Scale::Major, 25)); // 2 octaves + Minor 2nd not in major
    }


}