use crate::common::theme::get_current_color_scheme;
use crate::web::utils::rgb_to_css;

pub const SIDEBAR_WIDTH: i32 = 300;
pub const CANVAS_MARGIN: i32 = 50;
pub const ZOOM_CONTROL_OFFSET: i32 = 12;

fn add_style_to_document(css: &str) {
    let document = web_sys::window()
        .expect("no global window exists")
        .document()
        .expect("should have a document on window");
    let style = document.create_element("style").expect("Failed to create style element");
    style.set_text_content(Some(css));
    let head = document
        .query_selector("head")
        .expect("Failed to query head")
        .expect("document should have head");
    head.append_child(&style)
        .expect("Failed to append style to head");
}

pub fn apply_color_scheme_styles() {
    apply_css_variables();
}

fn create_css_variables_string() -> String {
    let color_scheme = get_current_color_scheme();
    format!(
        "
        --color-background: {};
        --color-surface: {};
        --color-primary: {};
        --color-secondary: {};
        --color-accent: {};
        --color-text: {};
        --color-muted: {};
        --color-border: {};
        --color-error: {};
        ",
        rgb_to_css(color_scheme.background),
        rgb_to_css(color_scheme.surface),
        rgb_to_css(color_scheme.primary),
        rgb_to_css(color_scheme.secondary),
        rgb_to_css(color_scheme.accent),
        rgb_to_css(color_scheme.text),
        rgb_to_css(color_scheme.muted),
        rgb_to_css(color_scheme.border),
        rgb_to_css(color_scheme.error)
    )
}

pub fn apply_css_variables() {
    let css = format!(":root {{{}}}", create_css_variables_string());
    add_style_to_document(&css);
}

pub fn update_css_variables() {
    let document = web_sys::window()
        .expect("no global window exists")
        .document()
        .expect("should have a document on window");
    if let Some(root) = document.document_element() {
        let _ = root.set_attribute("style", &create_css_variables_string());
    }
}


