/// High-resolution timing utilities for performance measurement
/// 
/// Provides cross-platform timing functions for both WASM and native environments.

/// Get high-resolution time in milliseconds
/// 
/// On WASM: Uses Performance.now() API for sub-millisecond precision
/// On native: Uses system time with millisecond precision
pub fn get_high_resolution_time() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(performance) = window.performance() {
                return performance.now();
            }
        }
        0.0
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64
    }
}