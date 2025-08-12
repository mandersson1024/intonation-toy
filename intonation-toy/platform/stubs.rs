//! Stub implementations for non-web platforms
//! 
//! These are placeholder implementations that will be replaced with
//! proper native platform support in future phases. Currently, they
//! provide minimal functionality to allow compilation on non-WASM targets.

use super::traits::*;

/// Stub timer implementation for non-web platforms
pub struct StubTimer;

impl Timer for StubTimer {
    fn now_ms() -> f64 {
        // Placeholder: would use std::time::Instant or similar
        0.0
    }
}

/// Stub performance monitor for non-web platforms
pub struct StubPerformanceMonitor;

impl PerformanceMonitor for StubPerformanceMonitor {
    fn sample_memory_usage() -> Option<(f64, f64)> {
        // Memory monitoring not yet implemented for native platforms
        None
    }
}

/// Stub UI controller for non-web platforms
pub struct StubUiController;

impl UiController for StubUiController {
    fn resize_canvas() {
        // Canvas resizing handled differently on native platforms
        // This will be implemented when native windowing is added
    }
    
    fn apply_theme_styles() {
        // Theme application will use native platform theming APIs
        // This will be implemented when native UI is added
    }
}