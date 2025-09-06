#[macro_export]
macro_rules! dev_log {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions, target_arch = "wasm32"))]
        web_sys::console::log_1(&format!($($arg)*).into());
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        println!($($arg)*);
    };
}

#[macro_export]
macro_rules! dev_log_bold {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions, target_arch = "wasm32"))]
        web_sys::console::log_2(
            &format!("%c{}", format!($($arg)*)).into(),
            &"font-weight: bold;".into()
        );
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        println!($($arg)*);
    };
}

#[macro_export]
macro_rules! trace_log {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions, target_arch = "wasm32"))]
        web_sys::console::debug_1(&format!("[TRACE] {}", format!($($arg)*)).into());
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        println!("[TRACE] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!($($arg)*).into());
        #[cfg(not(target_arch = "wasm32"))]
        println!($($arg)*);
    };
}

#[macro_export]
macro_rules! error_log {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::error_1(&format!($($arg)*).into());
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!($($arg)*);
    };
}

#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::warn_1(&format!($($arg)*).into());
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("[WARN] {}", format!($($arg)*));
    };
}

