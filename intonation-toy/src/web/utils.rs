
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
    wasm_bindgen::JsCast::dyn_into::<js_sys::Function>(js_sys::Reflect::get(&web_sys::window().unwrap(), &wasm_bindgen::JsValue::from_str("removePreloader")).unwrap()).unwrap()
        .call0(&wasm_bindgen::JsValue::NULL).unwrap();
}