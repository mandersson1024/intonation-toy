//! # Debug Interface Components
//!
//! Debug interface components for comprehensive debugging functionality.
//! Contains the main debug interface and debug panel components.

// Component exports - to be implemented during migration
#[cfg(debug_assertions)]
pub mod debug_interface;

#[cfg(debug_assertions)]
pub mod debug_panel;

// Re-exports for easy access
#[cfg(debug_assertions)]
pub use debug_interface::*;

#[cfg(debug_assertions)]
pub use debug_panel::*; 