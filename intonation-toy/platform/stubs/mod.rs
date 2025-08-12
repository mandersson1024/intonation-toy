//! Stub implementations for native (non-WASM) targets.
//!
//! This module provides minimal working implementations of platform traits
//! for non-WASM targets, enabling the application to compile and run tests
//! on native platforms without requiring browser APIs.
//!
//! The stub implementations are designed to:
//! - Provide real timing functionality for tests that depend on time progression
//! - Offer no-op implementations for UI and performance monitoring (not needed in headless tests)
//! - Maintain the exact same API as web implementations for seamless platform switching
//! - Enable development and testing on native platforms without browser dependencies

mod timer;
mod performance;
mod ui_controller;

pub use timer::StubTimer;
pub use performance::StubPerformanceMonitor;
pub use ui_controller::StubUiController;