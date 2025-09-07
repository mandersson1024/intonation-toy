#![cfg(target_arch = "wasm32")]

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
pub fn is_valid_midi_note(value: i32) -> bool {
    (0..=127).contains(&value)
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
#[derive(serde::Serialize, serde::Deserialize)]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Scale {
    Chromatic,
    Major,
    Minor,
    HarmonicMinor,
    MelodicMinor,
    MajorPentatonic,
    MinorPentatonic,
    Blues,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    WholeTone,
    Augmented,
    DiminishedHalfWhole,
    DiminishedWholeHalf,
    HungarianMinor,
    NeapolitanMinor,
    NeapolitanMajor,
    Enigmatic,
    Persian,
    DoubleHarmonicMajor,
    Altered,
    BebopMajor,
    BebopDominant,
}

impl Scale {
    /// Returns a boolean array indicating which semitones (0-11) from the root are included in the scale.
    /// Index 0 represents the root note (always true), index 1 represents +1 semitone from root, etc.
    pub fn pattern(&self) -> [bool; 12] {
        match self {
            Scale::Chromatic => [true; 12],
            Scale::Major => [true, false, true, false, true, true, false, true, false, true, false, true],
            Scale::Minor => [true, false, true, true, false, true, false, true, true, false, true, false],
            Scale::HarmonicMinor =>    [true, false, true, true, false, true, false, true, true, false, false, true],
            Scale::MelodicMinor =>     [true, false, true, true, false, true, false, true, false, true, false, true],
            Scale::MajorPentatonic =>  [true, false, true, false, true, false, false, true, false, true, false, false],
            Scale::MinorPentatonic =>  [true, false, false, true, false, true, false, true, false, false, true, false],
            Scale::Blues =>            [true, false, false, true, true, true, false, true, false, false, true, false],
            Scale::Dorian =>           [true, false, true, true, false, true, false, true, true, false, true, false],
            Scale::Phrygian =>         [true, true, false, true, false, true, false, true, true, false, true, false],
            Scale::Lydian =>           [true, false, true, false, true, false, true, true, false, true, false, true],
            Scale::Mixolydian =>       [true, false, true, false, true, true, false, true, false, true, true, false],
            Scale::Locrian =>          [true, true, false, true, false, true, true, false, true, false, true, false],
            Scale::WholeTone =>        [true, false, true, false, true, false, true, false, true, false, true, false],
            Scale::Augmented =>        [true, false, true, false, true, false, false, true, false, true, false, false],
            Scale::DiminishedHalfWhole => [true, true, false, true, false, true, true, false, true, false, true, false],
            Scale::DiminishedWholeHalf => [true, false, true, true, false, true, false, true, true, false, true, false],
            Scale::HungarianMinor =>   [true, false, true, true, false, false, true, true, false, true, false, true],
            Scale::NeapolitanMinor =>  [true, true, false, true, false, true, false, true, true, false, true, false],
            Scale::NeapolitanMajor =>  [true, true, false, true, false, true, false, true, false, true, false, true],
            Scale::Enigmatic =>        [true, false, false, true, false, true, true, true, true, false, true, false],
            Scale::Persian =>          [true, true, false, false, true, true, false, true, true, false, true, false],
            Scale::DoubleHarmonicMajor => [true, true, false, false, true, true, false, true, true, false, false, true],
            Scale::Altered =>          [true, true, false, true, false, true, true, false, true, true, true, false],
            Scale::BebopMajor =>       [true, false, true, false, true, true, true, false, true, false, true, false],
            Scale::BebopDominant =>    [true, false, true, false, true, true, false, true, true, false, true, false],
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
    pub closest_midi_note: Option<MidiNote>,
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


#[derive(Debug, Clone, PartialEq)]
pub struct EngineUpdateResult {
    pub audio_analysis: Option<AudioAnalysis>,
    pub audio_errors: Vec<Error>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelUpdateResult {
    pub volume: Volume,
    pub is_peaking: bool,
    pub pitch: Pitch,
    pub tuning_system: TuningSystem,
    pub scale: Scale,
    pub closest_midi_note: Option<MidiNote>,
    pub cents_offset: f32,
    pub interval_semitones: i32,
    pub tuning_fork_note: MidiNote,
}

/// Context data passed from presentation layer to main scene for rendering calculations
#[derive(Debug, Clone, PartialEq)]
pub struct PresentationContext {
    pub tuning_fork_note: MidiNote,
    pub tuning_system: TuningSystem,
    pub current_scale: Scale,
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

