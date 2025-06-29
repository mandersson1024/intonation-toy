use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, KeyboardEvent};
use wasm_bindgen::{closure::Closure, JsCast};

mod modules;

use modules::common::dev_log;

#[cfg(debug_assertions)]
use modules::console::DevConsole;

/// Render development console if in debug mode
#[cfg(debug_assertions)]
fn render_dev_console(visible: bool, on_toggle: Callback<()>) -> Html {
    html! { <DevConsole visible={visible} on_toggle={on_toggle} /> }
}

#[cfg(not(debug_assertions))]
fn render_dev_console(_visible: bool, _on_toggle: Callback<()>) -> Html {
    html! {}
}


/// Console visibility state
#[derive(Clone, PartialEq)]
struct ConsoleState {
    visible: bool,
}

/// Console visibility action for reducer
#[derive(Clone, PartialEq)]
enum ConsoleAction {
    Toggle,
}

impl Reducible for ConsoleState {
    type Action = ConsoleAction;
    
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            ConsoleAction::Toggle => {
                let new_state = !self.visible;
                web_sys::console::log_3(&"Reducer: Toggling console from".into(), &self.visible.into(), &format!("to {}", new_state).into());
                std::rc::Rc::new(ConsoleState { visible: new_state })
            }
        }
    }
}

/// Main application component for Pitch Toy
#[function_component]
fn App() -> Html {
    let console_state = use_reducer(|| ConsoleState { visible: true });
    let canvas_ref = use_node_ref();
    
    let toggle_console = {
        let console_state = console_state.clone();
        Callback::from(move |_| {
            console_state.dispatch(ConsoleAction::Toggle);
        })
    };
    
    // Global keyboard event handler for Escape key to toggle console
    {
        let console_state = console_state.clone();
        use_effect_with((), move |_| {
            let console_state_ref = console_state.clone();
            
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
                    web_sys::console::log_2(&"Key pressed:".into(), &keyboard_event.key().into());
                    if keyboard_event.key() == "Escape" {
                        web_sys::console::log_1(&"Escape key detected - toggling console".into());
                        keyboard_event.prevent_default();
                        
                        // Use dispatcher to avoid closure capture issues
                        console_state_ref.dispatch(ConsoleAction::Toggle);
                    }
                }
            }) as Box<dyn FnMut(_)>);
            
            let window = web_sys::window()
                .expect("Failed to get window");
            web_sys::console::log_1(&"Setting up global keydown event listener on window".into());
            window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .expect("Failed to add keydown event listener to window");
            
            // Store closure to prevent it from being dropped
            closure.forget();
            
            // Note: In a real application, we should properly manage the closure cleanup
            // For this development console, the memory leak is acceptable
            || {}
        });
    }
    
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
            { render_dev_console(console_state.visible, toggle_console) }
            
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