//! Common utilities and helper functions
//!
//! This module provides common utility functions, mathematical operations,
//! and helper structures used throughout the sprite renderer.

pub mod math;
pub mod color;

// Re-export commonly used types
pub use math::{Vec2, Rectangle, Transform2D, Mat4};
pub use color::Color;