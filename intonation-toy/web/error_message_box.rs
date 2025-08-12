#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, Window};
use crate::common::{dev_log, error_log};

pub fn show_error_message(title: &str, details: &str) {
    // Debug logging to console
    dev_log!("show_error_message called with title: {}, details: {}", title, details);
    
    let window: Window = match web_sys::window() {
        Some(w) => w,
        None => {
            error_log!("Failed to get window");
            return;
        }
    };

    let document: Document = match window.document() {
        Some(d) => d,
        None => {
            error_log!("Failed to get document");
            return;
        }
    };

    // Get existing error overlay elements
    
    let overlay = match document.get_element_by_id("error-message-overlay") {
        Some(el) => el,
        None => {
            error_log!("Failed to find error overlay element");
            return;
        }
    };

    let title_el = match document.get_element_by_id("error-title") {
        Some(el) => el,
        None => {
            error_log!("Failed to find error title element");
            return;
        }
    };

    let details_el = match document.get_element_by_id("error-details") {
        Some(el) => el,
        None => {
            error_log!("Failed to find error details element");
            return;
        }
    };

    // Update content of existing elements
    title_el.set_text_content(Some(title));
    details_el.set_text_content(Some(details));

    // Show the overlay
    if let Ok(html_element) = overlay.dyn_into::<HtmlElement>() {
        if let Err(e) = html_element.style().set_property("display", "flex") {
            error_log!("Failed to show overlay: {:?}", e);
        }
    } else {
        error_log!("Failed to cast overlay to HtmlElement");
    }
}

/// Convenience function to show an error message from an Error enum variant.
/// For errors with dynamic content, use show_error_message_with_params instead.
pub fn show_error(error: &crate::shared_types::Error) {
    show_error_message(error.title(), error.details());
}

/// Convenience function to show an error message from an Error enum variant with parameters.
/// Use this for errors that need dynamic content like missing API names.
pub fn show_error_with_params(error: &crate::shared_types::Error, params: &[&str]) {
    let details = error.details_with(params);
    show_error_message(error.title(), &details);
}

pub fn hide_error_message() {
    
    let window: Window = match web_sys::window() {
        Some(w) => w,
        None => {
            error_log!("Failed to get window in hide_error_message");
            return;
        }
    };

    let document: Document = match window.document() {
        Some(d) => d,
        None => {
            error_log!("Failed to get document in hide_error_message");
            return;
        }
    };

    if let Some(overlay) = document.get_element_by_id("error-message-overlay") {
        if let Ok(html_element) = overlay.dyn_into::<HtmlElement>() {
            if let Err(e) = html_element.style().set_property("display", "none") {
                error_log!("Failed to hide overlay: {:?}", e);
            }
        } else {
            error_log!("Failed to cast overlay to HtmlElement in hide");
        }
    }
}

