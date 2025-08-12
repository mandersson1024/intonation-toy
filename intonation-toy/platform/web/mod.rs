//! Web platform implementation for browser environments.
//! 
//! This module provides concrete implementations of platform traits for web browsers
//! using WebAssembly and browser APIs. It includes:
//! 
//! - High-resolution timing using Performance API
//! - Memory monitoring using Performance Memory API
//! - Canvas and UI management using DOM APIs
//! - Error message display using DOM overlays
//! - Color conversion utilities for CSS
//! 
//! All implementations are designed to work in modern browsers with graceful
//! fallbacks where appropriate.

pub mod timer;
pub mod performance;
pub mod utils;
pub mod ui_controller;
pub mod error_display;

// Re-export concrete implementations
pub use timer::WebTimer;
pub use performance::WebPerformanceMonitor;
pub use ui_controller::WebUiController;
pub use error_display::WebErrorDisplay;

// Re-export utility functions for internal use
pub use utils::{rgba_to_css, rgb_to_css};