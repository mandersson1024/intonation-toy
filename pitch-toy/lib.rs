use three_d::{self, Window, WindowSettings, GUI, ClearState, FrameOutput, egui, egui::Color32};
use std::rc::Rc;
use std::cell::RefCell;

// Three-layer architecture modules
pub mod engine;
pub mod model;
pub mod presentation;

// Module interfaces
pub(crate) mod shared_types;

// Supporting modules
pub(crate) mod common;
pub(crate) mod debug;
pub(crate) mod debug_data;

use common::dev_log;
use wasm_bindgen::prelude::*;
use egui_dev_console::ConsoleCommandRegistry;

use engine::platform::{Platform, PlatformValidationResult};

// Import action types for three-layer action processing
use debug::egui::HybridEguiLiveDataPanel;





// Legacy run_three_d function removed - using hybrid architecture only

/// Run three-d with three-layer architecture
/// 
/// This function orchestrates the three-layer architecture with the following sequence:
/// 1. Three-layer update (engine → model → presenter)
/// 2. User action processing (presenter → model → engine)
/// 3. Debug action processing (presenter → engine, debug builds only)
/// 4. GUI update and rendering
///
/// Action processing ensures proper validation and separation of concerns:
/// - User actions are collected from the presentation layer
/// - Actions are validated in the model layer
/// - Validated actions are executed in the engine layer
/// - Debug actions bypass validation for testing purposes
pub async fn run_three_d_with_layers(
    mut engine: Option<engine::AudioEngine>,
    mut model: Option<model::DataModel>,
    presenter: Option<Rc<RefCell<presentation::Presenter>>>,
) {
    dev_log!("Starting three-d with three-layer architecture");
    
    let window = Window::new(WindowSettings {
        title: "pitch-toy".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);
    
    let mut command_registry = ConsoleCommandRegistry::new();
    crate::engine::platform::commands::register_platform_commands(&mut command_registry);
    crate::engine::audio::register_audio_commands(&mut command_registry);

    let mut dev_console = egui_dev_console::DevConsole::new_with_registry(command_registry);
    
    
    // Create hybrid live data without legacy interface
    let hybrid_live_data = debug_data::DebugData::new();
    
    // Create hybrid debug panel
    let mut hybrid_live_data_panel = if let Some(ref presenter_ref) = presenter {
        Some(HybridEguiLiveDataPanel::new(
            hybrid_live_data,
            presenter_ref.clone(),
        ))
    } else {
        None
    };

    dev_log!("Starting three-d + egui render loop with three-layer architecture");
    
    // Set up user gesture handler for microphone permission
    let permission_granted = std::rc::Rc::new(std::cell::RefCell::new(false));
    setup_first_click_handler(permission_granted.clone(), &mut engine);
    
    // Performance tracking
    let mut frame_count = 0u32;
    let mut last_fps_update = 0.0;
    let mut fps = 0.0;
    
    window.render_loop(move |mut frame_input| {
        // Update FPS counter
        frame_count += 1;
        let current_time = frame_input.accumulated_time;
        
        // Update FPS every second
        if current_time - last_fps_update >= 1000.0 {
            fps = (frame_count as f64) / ((current_time - last_fps_update) / 1000.0);
            frame_count = 0;
            last_fps_update = current_time;
            
            // Update performance metrics
            let _metrics = debug::egui::data_types::PerformanceMetrics {
                fps,
                memory_usage: 0.0, // Placeholder
                audio_latency: 0.0, // Placeholder
                cpu_usage: 0.0, // Placeholder
            };
        }
        
        // Three-layer update sequence (engine → model → presenter)
        let timestamp = current_time / 1000.0;
        
        // Update engine layer and get results
        let engine_data = if let Some(ref mut engine) = engine {
            engine.update(timestamp)
        } else {
            // Provide default engine data when engine is not available
            crate::shared_types::EngineUpdateResult {
                audio_analysis: None,
                audio_errors: Vec::new(),
                permission_state: crate::shared_types::PermissionState::NotRequested,
            }
        };
        
        // Update model layer with engine data and capture result
        let model_data = if let Some(ref mut model) = model {
            model.update(timestamp, engine_data.clone())
        } else {
            // Provide default model data when model is not available
            crate::shared_types::ModelUpdateResult {
                volume: crate::shared_types::Volume { peak: -60.0, rms: -60.0 },
                pitch: crate::shared_types::Pitch::NotDetected,
                accuracy: crate::shared_types::Accuracy {
                    closest_note: crate::shared_types::Note::A,
                    accuracy: 1.0,
                },
                tuning_system: crate::shared_types::TuningSystem::EqualTemperament,
                errors: Vec::new(),
                permission_state: crate::shared_types::PermissionState::NotRequested,
            }
        };
        
        // Update debug panel data with engine and model results
        if let Some(ref mut panel) = hybrid_live_data_panel {
            panel.update_data(&engine_data, Some(&model_data));
        }
        
        // Update debug panel data with performance metrics
        let performance_metrics = debug::egui::data_types::PerformanceMetrics {
            fps,
            memory_usage: 0.0, // Placeholder
            audio_latency: 0.0, // Placeholder
            cpu_usage: 0.0, // Placeholder
        };
        if let Some(ref mut panel) = hybrid_live_data_panel {
            // Collect real debug data from the engine
            #[cfg(debug_assertions)]
            let (audio_devices, audioworklet_status, buffer_pool_stats) = if let Some(ref engine) = engine {
                let devices = engine.get_debug_audio_devices();
                let status = engine.get_debug_audioworklet_status().map(|s| {
                    // Convert from engine AudioWorkletStatus to debug AudioWorkletStatus
                    debug::egui::data_types::AudioWorkletStatus {
                        state: s.state,
                        processor_loaded: s.processor_loaded,
                        chunk_size: s.chunk_size,
                        chunks_processed: s.chunks_processed,
                        last_update: s.last_update,
                    }
                });
                let stats = engine.get_debug_buffer_pool_stats();
                (devices, status, stats)
            } else {
                (None, None, None)
            };
            
            #[cfg(not(debug_assertions))]
            let (audio_devices, audioworklet_status, buffer_pool_stats) = (None, None, None);
            
            panel.update_debug_data(
                audio_devices,
                Some(performance_metrics),
                audioworklet_status,
                buffer_pool_stats,
            );
        }
        
        // Update presentation layer with model data
        if let Some(ref presenter) = presenter {
            if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                presenter_ref.update(timestamp, model_data);
                presenter_ref.update_viewport(frame_input.viewport);
            }
        }
        
        // Process user actions through three-layer validation and execution
        if let (Some(presenter), Some(model), Some(engine)) = (&presenter, &mut model, &mut engine) {
            // Collect user actions from presentation layer
            let user_actions = if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                presenter_ref.get_user_actions()
            } else {
                presentation::PresentationLayerActions::new()
            };
            
            // Only process if there are actions to handle
            let has_user_actions = !user_actions.tuning_system_changes.is_empty() ||
                                  !user_actions.root_note_adjustments.is_empty();
            
            if has_user_actions {
                dev_log!("Processing {} user actions", 
                    user_actions.tuning_system_changes.len() + 
                    user_actions.root_note_adjustments.len()
                );
                
                // Process and validate actions in model layer
                let processed_actions = model.process_user_actions(user_actions);
                
                // Log validation errors if any
                for error in &processed_actions.validation_errors {
                    dev_log!("Action validation error: {:?}", error);
                }
                
                // Execute validated actions in engine layer
                let has_model_actions = !processed_actions.actions.audio_system_configurations.is_empty() ||
                                       !processed_actions.actions.tuning_configurations.is_empty();
                
                if has_model_actions {
                    dev_log!("Actions ready for execution: {} audio system, {} tuning", 
                        processed_actions.actions.audio_system_configurations.len(),
                        processed_actions.actions.tuning_configurations.len()
                    );
                    
                    // Execute actions synchronously
                    let total_sync = processed_actions.actions.audio_system_configurations.len() + 
                                   processed_actions.actions.tuning_configurations.len();
                    match engine.execute_actions(processed_actions.actions) {
                        Ok(()) => {
                            if total_sync > 0 {
                                dev_log!("✓ Executed {} actions", total_sync);
                            }
                        }
                        Err(e) => {
                            dev_log!("✗ Action execution failed: {}", e);
                        }
                    }
                }
            }
        } else {
            // Log if action processing is skipped due to missing layers
            if presenter.is_none() || model.is_none() || engine.is_none() {
                let missing = vec![
                    if presenter.is_none() { "presenter" } else { "" },
                    if model.is_none() { "model" } else { "" },
                    if engine.is_none() { "engine" } else { "" },
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(", ");
                
                if !missing.is_empty() {
                    dev_log!("Skipping user action processing - missing layers: {}", missing);
                }
            }
        }
        
        // Process debug actions with privileged access (debug builds only)
        #[cfg(debug_assertions)]
        {
            if let (Some(presenter), Some(_engine)) = (&presenter, &mut engine) {
                // Collect debug actions from presentation layer
                let debug_actions = if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                    presenter_ref.get_debug_actions()
                } else {
                    presentation::DebugLayerActions::new()
                };
                
                // Only process if there are debug actions to handle
                let has_debug_actions = !debug_actions.test_signal_configurations.is_empty() ||
                                       !debug_actions.speaker_output_configurations.is_empty() ||
                                       !debug_actions.background_noise_configurations.is_empty();
                
                if has_debug_actions {
                    dev_log!("[DEBUG] Processing {} debug actions", 
                        debug_actions.test_signal_configurations.len() + 
                        debug_actions.speaker_output_configurations.len() + 
                        debug_actions.background_noise_configurations.len()
                    );
                    
                    // Execute debug actions synchronously
                    match _engine.execute_debug_actions_sync(debug_actions) {
                        Ok(executed_debug_actions) => {
                            let total_debug = executed_debug_actions.test_signal_executions.len() + 
                                            executed_debug_actions.speaker_output_executions.len() + 
                                            executed_debug_actions.background_noise_executions.len();
                            if total_debug > 0 {
                                dev_log!("[DEBUG] ✓ Executed {} debug actions", total_debug);
                            }
                        }
                        Err(e) => {
                            dev_log!("[DEBUG] ✗ Debug action execution failed: {}", e);
                        }
                    }
                }
            } else {
                // Log if debug action processing is skipped due to missing layers
                if presenter.is_none() || engine.is_none() {
                    let missing = vec![
                        if presenter.is_none() { "presenter" } else { "" },
                        if engine.is_none() { "engine" } else { "" },
                    ]
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                    
                    if !missing.is_empty() {
                        dev_log!("[DEBUG] Skipping debug action processing - missing layers: {}", missing);
                    }
                }
            }
        }
        
        // Extract needed values before borrowing screen
        let accumulated_time = frame_input.accumulated_time;
        let viewport = frame_input.viewport;
        let device_pixel_ratio = frame_input.device_pixel_ratio;
        
        gui.update(
            &mut frame_input.events,
            accumulated_time,
            viewport,
            device_pixel_ratio,
            |gui_context| {
                gui_context.set_visuals(egui::Visuals {
                    window_fill: Color32::from_rgba_unmultiplied(27, 27, 27, 240),
                    override_text_color: Some(Color32::from_rgb(255, 255, 255)),
                    ..egui::Visuals::default()
                });
                
                dev_console.render(gui_context);
                if let Some(ref mut panel) = hybrid_live_data_panel {
                    panel.render(gui_context);
                }
            }
        );
        
        let mut screen = frame_input.screen();
        screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));
        
        // Render presentation layer
        if let Some(ref presenter) = presenter {
            if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                presenter_ref.render(&context, &mut screen);
            }
        }
        
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
    
    web_sys::console::log_1(&"DEBUG: Starting Pitch Toy application".into());
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    dev_log!("{}", Platform::get_platform_info());
    
    // Validate critical platform APIs before proceeding
    web_sys::console::log_1(&"DEBUG: Checking platform feature support...".into());
    if let PlatformValidationResult::MissingCriticalApis(missing_apis) = Platform::check_feature_support() {
        let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
        web_sys::console::error_1(&format!("✗ CRITICAL: Missing required browser APIs: {}", api_list.join(", ")).into());
        web_sys::console::error_1(&"✗ Application cannot start. Please upgrade your browser or use a supported browser:".into());
        // TODO: Add error screen rendering in future story when UI requirements are defined
        return;
    }

    web_sys::console::log_1(&"DEBUG: ✓ Platform validation passed - initializing three-layer architecture".into());
    
    // Create three-layer architecture instances
    dev_log!("Creating three-layer architecture instances...");
    
    // Create engine layer
    let engine = match engine::AudioEngine::create().await {
        Ok(engine) => {
            dev_log!("✓ Engine layer created successfully");
            Some(engine)
        }
        Err(e) => {
            dev_log!("✗ Engine layer creation failed: {}", e);
            dev_log!("Application will continue without engine layer");
            None
        }
    };
    
    // Create model layer
    let model = match model::DataModel::create() {
        Ok(model) => {
            dev_log!("✓ Model layer created successfully");
            Some(model)
        }
        Err(e) => {
            dev_log!("✗ Model layer creation failed: {}", e);
            dev_log!("Application will continue without model layer");
            None
        }
    };
    
    // Create presentation layer
    let presenter = match presentation::Presenter::create() {
        Ok(presenter) => {
            dev_log!("✓ Presentation layer created successfully");
            // Sprite scene will be initialized on first render to avoid variable shadowing
            Some(Rc::new(RefCell::new(presenter)))
        }
        Err(e) => {
            dev_log!("✗ Presentation layer creation failed: {}", e);
            dev_log!("Application will continue without presentation layer");
            None
        }
    };
    
    // Start three-d application with three-layer architecture
    run_three_d_with_layers(
        engine,
        model,
        presenter,
    ).await;
}

/// Set up a full-screen invisible overlay to capture the first user click
/// and request microphone permission while preserving the user gesture context
#[cfg(target_arch = "wasm32")]
fn setup_first_click_handler(
    permission_granted: std::rc::Rc<std::cell::RefCell<bool>>,
    engine: &mut Option<engine::AudioEngine>,
) {
    use web_sys::{window, HtmlElement, EventTarget};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    
    let window = match window() {
        Some(w) => w,
        None => {
            dev_log!("⚠ No window object available for first click handler");
            return;
        }
    };
    
    let document = match window.document() {
        Some(d) => d,
        None => {
            dev_log!("⚠ No document object available for first click handler");
            return;
        }
    };
    
    // Create full-screen overlay div
    let overlay = match document.create_element("div") {
        Ok(el) => el.dyn_into::<HtmlElement>().unwrap(),
        Err(_) => {
            dev_log!("⚠ Failed to create overlay div");
            return;
        }
    };
    
    // Style the overlay to be full-screen and invisible
    overlay.set_attribute("style", 
        "position: fixed; top: 0; left: 0; width: 100%; height: 100%; \
         background: transparent; z-index: 9999; cursor: pointer;"
    ).unwrap();
    
    // Add instructions text
    overlay.set_inner_html(
        "<div style='position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); \
         color: #fff; font-family: Arial, sans-serif; font-size: 18px; text-align: center; \
         background: rgba(0,0,0,0.8); padding: 20px; border-radius: 10px;'>
         Click anywhere to start<br>
         <small style='opacity: 0.7;'>(Microphone permission will be requested)</small>
         </div>"
    );
    
    // Get audio context from engine for the permission request
    let audio_context = engine.as_ref()
        .and_then(|e| e.get_audio_context());
    
    if audio_context.is_none() {
        dev_log!("⚠ No audio context available for permission request");
        return;
    }
    
    let audio_context = audio_context.unwrap();
    let permission_granted_clone = permission_granted.clone();
    
    // Get engine reference for passing MediaStream
    let engine_ref = engine.as_ref().map(|e| e.get_audio_context()).flatten();
    
    // Create click handler closure
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        let permission_granted = permission_granted_clone.clone();
        let audio_context = audio_context.clone();
        let engine_context = engine_ref.clone();
        
        dev_log!("First click detected - requesting microphone permission");
        
        // Remove the overlay immediately
        if let Some(browser_window) = web_sys::window() {
            if let Some(document) = browser_window.document() {
                if let Some(body) = document.body() {
                    // Use querySelectorAll to find all overlays
                    if let Ok(Some(overlay_element)) = document.query_selector("div[style*='z-index: 9999']") {
                        if let Some(parent) = overlay_element.parent_node() {
                            let _ = parent.remove_child(&overlay_element);
                        }
                    }
                }
            }
        }
        
        // Call getUserMedia directly in the synchronous click handler to preserve user activation
        use web_sys::MediaStreamConstraints;
        use wasm_bindgen_futures::JsFuture;
        
        // Get navigator.mediaDevices
        if let Some(navigator) = web_sys::window().and_then(|w| w.navigator().media_devices().ok()) {
            // Create constraints for audio only
            let mut constraints = MediaStreamConstraints::new();
            constraints.set_audio(&true.into());
            constraints.set_video(&false.into());
            
            // Call getUserMedia synchronously (preserves user gesture context)
            if let Ok(promise) = navigator.get_user_media_with_constraints(&constraints) {
                // Now we can spawn the async part that handles the Promise result
                wasm_bindgen_futures::spawn_local(async move {
                    match JsFuture::from(promise).await {
                        Ok(stream_js) => {
                            dev_log!("✓ Microphone permission granted on first click");
                            *permission_granted.borrow_mut() = true;
                            
                            // Convert JsValue to MediaStream and pass to engine
                            if let Ok(media_stream) = stream_js.dyn_into::<web_sys::MediaStream>() {
                                if let Some(engine_context) = engine_context {
                                    match crate::engine::audio::microphone::connect_existing_mediastream_to_audioworklet(media_stream, &engine_context).await {
                                        Ok(_) => {
                                            dev_log!("✓ MediaStream successfully connected to engine");
                                        }
                                        Err(e) => {
                                            dev_log!("✗ Failed to connect MediaStream to engine: {}", e);
                                        }
                                    }
                                } else {
                                    dev_log!("⚠ No engine context available to connect MediaStream");
                                }
                            } else {
                                dev_log!("✗ Failed to convert stream to MediaStream");
                            }
                        }
                        Err(e) => {
                            dev_log!("✗ Microphone permission failed on first click: {:?}", e);
                        }
                    }
                });
            } else {
                dev_log!("✗ Failed to call getUserMedia");
            }
        } else {
            dev_log!("✗ MediaDevices API not available");
        }
    }) as Box<dyn FnMut(_)>);
    
    // Add event listener to overlay
    let target: &EventTarget = overlay.as_ref();
    target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
    
    // Keep the closure alive
    closure.forget();
    
    // Append overlay to body
    if let Some(body) = document.body() {
        body.append_child(&overlay).unwrap();
        dev_log!("✓ First click handler overlay added");
    } else {
        dev_log!("⚠ No body element available to append overlay");
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn setup_first_click_handler(
    _permission_granted: std::rc::Rc<std::cell::RefCell<bool>>,
    _engine: &mut Option<engine::AudioEngine>,
) {
    // No-op for non-wasm targets
}


