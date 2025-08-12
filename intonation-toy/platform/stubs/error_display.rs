//! Error display stub implementation for native (non-WASM) targets.
//!
//! This module provides console-based error display implementations for native platforms.
//! Since native environments typically don't have browser-style DOM overlays,
//! error messages are logged to stdout/console instead.

use crate::platform::traits::ErrorDisplay;

/// A stub error display implementation for native targets.
///
/// This implementation provides console-based error display methods since:
/// - Native platforms don't have DOM overlays for error messages
/// - Console output is the most appropriate fallback for native/test environments
/// - Error information is still accessible to developers for debugging
///
/// All methods log error information to the console using `println!`, making
/// error details visible during testing and development on native platforms.
///
/// # Future Native Error Display Possibilities
///
/// If native error display becomes necessary, potential approaches include:
/// - Desktop notifications: Using system notification APIs
/// - Terminal UI: Using crates like `ratatui` for rich text-based error displays
/// - Log files: Writing errors to structured log files
/// - Native dialogs: Platform-specific error dialog boxes
///
/// However, console output provides sufficient error visibility for testing
/// and development use cases without additional dependencies.
///
/// # Example
///
/// ```ignore
/// let error = Error::MicrophonePermissionDenied;
/// StubErrorDisplay::show_error(&error);  // Logs error to console
/// StubErrorDisplay::show_error_with_params(&error, &["param1", "param2"]);  // Logs with parameters
/// ```
pub struct StubErrorDisplay;

impl ErrorDisplay for StubErrorDisplay {
    /// Logs error message to console for native platforms.
    ///
    /// This method outputs the error title and details to stdout using `println!`.
    /// The format provides clear error information for debugging and testing.
    ///
    /// # Arguments
    /// * `error` - The error to display, providing title and details
    ///
    /// # Output Format
    /// ```text
    /// [ERROR] <title>: <details>
    /// ```
    fn show_error(error: &crate::shared_types::Error) {
        // Log error to console since native platforms don't have DOM overlays
        // This provides error visibility during testing and development
        println!("[ERROR] {}: {}", error.title(), error.details());
    }
    
    /// Logs error message with parameter substitution to console for native platforms.
    ///
    /// This method substitutes parameters into the error message template and
    /// outputs the result to stdout using `println!`. The format provides
    /// clear error information with dynamic content for debugging and testing.
    ///
    /// # Arguments
    /// * `error` - The error to display, providing title and parameterized details
    /// * `params` - Array of string parameters to substitute into the error message
    ///
    /// # Output Format
    /// ```text
    /// [ERROR] <title>: <details_with_params>
    /// ```
    fn show_error_with_params(error: &crate::shared_types::Error, params: &[&str]) {
        // Log error with parameters to console since native platforms don't have DOM overlays
        // Parameter substitution provides context-specific error information
        let details = error.details_with(params);
        println!("[ERROR] {}: {}", error.title(), details);
    }
}

impl StubErrorDisplay {
    /// Creates a new StubErrorDisplay instance.
    pub fn new() -> Self {
        StubErrorDisplay
    }
}

impl Default for StubErrorDisplay {
    fn default() -> Self {
        Self::new()
    }
}