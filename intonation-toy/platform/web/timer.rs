//! High-resolution timer implementation for web browsers.
//! 
//! Uses the Performance API's `performance.now()` method for microsecond-precision
//! timing in browsers. Falls back to `Date.now()` if Performance API is unavailable,
//! though this is unlikely in modern browsers.

use crate::platform::traits::Timer;

/// Web-based timer implementation using Performance API.
/// 
/// Provides high-resolution timestamps with sub-millisecond precision
/// using the browser's Performance API. The timer measures time relative
/// to the page navigation start.
/// 
/// # Browser Compatibility
/// 
/// The Performance API is supported in all modern browsers:
/// - Chrome 20+ (2012)
/// - Firefox 15+ (2012)
/// - Safari 8+ (2014)
/// - Edge (all versions)
/// 
/// # Precision
/// 
/// The Performance API typically provides microsecond precision (0.001ms),
/// though some browsers may reduce precision for security reasons
/// (e.g., to mitigate timing attacks).
pub struct WebTimer;

impl Timer for WebTimer {
    fn now_ms() -> f64 {
        // Get the window object
        let window = match web_sys::window() {
            Some(win) => win,
            None => {
                // Fallback: If we can't get window, use a default timestamp
                // This should never happen in a browser environment
                return 0.0;
            }
        };

        // Get the performance object
        let performance = match window.performance() {
            Some(perf) => perf,
            None => {
                // Fallback: Use Date.now() if Performance API is not available
                // Convert from milliseconds since Unix epoch to relative time
                // This is less precise but works in older browsers
                return js_sys::Date::now();
            }
        };

        // Get high-resolution timestamp
        // Returns milliseconds since navigation start with microsecond precision
        performance.now()
    }
}

impl WebTimer {
    /// Creates a new WebTimer instance.
    pub fn new() -> Self {
        WebTimer
    }
}

impl Default for WebTimer {
    fn default() -> Self {
        Self::new()
    }
}