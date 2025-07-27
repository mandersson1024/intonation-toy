// Common utilities and shared types for pitch-toy application
//
// Logging Optimization Features:
// - `separation_log!`: Feature-gated macro for separation verification logging
//   (only enabled with --features separation-logging)
// - `trace_log!`: Verbose debugging for less critical logs (uses console.debug)
// - `dev_log!`: Standard development logging (uses console.log)

/// Development-only macro for logging
macro_rules! dev_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

/// Trace-level logging macro for verbose debugging
macro_rules! trace_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        web_sys::console::debug_1(&format!("[TRACE] {}", format!($($arg)*)).into());
    };
}

/// Separation verification logging macro - only enabled with feature flag
macro_rules! separation_log {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions, feature = "separation-logging"))]
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

pub(crate) use {dev_log, trace_log, separation_log};