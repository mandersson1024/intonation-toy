use crate::theme::get_current_color_scheme;
use crate::web::utils::rgb_to_css;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement};

/// Sidebar width in pixels - used consistently across styling and canvas sizing
pub const SIDEBAR_WIDTH: i32 = 300;

/// Canvas margin in pixels - now applies to the scene wrapper container
pub const CANVAS_MARGIN: i32 = 100;

/// Zoom control offset from canvas edge in pixels
pub const ZOOM_CONTROL_OFFSET: i32 = 12;

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
/// Works with static/style.css which provides fallback values for better browser compatibility.
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
            --color-border: {};
            --color-error: {};
        }}"#,
        rgb_to_css(color_scheme.background),
        rgb_to_css(color_scheme.surface),
        rgb_to_css(color_scheme.primary),
        rgb_to_css(color_scheme.secondary),
        rgb_to_css(color_scheme.accent),
        rgb_to_css(color_scheme.text),
        rgb_to_css(color_scheme.muted),
        rgb_to_css(color_scheme.border),
        rgb_to_css(color_scheme.error),
    );
    
    add_style_to_document(&css);
    crate::common::dev_log!("Applied CSS custom properties for theme initialization");
}

/// Update only the CSS custom properties for efficient theme switching.
/// This function directly updates the CSS variables on the document root element
/// without creating new style elements, making it more efficient than apply_css_variables().
/// Works seamlessly with static/style.css which references these CSS custom properties.
pub fn update_css_variables() {
    let color_scheme = get_current_color_scheme();
    
    // Apply to document.documentElement (html element) instead of :root selector
    let document = get_document();
    if let Some(root) = document.document_element() {
        // Use set_attribute to set style properties directly on the element
        let style_str = format!(
            "--color-background: {}; --color-surface: {}; --color-primary: {}; --color-secondary: {}; --color-accent: {}; --color-text: {}; --color-muted: {}; --color-border: {}; --color-error: {};",
            rgb_to_css(color_scheme.background),
            rgb_to_css(color_scheme.surface),
            rgb_to_css(color_scheme.primary),
            rgb_to_css(color_scheme.secondary),
            rgb_to_css(color_scheme.accent),
            rgb_to_css(color_scheme.text),
            rgb_to_css(color_scheme.muted),
            rgb_to_css(color_scheme.border),
            rgb_to_css(color_scheme.error)
        );
        
        if let Err(_) = root.set_attribute("style", &style_str) {
            crate::common::dev_log!("Failed to set CSS variables on root element");
        } else {
            crate::common::dev_log!("Successfully updated CSS custom properties");
        }
    }
}

/// Reapply the current theme by updating CSS custom properties.
/// All styling is now handled by static/style.css with CSS custom properties, 
/// so only updating the CSS variables is needed for efficient theme switching.
/// This approach ensures theme changes are applied instantly across all UI elements.
pub fn reapply_current_theme() {
    update_css_variables();
}


/// Helper function to safely apply styles to DOM elements with error handling.
/// Returns Result for better error management in theme switching operations.
fn try_apply_style_to_element(selector: &str, style: &str) -> Result<(), String> {
    let document = get_document();
    let element = document
        .query_selector(selector)
        .map_err(|_| "Failed to query selector")?
        .ok_or("Element not found")?;
    
    if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
        html_element
            .set_attribute("style", style)
            .map_err(|_| "Failed to set style attribute")?;
        Ok(())
    } else {
        Err("Element is not an HTML element".to_string())
    }
}
