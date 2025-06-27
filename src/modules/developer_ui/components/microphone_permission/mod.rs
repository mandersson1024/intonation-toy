//! # Microphone Permission Components
//!
//! Components for microphone permission handling and fallback UI.
//! Contains microphone permission and fallback UI components.

// Component exports - to be implemented during migration
#[cfg(debug_assertions)]
pub mod microphone_permission;

#[cfg(debug_assertions)]
pub mod fallback_ui;

// Re-exports for easy access
#[cfg(debug_assertions)]
pub use microphone_permission::*;

#[cfg(debug_assertions)]
pub use fallback_ui::*; 