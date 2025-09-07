#![cfg(target_arch = "wasm32")]

#[macro_export]
macro_rules! dev_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

#[macro_export]
macro_rules! dev_log_bold {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        web_sys::console::log_2(
            &format!("%c{}", format!($($arg)*)).into(),
            &"font-weight: bold;".into()
        );
    };
}

#[macro_export]
macro_rules! trace_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        web_sys::console::debug_1(&format!("[TRACE] {}", format!($($arg)*)).into());
    };
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

#[macro_export]
macro_rules! error_log {
    ($($arg:tt)*) => {
        web_sys::console::error_1(&format!($($arg)*).into());
    };
}

#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        web_sys::console::warn_1(&format!($($arg)*).into());
    };
}

