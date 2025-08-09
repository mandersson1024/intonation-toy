#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{Document, HtmlElement, Window};

use crate::common::dev_log;
use crate::web::styling;

#[cfg(target_arch = "wasm32")]
pub fn show_error_message(title: &str, details: &str) {
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
        if let Err(e) = html_element.style().set_property("display", "block") {
            web_sys::console::error_1(&format!("Failed to show overlay: {:?}", e).into());
        } else {
            web_sys::console::log_1(&"Successfully showed error overlay".into());
        }
    } else {
        web_sys::console::error_1(&"Failed to cast overlay to HtmlElement".into());
    }
}

/// Convenience function to show an error message from an Error enum variant.
/// For errors with dynamic content, use show_error_message_with_params instead.
#[cfg(target_arch = "wasm32")]
pub fn show_error(error: &crate::shared_types::Error) {
    show_error_message(error.title(), error.details());
}

/// Convenience function to show an error message from an Error enum variant with parameters.
/// Use this for errors that need dynamic content like missing API names.
#[cfg(target_arch = "wasm32")]
pub fn show_error_with_params(error: &crate::shared_types::Error, params: &[&str]) {
    let details = error.details_with(params);
    show_error_message(error.title(), &details);
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
pub fn show_error_message(_title: &str, _details: &str) {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn show_error(_error: &crate::shared_types::Error) {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn show_error_with_params(_error: &crate::shared_types::Error, _params: &[&str]) {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn hide_error_message() {
    // No-op for non-WASM targets
}