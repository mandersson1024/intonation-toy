#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

fn show_error_box(title: &str, details: &str) {
    crate::common::error_log!("Error: {} - {}", title, details);

    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    
    let Some(overlay) = document.get_element_by_id("error-message-overlay") else { return };
    let Some(title_el) = document.get_element_by_id("error-title") else { return };
    let Some(details_el) = document.get_element_by_id("error-details") else { return };

    title_el.set_text_content(Some(title));
    details_el.set_text_content(Some(details));

    if let Ok(html_element) = overlay.dyn_into::<HtmlElement>() {
        let _ = html_element.style().set_property("display", "flex");
    }
}

pub fn show_error(error: &crate::common::shared_types::Error) {
    show_error_box(error.title(), error.details());
}
pub fn show_error_with_params(error: &crate::common::shared_types::Error, params: &[&str]) {
    let details = error.details_with(params);
    show_error_box(error.title(), &details);
}

