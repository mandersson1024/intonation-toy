// Common utilities and shared types for pitch-toy application

/// Log a message to the browser console (development builds only)
#[cfg(debug_assertions)]
pub fn log(message: &str) {
    web_sys::console::log_1(&message.into());
}

/// Development-only macro for logging
#[cfg(debug_assertions)]
macro_rules! dev_log {
    ($($arg:tt)*) => {
        crate::modules::common::log(&format!($($arg)*));
    };
}

#[cfg(not(debug_assertions))]
macro_rules! dev_log {
    ($($arg:tt)*) => {};
}

pub(crate) use dev_log;