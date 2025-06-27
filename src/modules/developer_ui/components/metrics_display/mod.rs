//! # Metrics Display Components
//!
//! Components for performance monitoring and metrics visualization.
//! Contains metrics display and performance monitor components.

// Component exports - to be implemented during migration
#[cfg(debug_assertions)]
pub mod metrics_display;

#[cfg(debug_assertions)]
pub mod performance_monitor;

// Re-exports for easy access
#[cfg(debug_assertions)]
pub use metrics_display::*;

#[cfg(debug_assertions)]
pub use performance_monitor::*; 