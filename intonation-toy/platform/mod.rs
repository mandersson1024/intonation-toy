//! Platform abstraction layer
//! 
//! This module provides trait-based abstractions for platform-specific functionality,
//! allowing the application to run on different targets (web, native) without scattering
//! conditional compilation throughout the codebase.
//! 
//! The abstraction is organized around four core concepts:
//! - **Timer**: High-resolution timing functionality
//! - **PerformanceMonitor**: System performance metrics and monitoring
//! - **UiController**: Platform-specific UI operations like canvas management
//! - **ErrorDisplay**: Platform-specific error message display
//! 
//! Different implementations are selected at compile time based on the target platform:
//! - Web/WASM targets use browser-specific implementations
//! - Native targets use stub implementations for testing and development
//! 
//! ## Module Organization
//! 
//! The platform module maintains parallel structure between implementations:
//! - `web/` - Browser-based implementations using Web APIs
//! - `stubs/` - Minimal working implementations for native testing
//! 
//! Both implementations provide the same public API, allowing seamless switching
//! between platforms through conditional compilation.

pub mod traits;

// Re-export all traits for convenient access
pub use traits::*;

// Platform-specific implementations
#[cfg(target_arch = "wasm32")]
pub mod web;

// Re-export web platform implementations when on WASM targets
#[cfg(target_arch = "wasm32")]
pub use web::{WebTimer, WebPerformanceMonitor, WebUiController, WebErrorDisplay};

// Stub implementations for non-web targets
#[cfg(not(target_arch = "wasm32"))]
pub mod stubs;

// Re-export stub platform implementations when on non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub use stubs::{StubTimer, StubPerformanceMonitor, StubUiController, StubErrorDisplay};