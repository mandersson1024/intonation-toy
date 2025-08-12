//! Error display implementation for web browsers.
//! 
//! Provides error message display functionality using DOM APIs to show
//! error overlays and messages to users in a platform-appropriate way.
//! 
//! This implementation moves functionality from web/error_message_box.rs
//! into the platform abstraction layer.

use crate::platform::traits::ErrorDisplay;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, Window};

/// Web-based error display implementation.
/// 
/// Manages error message overlays and user notifications through
/// browser DOM APIs.
pub struct WebErrorDisplay;

impl ErrorDisplay for WebErrorDisplay {
    fn show_error(error: &crate::shared_types::Error) {
        show_error_message(error.title(), error.details());
    }
    
    fn show_error_with_params(error: &crate::shared_types::Error, params: &[&str]) {
        let details = error.details_with(params);
        show_error_message(error.title(), &details);
    }
}

impl WebErrorDisplay {
    /// Creates a new WebErrorDisplay instance.
    pub fn new() -> Self {
        WebErrorDisplay
    }
}

impl Default for WebErrorDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// Displays an error message using DOM overlay elements
/// 
/// This function updates existing HTML error overlay elements and makes them visible.
/// The error overlay elements must exist in the HTML document for this to work properly.
/// 
/// # Arguments
/// * `title` - The error title to display
/// * `details` - The error details/description to display
/// 
/// # Implementation Notes
/// - Uses existing DOM elements with IDs: error-message-overlay, error-title, error-details
/// - Shows the overlay by setting display: flex CSS property
/// - Logs debug information to browser console
/// - Gracefully handles missing DOM elements
fn show_error_message(title: &str, details: &str) {
    // Debug logging to console
    web_sys::console::log_1(&format!("show_error_message called with title: {}, details: {}", title, details).into());
    
    let window: Window = match web_sys::window() {
        Some(w) => w,
        None => {
            web_sys::console::error_1(&"Failed to get window".into());
            return;
        }
    };

    let document: Document = match window.document() {
        Some(d) => d,
        None => {
            web_sys::console::error_1(&"Failed to get document".into());
            return;
        }
    };

    // Get existing error overlay elements
    web_sys::console::log_1(&"Getting existing error overlay elements".into());
    
    let overlay = match document.get_element_by_id("error-message-overlay") {
        Some(el) => {
            web_sys::console::log_1(&"Found existing error overlay".into());
            el
        },
        None => {
            web_sys::console::error_1(&"Failed to find error overlay element".into());
            return;
        }
    };

    let title_el = match document.get_element_by_id("error-title") {
        Some(el) => {
            web_sys::console::log_1(&"Found existing error title element".into());
            el
        },
        None => {
            web_sys::console::error_1(&"Failed to find error title element".into());
            return;
        }
    };

    let details_el = match document.get_element_by_id("error-details") {
        Some(el) => {
            web_sys::console::log_1(&"Found existing error details element".into());
            el
        },
        None => {
            web_sys::console::error_1(&"Failed to find error details element".into());
            return;
        }
    };

    // Update content of existing elements
    title_el.set_text_content(Some(title));
    details_el.set_text_content(Some(details));

    // Show the overlay
    if let Ok(html_element) = overlay.dyn_into::<HtmlElement>() {
        if let Err(e) = html_element.style().set_property("display", "flex") {
            web_sys::console::error_1(&format!("Failed to show overlay: {:?}", e).into());
        } else {
            web_sys::console::log_1(&"Successfully showed error overlay".into());
        }
    } else {
        web_sys::console::error_1(&"Failed to cast overlay to HtmlElement".into());
    }
}

/// Hides the error message overlay
/// 
/// This function hides the error overlay by setting display: none CSS property.
/// Used internally for error message management.
/// 
/// # Implementation Notes
/// - Sets display: none on the error-message-overlay element
/// - Logs debug information to browser console
/// - Gracefully handles missing DOM elements
#[allow(dead_code)]
pub fn hide_error_message() {
    web_sys::console::log_1(&"hide_error_message called".into());
    
    let window: Window = match web_sys::window() {
        Some(w) => w,
        None => {
            web_sys::console::error_1(&"Failed to get window in hide_error_message".into());
            return;
        }
    };

    let document: Document = match window.document() {
        Some(d) => d,
        None => {
            web_sys::console::error_1(&"Failed to get document in hide_error_message".into());
            return;
        }
    };

    if let Some(overlay) = document.get_element_by_id("error-message-overlay") {
        web_sys::console::log_1(&"Found overlay to hide - hiding it".into());
        if let Ok(html_element) = overlay.dyn_into::<HtmlElement>() {
            if let Err(e) = html_element.style().set_property("display", "none") {
                web_sys::console::error_1(&format!("Failed to hide overlay: {:?}", e).into());
            } else {
                web_sys::console::log_1(&"Successfully hid error overlay".into());
            }
        } else {
            web_sys::console::error_1(&"Failed to cast overlay to HtmlElement in hide".into());
        }
    } else {
        web_sys::console::log_1(&"No overlay found to hide".into());
    }
}