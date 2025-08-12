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
        #[cfg(all(debug_assertions, target_arch = "wasm32"))]
        web_sys::console::log_1(&format!($($arg)*).into());
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        println!("{}", format!($($arg)*));
    };
}

/// Trace-level logging macro for verbose debugging
macro_rules! trace_log {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions, target_arch = "wasm32"))]
        web_sys::console::debug_1(&format!("[TRACE] {}", format!($($arg)*)).into());
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        println!("[TRACE] {}", format!($($arg)*));
    };
}

/// General logging macro for both release and debug builds
macro_rules! log {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!($($arg)*).into());
        #[cfg(not(target_arch = "wasm32"))]
        println!("{}", format!($($arg)*));
    };
}

/// Error logging macro for both release and debug builds
macro_rules! error_log {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::error_1(&format!($($arg)*).into());
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("{}", format!($($arg)*));
    };
}

/// Warning logging macro for both release and debug builds
macro_rules! warn_log {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::warn_1(&format!($($arg)*).into());
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("{}", format!($($arg)*));
    };
}

pub(crate) use {dev_log, trace_log, log, error_log, warn_log};