//! Timer implementation for native (non-WASM) targets.
//!
//! This module provides a functional timer implementation using `std::time::Instant`
//! for high-resolution monotonic timing on native platforms. Unlike the placeholder
//! implementation that returns constant values, this provides real timing functionality
//! needed for tests and native applications.

use crate::platform::traits::Timer;
use std::sync::OnceLock;
use std::time::Instant;

/// Static initialization of the start time.
/// This provides a consistent reference point for all timer measurements.
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// A timer implementation for native targets using `std::time::Instant`.
///
/// This timer provides:
/// - Monotonic timing that always increases and is not affected by system clock adjustments
/// - High-resolution timing with nanosecond precision (actual precision depends on the platform)
/// - Relative timestamps measured from the first call to `now_ms()`
/// - Thread-safe operation through the use of `OnceLock` for initialization
///
/// # Platform Support
///
/// The underlying `std::time::Instant` provides different levels of precision
/// depending on the operating system:
/// - Linux: typically nanosecond precision
/// - macOS: typically nanosecond precision
/// - Windows: typically 100-nanosecond precision
///
/// # Example
///
/// ```ignore
/// let start = StubTimer::now_ms();
/// // ... do some work ...
/// let elapsed = StubTimer::now_ms() - start;
/// println!("Work took {} ms", elapsed);
/// ```
pub struct StubTimer;

impl Timer for StubTimer {
    /// Returns the current time in milliseconds since the timer was first called.
    ///
    /// The first call to this method establishes the reference point (time 0.0).
    /// All subsequent calls return the elapsed time in milliseconds from that
    /// initial reference point.
    ///
    /// This provides consistent relative timing for tests and native applications
    /// without requiring access to browser performance APIs.
    fn now_ms() -> f64 {
        // Initialize the start time on first call
        let start = START_TIME.get_or_init(|| Instant::now());
        
        // Calculate elapsed time since start
        let elapsed = start.elapsed();
        
        // Convert to milliseconds with sub-millisecond precision
        // Using as_secs_f64() * 1000.0 preserves nanosecond precision
        elapsed.as_secs_f64() * 1000.0
    }
}