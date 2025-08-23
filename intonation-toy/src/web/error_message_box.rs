#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlElement;

#[cfg(target_arch = "wasm32")]
fn show_error_message(title: &str, details: &str) {
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

#[cfg(target_arch = "wasm32")]
pub fn show_error(error: &crate::common::shared_types::Error) {
    show_error_message(error.title(), error.details());
}
#[cfg(target_arch = "wasm32")]
pub fn show_error_with_params(error: &crate::common::shared_types::Error, params: &[&str]) {
    let details = error.details_with(params);
    show_error_message(error.title(), &details);
}

#[cfg(not(target_arch = "wasm32"))]
fn show_error_message(title: &str, details: &str) {
    crate::common::error_log!("Error: {} - {}", title, details);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn show_error(error: &crate::common::shared_types::Error) {
    crate::common::error_log!("Error: {} - {}", error.title(), error.details());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn show_error_with_params(error: &crate::common::shared_types::Error, params: &[&str]) {
    crate::common::error_log!("Error: {} - {}", error.title(), error.details_with(params));
}
