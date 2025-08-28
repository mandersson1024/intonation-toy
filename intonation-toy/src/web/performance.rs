//! Performance monitoring utilities for web browsers
//! 
//! This module provides browser-specific performance monitoring functionality,
//! including memory usage sampling via the Performance API.

/// Sample current JavaScript heap memory usage from the browser's Performance API.
/// 
/// This function attempts to access the `performance.memory` API to get information
/// about JavaScript heap usage. This API is not standardized and may not be available
/// in all browsers, particularly in privacy-focused configurations.
/// 
/// # Returns
/// 
/// Returns `Option<(f64, f64)>` containing:
/// - `memory_mb`: Used heap size in megabytes
/// - `memory_percent`: Percentage of total heap used (0-100)
/// 
/// Returns `None` if:
/// - The Performance API is not available
/// - The `performance.memory` extension is not supported
/// - Any of the required properties are undefined/null
/// - Type conversions fail
/// 
/// # Browser Compatibility
/// 
/// The `performance.memory` API is a non-standard extension primarily supported in:
/// - Chrome/Chromium-based browsers
/// - Some versions of Edge
/// 
/// It may not be available in:
/// - Firefox (disabled by default for privacy)
/// - Safari
/// - Privacy-focused browser configurations
/// 
/// # Example
/// 
/// ```rust
/// if let Some((memory_mb, memory_percent)) = sample_memory_usage() {
///     println!("Memory usage: {:.1} MB ({:.1}%)", memory_mb, memory_percent);
/// } else {
///     println!("Memory information not available");
/// }
/// ```
pub fn sample_memory_usage() -> Option<(f64, f64)> {
    use wasm_bindgen::{JsValue, JsCast};
    
    let window = web_sys::window()?;
    let performance = window.performance()?;
    
    // Try to get memory information (not available in all browsers)
    let memory = js_sys::Reflect::get(&performance, &JsValue::from_str("memory")).ok()?;
    if memory.is_undefined() || memory.is_null() {
        return None;
    }
    
    let memory_obj = memory.dyn_into::<web_sys::js_sys::Object>().ok()?;
    
    // Get used heap size
    let used_heap_size = js_sys::Reflect::get(&memory_obj, &JsValue::from_str("usedJSHeapSize")).ok()?;
    if used_heap_size.is_undefined() || used_heap_size.is_null() {
        return None;
    }
    
    // Get total heap size
    let total_heap_size = js_sys::Reflect::get(&memory_obj, &JsValue::from_str("totalJSHeapSize")).ok()?;
    if total_heap_size.is_undefined() || total_heap_size.is_null() {
        return None;
    }
    
    // Convert from bytes to MB and calculate percentage
    let used_bytes = used_heap_size.as_f64()?;
    let total_bytes = total_heap_size.as_f64()?;
    
    let memory_mb = used_bytes / (1024.0 * 1024.0);
    let memory_percent = if total_bytes > 0.0 {
        (used_bytes / total_bytes) * 100.0
    } else {
        0.0
    };
    
    Some((memory_mb, memory_percent))
}

/// Get complete performance metrics including FPS and memory usage.
/// 
/// # Parameters
/// 
/// - `fps`: Current frames per second value
/// 
/// # Returns
/// 
/// Returns a `PerformanceMetrics` struct containing:
/// - FPS value
/// - Memory usage in MB (0.0 if not available)
/// - Memory usage percentage (0.0 if not available)
pub fn get_performance_metrics(fps: f64) -> crate::debug::data_types::PerformanceMetrics {
    let (memory_usage_mb, memory_usage_percent) = sample_memory_usage().unwrap_or((0.0, 0.0));
    
    crate::debug::data_types::PerformanceMetrics {
        fps,
        memory_usage_mb,
        memory_usage_percent,
    }
}