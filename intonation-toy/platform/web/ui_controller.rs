//! UI controller implementation for web browsers.
//! 
//! Provides canvas management, theme application, and other UI-related
//! functionality using DOM APIs.

use crate::platform::traits::UiController;
use super::utils::rgb_to_css;
use wasm_bindgen::JsCast;

/// Web-based UI controller implementation.
/// 
/// Manages canvas sizing, theme styling, and other UI operations
/// through browser DOM APIs.
pub struct WebUiController;

impl UiController for WebUiController {
    fn resize_canvas() {
        resize_canvas_impl();
    }

    fn apply_theme_styles() {
        reapply_current_theme();
    }
}

impl WebUiController {
    /// Creates a new WebUiController instance.
    pub fn new() -> Self {
        WebUiController
    }
}

impl Default for WebUiController {
    fn default() -> Self {
        Self::new()
    }
}

/// Resize the 3D canvas to fit the available window space.
/// 
/// This function calculates the optimal canvas size based on:
/// - Window dimensions
/// - Sidebar width
/// - Canvas margins
/// - Zoom control space
/// 
/// The canvas maintains a square aspect ratio and respects minimum/maximum size constraints.
fn resize_canvas_impl() {
    use crate::common::dev_log;
    
    let window_obj = match web_sys::window() {
        Some(win) => win,
        None => {
            dev_log!("RESIZE: No window object available");
            return;
        }
    };
    
    let document = match window_obj.document() {
        Some(doc) => doc,
        None => {
            dev_log!("RESIZE: No document object available");
            return;
        }
    };
    
    // Get the canvas element
    let canvas = match document.get_element_by_id("three-d-canvas") {
        Some(elem) => match elem.dyn_into::<web_sys::HtmlCanvasElement>() {
            Ok(canvas) => canvas,
            Err(_) => {
                dev_log!("RESIZE: Element with id 'three-d-canvas' is not a canvas");
                return;
            }
        },
        None => {
            dev_log!("RESIZE: Canvas element 'three-d-canvas' not found");
            return;
        }
    };
    
    dev_log!("RESIZE: resize_canvas called");
    
    // Import constants from web styling module
    // TODO: These should be moved to platform configuration in future phases
    const SIDEBAR_WIDTH: i32 = 300;
    const CANVAS_MARGIN: i32 = 100;
    
    // Estimate zoom control width (padding + slider + margins)
    let zoom_control_width = 80; // Approximate width of zoom control
    let gap = 16; // Gap between canvas and zoom control
    
    // Calculate available space (subtract sidebar width, margins, zoom control, and gap)
    let available_width = window_obj.inner_width().unwrap().as_f64().unwrap() as i32 - SIDEBAR_WIDTH - (CANVAS_MARGIN * 2) - zoom_control_width - gap;
    let available_height = window_obj.inner_height().unwrap().as_f64().unwrap() as i32 - (CANVAS_MARGIN * 2);
    
    dev_log!("RESIZE: available {}x{}", available_width, available_height);
    
    // Take the smaller dimension to maintain square aspect ratio
    let mut canvas_size = std::cmp::min(available_width, available_height);
    canvas_size = std::cmp::min(canvas_size, crate::app_config::CANVAS_MAX_SIZE);
    canvas_size = std::cmp::max(canvas_size, crate::app_config::CANVAS_MIN_SIZE);
    
    // Scene wrapper width includes canvas + gap + zoom control
    let wrapper_width = canvas_size + gap + zoom_control_width;
    let wrapper_height = canvas_size;
    
    dev_log!("RESIZE: setting canvas size to {}px, wrapper size to {}x{}", canvas_size, wrapper_width, wrapper_height);
    
    // Get the scene wrapper element
    let scene_wrapper = match document.get_element_by_id("scene-wrapper") {
        Some(elem) => elem,
        None => {
            dev_log!("RESIZE: Scene wrapper element not found");
            return;
        }
    };
    
    // Set CSS positioning and sizing for scene wrapper
    if let Err(e) = scene_wrapper.set_attribute("style", &format!(
        "position: absolute; top: {}px; left: {}px; width: {}px; height: {}px; display: flex; flex-direction: row; align-items: center; gap: 16px;",
        CANVAS_MARGIN, CANVAS_MARGIN, wrapper_width, wrapper_height
    )) {
        dev_log!("RESIZE: Failed to set scene wrapper style: {:?}", e);
        return;
    }
    
    // Set canvas to specific size
    let style = canvas.style();
    if let Err(e) = style.set_property("width", &format!("{}px", canvas_size)) {
        dev_log!("RESIZE: Failed to set canvas width: {:?}", e);
    }
    if let Err(e) = style.set_property("height", &format!("{}px", canvas_size)) {
        dev_log!("RESIZE: Failed to set canvas height: {:?}", e);
    }
}

/// Reapply the current theme by updating CSS custom properties.
/// 
/// All styling is handled by static/style.css with CSS custom properties,
/// so only updating the CSS variables is needed for efficient theme switching.
/// This approach ensures theme changes are applied instantly across all UI elements.
fn reapply_current_theme() {
    update_css_variables();
}

/// Update CSS custom properties based on the current color scheme.
/// 
/// Sets CSS variables on the document root element that control theming
/// throughout the application. Variables include:
/// - --color-background
/// - --color-surface
/// - --color-primary
/// - --color-secondary
/// - --color-accent
/// - --color-text
/// - --color-muted
/// - --color-border
/// - --color-error
fn update_css_variables() {
    use crate::theme::get_current_color_scheme;
    use crate::common::dev_log;
    
    let color_scheme = get_current_color_scheme();
    
    // Apply to document.documentElement (html element) instead of :root selector
    let document = match web_sys::window() {
        Some(win) => match win.document() {
            Some(doc) => doc,
            None => {
                dev_log!("No document available for theme update");
                return;
            }
        },
        None => {
            dev_log!("No window available for theme update");
            return;
        }
    };
    
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
        
        if root.set_attribute("style", &style_str).is_err() {
            dev_log!("Failed to set CSS variables on root element");
        } else {
            dev_log!("Successfully updated CSS custom properties");
        }
    }
}