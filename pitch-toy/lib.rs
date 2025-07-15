use yew::prelude::*;
use web_sys::HtmlCanvasElement;
use three_d::*;

pub mod audio;
pub mod common;
pub mod platform;
pub mod events;
pub mod debug;
pub mod graphics;
pub mod live_data;

use common::dev_log;
use wasm_bindgen::prelude::*;
use egui_dev_console::ConsoleCommandRegistry;

use platform::{Platform, PlatformValidationResult};
use debug::egui::{EguiMicrophoneButton, EguiLiveDataPanel};

use graphics::SpriteScene;

// Import LiveData type
use live_data::LiveData;

/// Main application component for Pitch Toy
#[function_component]
fn App() -> Html {
    let canvas_ref = use_node_ref();
    
    // Create data sources and LiveData in one memo (run once)  
    let memo_result = use_memo((), |_| {
        use observable_data::DataSource;
        
        let microphone_permission_source = DataSource::new(audio::AudioPermission::Uninitialized);
        let audio_devices_source = DataSource::new(audio::AudioDevices {
            input_devices: vec![],
            output_devices: vec![],
        });
        let performance_metrics_source = DataSource::new(debug::egui::live_data_panel::PerformanceMetrics::default());
        let volume_level_source = DataSource::new(None::<debug::egui::live_data_panel::VolumeLevelData>);
        let pitch_data_source = DataSource::new(None::<debug::egui::live_data_panel::PitchData>);
        let audioworklet_status_source = DataSource::new(debug::egui::live_data_panel::AudioWorkletStatus::default());
        
        let live_data = live_data::LiveData {
            microphone_permission: microphone_permission_source.observer(),
            audio_devices: audio_devices_source.observer(),
            performance_metrics: performance_metrics_source.observer(),
            volume_level: volume_level_source.observer(),
            pitch_data: pitch_data_source.observer(),
            audioworklet_status: audioworklet_status_source.observer(),
        };
        
        (
            live_data, 
            microphone_permission_source.setter(), 
            audio_devices_source.setter(), 
            audioworklet_status_source.setter(),
            performance_metrics_source.setter(),
            pitch_data_source.setter(),
            volume_level_source.setter()
        )
    });
    
    let live_data = &memo_result.0;
    let microphone_permission_setter = &memo_result.1;
    let audio_devices_setter = &memo_result.2;
    let audioworklet_status_setter = &memo_result.3;
    let performance_metrics_setter = &memo_result.4;
    let pitch_data_setter = &memo_result.5;
    let volume_level_setter = &memo_result.6;
    
    
    // Initialize wgpu canvas after component is rendered
    use_effect_with(canvas_ref.clone(), {
        let canvas_ref = canvas_ref.clone();
        let live_data_clone = live_data.clone();
        let mic_perm_setter = microphone_permission_setter.clone();
        let perf_metrics_setter = performance_metrics_setter.clone();
        let pitch_setter = pitch_data_setter.clone();
        
        move |_| {
            if let Some(canvas_element) = canvas_ref.cast::<HtmlCanvasElement>() {
                dev_log!("Canvas element found via ref: {}x{}", canvas_element.width(), canvas_element.height());
                initialize_canvas(&canvas_element);
                wasm_bindgen_futures::spawn_local(async move {
                    run_three_d(live_data_clone, mic_perm_setter, perf_metrics_setter, pitch_setter).await;
                });
            } else {
                dev_log!("Warning: Canvas element not found via ref");
            }
        }
    });

    html! {
        <div>
            // Debug interface (LivePanel) for debug builds only
            {
                if cfg!(debug_assertions) {
                    // Get global shared event dispatcher
                    let event_dispatcher = crate::events::get_global_event_dispatcher();
                    
                    // Create audio service with event dispatcher and setters
                    let audio_service = std::rc::Rc::new(crate::audio::create_console_audio_service_with_audioworklet_setter(
                        event_dispatcher.clone(),
                        audio_devices_setter.clone(),
                        audioworklet_status_setter.clone(),
                        volume_level_setter.clone()
                    ));
                    
                    html! { 
                        <debug::DebugInterface
                            audio_service={audio_service}
                            event_dispatcher={Some(event_dispatcher)}
                            live_data={live_data.clone()}
                        />
                    }
                } else {
                    html! {}
                }
            }
            
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


pub async fn run_three_d(
    live_data: LiveData, 
    microphone_permission_setter: impl observable_data::DataSetter<audio::AudioPermission> + Clone + 'static,
    performance_metrics_setter: impl observable_data::DataSetter<debug::egui::live_data_panel::PerformanceMetrics> + Clone + 'static,
    pitch_data_setter: impl observable_data::DataSetter<Option<debug::egui::live_data_panel::PitchData>> + Clone + 'static
) {
    dev_log!("Starting three-d with red sprites");
    
    
    let window = Window::new(WindowSettings {
        title: "pitch-toy".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();
    let mut scene = SpriteScene::new(&context, window.viewport());
    let mut gui = three_d::GUI::new(&context);
    
    let mut command_registry = ConsoleCommandRegistry::new();
    crate::platform::commands::register_platform_commands(&mut command_registry);
    crate::audio::register_audio_commands(&mut command_registry);

    let mut dev_console = egui_dev_console::EguiDevConsole::new_with_registry(command_registry);
    let mut microphone_button = EguiMicrophoneButton::new(
        live_data.microphone_permission.clone(),
        microphone_permission_setter,
    );
    
    // Create audio service for LiveDataPanel
    let audio_service = std::rc::Rc::new(audio::create_console_audio_service());
    
    // Set the pitch data setter on the global pitch analyzer (it should be initialized by now)
    audio::set_pitch_data_setter(std::rc::Rc::new(pitch_data_setter.clone()));
    
    // Create LiveDataPanel
    let mut live_data_panel = EguiLiveDataPanel::new(audio_service.clone(), live_data);

    dev_log!("Starting three-d + egui render loop");
    
    // Performance tracking
    let mut frame_count = 0u32;
    let mut last_fps_update = 0.0;
    let mut fps = 0.0;
    
    window.render_loop(move |mut frame_input| {
        // Update FPS counter
        frame_count += 1;
        let current_time = frame_input.accumulated_time as f64;
        
        // Update FPS every second
        if current_time - last_fps_update >= 1000.0 {
            fps = (frame_count as f64) / ((current_time - last_fps_update) / 1000.0);
            frame_count = 0;
            last_fps_update = current_time;
            
            // Update performance metrics
            let metrics = debug::egui::live_data_panel::PerformanceMetrics {
                fps,
                memory_usage: 0.0, // TODO: Implement when Performance.memory is available
                audio_latency: 0.0, // TODO: Get from audio system
                cpu_usage: 0.0, // TODO: Estimate from frame time
            };
            performance_metrics_setter.set(metrics);
        }
        scene.update_viewport(frame_input.viewport);
        scene.render(&mut frame_input.screen());

        gui.update(&mut frame_input.events, frame_input.accumulated_time, frame_input.viewport, frame_input.device_pixel_ratio,
            |gui_context| {
                dev_console.render(gui_context);
                microphone_button.render(gui_context);
                live_data_panel.render(gui_context);
            }
        );
        
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
    if let Err(e) = initialize_audio_systems(None).await {
        dev_log!("✗ Audio system initialization failed: {}", e);
        dev_log!("Application cannot continue without audio system");
        // TODO: Add error screen rendering in future story when UI requirements are defined
        return;
    }
    
    // Start Yew application
    yew::Renderer::<App>::new().render();
}

/// Initialize all audio systems in sequence with proper error handling
async fn initialize_audio_systems(
    pitch_data_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<debug::egui::live_data_panel::PitchData>>>>
) -> Result<(), String> {
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
    
    // Set the pitch data setter if provided
    if let Some(setter) = pitch_data_setter {
        audio::set_pitch_data_setter(setter);
        dev_log!("✓ Pitch data setter configured");
    }
    
    Ok(())
}
