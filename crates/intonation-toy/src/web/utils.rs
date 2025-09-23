#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

pub fn rgb_to_css(rgb: [f32; 3]) -> String {
    format!("rgb({}, {}, {})",
        (rgb[0] * 255.0) as u8,
        (rgb[1] * 255.0) as u8,
        (rgb[2] * 255.0) as u8
    )
}

pub fn rgb_to_hex(rgb: [f32; 3]) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (rgb[0] * 255.0) as u8,
        (rgb[1] * 255.0) as u8,
        (rgb[2] * 255.0) as u8,
    )
}

pub fn show_first_click_overlay() {
    web_sys::window().unwrap().document().unwrap()
        .query_selector(".first-click-overlay").unwrap().unwrap()
        .class_list().remove_1("first-click-overlay-hidden").unwrap();
}

pub fn hide_first_click_overlay() {
    web_sys::window().unwrap().document().unwrap()
        .query_selector(".first-click-overlay").unwrap().unwrap()
        .class_list().add_1("first-click-overlay-hidden").unwrap();
}

pub fn hide_preloader() {
    let document = web_sys::window().unwrap().document().unwrap();
    
    if let Ok(Some(preloader)) = document.query_selector("#preloader-overlay") {
        if let Some(parent) = preloader.parent_node() {
            parent.remove_child(&preloader).ok();
        }
    }
}

pub fn get_canvas() -> web_sys::HtmlCanvasElement {
    let window_obj = web_sys::window().unwrap();
    let document = window_obj.document().unwrap();
    
    document.get_element_by_id("three-d-canvas").unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>().unwrap()
}

pub fn get_canvas_style_size() -> f32 {
    let window_obj = web_sys::window().unwrap();
    
    let available_width = window_obj.inner_width().unwrap().as_f64().unwrap() as i32 - crate::web::styling::SIDEBAR_WIDTH - (crate::web::styling::CANVAS_MARGIN * 2);
    let available_height = window_obj.inner_height().unwrap().as_f64().unwrap() as i32 - (crate::web::styling::CANVAS_MARGIN * 2);
    
    std::cmp::min(available_width, available_height)
        .clamp(crate::app_config::CANVAS_MIN_SIZE, crate::app_config::CANVAS_MAX_SIZE) as f32
}

pub fn resize_canvas() {
    let canvas = get_canvas();
    let document = web_sys::window().unwrap().document().unwrap();
    
    let canvas_size = get_canvas_style_size() as i32;
    
    let scene_wrapper = document.get_element_by_id("scene-wrapper").unwrap();
    
    scene_wrapper.set_attribute("style", &format!(
        "position: absolute; top: {}px; left: {}px; width: {}px; height: {}px;",
        crate::web::styling::CANVAS_MARGIN, crate::web::styling::CANVAS_MARGIN, canvas_size, canvas_size
    )).unwrap();
    
    let html_element = canvas.dyn_ref::<web_sys::HtmlElement>().unwrap();
    html_element.style().set_property("width", &format!("{}px", canvas_size)).unwrap();
    html_element.style().set_property("height", &format!("{}px", canvas_size)).unwrap();
}

/// Copy text to the system clipboard using the browser's Clipboard API
pub fn copy_to_clipboard(text: String) {
    spawn_local(async move {
        if let Some(window) = web_sys::window() {
            if let Ok(navigator) = js_sys::Reflect::get(&window.navigator(), &"clipboard".into()) {
                if !navigator.is_undefined() {
                    if let Ok(write_text_fn) = js_sys::Reflect::get(&navigator, &"writeText".into()) {
                        if let Ok(function) = write_text_fn.dyn_into::<js_sys::Function>() {
                            if let Ok(promise) = js_sys::Reflect::apply(
                                &function,
                                &navigator,
                                &js_sys::Array::of1(&text.into())
                            ) {
                                // Convert to Promise and await it
                                if let Ok(promise) = promise.dyn_into::<js_sys::Promise>() {
                                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}