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

use common::dev_log;

/// Convert main crate AudioPermission to egui console AudioPermission
fn convert_permission_to_egui(permission: &audio::AudioPermission) -> egui_dev_console::AudioPermission {
    match permission {
        audio::AudioPermission::Uninitialized => egui_dev_console::AudioPermission::Uninitialized,
        audio::AudioPermission::Requesting => egui_dev_console::AudioPermission::Requesting,
        audio::AudioPermission::Granted => egui_dev_console::AudioPermission::Granted,
        audio::AudioPermission::Denied => egui_dev_console::AudioPermission::Denied,
        audio::AudioPermission::Unavailable => egui_dev_console::AudioPermission::Unavailable,
    }
}


/// Request microphone permission and publish the result via events
/// This function is called synchronously from the user click callback
fn request_microphone_permission_and_publish_result(permission_source: std::sync::Arc<std::sync::Mutex<observable_data::DataSource<audio::AudioPermission>>>) {
    use crate::events::{get_global_event_dispatcher, audio_events::AudioEvent};
    
    let event_dispatcher = get_global_event_dispatcher();
    
    // Set state to requesting immediately (synchronously)
    permission_source.lock().unwrap().set(crate::audio::AudioPermission::Requesting);
    let event = AudioEvent::PermissionChanged(crate::audio::AudioPermission::Requesting);
    event_dispatcher.borrow().publish(&event);
    
    // Clone permission source reference for async block
    let permission_source_clone = permission_source.clone();
    
    // Start the async permission request (this should maintain the user gesture context)
    wasm_bindgen_futures::spawn_local(async move {
        match connect_microphone_to_audioworklet().await {
            Ok(_) => {
                web_sys::console::log_1(&"✓ Microphone connected successfully".into());
                // Update permission state and publish event
                permission_source_clone.lock().unwrap().set(crate::audio::AudioPermission::Granted);
                let event_dispatcher = get_global_event_dispatcher();
                let event = AudioEvent::PermissionChanged(crate::audio::AudioPermission::Granted);
                event_dispatcher.borrow().publish(&event);
            }
            Err(e) => {
                web_sys::console::error_1(&format!("✗ Microphone connection failed: {}", e).into());
                
                // Map error to permission state
                let permission_state = if e.contains("denied") || e.contains("NotAllowedError") {
                    crate::audio::AudioPermission::Denied
                } else if e.contains("NotFoundError") || e.contains("unavailable") {
                    crate::audio::AudioPermission::Unavailable
                } else {
                    crate::audio::AudioPermission::Unavailable
                };
                
                // Update permission state and publish event
                permission_source_clone.lock().unwrap().set(permission_state.clone());
                let event_dispatcher = get_global_event_dispatcher();
                let event = AudioEvent::PermissionChanged(permission_state);
                event_dispatcher.borrow().publish(&event);
            }
        }
    });
}

use wasm_bindgen::prelude::*;

use platform::{Platform, PlatformValidationResult};

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


/// Connect microphone stream to AudioWorklet for audio processing
pub async fn connect_microphone_to_audioworklet() -> Result<(), String> {
    use crate::audio::permission::PermissionManager;
    
    dev_log!("Requesting microphone permission and connecting to AudioWorklet");
    
    // Request microphone permission and get stream
    let media_stream = match PermissionManager::request_microphone_permission().await {
        Ok(stream) => {
            dev_log!("✓ Microphone permission granted, received MediaStream");
            stream
        }
        Err(e) => {
            dev_log!("✗ Microphone permission failed: {:?}", e);
            return Err(format!("Failed to get microphone permission: {:?}", e));
        }
    };
    
    // Get audio context and AudioWorklet manager
    let audio_context_manager = audio::get_audio_context_manager()
        .ok_or_else(|| "AudioContext manager not initialized".to_string())?;
    
    // Resume AudioContext if suspended (required for processing to start)
    {
        let mut manager = audio_context_manager.borrow_mut();
        if let Err(e) = manager.resume().await {
            dev_log!("⚠️ Failed to resume AudioContext: {:?}", e);
        } else {
            dev_log!("✓ AudioContext resumed for microphone processing");
        }
    }
    
    let audioworklet_manager = audio::get_global_audioworklet_manager()
        .ok_or_else(|| "AudioWorklet manager not initialized".to_string())?;
    
    // Create audio source from MediaStream
    let audio_context = {
        let manager = audio_context_manager.borrow();
        manager.get_context()
            .ok_or_else(|| "AudioContext not available".to_string())?
            .clone()
    };
    
    let source = match audio_context.create_media_stream_source(&media_stream) {
        Ok(source_node) => {
            dev_log!("✓ Created MediaStreamAudioSourceNode from microphone");
            source_node
        }
        Err(e) => {
            dev_log!("✗ Failed to create audio source: {:?}", e);
            return Err(format!("Failed to create audio source: {:?}", e));
        }
    };
    
    // Connect microphone source to AudioWorklet
    let mut worklet_manager = audioworklet_manager.borrow_mut();
    match worklet_manager.connect_microphone(source.as_ref()) {
        Ok(_) => {
            dev_log!("✓ Microphone successfully connected to AudioWorklet");
            
            // Note: No need to connect to destination - microphone → AudioWorklet is sufficient for processing
            
            // Ensure processing is active after connection
            if !worklet_manager.is_processing() {
                dev_log!("Starting AudioWorklet processing after microphone connection...");
                match worklet_manager.start_processing() {
                    Ok(_) => {
                        dev_log!("✓ AudioWorklet processing started - audio pipeline active");
                    }
                    Err(e) => {
                        dev_log!("⚠️ Failed to start processing after microphone connection: {:?}", e);
                    }
                }
            } else {
                dev_log!("✓ AudioWorklet already processing - audio pipeline active");
            }
            
            // Publish success event
            let event_dispatcher = crate::events::get_global_event_dispatcher();
            publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Processing, true, 0);
            
            Ok(())
        }
        Err(e) => {
            dev_log!("✗ Failed to connect microphone to AudioWorklet: {:?}", e);
            Err(format!("Failed to connect microphone: {:?}", e))
        }
    }
}

/// Publish AudioWorklet status update to Live Data Panel
#[cfg(not(test))]
fn publish_audioworklet_status(
    event_dispatcher: &crate::events::AudioEventDispatcher,
    state: audio::worklet::AudioWorkletState,
    processor_loaded: bool,
    chunks_processed: u32
) {
    #[cfg(target_arch = "wasm32")]
    let timestamp = js_sys::Date::now();
    #[cfg(not(target_arch = "wasm32"))]
    let timestamp = 0.0;
    
    let status = crate::debug::live_panel::AudioWorkletStatus {
        state: state.clone(),
        processor_loaded,
        chunk_size: 128, // Web Audio API standard
        chunks_processed,
        last_update: timestamp,
    };
    
    let status_event = crate::events::audio_events::AudioEvent::AudioWorkletStatusChanged(status);
    event_dispatcher.borrow().publish(&status_event);
    
    dev_log!("Published AudioWorklet status: {} (processor: {})", state, processor_loaded);
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

/// Initialize AudioWorklet manager with buffer pool and event dispatcher integration
async fn initialize_audioworklet_manager() -> Result<(), String> {
    dev_log!("Initializing AudioWorklet manager");
    
    // Get audio context manager
    let audio_context_manager = audio::get_audio_context_manager()
        .ok_or_else(|| "AudioContext manager not initialized".to_string())?;
    
    // Create AudioWorklet manager
    let mut worklet_manager = audio::AudioWorkletManager::new();
    
    // Get buffer pool and event dispatcher
    let buffer_pool = audio::get_global_buffer_pool()
        .ok_or_else(|| "Buffer pool not initialized".to_string())?;
    let event_dispatcher = crate::events::get_global_event_dispatcher();
    
    // Configure AudioWorklet manager
    worklet_manager.set_buffer_pool(buffer_pool);
    worklet_manager.set_event_dispatcher(event_dispatcher.clone());
    
    // Add volume detector for real-time volume analysis
    let volume_detector = audio::VolumeDetector::new_default();
    worklet_manager.set_volume_detector(volume_detector);
    
    // Publish initial status
    publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Initializing, false, 0);
    
    // Initialize AudioWorklet
    let audio_context_ref = audio_context_manager.borrow();
    match worklet_manager.initialize(&*audio_context_ref).await {
        Ok(_) => {
            dev_log!("✓ AudioWorklet processor loaded and ready");
            
            
            // Publish ready status
            publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Ready, true, 0);
            
            // Note: We don't connect AudioWorklet to destination to avoid audio feedback
            // The AudioWorklet will still process audio when microphone is connected to it
            
            // Start audio processing automatically
            match worklet_manager.start_processing() {
                Ok(_) => {
                    dev_log!("✓ Audio processing started automatically");
                    
                    // Publish processing status
                    publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Processing, true, 0);
                }
                Err(e) => {
                    dev_log!("✗ Failed to start audio processing: {:?}", e);
                    
                    // Still store the manager but in Ready state
                    publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Ready, true, 0);
                }
            }
            
            // Store globally for microphone connection
            audio::set_global_audioworklet_manager(std::rc::Rc::new(std::cell::RefCell::new(worklet_manager)));
            
            Ok(())
        }
        Err(e) => {
            dev_log!("✗ AudioWorklet initialization failed: {:?}", e);
            
            // Publish failed status
            publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Failed, false, 0);
            
            Err(format!("Failed to initialize AudioWorklet: {:?}", e))
        }
    }
}

pub async fn run_three_d() {
    dev_log!("Starting three-d with red sprites");
    
    // Create application data with observable permission state
    use crate::app_data::LiveData;
    use observable_data::DataSource;
    let permission_source = std::sync::Arc::new(std::sync::Mutex::new(
        DataSource::new(audio::AudioPermission::Uninitialized)
    ));
    let live_data = LiveData {
        microphone_permission: permission_source.lock().unwrap().observer(),
    };
    
    let window = Window::new(WindowSettings {
        title: "Sprites!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 15.0, 15.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(60.0),
        0.1,
        1000.0,
    );

    let axes = Axes::new(&context, 0.1, 1.0);

    let material = ColorMaterial {
        color: Srgba::new(255, 100, 100, 255), // Red color
        ..Default::default()
    };

    let billboards = Sprites::new(
        &context,
        &[
            vec3(-20.0, 0.0, -5.0),
            vec3(-15.0, 0.0, -10.0),
            vec3(-10.0, 0.0, -5.0),
        ],
        None,
    );

    let sprites_up = Sprites::new(
        &context,
        &[
            vec3(5.0, 0.0, -5.0),
            vec3(0.0, 0.0, -10.0),
            vec3(-5.0, 0.0, -5.0),
        ],
        Some(vec3(0.0, 1.0, 0.0)),
    );

    let sprites = Sprites::new(
        &context,
        &[
            vec3(20.0, 0.0, -5.0),
            vec3(15.0, 0.0, -10.0),
            vec3(10.0, 0.0, -5.0),
        ],
        Some(vec3(1.0, 1.0, 0.0).normalize()),
    );

    let ambient = AmbientLight::new(&context, 1.0, Srgba::WHITE);

    // Create egui GUI
    let mut gui = three_d::GUI::new(&context);
    
    // Create egui dev console with the same registry as the Yew console
    let registry = crate::console_commands::create_console_registry_with_audio();
    let mut dev_console = egui_dev_console::EguiDevConsole::new_with_registry(registry);

    
    // Set up microphone button click callback
    dev_console.set_microphone_click_callback({
        let permission_source = permission_source.clone();
        move || {
            // This function will be called directly by the user click
            request_microphone_permission_and_publish_result(permission_source.clone());
        }
    });

    dev_log!("Starting three-d + egui render loop");
    
    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);


        // Render 3D scene first
        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &camera,
                axes.into_iter()
                    .chain(&Gm {
                        geometry: &billboards,
                        material: &material,
                    })
                    .chain(&Gm {
                        geometry: &sprites_up,
                        material: &material,
                    })
                    .chain(&Gm {
                        geometry: &sprites,
                        material: &material,
                    }),
                &[&ambient],
            );

        // Render egui overlay  
        gui.update(&mut frame_input.events, frame_input.accumulated_time, frame_input.viewport, frame_input.device_pixel_ratio, |gui_context| {
            // Sync permission state from LiveData to console
            let current_permission = live_data.microphone_permission.get();
            let console_permission = convert_permission_to_egui(&current_permission);
            
            // Update console permission if it's different
            if dev_console.microphone_permission() != &console_permission {
                dev_console.update_microphone_permission(console_permission);
            }
            
            // Render development console
            dev_console.show(gui_context);
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
    initialize_audioworklet_manager().await
        .map_err(|e| format!("AudioWorklet manager initialization failed: {}", e))?;
    dev_log!("✓ AudioWorklet manager initialized successfully");
    
    // Initialize pitch analyzer (required)
    audio::initialize_pitch_analyzer().await
        .map_err(|e| format!("Pitch analyzer initialization failed: {}", e))?;
    dev_log!("✓ Pitch analyzer initialized successfully");
    
    Ok(())
}
