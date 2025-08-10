/// Application configuration constants
/// 
/// This module contains all configuration constants used throughout the application

use crate::shared_types::Theme;

/// Default theme configuration
pub const DEFAULT_THEME: Theme = Theme::Dark;

/// Configuration constants
/// 
/// This module should contain only configuration constants, not functions.
/// Functions that compute or retrieve dynamic configuration values belong 
/// in their respective domain modules (e.g., theme module for color schemes).

/// Viewport configuration
pub const VIEWPORT_WIDTH: u32 = 768;
pub const VIEWPORT_HEIGHT: u32 = 768;
pub const CANVAS_MIN_SIZE: i32 = 256;
pub const CANVAS_MAX_SIZE: i32 = 768;

/// Window configuration
pub const WINDOW_TITLE: &str = "intonation-toy";

/// Pitch detection configuration
pub const CLARITY_THRESHOLD: f32 = 0.7;
pub const POWER_THRESHOLD: f32 = 5.0;

/// User pitch line thickness configuration
pub const USER_PITCH_LINE_THICKNESS_MIN: f32 = 3.0;
pub const USER_PITCH_LINE_THICKNESS_MAX: f32 = 20.0;

/// User pitch line transparency configuration
pub const USER_PITCH_LINE_TRANSPARENCY_MIN: f32 = 0.0;
pub const USER_PITCH_LINE_TRANSPARENCY_MAX: f32 = 1.0;

/// Octave line thickness configuration
pub const OCTAVE_LINE_THICKNESS: f32 = 4.0;
pub const REGULAR_LINE_THICKNESS: f32 = 2.0;

/// Overlay alpha configuration
pub const OVERLAY_BACKGROUND_ALPHA: f32 = 0.8;