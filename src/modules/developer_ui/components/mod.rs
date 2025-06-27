//! # Developer UI Components
//!
//! This module contains all debug components migrated from the legacy system.
//! Components are organized by functional area and conditionally compiled
//! for debug builds only.

// Audio control components
#[cfg(debug_assertions)]
pub mod audio_controls;

// Debug interface components
#[cfg(debug_assertions)]
pub mod debug_interface;

// Error display components
#[cfg(debug_assertions)]
pub mod error_display;

// Metrics and performance monitoring components
#[cfg(debug_assertions)]
pub mod metrics_display;

// Microphone permission components
#[cfg(debug_assertions)]
pub mod microphone_permission;

// Re-export components for easier access
#[cfg(debug_assertions)]
pub use audio_controls::*;

#[cfg(debug_assertions)]
pub use debug_interface::*;

#[cfg(debug_assertions)]
pub use error_display::*;

#[cfg(debug_assertions)]
pub use metrics_display::*;

#[cfg(debug_assertions)]
pub use microphone_permission::*; 