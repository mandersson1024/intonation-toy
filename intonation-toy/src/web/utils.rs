
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