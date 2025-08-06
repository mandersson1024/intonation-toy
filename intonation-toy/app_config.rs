/// Application configuration constants
/// 
/// This module contains all configuration constants used throughout the application

use crate::shared_types::ColorScheme;

/// Color scheme configuration
pub const COLOR_SCHEME: ColorScheme = ColorScheme::light();

/// Viewport configuration
pub const VIEWPORT_WIDTH: u32 = 1024;
pub const VIEWPORT_HEIGHT: u32 = 1024;
pub const VIEWPORT_MIN_SIZE: u32 = 256;
pub const VIEWPORT_MAX_SIZE: u32 = 1024;

/// Window configuration
pub const WINDOW_TITLE: &str = "intonation-toy";

/// Pitch detection configuration
pub const CLARITY_THRESHOLD: f32 = 0.7;