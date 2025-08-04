#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{Document, HtmlElement, Window};

use crate::common::dev_log;

#[cfg(target_arch = "wasm32")]
pub fn show_error_message(title: &str, details: &str) {
    let window: Window = match web_sys::window() {
        Some(w) => w,
        None => {
            dev_log!("Failed to get window");
            return;
        }
    };

    let document: Document = match window.document() {
        Some(d) => d,
        None => {
            dev_log!("Failed to get document");
            return;
        }
    };

    // Remove any existing error message
    hide_error_message();

    // Create overlay
    let overlay = match document.create_element("div") {
        Ok(el) => el,
        Err(e) => {
            dev_log!("Failed to create overlay element: {:?}", e);
            return;
        }
    };

    if let Err(e) = overlay.set_attribute("id", "error-message-overlay") {
        dev_log!("Failed to set overlay id: {:?}", e);
    }

    if let Err(e) = overlay.set_attribute(
        "style",
        "position: fixed; top: 0; left: 0; width: 100%; height: 100%; \
         background-color: rgba(0, 0, 0, 0.8); display: flex; \
         justify-content: center; align-items: center; z-index: 10000;",
    ) {
        dev_log!("Failed to set overlay style: {:?}", e);
    }

    // Create error panel
    let panel = match document.create_element("div") {
        Ok(el) => el,
        Err(e) => {
            dev_log!("Failed to create panel element: {:?}", e);
            return;
        }
    };

    if let Err(e) = panel.set_attribute(
        "style",
        "background-color: #ef4444; color: white; padding: 30px; \
         border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); \
         text-align: center; min-width: 400px; max-width: 600px; \
         font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; \
         transition: transform 0.2s ease;",
    ) {
        dev_log!("Failed to set panel style: {:?}", e);
    }

    // Create title element
    let title_el = match document.create_element("h2") {
        Ok(el) => el,
        Err(e) => {
            dev_log!("Failed to create title element: {:?}", e);
            return;
        }
    };

    title_el.set_text_content(Some(title));

    if let Err(e) = title_el.set_attribute(
        "style",
        "margin: 0 0 16px 0; font-size: 24px; font-weight: bold;",
    ) {
        dev_log!("Failed to set title style: {:?}", e);
    }

    // Create details element
    let details_el = match document.create_element("p") {
        Ok(el) => el,
        Err(e) => {
            dev_log!("Failed to create details element: {:?}", e);
            return;
        }
    };

    details_el.set_text_content(Some(details));

    if let Err(e) = details_el.set_attribute(
        "style",
        "margin: 0; font-size: 16px; line-height: 1.5; opacity: 0.95;",
    ) {
        dev_log!("Failed to set details style: {:?}", e);
    }

    // Assemble the error panel
    if let Err(e) = panel.append_child(&title_el) {
        dev_log!("Failed to append title to panel: {:?}", e);
    }

    if let Err(e) = panel.append_child(&details_el) {
        dev_log!("Failed to append details to panel: {:?}", e);
    }

    if let Err(e) = overlay.append_child(&panel) {
        dev_log!("Failed to append panel to overlay: {:?}", e);
    }

    // Add to document body
    match document.body() {
        Some(body) => {
            if let Err(e) = body.append_child(&overlay) {
                dev_log!("Failed to append overlay to body: {:?}", e);
            }
        }
        None => {
            dev_log!("Failed to get document body");
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn hide_error_message() {
    let window: Window = match web_sys::window() {
        Some(w) => w,
        None => {
            dev_log!("Failed to get window");
            return;
        }
    };

    let document: Document = match window.document() {
        Some(d) => d,
        None => {
            dev_log!("Failed to get document");
            return;
        }
    };

    if let Some(overlay) = document.get_element_by_id("error-message-overlay") {
        if let Err(e) = overlay.remove() {
            dev_log!("Failed to remove error overlay: {:?}", e);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn show_error_message(_title: &str, _details: &str) {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn hide_error_message() {
    // No-op for non-WASM targets
}