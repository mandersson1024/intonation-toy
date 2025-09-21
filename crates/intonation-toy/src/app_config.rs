#![cfg(target_arch = "wasm32")]

//! Application configuration constants
//! 
//! This module contains all configuration constants used throughout the application

use crate::common::shared_types::{Theme, MidiNote, Scale, DisplayRange};

/// Default theme configuration
pub const DEFAULT_THEME: Theme = Theme::Dark;

/// Default musical configuration
///
/// Default tonal center for the tuning system and pitch analysis.
/// C4 = 60
pub const DEFAULT_TONAL_CENTER_NOTE: MidiNote = 60;

/// Default scale for pitch visualization and analysis.
/// Set to Chromatic scale, which includes all 12 semitones and provides
/// the most comprehensive pitch reference for users. Other scales can be
/// selected through the UI to focus on specific musical contexts.
pub const DEFAULT_SCALE: Scale = Scale::Major;

/// Default display range for the pitch visualization.
pub const DEFAULT_DISPLAY_RANGE: DisplayRange = DisplayRange::TwoOctaves;

/// Viewport configuration
pub const VIEWPORT_RENDER_SIZE: u32 = 1024;
pub const VIEWPORT_RENDER_SIZE_RETINA: u32 = 512;
pub const CANVAS_MIN_SIZE: i32 = 384;
pub const CANVAS_MAX_SIZE: i32 = 4096;

/// Window configuration
pub const WINDOW_TITLE: &str = "intonation-toy";

/// Audio processing configuration
pub const AUDIO_CHUNK_SIZE: usize = 128;                // AudioWorklet fixed chunk size
pub const BUFFER_SIZE: usize = AUDIO_CHUNK_SIZE * 16;   // IMPORTANT: Also update BUFFER_SIZE in static/audio-processor.js

/// Pitch detection configuration
pub const POWER_THRESHOLD: f32 = 0.3;
pub const CLARITY_THRESHOLD: f32 = 0.2;

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
pub const PITCH_SMOOTHING_FACTOR: f32 = 0.2;

/// Adaptive EMA configuration for advanced smoothing
/// These parameters control the adaptive EMA algorithm that reduces jitter and outliers

/// Minimum alpha (EMA factor) for strong smoothing when changes are small
pub const ADAPTIVE_EMA_ALPHA_MIN: f32 = 0.02;

/// Maximum alpha (EMA factor) for quick response when changes are large
pub const ADAPTIVE_EMA_ALPHA_MAX: f32 = 0.6;

/// Soft threshold for "jitter size" in Hz - typical noise scale for pitch detection
pub const ADAPTIVE_EMA_D: f32 = 0.5;

/// Softness of the transition between min and max alpha (smaller => steeper)
pub const ADAPTIVE_EMA_S: f32 = 0.15;

/// Enable median-of-3 prefilter for cheap jitter reduction
pub const ADAPTIVE_EMA_USE_MEDIAN3: bool = true;

/// Enable Hampel outlier suppression for removing transient spikes
pub const ADAPTIVE_EMA_USE_HAMPEL: bool = true;

/// Window size for Hampel filter (must be odd, e.g., 5, 7, 9)
pub const ADAPTIVE_EMA_HAMPEL_WINDOW: usize = 7;

/// Sensitivity for Hampel filter (larger => fewer points flagged as outliers)
pub const ADAPTIVE_EMA_HAMPEL_NSIGMA: f32 = 3.0;

/// Deadband threshold in Hz - when changes are smaller than this, use minimum smoothing
pub const ADAPTIVE_EMA_DEADBAND: f32 = 1.0;

/// Hysteresis thresholds (d_down, d_up) to reduce flicker near threshold
pub const ADAPTIVE_EMA_HYSTERESIS_DOWN: f32 = 0.25;
pub const ADAPTIVE_EMA_HYSTERESIS_UP: f32 = 0.45;

/// Enable adaptive EMA smoothing (set to false to use simple EMA)
pub const USE_ADAPTIVE_EMA: bool = false;

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
pub const USER_PITCH_LINE_THICKNESS: f32 = 10.0;

/// Note label configuration
pub const NOTE_LABEL_FONT_SIZE: f32 = 22.0;
pub const NOTE_LABEL_X_OFFSET: f32 = 12.0;
pub const NOTE_LABEL_Y_OFFSET: f32 = 13.0;
pub const INTERVAL_LABEL_X_OFFSET: f32 = 18.0;

/// Tuning lines layout configuration
pub const NOTE_LINE_LEFT_MARGIN: f32 = 64.0;
pub const NOTE_LINE_RIGHT_MARGIN: f32 = 54.0;

pub const USER_PITCH_LINE_LEFT_MARGIN: f32 = 970.0;
pub const USER_PITCH_LINE_RIGHT_MARGIN: f32 = 0.0;

/// Line thickness configuration
pub const OCTAVE_LINE_THICKNESS: f32 = 8.0;
pub const REGULAR_LINE_THICKNESS: f32 = 4.0;
pub const DEFAULT_LINE_THICKNESS: f32 = 1.0;

/// Overlay alpha configuration
pub const OVERLAY_BACKGROUND_ALPHA: f32 = 0.8;

