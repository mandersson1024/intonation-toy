//! Application configuration constants
//! 
//! This module contains all configuration constants used throughout the application

use crate::shared_types::{Theme, MidiNote, Scale};

/// Default theme configuration
pub const DEFAULT_THEME: Theme = Theme::Dark;

/// Default musical configuration
/// 
/// Default root note for the tuning system and pitch analysis.
/// C4 = 60
pub const DEFAULT_ROOT_NOTE: MidiNote = 60;

/// Default scale for pitch visualization and analysis.
/// Set to Chromatic scale, which includes all 12 semitones and provides
/// the most comprehensive pitch reference for users. Other scales can be
/// selected through the UI to focus on specific musical contexts.
pub const DEFAULT_SCALE: Scale = Scale::Major;

/// Viewport configuration
pub const VIEWPORT_RENDER_SIZE: u32 = 768;
pub const CANVAS_MIN_SIZE: i32 = 384;
pub const CANVAS_MAX_SIZE: i32 = 4096;

/// Window configuration
pub const WINDOW_TITLE: &str = "intonation-toy";

/// Pitch detection configuration
pub const CLARITY_THRESHOLD: f32 = 0.3;
pub const POWER_THRESHOLD: f32 = 1.0;

/// Pitch smoothing factor for exponential moving average (EMA)
/// 
/// Controls how much the pitch detection is smoothed over time to reduce jitter
/// and noise while maintaining responsiveness to actual pitch changes.
/// 
/// Range: 0.0 to 1.0
/// - Higher values (closer to 1.0): More responsive to changes, less smoothing
/// - Lower values (closer to 0.0): More smoothing, slower response to changes
/// - Default 0.1: Provides moderate smoothing while maintaining good responsiveness
/// 
/// This factor is used in the EMA formula: smoothed = factor * new_value + (1 - factor) * old_value
pub const PITCH_SMOOTHING_FACTOR: f32 = 0.07;

/// Intonation accuracy configuration
/// Threshold in cents for considering pitch "accurate" and showing accent color
/// When the detected pitch is within ±INTONATION_ACCURACY_THRESHOLD cents of a note,
/// the user pitch line will display in accent color (unless volume is peaking)
pub const INTONATION_ACCURACY_THRESHOLD: f32 = 15.0;

/// Volume peak threshold configuration
/// (Since we don't calculate true peak)
/// Peak amplitude threshold for determining when volume is considered "peaking"
/// This affects the color of the user pitch line (error color when above threshold)
/// Set to -0.1dB converted to amplitude: 10^(-0.1/20) ≈ 0.9886
pub const VOLUME_PEAK_THRESHOLD: f32 = 0.9886;

/// User pitch line thickness configuration
pub const USER_PITCH_LINE_THICKNESS_MIN: f32 = 6.0;
pub const USER_PITCH_LINE_THICKNESS_MAX: f32 = 25.0;

/// User pitch line transparency configuration
pub const USER_PITCH_LINE_TRANSPARENCY_MIN: f32 = 0.0;
pub const USER_PITCH_LINE_TRANSPARENCY_MAX: f32 = 1.0;


/// Overlay alpha configuration
pub const OVERLAY_BACKGROUND_ALPHA: f32 = 0.8;

pub const PITCH_VISUALIZATION_ZOOM_DEFAULT: f32 = 0.92;
pub const PITCH_VISUALIZATION_ZOOM_MAX: f32 = 2.85;