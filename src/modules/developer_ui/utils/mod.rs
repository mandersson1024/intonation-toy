//! # Developer UI Utilities
//!
//! Utility modules for the Developer UI providing type-safe event handling,
//! performance optimization, and debug component support.

#[cfg(debug_assertions)]
pub mod debug_event_publisher;

#[cfg(debug_assertions)]
pub mod event_type_registry;

#[cfg(debug_assertions)]
pub mod debug_performance_utils;

#[cfg(debug_assertions)]
pub mod memory_leak_prevention;

#[cfg(debug_assertions)]
pub mod debug_event_performance_monitor;

// Re-exports for easy access
#[cfg(debug_assertions)]
pub use debug_event_publisher::*;

#[cfg(debug_assertions)]
pub use event_type_registry::*;

#[cfg(debug_assertions)]
pub use debug_performance_utils::*;

#[cfg(debug_assertions)]
pub use memory_leak_prevention::*;

#[cfg(debug_assertions)]
pub use debug_event_performance_monitor::*;