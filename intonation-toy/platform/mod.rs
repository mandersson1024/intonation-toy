//! Platform abstraction layer
//! 
//! This module provides trait-based abstractions for platform-specific functionality,
//! allowing the application to run on different targets (web, native) without scattering
//! conditional compilation throughout the codebase.
//! 
//! The abstraction is organized around three core concepts:
//! - **Timer**: High-resolution timing functionality
//! - **PerformanceMonitor**: System performance metrics and monitoring
//! - **UiController**: Platform-specific UI operations like canvas management
//! 
//! Different implementations are selected at compile time based on the target platform:
//! - Web/WASM targets use browser-specific implementations
//! - Native targets will use OS-specific implementations (future)

pub mod traits;

// Re-export all traits for convenient access
pub use traits::*;

// Platform-specific implementations
#[cfg(target_arch = "wasm32")]
pub mod web_impl;

// Stub implementations for non-web targets (placeholder for future phases)
#[cfg(not(target_arch = "wasm32"))]
pub mod stubs;