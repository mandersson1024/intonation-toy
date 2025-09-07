#![cfg(target_arch = "wasm32")]

use web_sys::window;

/// Execute a function and wrap it with User Timing marks
/// for profiling in Chrome DevTools Performance panel.
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
    
    // Execute the function
    let result = f();
    
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


/// Macro to conditionally profile code based on the "profiling" feature flag.
/// When profiling is enabled, wraps the expression with profiling marks.
/// When disabled, executes the expression directly without overhead.
///
/// # Examples
/// ```
/// let result = profile!("my_operation", {
///     expensive_computation()
/// });
/// ```
#[macro_export]
macro_rules! profile {
    ($name:expr, $expr:expr) => {
        if cfg!(feature = "profiling") {
            crate::web::profiling::profiled($name, || $expr)
        } else {
            $expr
        }
    };
}