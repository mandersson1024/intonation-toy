use crate::theme::get_current_color_scheme;
use crate::web::utils::rgb_to_css;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement};

/// Sidebar width in pixels - used consistently across styling and canvas sizing
pub const SIDEBAR_WIDTH: i32 = 300;

fn get_document() -> Document {
    web_sys::window()
        .expect("no global window exists")
        .document()
        .expect("should have a document on window")
}

fn apply_style_to_element(selector: &str, style: &str) {
    let document = get_document();
    if let Some(element) = document.query_selector(selector).ok().flatten() {
        if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
            html_element.set_attribute("style", style).ok();
        }
    }
}

fn add_style_to_document(css: &str) {
    let document = get_document();
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
    // Apply CSS variables - all other styles are now handled by static/style.css
    apply_css_variables();
}





/// Create and set CSS custom properties on the document root based on the current theme's ColorScheme.
/// This function creates CSS variables that can be used by CSS classes for efficient theme switching.
/// Variables follow the pattern: --color-background, --color-surface, etc.
pub fn apply_css_variables() {
    let color_scheme = get_current_color_scheme();
    
    let css = format!(
        r#":root {{
            --color-background: {};
            --color-surface: {};
            --color-primary: {};
            --color-secondary: {};
            --color-accent: {};
            --color-text: {};
            --color-muted: {};
        }}"#,
        rgb_to_css(color_scheme.background),
        rgb_to_css(color_scheme.surface),
        rgb_to_css(color_scheme.primary),
        rgb_to_css(color_scheme.secondary),
        rgb_to_css(color_scheme.accent),
        rgb_to_css(color_scheme.text),
        rgb_to_css(color_scheme.muted)
    );
    
    add_style_to_document(&css);
}

/// Update only the CSS custom properties for efficient theme switching.
/// This function directly updates the CSS variables on the document root element
/// without creating new style elements, making it more efficient than apply_css_variables().
pub fn update_css_variables() {
    let color_scheme = get_current_color_scheme();
    
    let style = format!(
        "--color-background: {}; --color-surface: {}; --color-primary: {}; --color-secondary: {}; --color-accent: {}; --color-text: {}; --color-muted: {};",
        rgb_to_css(color_scheme.background),
        rgb_to_css(color_scheme.surface),
        rgb_to_css(color_scheme.primary),
        rgb_to_css(color_scheme.secondary),
        rgb_to_css(color_scheme.accent),
        rgb_to_css(color_scheme.text),
        rgb_to_css(color_scheme.muted)
    );
    
    apply_style_to_element(":root", &style);
}

/// Reapply the current theme by updating CSS custom properties.
/// All styling is now CSS class-based with custom properties, so only updating
/// the CSS variables is needed for efficient theme switching.
pub fn reapply_current_theme() {
    update_css_variables();
}