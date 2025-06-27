//! # Error Display Components
//!
//! Components for error presentation and debugging.
//! Contains error display and error toast notification components.

// Component exports - to be implemented during migration
#[cfg(debug_assertions)]
pub mod error_display;

#[cfg(debug_assertions)]
pub mod error_toast;

// Re-exports for easy access
#[cfg(debug_assertions)]
pub use error_display::*;

#[cfg(debug_assertions)]
pub use error_toast::*; 