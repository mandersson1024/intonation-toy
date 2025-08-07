use crate::app_config::COLOR_SCHEME;
use crate::web::utils::{rgb_to_css, rgba_to_css};
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement};

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

pub fn apply_css_reset() {
    let css = r#"
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
    "#;
    add_style_to_document(css);
}

pub fn apply_body_styles() {
    let style = format!(
        "background-color: {}; color: {}; font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; display: flex; flex-direction: row; min-height: 100vh; overflow: hidden;",
        rgb_to_css(COLOR_SCHEME.background),
        rgb_to_css(COLOR_SCHEME.text)
    );
    apply_style_to_element("body", &style);
}

pub fn apply_sidebar_styles() {
    let gradient_start = rgba_to_css(COLOR_SCHEME.surface, 0.95);
    let gradient_end = rgba_to_css(COLOR_SCHEME.surface, 0.85);
    let border_color = rgba_to_css(COLOR_SCHEME.muted, 0.3);
    
    let style = format!(
        "position: fixed; top: 0; left: 0; bottom: 0; width: 300px; background: linear-gradient(to right, {}, {}); border-right: 1px solid {}; display: flex; flex-direction: column; padding: 20px; z-index: 1000; backdrop-filter: blur(10px); overflow-y: auto;",
        gradient_start,
        gradient_end,
        border_color
    );
    apply_style_to_element("#sidebar", &style);
}

pub fn apply_canvas_container_styles() {
    let style = "position: fixed; top: 0; left: 300px; right: 0; bottom: 0; display: flex; justify-content: center; align-items: center;";
    apply_style_to_element("#canvas-container", &style);
}

pub fn apply_canvas_styles() {
    // Using red border and black background as specified in index.css
    let style = "border: 3px solid red; border-radius: 4px; background-color: #000; display: block; width: 100%; height: 100%;";
    apply_style_to_element("#three-d-canvas", &style);
}

pub fn get_status_success_style() -> String {
    "color: #10b981;".to_string()
}

pub fn get_status_neutral_style() -> String {
    "color: #ffffff;".to_string()
}

pub fn get_status_active_style() -> String {
    "color: #06b6d4;".to_string()
}

pub fn get_status_pending_style() -> String {
    "color: #f59e0b;".to_string()
}

pub fn get_status_warning_style() -> String {
    "color: #fb923c;".to_string()
}

pub fn get_status_error_style() -> String {
    "color: #ef4444;".to_string()
}

pub fn get_status_inactive_style() -> String {
    "color: #6b7280;".to_string()
}

pub fn apply_status_classes() {
    // Enhanced status classes with !important and font-weight as in index.css
    let css = r#"
        .status-value.status-success,
        span.status-success {
            color: #22c55e !important;
            font-weight: 600;
        }
        .status-value.status-neutral,
        span.status-neutral {
            color: #ffffff !important;
            font-weight: 600;
        }
        .status-value.status-active,
        span.status-active {
            color: #06b6d4 !important;
            font-weight: 600;
        }
        .status-value.status-pending,
        span.status-pending {
            color: #f59e0b !important;
            font-weight: 600;
        }
        .status-value.status-warning,
        span.status-warning {
            color: #f97316 !important;
            font-weight: 600;
        }
        .status-value.status-error,
        span.status-error {
            color: #ef4444 !important;
            font-weight: 600;
        }
        .status-value.status-inactive,
        span.status-inactive {
            color: #6b7280 !important;
            font-weight: 400;
        }
        .permission-required {
            color: #6b7280 !important;
            font-style: italic !important;
            font-weight: 400;
        }
    "#;
    add_style_to_document(css);
}

pub fn get_button_style() -> String {
    format!(
        "background-color: {}; color: {}; border: 1px solid {}; padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: 500; transition: all 0.2s;",
        rgb_to_css(COLOR_SCHEME.surface),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.primary, 0.3)
    )
}

pub fn get_select_style() -> String {
    format!(
        "background-color: {}; color: {}; border: 1px solid {}; padding: 6px 12px; border-radius: 6px; font-size: 14px; cursor: pointer; min-width: 120px;",
        rgb_to_css(COLOR_SCHEME.surface),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.primary, 0.3)
    )
}

pub fn get_input_style() -> String {
    format!(
        "background-color: {}; color: {}; border: 1px solid {}; padding: 6px 12px; border-radius: 6px; font-size: 14px;",
        rgb_to_css(COLOR_SCHEME.surface),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.primary, 0.3)
    )
}

pub fn get_label_style() -> String {
    format!(
        "color: {}; font-size: 14px; font-weight: 500; margin-right: 8px;",
        rgb_to_css(COLOR_SCHEME.text)
    )
}

pub fn get_control_range_style() -> String {
    "display: flex; align-items: center; gap: 12px;".to_string()
}

pub fn apply_control_range_styles() {
    // Enhanced control range styling with proper layout from index.css
    let css = r#"
        .control-item.control-range {
            display: flex !important;
            flex-direction: row !important;
            align-items: center !important;
            gap: 12px !important;
            justify-content: space-between !important;
        }
        .control-item.control-range .control-label {
            min-width: 120px !important;
            flex-shrink: 0 !important;
            margin-bottom: 0 !important;
        }
        .control-item.control-range .control-slider-container {
            flex: 1 !important;
            display: flex !important;
            align-items: center !important;
            gap: 8px !important;
            margin-top: 0 !important;
        }
    "#;
    add_style_to_document(css);
}

pub fn apply_control_styles() {
    let button_hover = format!(
        "background-color: {}; transform: translateY(-1px); box-shadow: 0 4px 12px {};",
        rgba_to_css(COLOR_SCHEME.primary, 0.1),
        rgba_to_css(COLOR_SCHEME.primary, 0.2)
    );
    
    let css = format!(
        r#"
        .control-button {{ {} }}
        .control-button:hover {{ {} }}
        .control-select {{ {} }}
        .control-input {{ {} }}
        .control-label {{ {} }}
        .control-range {{ {} }}
        "#,
        get_button_style(),
        button_hover,
        get_select_style(),
        get_input_style(),
        get_label_style(),
        get_control_range_style()
    );
    add_style_to_document(&css);
}

pub fn get_permission_overlay_style() -> String {
    // Overlay only covers the canvas area, not the sidebar
    "position: fixed; top: 0; left: 300px; right: 0; bottom: 0; background: transparent; z-index: 9999; cursor: pointer;".to_string()
}

pub fn get_permission_panel_style() -> String {
    // Permission panel style with position and transform as in index.css
    "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); color: #fff; font-family: Arial, sans-serif; font-size: 18px; text-align: center; background: rgba(0, 0, 0, 0.8); padding: 30px; border-radius: 10px; min-width: 400px; box-shadow: 0 5px 25px rgba(0, 0, 0, 0.4); transition: background 0.3s ease, box-shadow 0.3s ease; cursor: pointer;".to_string()
}

pub fn apply_permission_styles() {
    let css = format!(
        r#"
        .permission-overlay {{ {} }}
        .permission-panel, #permission-panel {{ {} }}
        .permission-panel:hover, #permission-panel:hover {{
            background: rgba(30, 30, 80, 0.9);
            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.6), 0 0 0 2px rgba(100, 100, 255, 0.5);
        }}
        .permission-title {{ font-size: 24px; font-weight: 600; margin-bottom: 16px; color: {}; }}
        .permission-description {{ font-size: 16px; margin-bottom: 24px; color: {}; }}
        .permission-button {{ {}; margin: 0 8px; }}
        
        /* Disabled sidebar styles during permission request */
        body.permission-required #sidebar {{
            opacity: 0.5;
            pointer-events: none;
        }}
        body.permission-required #sidebar button,
        body.permission-required #sidebar select,
        body.permission-required #sidebar input {{
            opacity: 0.4;
            cursor: not-allowed;
        }}
        "#,
        get_permission_overlay_style(),
        get_permission_panel_style(),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.text, 0.8),
        get_button_style()
    );
    add_style_to_document(&css);
}

pub fn apply_permission_overlay_animations() {
    // Permission panel pulse animation from index.css
    let css = r#"
        @keyframes permission-panel-pulse {
            0% {
                box-shadow: 0 5px 25px rgba(0, 0, 0, 0.4);
            }
            50% {
                box-shadow: 0 5px 30px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.1);
            }
            100% {
                box-shadow: 0 5px 25px rgba(0, 0, 0, 0.4);
            }
        }
        .permission-panel.pulsing {
            animation: permission-panel-pulse 2s ease-in-out infinite;
        }
    "#;
    add_style_to_document(css);
}

// Error Message Box Styles
pub fn get_error_overlay_style() -> String {
    format!(
        "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background-color: {}; display: flex; justify-content: center; align-items: center; z-index: 10000;",
        rgba_to_css(COLOR_SCHEME.background, 0.95)
    )
}

pub fn get_error_panel_style() -> String {
    format!(
        "background-color: {}; color: {}; padding: 40px; border-radius: 12px; box-shadow: 0 20px 60px {}; max-width: 600px; text-align: center;",
        rgb_to_css(COLOR_SCHEME.surface),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.background, 0.5)
    )
}

pub fn get_error_title_style() -> String {
    format!(
        "font-size: 28px; font-weight: 700; margin-bottom: 20px; color: {};",
        rgb_to_css(COLOR_SCHEME.secondary)
    )
}

pub fn get_error_details_style() -> String {
    format!(
        "font-size: 16px; line-height: 1.5; color: {}; white-space: pre-wrap;",
        rgba_to_css(COLOR_SCHEME.text, 0.8)
    )
}

// First Click Handler Styles
pub fn get_first_click_overlay_style() -> String {
    format!(
        "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background-color: {}; display: flex; justify-content: center; align-items: center; z-index: 9999; cursor: pointer;",
        rgba_to_css(COLOR_SCHEME.background, 0.95)
    )
}

pub fn get_first_click_panel_style() -> String {
    format!(
        "background-color: {}; color: {}; padding: 60px; border-radius: 16px; box-shadow: 0 25px 75px {}; max-width: 600px; text-align: center; cursor: pointer; transition: transform 0.3s ease, box-shadow 0.3s ease;",
        rgb_to_css(COLOR_SCHEME.surface),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.background, 0.6)
    )
}

pub fn get_first_click_panel_hover_style() -> String {
    format!(
        "transform: scale(1.02); box-shadow: 0 30px 90px {};",
        rgba_to_css(COLOR_SCHEME.background, 0.7)
    )
}

pub fn apply_first_click_styles() {
    let css = format!(
        r#"
        .first-click-overlay {{ {} }}
        .first-click-panel {{ {} }}
        .first-click-panel:hover {{ {} }}
        .first-click-svg {{ margin-bottom: 30px; }}
        .first-click-title {{ font-size: 32px; font-weight: 700; margin-bottom: 20px; color: {}; }}
        .first-click-description {{ font-size: 18px; line-height: 1.6; color: {}; }}
        "#,
        get_first_click_overlay_style(),
        get_first_click_panel_style(),
        get_first_click_panel_hover_style(),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.text, 0.8)
    );
    add_style_to_document(&css);
}

pub fn get_svg_muted_color() -> String {
    rgb_to_css(COLOR_SCHEME.muted)
}

pub fn get_svg_secondary_color() -> String {
    rgb_to_css(COLOR_SCHEME.secondary)
}

pub fn get_svg_surface_color() -> String {
    rgb_to_css(COLOR_SCHEME.surface)
}

pub fn get_svg_accent_color() -> String {
    rgb_to_css(COLOR_SCHEME.accent)
}

// Main Scene UI Styles
pub fn get_root_note_display_style() -> String {
    format!(
        "background-color: {}; color: {}; padding: 6px 12px; border-radius: 6px; font-size: 16px; font-weight: 600; min-width: 60px; text-align: center; font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;",
        rgba_to_css(COLOR_SCHEME.primary, 0.1),
        rgb_to_css(COLOR_SCHEME.primary)
    )
}

pub fn get_small_button_style() -> String {
    format!(
        "background-color: {}; color: {}; border: 1px solid {}; padding: 4px 12px; border-radius: 4px; cursor: pointer; font-size: 16px; font-weight: 600; transition: all 0.2s;",
        rgb_to_css(COLOR_SCHEME.surface),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.primary, 0.3)
    )
}

pub fn get_container_style() -> String {
    format!(
        "display: flex; flex-direction: column; gap: 20px; width: 100%;"
    )
}

pub fn get_control_container_style() -> String {
    format!(
        "display: flex; flex-direction: column; gap: 12px; width: 100%;"
    )
}

pub fn get_monospace_display_style() -> String {
    format!(
        "font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace; font-size: 14px; color: {};",
        rgb_to_css(COLOR_SCHEME.text)
    )
}

pub fn get_checkbox_style() -> String {
    format!(
        "width: 18px; height: 18px; cursor: pointer; accent-color: {};",
        rgb_to_css(COLOR_SCHEME.primary)
    )
}

pub fn apply_color_scheme_styles() {
    // Apply CSS reset first
    apply_css_reset();
    apply_body_styles();
    apply_sidebar_styles();
    apply_canvas_container_styles();
    apply_canvas_styles();
    apply_status_classes();
    apply_control_styles();
    apply_control_range_styles();
    apply_permission_styles();
    apply_permission_overlay_animations();
    apply_first_click_styles();
}