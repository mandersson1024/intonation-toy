/// Application configuration constants
/// 
/// This module contains all configuration constants used throughout the application

use crate::shared_types::{ColorScheme, Theme};

/// Default theme configuration
pub const DEFAULT_THEME: Theme = Theme::Light;

/// Configuration constants
/// 
/// This module should contain only configuration constants, not functions.
/// Functions that compute or retrieve dynamic configuration values belong 
/// in their respective domain modules (e.g., theme module for color schemes).

/// Viewport configuration
pub const VIEWPORT_WIDTH: u32 = 1024;
pub const VIEWPORT_HEIGHT: u32 = 1024;
pub const VIEWPORT_MIN_SIZE: u32 = 256;
pub const VIEWPORT_MAX_SIZE: u32 = 1024;

/// Window configuration
pub const WINDOW_TITLE: &str = "intonation-toy";

/// Pitch detection configuration
pub const CLARITY_THRESHOLD: f32 = 0.7;

/// User pitch line thickness configuration
pub const USER_PITCH_LINE_THICKNESS_MIN: f32 = 3.0;
pub const USER_PITCH_LINE_THICKNESS_MAX: f32 = 30.0;