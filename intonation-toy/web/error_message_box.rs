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

    // Remove any existing error message
    web_sys::console::log_1(&"About to hide existing error messages".into());
    hide_error_message();

    // Create overlay
    web_sys::console::log_1(&"Creating overlay element".into());
    let overlay = match document.create_element("div") {
        Ok(el) => {
            web_sys::console::log_1(&"Overlay element created successfully".into());
            el
        },
        Err(e) => {
            web_sys::console::error_1(&format!("Failed to create overlay element: {:?}", e).into());
            return;
        }
    };

    if let Err(e) = overlay.set_attribute("id", "error-message-overlay") {
        dev_log!("Failed to set overlay id: {:?}", e);
    }

    let overlay_style = styling::get_error_overlay_style();
    
    if let Err(e) = overlay.set_attribute("style", &overlay_style) {
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

    let panel_style = styling::get_error_panel_style();
    
    if let Err(e) = panel.set_attribute("style", &panel_style) {
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
        &styling::get_error_title_style(),
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
        &styling::get_error_details_style(),
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
    web_sys::console::log_1(&"About to append overlay to body".into());
    match document.body() {
        Some(body) => {
            if let Err(e) = body.append_child(&overlay) {
                web_sys::console::error_1(&format!("Failed to append overlay to body: {:?}", e).into());
            } else {
                web_sys::console::log_1(&"Successfully appended overlay to body".into());
                // Verify it's actually there
                if let Some(check_overlay) = document.get_element_by_id("error-message-overlay") {
                    web_sys::console::log_1(&"Overlay confirmed to exist in DOM".into());
                } else {
                    web_sys::console::error_1(&"Overlay NOT found in DOM immediately after append!".into());
                }
            }
        }
        None => {
            web_sys::console::error_1(&"Failed to get document body".into());
        }
    }
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
        web_sys::console::log_1(&"Found overlay to hide - removing it".into());
        overlay.remove();
    } else {
        web_sys::console::log_1(&"No overlay found to hide".into());
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