
pub fn rgb_to_css(rgb: [f32; 3]) -> String {
    format!("rgb({}, {}, {})", 
        (rgb[0] * 255.0) as u8, 
        (rgb[1] * 255.0) as u8, 
        (rgb[2] * 255.0) as u8
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

pub fn resize_canvas(canvas: &web_sys::HtmlCanvasElement) {
    use wasm_bindgen::JsCast;
    
    let window_obj = web_sys::window().unwrap();
    let document = window_obj.document().unwrap();
    
    let available_width = window_obj.inner_width().unwrap().as_f64().unwrap() as i32 - crate::web::styling::SIDEBAR_WIDTH - (crate::web::styling::CANVAS_MARGIN * 2);
    let available_height = window_obj.inner_height().unwrap().as_f64().unwrap() as i32 - (crate::web::styling::CANVAS_MARGIN * 2);
    
    let canvas_size = std::cmp::min(available_width, available_height)
        .min(crate::app_config::CANVAS_MAX_SIZE)
        .max(crate::app_config::CANVAS_MIN_SIZE);
    
    let scene_wrapper = document.get_element_by_id("scene-wrapper").unwrap();
    
    scene_wrapper.set_attribute("style", &format!(
        "position: absolute; top: {}px; left: {}px; width: {}px; height: {}px;",
        crate::web::styling::CANVAS_MARGIN, crate::web::styling::CANVAS_MARGIN, canvas_size, canvas_size
    )).unwrap();
    
    let html_element = canvas.dyn_ref::<web_sys::HtmlElement>().unwrap();
    html_element.style().set_property("width", &format!("{}px", canvas_size)).unwrap();
    html_element.style().set_property("height", &format!("{}px", canvas_size)).unwrap();
}