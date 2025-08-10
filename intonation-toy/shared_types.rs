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
/// assert_eq!(increment_midi_note(69), Some(70));  // A4 to Bb4
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
/// assert_eq!(decrement_midi_note(69), Some(68));  // A4 to Ab4
/// assert_eq!(decrement_midi_note(0), None);       // C-1 cannot decrement
/// ```
pub fn decrement_midi_note(midi_note: MidiNote) -> Option<MidiNote> {
    if midi_note > 0 {
        Some(midi_note - 1)
    } else {
        None
    }
}

/// Converts a MIDI note number (0-127) to its standard note name with octave.
/// 
/// Uses the standard MIDI mapping where:
/// - C4 = 60 (middle C)
/// - A4 = 69 (440 Hz)
/// - MIDI note 0 = C-1
/// - MIDI note 127 = G9
/// 
/// # Examples
/// ```
/// assert_eq!(midi_note_to_name(60), "C4");  // Middle C
/// assert_eq!(midi_note_to_name(69), "A4");  // Concert A
/// assert_eq!(midi_note_to_name(0), "C-1");  // Lowest MIDI note
/// assert_eq!(midi_note_to_name(127), "G9"); // Highest MIDI note
/// ```
pub fn midi_note_to_name(midi_note: MidiNote) -> String {
    const NOTE_NAMES: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];
    
    let octave = (midi_note as i32 / 12) - 1;
    let note_index = midi_note % 12;
    let note_name = NOTE_NAMES[note_index as usize];
    
    format!("{}{}", note_name, octave)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Theme {
    Dark,
    Light,
    Autumn,
    Sunset,
}

impl Theme {
    pub fn name(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Autumn => "autumn",
            Theme::Sunset => "sunset",
        }
    }

    pub fn color_scheme(&self) -> ColorScheme {
        match self {
            Theme::Dark => ColorScheme::dark(),
            Theme::Light => ColorScheme::light(),
            Theme::Autumn => ColorScheme::autumn(),
            Theme::Sunset => ColorScheme::sunset(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorScheme {
    pub background: [f32; 3],
    pub surface: [f32; 3],
    pub primary: [f32; 3],
    pub secondary: [f32; 3],
    pub accent: [f32; 3],
    pub text: [f32; 3],
    pub muted: [f32; 3],
    pub border: [f32; 3],
    pub error: [f32; 3],
    
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl ColorScheme {
    pub const fn dark() -> Self {
        Self {
            background: [0.16, 0.18, 0.21],  // #2E3440
            surface:    [0.19, 0.21, 0.25],  // #3B4252
            primary:    [0.58, 0.74, 0.81],  // #94B7CF (Frost Blue)
            secondary:  [0.78, 0.87, 0.90],  // #C7E0E6 (Lighter Frost)
            accent:     [0.88, 0.80, 0.62],  // #E1CC9F (Sand)
            text:       [0.89, 0.91, 0.95],  // #E5E9F0
            muted:      [0.64, 0.68, 0.75],  // #A3ADBf
            border:     [0.28, 0.28, 0.32],  // #474852 (Subtle outline for panels/inputs)
            error:      [0.90, 0.35, 0.38],  // #E65A60 (Desaturated red for errors)
        }
    }
    
    pub const fn light() -> Self {
        Self {
            background: [0.95, 0.95, 0.95],
            surface: [0.8, 0.8, 0.8],
            primary: [0.0, 0.4, 0.8],
            secondary: [0.8, 0.0, 0.0],
            accent: [0.0, 1.0, 0.0],
            text: [0.0, 0.0, 0.0],
            muted: [0.6, 0.6, 0.6],
            border:     [0.28, 0.28, 0.32],  // #474852 (Subtle outline for panels/inputs)
            error:      [0.90, 0.35, 0.38],  // #E65A60 (Desaturated red for errors)
        }
    }
    
    pub const fn autumn() -> Self {
        Self {
            background: [0.12, 0.08, 0.06],
            surface: [0.3, 0.2, 0.15],
            primary: [0.9, 0.6, 0.2],
            secondary: [0.8, 0.3, 0.1],
            accent: [0.95, 0.8, 0.3],
            text: [0.95, 0.9, 0.8],
            muted: [0.5, 0.4, 0.3],
            border:     [0.28, 0.28, 0.32],  // #474852 (Subtle outline for panels/inputs)
            error:      [0.90, 0.35, 0.38],  // #E65A60 (Desaturated red for errors)
        }
    }
    
    pub const fn sunset() -> Self {
        Self {
            background: [0.15, 0.05, 0.08],
            surface: [0.25, 0.15, 0.18],
            primary: [1.0, 0.4, 0.2],
            secondary: [0.9, 0.2, 0.4],
            accent: [1.0, 0.7, 0.0],
            text: [1.0, 0.95, 0.9],
            muted: [0.6, 0.4, 0.4],
            border:     [0.28, 0.28, 0.32],  // #474852 (Subtle outline for panels/inputs)
            error:      [0.90, 0.35, 0.38],  // #E65A60 (Desaturated red for errors)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scale {
    Chromatic,
    Major,
    Minor,
    MajorPentatonic,
    MinorPentatonic,
}

impl Scale {
    /// Returns a boolean array indicating which semitones (0-11) from the root are included in the scale.
    /// Index 0 represents the root note (always true), index 1 represents +1 semitone from root, etc.
    pub fn pattern(&self) -> [bool; 12] {
        match self {
            // Chromatic scale: all semitones including root
            Scale::Chromatic => [true; 12],
            // Major scale: Root + W-W-H-W-W-W-H (semitones: 0,2,4,5,7,9,11)
            Scale::Major => [true, false, true, false, true, true, false, true, false, true, false, true],
            // Minor scale: Root + W-H-W-W-H-W-W (semitones: 0,2,3,5,7,8,10)
            Scale::Minor => [true, false, true, true, false, true, false, true, true, false, true, false],
            // Major Pentatonic scale: Root + W-W-m3-W-m3 (semitones: 0,2,4,7,9)
            Scale::MajorPentatonic => [true, false, true, false, true, false, false, true, false, true, false, false],
            // Minor Pentatonic scale: Root + m3-W-W-m3-W (semitones: 0,3,5,7,10)
            Scale::MinorPentatonic => [true, false, false, true, false, true, false, true, false, false, true, false],
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
    MobileDeviceNotSupported,
    BrowserError,
}

impl Error {
    /// Returns the error dialog title for this error variant.
    pub fn title(&self) -> &'static str {
        match self {
            Error::MicrophonePermissionDenied => "Microphone Access Required",
            Error::MicrophoneNotAvailable => "Microphone Not Available",
            Error::ProcessingError(_) => "Processing Error",
            Error::BrowserApiNotSupported => "Browser Not Supported",
            Error::MobileDeviceNotSupported => "Mobile Devices Not Supported",
            Error::BrowserError => "Browser Error",
        }
    }

    /// Returns the error message template for this error variant.
    /// Some messages contain `{}` placeholders for dynamic content.
    /// Note: ProcessingError returns a dynamically allocated string, not a static string.
    pub fn details(&self) -> &str {
        match self {
            Error::MicrophonePermissionDenied => "Please allow microphone access to use the pitch detection features. Refresh the page and click 'Allow' when prompted. (Mac users: the microphone may be blocked in System Settings.)",
            Error::MicrophoneNotAvailable => "No microphone device found. Please ensure a microphone is connected and try again.",
            Error::ProcessingError(msg) => msg,
            Error::BrowserApiNotSupported => "This browser doesn't support the required audio features ({}). Please try Chrome, Firefox, or Edge.",
            Error::MobileDeviceNotSupported => "This application is not optimized for mobile devices. Please use a desktop computer.",
            Error::BrowserError => "An unexpected browser error occurred. Please try refreshing the page.",
        }
    }

    /// Returns the error message with dynamic parameters filled in.
    /// Use this for messages that need dynamic content like missing API names.
    pub fn details_with(&self, params: &[&str]) -> String {
        let template = self.details();
        
        // For ProcessingError, return the message as-is since it's already dynamic
        if matches!(self, Error::ProcessingError(_)) {
            return template.to_string();
        }
        
        // Replace {} placeholders with provided parameters
        let mut result = template.to_string();
        for param in params {
            if let Some(pos) = result.find("{}") {
                result.replace_range(pos..pos+2, param);
            }
        }
        result
    }
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
    pub volume_peak: bool,
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
            volume_peak: false,
            pitch: test_pitch.clone(),
            accuracy: test_accuracy.clone(),
            tuning_system: test_tuning_system.clone(),
            scale: Scale::Chromatic,
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
        assert_eq!(increment_midi_note(69), Some(70));  // A4 to Bb4
        assert_eq!(increment_midi_note(126), Some(127));
        assert_eq!(increment_midi_note(127), None);     // Cannot increment max value
    }

    #[wasm_bindgen_test]
    fn test_decrement_midi_note() {
        assert_eq!(decrement_midi_note(127), Some(126));
        assert_eq!(decrement_midi_note(69), Some(68));  // A4 to Ab4
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
        // Test Chromatic scale pattern (all semitones including root)
        let chromatic = Scale::Chromatic.pattern();
        assert_eq!(chromatic, [true; 12]);
        
        // Test Major scale pattern (Root + W-W-H-W-W-W-H)
        let major = Scale::Major.pattern();
        assert_eq!(major, [true, false, true, false, true, true, false, true, false, true, false, true]);
        
        // Test Minor scale pattern (Root + W-H-W-W-H-W-W)
        let minor = Scale::Minor.pattern();
        assert_eq!(minor, [true, false, true, true, false, true, false, true, true, false, true, false]);
        
        // Test Major Pentatonic scale pattern (semitones: 0,2,4,7,9)
        let major_pent = Scale::MajorPentatonic.pattern();
        assert_eq!(major_pent, [true, false, true, false, true, false, false, true, false, true, false, false]);
        
        // Test Minor Pentatonic scale pattern (semitones: 0,3,5,7,10)
        let minor_pent = Scale::MinorPentatonic.pattern();
        assert_eq!(minor_pent, [true, false, false, true, false, true, false, true, false, false, true, false]);
    }

    #[wasm_bindgen_test]
    fn test_semitone_in_scale() {
        // Test root is always in scale
        assert!(semitone_in_scale(Scale::Chromatic, 0));
        assert!(semitone_in_scale(Scale::Major, 0));
        assert!(semitone_in_scale(Scale::Minor, 0));
        assert!(semitone_in_scale(Scale::MajorPentatonic, 0));
        assert!(semitone_in_scale(Scale::MinorPentatonic, 0));
        
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
        
        // Test Major Pentatonic scale semitones (0,2,4,7,9)
        assert!(semitone_in_scale(Scale::MajorPentatonic, 0));   // Root
        assert!(!semitone_in_scale(Scale::MajorPentatonic, 1));  // Not in scale
        assert!(semitone_in_scale(Scale::MajorPentatonic, 2));   // Major 2nd
        assert!(!semitone_in_scale(Scale::MajorPentatonic, 3));  // Not in scale
        assert!(semitone_in_scale(Scale::MajorPentatonic, 4));   // Major 3rd
        assert!(!semitone_in_scale(Scale::MajorPentatonic, 5));  // Not in scale
        assert!(!semitone_in_scale(Scale::MajorPentatonic, 6));  // Not in scale
        assert!(semitone_in_scale(Scale::MajorPentatonic, 7));   // Perfect 5th
        assert!(!semitone_in_scale(Scale::MajorPentatonic, 8));  // Not in scale
        assert!(semitone_in_scale(Scale::MajorPentatonic, 9));   // Major 6th
        assert!(!semitone_in_scale(Scale::MajorPentatonic, 10)); // Not in scale
        assert!(!semitone_in_scale(Scale::MajorPentatonic, 11)); // Not in scale
        
        // Test Minor Pentatonic scale semitones (0,3,5,7,10)
        assert!(semitone_in_scale(Scale::MinorPentatonic, 0));   // Root
        assert!(!semitone_in_scale(Scale::MinorPentatonic, 1));  // Not in scale
        assert!(!semitone_in_scale(Scale::MinorPentatonic, 2));  // Not in scale
        assert!(semitone_in_scale(Scale::MinorPentatonic, 3));   // Minor 3rd
        assert!(!semitone_in_scale(Scale::MinorPentatonic, 4));  // Not in scale
        assert!(semitone_in_scale(Scale::MinorPentatonic, 5));   // Perfect 4th
        assert!(!semitone_in_scale(Scale::MinorPentatonic, 6));  // Not in scale
        assert!(semitone_in_scale(Scale::MinorPentatonic, 7));   // Perfect 5th
        assert!(!semitone_in_scale(Scale::MinorPentatonic, 8));  // Not in scale
        assert!(!semitone_in_scale(Scale::MinorPentatonic, 9));  // Not in scale
        assert!(semitone_in_scale(Scale::MinorPentatonic, 10));  // Minor 7th
        assert!(!semitone_in_scale(Scale::MinorPentatonic, 11)); // Not in scale
        
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

    #[test]
    fn test_midi_note_to_name() {
        // Test standard reference points
        assert_eq!(midi_note_to_name(60), "C4");   // Middle C
        assert_eq!(midi_note_to_name(69), "A4");   // Concert A (440 Hz)
        
        // Test all chromatic notes in the 4th octave
        assert_eq!(midi_note_to_name(60), "C4");   // C4
        assert_eq!(midi_note_to_name(61), "Db4");  // Db4
        assert_eq!(midi_note_to_name(62), "D4");   // D4
        assert_eq!(midi_note_to_name(63), "Eb4");  // Eb4
        assert_eq!(midi_note_to_name(64), "E4");   // E4
        assert_eq!(midi_note_to_name(65), "F4");   // F4
        assert_eq!(midi_note_to_name(66), "Gb4");  // Gb4
        assert_eq!(midi_note_to_name(67), "G4");   // G4
        assert_eq!(midi_note_to_name(68), "Ab4");  // Ab4
        assert_eq!(midi_note_to_name(69), "A4");   // A4
        assert_eq!(midi_note_to_name(70), "Bb4");  // Bb4
        assert_eq!(midi_note_to_name(71), "B4");   // B4
        
        // Test different octaves
        assert_eq!(midi_note_to_name(0), "C-1");   // Lowest MIDI note
        assert_eq!(midi_note_to_name(12), "C0");   // C0
        assert_eq!(midi_note_to_name(24), "C1");   // C1
        assert_eq!(midi_note_to_name(36), "C2");   // C2
        assert_eq!(midi_note_to_name(48), "C3");   // C3
        assert_eq!(midi_note_to_name(72), "C5");   // C5
        assert_eq!(midi_note_to_name(84), "C6");   // C6
        assert_eq!(midi_note_to_name(96), "C7");   // C7
        assert_eq!(midi_note_to_name(108), "C8");  // C8
        assert_eq!(midi_note_to_name(120), "C9");  // C9
        
        // Test edge cases
        assert_eq!(midi_note_to_name(127), "G9");  // Highest MIDI note
        assert_eq!(midi_note_to_name(1), "Db-1");  // Second lowest MIDI note
        assert_eq!(midi_note_to_name(11), "B-1");  // B in -1 octave
        
        // Test A notes across octaves (useful for tuning reference)
        assert_eq!(midi_note_to_name(21), "A0");   // A0
        assert_eq!(midi_note_to_name(33), "A1");   // A1
        assert_eq!(midi_note_to_name(45), "A2");   // A2
        assert_eq!(midi_note_to_name(57), "A3");   // A3
        assert_eq!(midi_note_to_name(69), "A4");   // A4 (440 Hz)
        assert_eq!(midi_note_to_name(81), "A5");   // A5
        assert_eq!(midi_note_to_name(93), "A6");   // A6
        assert_eq!(midi_note_to_name(105), "A7");  // A7
        assert_eq!(midi_note_to_name(117), "A8");  // A8
    }


}