// Common utilities and shared types for pitch-toy application
//
// Logging Optimization Features:
// - `trace_log!`: Verbose debugging for less critical logs (uses console.debug) - debug only
// - `dev_log!`: Standard development logging (uses console.log) - debug only
// - `log!`: General logging for both release and debug builds (uses console.log)
// - `error_log!`: Error logging for both release and debug builds (uses console.error)
// - `warn_log!`: Warning logging for both release and debug builds (uses console.warn)

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

/// General logging macro for both release and debug builds
macro_rules! log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

/// Error logging macro for both release and debug builds
macro_rules! error_log {
    ($($arg:tt)*) => {
        web_sys::console::error_1(&format!($($arg)*).into());
    };
}

/// Warning logging macro for both release and debug builds
macro_rules! warn_log {
    ($($arg:tt)*) => {
        web_sys::console::warn_1(&format!($($arg)*).into());
    };
}

pub(crate) use {dev_log, trace_log, log, error_log, warn_log};

pub mod smoothing;