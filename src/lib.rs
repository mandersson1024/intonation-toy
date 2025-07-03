use yew::prelude::*;
use web_sys::HtmlCanvasElement;

pub mod audio;
pub mod console;
pub mod console_commands;
pub mod common;
pub mod platform;

use common::dev_log;

#[cfg(not(test))]
use wasm_bindgen::prelude::*;

#[cfg(not(test))]
use platform::{Platform, PlatformValidationResult};

#[cfg(debug_assertions)]
use console::DevConsole;


#[cfg(debug_assertions)]
use std::rc::Rc;

/// Render development console if in debug mode
fn render_dev_console() -> Html {
    #[cfg(debug_assertions)]
    {
        let registry = Rc::new(crate::console_commands::create_console_registry());
        html! { <DevConsole registry={registry} /> }
    }
    
    #[cfg(not(debug_assertions))]
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
#[cfg(not(test))]
#[wasm_bindgen(start)]
pub fn main() {
    // Initialize console logging for development
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    
    dev_log!("Starting Pitch Toy application");
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    dev_log!("{}", Platform::get_platform_info());
    
    // Validate critical platform APIs before proceeding
    match Platform::check_feature_support() {
        PlatformValidationResult::AllSupported => {
            dev_log!("✓ Platform validation passed - initializing application");
            
            // Initialize audio system asynchronously
            wasm_bindgen_futures::spawn_local(async {
                match audio::initialize_audio_system().await {
                    Ok(_) => {
                        dev_log!("✓ Audio system initialized successfully");
                        yew::Renderer::<App>::new().render();
                    }
                    Err(_e) => {
                        dev_log!("✗ Audio system initialization failed: {}", _e);
                        dev_log!("Application cannot continue without audio system");
                        // TODO: Add error screen rendering in future story when UI requirements are defined
                    }
                }
            });
        }
        PlatformValidationResult::MissingCriticalApis(_missing_apis) => {
            let _api_list: Vec<String> = _missing_apis.iter().map(|api| api.to_string()).collect();
            dev_log!("✗ CRITICAL: Missing required browser APIs: {}", _api_list.join(", "));
            dev_log!("✗ Application cannot start. Please upgrade your browser or use a supported browser:");
            // TODO: Add error screen rendering in future story when UI requirements are defined
        }
    }
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