//! # Developer UI Hooks
//!
//! Custom hooks for debug functionality, migrated from the legacy system.
//! These hooks provide reusable debug logic for the developer UI components.

// Hook exports - to be implemented during migration
#[cfg(debug_assertions)]
pub mod use_error_handler;

#[cfg(debug_assertions)]
pub mod use_microphone_permission;

// Re-exports for easy access
#[cfg(debug_assertions)]
pub use use_error_handler::*;

#[cfg(debug_assertions)]
pub use use_microphone_permission::*; 