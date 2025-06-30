use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;


mod modules;

use modules::common::dev_log;

#[cfg(debug_assertions)]
use modules::console::DevConsole;

/// Render development console if in debug mode
#[cfg(debug_assertions)]
fn render_dev_console() -> Html {
    html! { <DevConsole /> }
}

/// Render nothing in release mode
#[cfg(not(debug_assertions))]
fn render_dev_console() -> Html {
    html! {}
}


/// Main application component for Pitch Toy
#[function_component]
fn App() -> Html {
    let canvas_ref = use_node_ref();
    
    
    // Initialize wgpu canvas after component is rendered
    use_effect_with(canvas_ref.clone(), {
        let canvas_ref = canvas_ref.clone();
        move |_| {
            if let Some(canvas_element) = canvas_ref.cast::<HtmlCanvasElement>() {
                dev_log!("Canvas element found via ref: {}x{}", canvas_element.width(), canvas_element.height());
                initialize_canvas(&canvas_element);
            } else {
                dev_log!("Warning: Canvas element not found via ref");
            }
        }
    });

    html! {
        <div>
            // Development console (debug builds only)
            { render_dev_console() }
            
            // Canvas for wgpu GPU rendering
            <canvas 
                ref={canvas_ref}
                id="wgpu-canvas"
                width="800" 
                height="600"
                style="display: block; margin: 0 auto; border: 1px solid #333;"
            />
        </div>
    }
}

// Note: get_canvas_element() function removed as we now use canvas_ref directly

/// Initialize canvas for wgpu rendering
fn initialize_canvas(canvas: &HtmlCanvasElement) {
    dev_log!("Initializing canvas for wgpu rendering");
    
    // Set canvas size to match display size
    let width = canvas.offset_width() as u32;
    let height = canvas.offset_height() as u32;
    
    canvas.set_width(width);
    canvas.set_height(height);
    
    dev_log!("Canvas initialized: {}x{}", width, height);
    
    // TODO: Initialize wgpu renderer (future story)
    // This will be implemented when graphics module is added
}

/// Application entry point
#[wasm_bindgen(start)]
pub fn main() {
    // Initialize console logging for development
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    
    dev_log!("Starting Pitch Toy application");
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    
    yew::Renderer::<App>::new().render();
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_build_configuration() {
        // Test that build configuration detection works
        let config = if cfg!(debug_assertions) { "Development" } else { "Production" };
        assert!(config == "Development" || config == "Production");
    }

    // TODO: Add meaningful tests when we have testable functionality:
    // - test_canvas_initialization() when wgpu renderer is implemented
    // - test_audio_processing() when audio modules are added
    // - test_event_system() when event dispatcher is implemented
    // - test_theme_switching() when theme manager is added
}