//! Core platform abstraction traits
//! 
//! These traits define the interface for platform-specific functionality,
//! enabling different implementations for web (WASM) and native targets.
//! The traits are designed to match existing usage patterns in the codebase
//! while providing a clean abstraction boundary.
//!
//! The platform abstraction covers:
//! - Timer functionality for high-resolution time measurements
//! - Performance monitoring for system resource tracking
//! - UI controller operations for platform-specific interface management
//! - Error display functionality for user-facing error messages

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
    
    /// Sets up the main scene UI elements and event handlers
    /// 
    /// This operation should:
    /// - Initialize UI components required for the main scene
    /// - Set up DOM elements and their initial state
    /// - Configure any platform-specific UI behaviors
    /// 
    /// # Integration Notes
    /// - Called during presentation layer initialization
    /// - Must be idempotent - safe to call multiple times
    /// - Web implementation manages HTML elements and CSS classes
    fn setup_ui();
    
    /// Cleans up main scene UI elements and removes event handlers
    /// 
    /// This operation should:
    /// - Remove dynamically created UI elements
    /// - Clean up event listeners to prevent memory leaks
    /// - Reset UI state to initial conditions
    /// 
    /// # Memory Management
    /// - Essential for preventing memory leaks on page transitions
    /// - Should undo all operations performed by setup_ui()
    /// - Must handle cases where setup_ui() was never called
    fn cleanup_ui();
    
    /// Sets up event listeners for UI interaction
    /// 
    /// This operation should:
    /// - Register event handlers for user input (clicks, key presses)
    /// - Set up communication between UI and presentation layer
    /// - Handle platform-specific input methods
    /// 
    /// # Architecture Integration
    /// - Takes a presenter reference for callbacks
    /// - Bridges UI events to presentation layer methods
    /// - Platform implementations handle the specific event registration
    fn setup_event_listeners(presenter: std::rc::Rc<std::cell::RefCell<crate::presentation::Presenter>>);
    
    /// Synchronizes HTML UI state with model data
    /// 
    /// This operation should:
    /// - Update UI elements to reflect current model state
    /// - Handle visual feedback for user interactions
    /// - Maintain consistency between model and view
    /// 
    /// # Three-Layer Architecture
    /// - Called from presentation layer with fresh model data
    /// - Ensures UI reflects the current state without caching
    /// - Platform implementations handle specific UI element updates
    fn sync_ui_state(model_data: &crate::shared_types::ModelUpdateResult);
    
    /// Gets the current UI zoom factor
    /// 
    /// Returns the current zoom level applied to the UI, where:
    /// - 1.0 represents normal/default zoom
    /// - Values > 1.0 represent zoomed in
    /// - Values < 1.0 represent zoomed out
    /// 
    /// # Platform Behavior
    /// - Web platforms return browser zoom level
    /// - Native platforms typically return 1.0 (no zoom)
    /// - Used for scaling calculations in the presentation layer
    fn get_zoom_factor() -> f32;
}

/// Platform-specific error message display
/// 
/// Provides functionality for displaying error messages to users in a
/// platform-appropriate way. This enables consistent error handling
/// across the application while allowing platform-specific presentation.
/// 
/// # Integration with Application Architecture
/// Error display is used throughout all layers of the application:
/// - Engine layer for audio system errors
/// - Model layer for processing errors  
/// - Presentation layer for UI-related errors
/// - Main application for initialization and runtime errors
pub trait ErrorDisplay {
    /// Displays an error message to the user
    /// 
    /// This operation should:
    /// - Present the error in a user-friendly format
    /// - Provide appropriate visual styling and positioning
    /// - Handle error message localization if supported
    /// 
    /// # Platform Behavior
    /// - Web platforms typically show modal overlays or notification UI
    /// - Native platforms may use system notifications or console output
    /// - The display method should be non-blocking where possible
    /// 
    /// # Error Handling
    /// - If display fails, should fall back to console/log output
    /// - Must not throw or panic on display errors
    fn show_error(error: &crate::shared_types::Error);
    
    /// Displays an error message with parameter substitution
    /// 
    /// This operation should:
    /// - Substitute parameters into error message templates
    /// - Handle missing or malformed parameters gracefully
    /// - Maintain consistent formatting with simple error display
    /// 
    /// # Parameter Substitution
    /// - Parameters are substituted in order of appearance
    /// - Missing parameters should be handled without crashing
    /// - Parameter formatting should match the platform's conventions
    /// 
    /// # Usage Notes
    /// - Used for errors that require dynamic content (filenames, values, etc.)
    /// - Parameters are provided as string references for flexibility
    /// - Implementation should validate parameter count and types
    fn show_error_with_params(error: &crate::shared_types::Error, params: &[&str]);
}