//! Core platform abstraction traits
//! 
//! These traits define the interface for platform-specific functionality,
//! enabling different implementations for web (WASM) and native targets.
//! The traits are designed to match existing usage patterns in the codebase
//! while providing a clean abstraction boundary.

/// High-resolution timer functionality
/// 
/// Provides access to monotonic time measurements for frame timing,
/// performance monitoring, and animation scheduling. The timer should
/// provide millisecond precision at minimum, with microsecond precision
/// preferred where available.
/// 
/// # Implementation Notes
/// - Web platforms typically use `performance.now()`
/// - Native platforms may use OS-specific high-resolution timers
/// - The returned time value should be monotonically increasing
pub trait Timer {
    /// Returns the current time in milliseconds
    /// 
    /// The absolute value is not meaningful; only differences between
    /// successive calls should be used for timing calculations.
    /// This matches the behavior of `frame_input.accumulated_time` used
    /// throughout the presentation layer.
    fn now_ms() -> f64;
}

/// System performance monitoring capabilities
/// 
/// Provides access to runtime performance metrics such as memory usage.
/// Not all platforms support all metrics, so methods return `Option` types
/// to indicate availability.
/// 
/// # Browser Compatibility
/// Memory monitoring requires browser support for the Performance Memory API,
/// which may not be available in all browsers or may require specific flags.
pub trait PerformanceMonitor {
    /// Samples current memory usage if available
    /// 
    /// Returns a tuple of (memory_mb, memory_percent) where:
    /// - `memory_mb`: Current memory usage in megabytes
    /// - `memory_percent`: Memory usage as a percentage (0.0-100.0)
    /// 
    /// Returns `None` if memory monitoring is not available on the platform.
    /// 
    /// # Implementation Notes
    /// - Web implementation uses `performance.memory` API
    /// - Availability depends on browser support and security context
    /// - Values should be sampled, not cached, to provide real-time data
    fn sample_memory_usage() -> Option<(f64, f64)>;
}

/// Platform-specific UI controller operations
/// 
/// Manages UI elements that require platform-specific handling,
/// such as canvas resizing and theme application. These operations
/// integrate with the presentation layer but require platform-specific
/// implementation details.
/// 
/// # Integration with Three-Layer Architecture
/// This trait bridges the presentation layer with platform-specific
/// UI operations, maintaining the separation of concerns while
/// allowing for platform-optimized implementations.
pub trait UiController {
    /// Resizes the rendering canvas to match the current window size
    /// 
    /// This operation should:
    /// - Query the current window or container dimensions
    /// - Update the canvas element size attributes
    /// - Handle high-DPI displays appropriately (device pixel ratio)
    /// 
    /// # Browser-Specific Notes
    /// Web implementations need to handle both CSS size and backing
    /// store size for proper rendering on high-DPI displays.
    fn resize_canvas();
    
    /// Applies the current theme styles to the UI
    /// 
    /// This operation should:
    /// - Apply CSS custom properties for theming
    /// - Update any dynamically styled elements
    /// - Ensure theme consistency across all UI components
    /// 
    /// # Implementation Notes
    /// - Web implementation modifies CSS variables on the document root
    /// - Native implementations may use platform-specific theming APIs
    fn apply_theme_styles();
}