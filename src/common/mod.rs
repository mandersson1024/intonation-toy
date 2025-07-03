// Common utilities and shared types for pitch-toy application

/// Development-only macro for logging
/// Uses browser console for wasm32, stdout for native targets
macro_rules! dev_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(&format!($($arg)*).into());
            }
            
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!($($arg)*);
            }
        }
    };
}

pub(crate) use dev_log;