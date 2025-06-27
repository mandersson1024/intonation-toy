//! # Audio Controls Components
//!
//! Debug components for audio control and testing functionality.
//! These components will contain the migrated audio control panel,
//! microphone panel, and test signal generator.

// Component exports - to be implemented during migration
#[cfg(debug_assertions)]
pub mod audio_control_panel;

#[cfg(debug_assertions)]
pub mod microphone_panel;

#[cfg(debug_assertions)]
pub mod test_signal_generator;

// Re-exports for easy access
#[cfg(debug_assertions)]
pub use audio_control_panel::*;

#[cfg(debug_assertions)]
pub use microphone_panel::*;

#[cfg(debug_assertions)]
pub use test_signal_generator::*; 