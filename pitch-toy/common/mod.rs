// Common utilities and shared types for pitch-toy application

/// Development-only macro for logging
macro_rules! dev_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

pub(crate) use dev_log;