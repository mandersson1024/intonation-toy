use crate::app_config::OVERLAY_BACKGROUND_ALPHA;
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
    let css = r#"
        .app-body {
            background-color: var(--color-background);
            color: var(--color-text);
            font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 0;
            display: flex;
            flex-direction: row;
            min-height: 100vh;
            overflow: hidden;
        }
    "#;
    add_style_to_document(css);
}

pub fn apply_sidebar_styles() {
    let css = format!(
        r#"
        .app-sidebar {{
            position: fixed;
            top: 0;
            left: 0;
            bottom: 0;
            width: {}px;
            background: linear-gradient(to right, 
                color-mix(in srgb, var(--color-surface) 95%, transparent),
                color-mix(in srgb, var(--color-surface) 85%, transparent));
            border-right: 1px solid color-mix(in srgb, var(--color-muted) 30%, transparent);
            display: flex;
            flex-direction: column;
            padding: 20px;
            z-index: 1000;
            backdrop-filter: blur(10px);
            overflow-y: auto;
        }}
        "#,
        SIDEBAR_WIDTH
    );
    add_style_to_document(&css);
}

pub fn apply_canvas_container_styles() {
    let css = format!(
        r#"
        .app-canvas-container {{
            position: fixed;
            top: 0;
            left: {}px;
            right: 0;
            bottom: 0;
            display: flex;
            justify-content: center;
            align-items: center;
        }}
        "#,
        SIDEBAR_WIDTH
    );
    add_style_to_document(&css);
}

pub fn apply_canvas_styles() {
    // Using red border and black background as specified in index.css
    let style = "border: 3px solid red; border-radius: 4px; background-color: #000; display: block; width: 100%; height: 100%;";
    apply_style_to_element("#three-d-canvas", &style);
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
            font-weight: 400;
        }
    "#;
    add_style_to_document(css);
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
    let css = r#"
        .control-button {
            background-color: var(--color-surface);
            color: var(--color-text);
            border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
            padding: 8px 16px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: all 0.2s;
        }
        .control-button:hover {
            background-color: color-mix(in srgb, var(--color-primary) 10%, transparent);
            transform: translateY(-1px);
            box-shadow: 0 4px 12px color-mix(in srgb, var(--color-primary) 20%, transparent);
        }
        .control-select {
            background-color: var(--color-surface);
            color: var(--color-text);
            border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
            padding: 6px 12px;
            border-radius: 6px;
            font-size: 14px;
            cursor: pointer;
            min-width: 120px;
        }
        .control-input {
            background-color: var(--color-surface);
            color: var(--color-text);
            border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
            padding: 6px 12px;
            border-radius: 6px;
            font-size: 14px;
        }
        .control-label {
            color: var(--color-text);
            font-size: 14px;
            font-weight: 500;
            margin-right: 8px;
        }
        .control-range {
            display: flex;
            align-items: center;
            gap: 12px;
        }
        "#;
    add_style_to_document(css);
}



pub fn apply_permission_styles() {
    let css = format!(
        r#"
        .permission-overlay {{
            position: fixed;
            top: 0;
            left: {}px;
            right: 0;
            bottom: 0;
            background: transparent;
            z-index: 9999;
            cursor: pointer;
        }}
        .permission-panel, #permission-panel {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: #fff;
            font-family: Arial, sans-serif;
            font-size: 18px;
            text-align: center;
            background: rgba(0, 0, 0, 0.8);
            padding: 30px;
            border-radius: 10px;
            min-width: 400px;
            box-shadow: 0 5px 25px rgba(0, 0, 0, 0.4);
            transition: background 0.3s ease, box-shadow 0.3s ease;
            cursor: pointer;
        }}
        .permission-panel:hover, #permission-panel:hover {{
            background: rgba(30, 30, 80, 0.9);
            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.6), 0 0 0 2px rgba(100, 100, 255, 0.5);
        }}
        .permission-title {{
            font-size: 24px;
            font-weight: 600;
            margin-bottom: 16px;
            color: var(--color-text);
        }}
        .permission-description {{
            font-size: 16px;
            margin-bottom: 24px;
            color: color-mix(in srgb, var(--color-text) 80%, transparent);
        }}
        .permission-button {{
            background-color: var(--color-surface);
            color: var(--color-text);
            border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
            padding: 8px 16px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: all 0.2s;
            margin: 0 8px;
        }}
        
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
        SIDEBAR_WIDTH
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
pub fn apply_error_message_styles() {
    let css = format!(
        r#"
        .error-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: color-mix(in srgb, var(--color-background) {}%, transparent);
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 10000;
        }}
        .error-panel {{
            background-color: var(--color-surface);
            color: var(--color-text);
            padding: 40px;
            border-radius: 12px;
            box-shadow: 0 20px 60px color-mix(in srgb, var(--color-background) 50%, transparent);
            max-width: 600px;
            text-align: center;
        }}
        .error-title {{
            font-size: 28px;
            font-weight: 700;
            margin-bottom: 20px;
            color: var(--color-secondary);
        }}
        .error-details {{
            font-size: 16px;
            line-height: 1.5;
            color: color-mix(in srgb, var(--color-text) 80%, transparent);
            white-space: pre-wrap;
        }}
        "#,
        (OVERLAY_BACKGROUND_ALPHA * 100.0) as u8
    );
    add_style_to_document(&css);
}

// First Click Handler Styles

pub fn apply_first_click_styles() {
    let css = format!(
        r#"
        .first-click-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: color-mix(in srgb, var(--color-background) {}%, transparent);
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 9999;
            cursor: pointer;
        }}
        .first-click-panel {{
            background-color: var(--color-surface);
            color: var(--color-text);
            padding: 60px;
            border-radius: 16px;
            box-shadow: 0 25px 75px color-mix(in srgb, var(--color-background) 60%, transparent);
            max-width: 600px;
            text-align: center;
            cursor: pointer;
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }}
        .first-click-panel:hover {{
            transform: scale(1.02);
            box-shadow: 0 30px 90px color-mix(in srgb, var(--color-background) 70%, transparent);
        }}
        .first-click-svg {{ margin-bottom: 30px; }}
        .first-click-title {{
            font-size: 32px;
            font-weight: 700;
            margin-bottom: 20px;
            color: var(--color-text);
        }}
        .first-click-description {{
            font-size: 18px;
            line-height: 1.6;
            color: color-mix(in srgb, var(--color-text) 80%, transparent);
        }}
        "#,
        (OVERLAY_BACKGROUND_ALPHA * 100.0) as u8
    );
    add_style_to_document(&css);
}

// Main Scene UI Styles
pub fn apply_component_styles() {
    let css = r#"
        .root-note-display {
            background-color: color-mix(in srgb, var(--color-primary) 10%, transparent);
            color: var(--color-primary);
            padding: 6px 12px;
            border-radius: 6px;
            font-size: 16px;
            font-weight: 600;
            min-width: 60px;
            text-align: center;
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
        }
        .small-button {
            background-color: var(--color-surface);
            color: var(--color-text);
            border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
            padding: 4px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
            font-weight: 600;
            transition: all 0.2s;
        }
        .small-button:hover {
            background-color: color-mix(in srgb, var(--color-primary) 10%, transparent);
            transform: translateY(-1px);
        }
        .ui-container {
            display: flex;
            flex-direction: column;
            gap: 20px;
            width: 100%;
        }
        .control-container {
            display: flex;
            flex-direction: column;
            gap: 12px;
            width: 100%;
        }
        .monospace-display {
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
            font-size: 14px;
            color: var(--color-text);
        }
        .volume-display {
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
            font-size: 12px;
            color: color-mix(in srgb, var(--color-text) 80%, transparent);
            min-width: 40px;
            text-align: right;
        }
        .tuning-fork-container {
            display: flex;
            flex-direction: column;
            gap: 8px;
            width: 100%;
        }
        .volume-slider-container {
            display: flex;
            align-items: center;
            gap: 8px;
            width: 100%;
        }
        .sidebar-header {
            color: var(--color-text);
            font-size: 24px;
            font-weight: 700;
            margin: 0 0 20px 0;
            padding: 16px 0;
            text-align: center;
            border-bottom: 2px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
            font-family: system-ui, -apple-system, sans-serif;
        }
        .subsection-header {
            color: var(--color-text);
            font-size: 13px;
            font-weight: 600;
            margin-bottom: 8px;
            margin-top: 24px;
            display: block;
        }
        "#;
    add_style_to_document(css);
}










pub fn apply_color_scheme_styles() {
    // Apply CSS reset first
    apply_css_reset();
    // Apply CSS variables before other styles that depend on them
    apply_css_variables();
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
    apply_component_styles();
    apply_error_message_styles();
    apply_about_section_styles();
    apply_range_input_styles();
}



// About Section Styles

pub fn apply_about_section_styles() {
    let css = r#"
        .about-section {
            margin-top: 32px;
            padding-top: 20px;
            border-top: 1px solid color-mix(in srgb, var(--color-muted) 30%, transparent);
            width: 100%;
            flex: 1;
            display: flex;
            flex-direction: column;
            min-height: 0;
        }
        .about-header {
            color: var(--color-text);
            font-size: 15px;
            font-weight: 600;
            margin-bottom: 12px;
        }
        .about-content {
            padding: 16px;
            background-color: color-mix(in srgb, var(--color-surface) 50%, transparent);
            border-radius: 8px;
            overflow-y: auto;
            flex: 1;
        }
        .about-text {
            color: color-mix(in srgb, var(--color-text) 90%, transparent);
            font-size: 13px;
            line-height: 1.6;
            margin-bottom: 12px;
        }
        .about-list {
            color: color-mix(in srgb, var(--color-text) 90%, transparent);
            font-size: 13px;
            line-height: 1.8;
            margin: 8px 0 8px 20px;
            padding-left: 0;
        }
        .about-list li {
            margin-bottom: 4px;
        }
        .about-section h3 {
            color: var(--color-primary);
            font-size: 14px;
            font-weight: 600;
            margin: 16px 0 8px 0;
        }
        .about-section strong {
            color: var(--color-text);
            font-weight: 600;
        }
        "#;
    add_style_to_document(css);
}

pub fn apply_range_input_styles() {
    let css = r#"
        input[type="range"] {
            width: 100%;
            height: 6px;
            cursor: pointer;
            appearance: none;
            background: color-mix(in srgb, var(--color-muted) 30%, transparent);
            border-radius: 3px;
            outline: none;
            transition: all 0.2s;
        }
        input[type="range"]::-webkit-slider-thumb {
            appearance: none;
            width: 18px;
            height: 18px;
            border-radius: 50%;
            background: var(--color-primary);
            cursor: pointer;
            border: 2px solid #fff;
            box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
        }
        input[type="range"]::-moz-range-thumb {
            width: 18px;
            height: 18px;
            border-radius: 50%;
            background: var(--color-primary);
            cursor: pointer;
            border: 2px solid #fff;
            box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
        }
        input[type="range"]:focus::-webkit-slider-thumb {
            box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 30%, transparent);
        }
        input[type="range"]:focus::-moz-range-thumb {
            box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 30%, transparent);
        }
        "#;
    add_style_to_document(css);
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