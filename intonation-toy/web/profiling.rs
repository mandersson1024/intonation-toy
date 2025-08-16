#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{console, window};

/// Execute a function and wrap it with User Timing marks and console.time measurements
/// for profiling in Chrome DevTools Performance panel.
#[cfg(target_arch = "wasm32")]
pub fn profiled<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Start User Timing mark
    if let Some(window) = window() {
        if let Some(performance) = window.performance() {
            let start_mark = format!("{}-start", name);
            let _ = performance.mark(&start_mark);
        }
    }
    
    // Start console timer
    console::time_with_label(name);
    
    // Execute the function
    let result = f();
    
    // End console timer
    console::time_end_with_label(name);
    
    // End User Timing mark and create measure
    if let Some(window) = window() {
        if let Some(performance) = window.performance() {
            let start_mark = format!("{}-start", name);
            let end_mark = format!("{}-end", name);
            let _ = performance.mark(&end_mark);
            let _ = performance.measure_with_start_mark_and_end_mark(name, &start_mark, &end_mark);
        }
    }
    
    result
}

/// No-op fallback for non-wasm targets
#[cfg(not(target_arch = "wasm32"))]
pub fn profiled<F, R>(_name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}