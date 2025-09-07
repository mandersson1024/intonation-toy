#![cfg(target_arch = "wasm32")]

/// Get high-resolution time in milliseconds
pub fn get_high_resolution_time() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}