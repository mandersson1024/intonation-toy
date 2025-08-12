//! Performance monitoring stub implementation for native (non-WASM) targets.
//!
//! This module provides a no-op performance monitor implementation for native platforms.
//! Memory monitoring is not implemented as it would require platform-specific APIs
//! and is not necessary for basic testing and development.

use crate::platform::traits::PerformanceMonitor;

/// A stub performance monitor implementation for native targets.
///
/// This implementation always returns `None` for memory usage sampling since:
/// - Native memory monitoring would require platform-specific system APIs
/// - Memory profiling is not needed for unit tests and basic development
/// - Cross-platform memory monitoring is complex and out of scope for stub implementations
///
/// # Future Implementation Possibilities
///
/// If native memory monitoring becomes necessary, potential approaches include:
/// - Linux: Reading from `/proc/self/status` or using `getrusage()`
/// - macOS: Using `mach_task_info()` or `task_info()`
/// - Windows: Using `GetProcessMemoryInfo()` from the Windows API
/// - Cross-platform: Using crates like `sysinfo` or `psutil`
///
/// However, these would add dependencies and complexity that are not justified
/// for the current use case of enabling tests and development on native platforms.
///
/// # Example
///
/// ```ignore
/// match StubPerformanceMonitor::sample_memory_usage() {
///     Some((mb, percent)) => println!("Memory: {} MB ({}%)", mb, percent),
///     None => println!("Memory monitoring not available"),
/// }
/// // Will always print: "Memory monitoring not available"
/// ```
pub struct StubPerformanceMonitor;

impl PerformanceMonitor for StubPerformanceMonitor {
    /// Always returns `None` as memory monitoring is not implemented for native targets.
    ///
    /// This is a no-op implementation that maintains API compatibility with the
    /// web implementation while acknowledging that memory monitoring is not
    /// available or necessary for headless testing scenarios.
    fn sample_memory_usage() -> Option<(f64, f64)> {
        // Memory monitoring not available for native targets
        // This would require platform-specific APIs that are beyond
        // the scope of minimal stub implementations for testing
        None
    }
}