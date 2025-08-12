//! UI controller stub implementation for native (non-WASM) targets.
//!
//! This module provides no-op UI controller implementations for native platforms.
//! Since tests typically run in headless environments without a GUI, all UI
//! operations are safely ignored.

use crate::platform::traits::UiController;

/// A stub UI controller implementation for native targets.
///
/// This implementation provides no-op methods for all UI operations since:
/// - Tests typically run in headless environments without a display
/// - Native GUI implementation would require platform-specific windowing libraries
/// - UI operations are not necessary for unit testing business logic
///
/// All methods can be safely called without side effects, making this implementation
/// suitable for testing code that may invoke UI operations as part of its normal flow.
///
/// # Future Native UI Possibilities
///
/// If native UI support becomes necessary, potential approaches include:
/// - Desktop GUI: Using crates like `egui`, `iced`, or `tauri` for cross-platform UIs
/// - Terminal UI: Using crates like `ratatui` or `cursive` for text-based interfaces
/// - Native widgets: Platform-specific libraries (GTK, Qt, Cocoa, Win32)
///
/// However, these would significantly increase complexity and dependencies,
/// which is not justified for the current testing and development use case.
///
/// # Example
///
/// ```ignore
/// StubUiController::resize_canvas();  // No-op, safe to call
/// StubUiController::apply_theme_styles();  // No-op, safe to call
/// // Both calls complete without any effect
/// ```
pub struct StubUiController;

impl UiController for StubUiController {
    /// No-op implementation for canvas resizing.
    ///
    /// This method does nothing since there is no canvas to resize in headless
    /// test environments. The method can be safely called without errors.
    fn resize_canvas() {
        // No-op: No canvas exists in headless/test environments
        // This allows tests to run without requiring a display server
        // or any GUI components
    }

    /// No-op implementation for theme style application.
    ///
    /// This method does nothing since there are no UI elements to style in
    /// headless test environments. The method can be safely called without errors.
    fn apply_theme_styles() {
        // No-op: No UI elements exist in headless/test environments
        // This allows theme-switching code to be tested without
        // requiring actual style application
    }
}