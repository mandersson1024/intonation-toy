use yew::prelude::*;
use web_sys::HtmlCanvasElement;
use three_d::*;

pub mod audio;
pub mod console_commands;
pub mod common;
pub mod platform;
pub mod events;
pub mod debug;
pub mod app_data;
pub mod graphics;

use common::dev_log;
use wasm_bindgen::prelude::*;

use platform::{Platform, PlatformValidationResult};
use debug::egui::EguiMicrophoneButton;
use graphics::SpriteScene;

#[cfg(debug_assertions)]
use debug::DebugInterface;

#[cfg(debug_assertions)]
use std::rc::Rc;

/// Render development console if in debug mode
fn render_dev_console() -> Html {
    #[cfg(debug_assertions)]
    {
        // Get global shared event dispatcher
        let event_dispatcher = crate::events::get_global_event_dispatcher();
        
        // Create audio service with event dispatcher
        let audio_service = Rc::new(crate::audio::create_console_audio_service_with_events(event_dispatcher.clone()));
        let registry = Rc::new(crate::console_commands::create_console_registry_with_audio());
        html! { 
            <DebugInterface
                registry={registry}
                audio_service={audio_service}
                event_dispatcher={Some(event_dispatcher)}
            />
        }
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
                wasm_bindgen_futures::spawn_local(async {
                    run_three_d().await;
                });
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
                width="1280" 
                height="720"
                style="display: block; margin: 0 auto; border: 1px solid #333;"
            />
        </div>
    }
}



/// Initialize canvas for three-d graphics rendering
fn initialize_canvas(canvas: &HtmlCanvasElement) {
    dev_log!("Initializing canvas for three-d hello-world proof-of-concept");
    
    // Set canvas size to match display size
    let width = canvas.offset_width() as u32;
    let height = canvas.offset_height() as u32;
    
    canvas.set_width(width);
    canvas.set_height(height);
    
    dev_log!("Canvas initialized: {}x{}", width, height);
}


pub async fn run_three_d() {
    dev_log!("Starting three-d with red sprites");
    
    // Create application data with observable permission state
    use crate::app_data::LiveData;
    use observable_data::DataSource;
    
    let permission_source = DataSource::new(audio::AudioPermission::Uninitialized);
    let _live_data = LiveData {
        microphone_permission: permission_source.observer(),
    };

    let window = Window::new(WindowSettings {
        title: "Sprites!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();
    let mut sprite_scene = SpriteScene::new(&context, window.viewport());
    let mut gui = three_d::GUI::new(&context);
    
    // Create egui dev console with the same registry as the Yew console
    let registry = crate::console_commands::create_console_registry_with_audio();

    let mut dev_console = egui_dev_console::EguiDevConsole::new_with_registry(registry);
    let mut microphone_button = EguiMicrophoneButton::new(&permission_source);

    dev_log!("Starting three-d + egui render loop");
    
    window.render_loop(move |mut frame_input| {
        sprite_scene.update_viewport(frame_input.viewport);

        // Render 3D scene first
        sprite_scene.render(&mut frame_input.screen());

        // Render egui overlay  
        gui.update(&mut frame_input.events, frame_input.accumulated_time, frame_input.viewport, frame_input.device_pixel_ratio, |gui_context| {
            dev_console.render(gui_context);
            microphone_button.render(gui_context);
        });

        let _ = gui.render();

        FrameOutput::default()
    });
}

/// Application entry point
#[wasm_bindgen(start)]
pub async fn start() {
    // Initialize console logging for development
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    
    dev_log!("Starting Pitch Toy application");
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    dev_log!("{}", Platform::get_platform_info());
    
    // Validate critical platform APIs before proceeding
    if let PlatformValidationResult::MissingCriticalApis(missing_apis) = Platform::check_feature_support() {
        let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
        dev_log!("✗ CRITICAL: Missing required browser APIs: {}", api_list.join(", "));
        dev_log!("✗ Application cannot start. Please upgrade your browser or use a supported browser:");
        // TODO: Add error screen rendering in future story when UI requirements are defined
        return;
    }

    dev_log!("✓ Platform validation passed - initializing application");
    
    // Initialize audio systems first
    if let Err(e) = initialize_audio_systems().await {
        dev_log!("✗ Audio system initialization failed: {}", e);
        dev_log!("Application cannot continue without audio system");
        // TODO: Add error screen rendering in future story when UI requirements are defined
        return;
    }
    
    // Start Yew application
    yew::Renderer::<App>::new().render();
}

/// Initialize all audio systems in sequence with proper error handling
async fn initialize_audio_systems() -> Result<(), String> {
    // Initialize audio system
    audio::initialize_audio_system().await
        .map_err(|e| format!("Audio system initialization failed: {}", e))?;
    dev_log!("✓ Audio system initialized successfully");
    
    // Initialize buffer pool
    audio::initialize_buffer_pool().await
        .map_err(|e| format!("Buffer pool initialization failed: {}", e))?;
    dev_log!("✓ Buffer pool initialized successfully");
    
    // Initialize AudioWorklet manager (required)
    audio::worklet::initialize_audioworklet_manager().await
        .map_err(|e| format!("AudioWorklet manager initialization failed: {}", e))?;
    dev_log!("✓ AudioWorklet manager initialized successfully");
    
    // Initialize pitch analyzer (required)
    audio::initialize_pitch_analyzer().await
        .map_err(|e| format!("Pitch analyzer initialization failed: {}", e))?;
    dev_log!("✓ Pitch analyzer initialized successfully");
    
    Ok(())
}
