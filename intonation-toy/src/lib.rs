use three_d::{Window, WindowSettings, FrameOutput, egui};
use std::rc::Rc;
use std::cell::RefCell;

// Configuration module
pub mod app_config;

// Three-layer architecture modules
pub mod engine;
pub mod model;
pub mod presentation;

// Platform-specific modules
pub mod web;

// Module interfaces
pub mod shared_types;

// Theme management
pub mod theme;

// Music theory module
pub mod music_theory;

// Supporting modules
pub(crate) mod common;
#[cfg(debug_assertions)]
pub(crate) mod debug;

use wasm_bindgen::prelude::*;
#[cfg(all(debug_assertions, not(feature = "profiling")))]
use egui_dev_console::ConsoleCommandRegistry;

use engine::platform::{Platform, PlatformValidationResult};

#[cfg(target_arch = "wasm32")]
fn resize_canvas(canvas: &web_sys::HtmlCanvasElement) {
    dev_log!("RESIZE: resize_canvas called");
    let window_obj = web_sys::window().unwrap();
    let document = window_obj.document().unwrap();
    
    let sidebar_width = crate::web::styling::SIDEBAR_WIDTH;
    let margin = crate::web::styling::CANVAS_MARGIN;
    
    // Calculate available space (subtract sidebar width and margins)
    let available_width = window_obj.inner_width().unwrap().as_f64().unwrap() as i32 - sidebar_width - (margin * 2);
    let available_height = window_obj.inner_height().unwrap().as_f64().unwrap() as i32 - (margin * 2);
    
    dev_log!("RESIZE: available {}x{}", available_width, available_height);
    
    // Take the smaller dimension to maintain square aspect ratio
    let mut canvas_size = std::cmp::min(available_width, available_height);
    canvas_size = std::cmp::min(canvas_size, crate::app_config::CANVAS_MAX_SIZE);
    canvas_size = std::cmp::max(canvas_size, crate::app_config::CANVAS_MIN_SIZE);
    
    // Scene wrapper size is just the canvas size
    let wrapper_width = canvas_size;
    let wrapper_height = canvas_size;
    
    dev_log!("RESIZE: setting canvas size to {}px, wrapper size to {}x{}", canvas_size, wrapper_width, wrapper_height);
    
    // Get the scene wrapper element
    let scene_wrapper = document.get_element_by_id("scene-wrapper").unwrap();
    
    // Set CSS positioning and sizing for scene wrapper
    scene_wrapper.set_attribute("style", &format!(
        "position: absolute; top: {}px; left: {}px; width: {}px; height: {}px;",
        margin, margin, wrapper_width, wrapper_height
    )).unwrap();
    
    // Set canvas to specific size
    canvas.style().set_property("width", &format!("{}px", canvas_size)).unwrap();
    canvas.style().set_property("height", &format!("{}px", canvas_size)).unwrap();
}

#[cfg(all(debug_assertions, not(feature = "profiling")))]
use debug::debug_panel::DebugPanel;

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
pub async fn start_render_loop(
    mut engine: Option<engine::AudioEngine>,
    mut model: Option<model::DataModel>,
    presenter: Option<Rc<RefCell<presentation::Presenter>>>,
) {
    dev_log!("Starting three-d with three-layer architecture");
    
    // Get existing canvas element and set up resize handling
    #[cfg(target_arch = "wasm32")]
    let canvas = {
        let window_obj = web_sys::window().unwrap();
        let document = window_obj.document().unwrap();
        
        let canvas = document.get_element_by_id("three-d-canvas").unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        
        // Set up resize event handler
        let canvas_clone = canvas.clone();
        let resize_callback = Closure::wrap(Box::new(move || {
            resize_canvas(&canvas_clone);
        }) as Box<dyn FnMut()>);
        
        window_obj.add_event_listener_with_callback("resize", resize_callback.as_ref().unchecked_ref()).unwrap();
        resize_callback.forget(); // Keep the closure alive
        
        Some(canvas)
    };

    let dpr = web_sys::window().unwrap().device_pixel_ratio();
    dev_log!("device pixel ratio: {}", dpr);
    
    let render_size: u32 = if dpr <= 1.0 { app_config::VIEWPORT_RENDER_SIZE } else { app_config::VIEWPORT_RENDER_SIZE_RETINA };

    let window = Window::new(WindowSettings {
        title: app_config::WINDOW_TITLE.to_string(),
        max_size: Some((render_size, render_size)),
        ..Default::default()
    })
    .unwrap();
    
    // Apply initial canvas sizing after three_d window initialization
    #[cfg(target_arch = "wasm32")]
    if let Some(ref canvas_element) = canvas {
        resize_canvas(canvas_element);
    }
    
    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);
    
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut command_registry = ConsoleCommandRegistry::new();
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    crate::engine::platform::commands::register_platform_commands(&mut command_registry);
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    crate::engine::audio::register_audio_commands(&mut command_registry);

    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut dev_console = egui_dev_console::DevConsole::new(command_registry);
    
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let debug_data = debug::debug_data::DebugData::new();
    
    // Create debug panel
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut debug_panel = presenter.as_ref().map(|presenter_ref| DebugPanel::new(
            debug_data,
            presenter_ref.clone(),
        ));

    dev_log!("Starting render loop");
    
    // Set up user gesture handler for microphone permission
    let permission_granted = std::rc::Rc::new(std::cell::RefCell::new(false));
    web::first_click_handler::setup_first_click_handler(permission_granted.clone(), &mut engine);
    
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
            
            // Performance metrics update happens later in the debug panel section
        }
        
        let timestamp = current_time / 1000.0;
        
        // Update engine layer and get results
        let engine_data = if let Some(ref mut engine) = engine {
            #[cfg(feature = "profiling")]
            {
                crate::web::profiling::profiled("engine_update", || {
                    engine.update(timestamp)
                })
            }
            #[cfg(not(feature = "profiling"))]
            {
                engine.update(timestamp)
            }
        } else {
            // Provide default engine data when engine is not available
            crate::shared_types::EngineUpdateResult {
                audio_analysis: None,
                audio_errors: Vec::new(),
                permission_state: crate::shared_types::PermissionState::NotRequested,
            }
        };
        
        // Process user actions through three-layer validation and execution
        if let (Some(presenter), Some(model), Some(engine)) = (&presenter, &mut model, &mut engine) {
            let mut user_action_processing = || {
                // Collect user actions from presentation layer
                let user_actions = if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                    presenter_ref.get_user_actions()
                } else {
                    presentation::PresentationLayerActions::default()
                };
                
                // Only process if there are actions to handle
                let has_user_actions = !user_actions.tuning_system_changes.is_empty() ||
                                      !user_actions.tuning_fork_adjustments.is_empty() ||
                                      !user_actions.scale_changes.is_empty() ||
                                      !user_actions.tuning_fork_configurations.is_empty();
                
                if has_user_actions {
                    trace_log!("Processing {} user actions (tuning: {}, tuning_fork: {}, scale: {}, audio: {})", 
                        user_actions.tuning_system_changes.len() + 
                        user_actions.tuning_fork_adjustments.len() +
                        user_actions.scale_changes.len() +
                        user_actions.tuning_fork_configurations.len(),
                        user_actions.tuning_system_changes.len(),
                        user_actions.tuning_fork_adjustments.len(),
                        user_actions.scale_changes.len(),
                        user_actions.tuning_fork_configurations.len()
                    );
                    
                    // Process and validate actions in model layer
                    let processed_actions = model.process_user_actions(user_actions);
                    
                    // Log validation errors if any
                    for error in &processed_actions.validation_errors {
                        dev_log!("Action validation error: {:?}", error);
                    }
                    
                    // Execute validated actions in engine layer
                    let has_model_actions = !processed_actions.actions.audio_system_configurations.is_empty() ||
                                           !processed_actions.actions.tuning_configurations.is_empty() ||
                                           !processed_actions.actions.tuning_fork_configurations.is_empty();
                    
                    if has_model_actions {
                        trace_log!("Actions ready for execution: {} audio system, {} tuning, {} tuning fork audio", 
                            processed_actions.actions.audio_system_configurations.len(),
                            processed_actions.actions.tuning_configurations.len(),
                            processed_actions.actions.tuning_fork_configurations.len()
                        );
                        
                        // Execute actions synchronously
                        let total_sync = processed_actions.actions.audio_system_configurations.len() + 
                                       processed_actions.actions.tuning_configurations.len() +
                                       processed_actions.actions.tuning_fork_configurations.len();
                        match engine.execute_actions(processed_actions.actions) {
                            Ok(()) => {
                                if total_sync > 0 {
                                    trace_log!("✓ Executed {} actions", total_sync);
                                }
                            }
                            Err(e) => {
                                dev_log!("✗ Action execution failed: {}", e);
                            }
                        }
                    }
                }
            };

            #[cfg(feature = "profiling")]
            {
                crate::web::profiling::profiled("user_action_processing", user_action_processing);
            }
            
            #[cfg(not(feature = "profiling"))]
            {
                user_action_processing();
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
                    trace_log!("Skipping user action processing - missing layers: {}", missing);
                }
            }
        }
        
        // Update model layer with engine data and capture result
        let model_data = if let Some(ref mut model) = model {
            #[cfg(feature = "profiling")]
            {
                Some(crate::web::profiling::profiled("model_update", || {
                    model.update(timestamp, engine_data.clone())
                }))
            }
            #[cfg(not(feature = "profiling"))]
            {
                Some(model.update(timestamp, engine_data.clone()))
            }
        } else {
            None
        };
        
        // Update debug panel data with engine and model results
        #[cfg(all(debug_assertions, not(feature = "profiling")))]
        if let Some(ref mut panel) = debug_panel {
            panel.update_data(&engine_data, model_data.as_ref());
        }
        
        // Update debug panel data with performance metrics
        #[cfg(all(debug_assertions, not(feature = "profiling")))]
        {
            let (memory_usage_mb, memory_usage_percent) = web::performance::sample_memory_usage().unwrap_or((0.0, 0.0));
            
            let performance_metrics = debug::data_types::PerformanceMetrics {
                fps,
                memory_usage_mb,
                memory_usage_percent,
            };
            if let Some(ref mut panel) = debug_panel {
                // Collect real debug data from the engine
                let (audio_devices, audioworklet_status, buffer_pool_stats) = if let Some(ref engine) = engine {
                    let devices = engine.get_debug_audio_devices();
                    let status = engine.get_debug_audioworklet_status().map(|s| {
                        // Convert from engine AudioWorkletStatus to debug AudioWorkletStatus
                        debug::data_types::AudioWorkletStatus {
                            state: s.state,
                            processor_loaded: s.processor_loaded,
                            chunk_size: s.chunk_size,
                            batch_size: s.batch_size,
                            batches_processed: s.batches_processed,
                        }
                    });
                    let stats = engine.get_debug_buffer_pool_stats();
                    (devices, status, stats)
                } else {
                    (None, None, None)
                };
                
                // Update debug-specific data
                panel.update_debug_data(
                    audio_devices,
                    Some(performance_metrics),
                    audioworklet_status,
                    buffer_pool_stats,
                );
            }
        }
        
        // Update presentation layer with model data
        if let Some(ref presenter) = presenter {
            if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                if let Some(ref data) = model_data {
                    presenter_ref.process_data(timestamp, data.clone());
                    presenter_ref.update_graphics(frame_input.viewport, data);
                }
            }
        }
        
        // Process debug actions with privileged access (debug builds only)
        #[cfg(debug_assertions)]
        {
            let (Some(presenter), Some(_engine)) = (&presenter, &mut engine) else {
                panic!("Critical error: presenter or engine is None during render loop. This indicates a serious initialization bug.");
            };
            
            // Collect debug actions from presentation layer
            let debug_actions = if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                presenter_ref.get_debug_actions()
            } else {
                presentation::DebugLayerActions::new()
            };
            
            // Only process if there are debug actions to handle
            let has_debug_actions = !debug_actions.test_signal_configurations.is_empty();
            
            if has_debug_actions {
                trace_log!("[DEBUG] Processing {} debug actions", 
                    debug_actions.test_signal_configurations.len()
                );
                
                // Execute debug actions synchronously
                match _engine.execute_debug_actions_sync(debug_actions) {
                    Ok(executed_debug_actions) => {
                        let total_debug = executed_debug_actions.test_signal_executions.len();
                        if total_debug > 0 {
                            trace_log!("[DEBUG] ✓ Executed {} debug actions", total_debug);
                        }
                    }
                    Err(e) => {
                        dev_log!("[DEBUG] ✗ Debug action execution failed: {}", e);
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
                // In debug mode (but not profiling), use dark theme for debug panels and console
                #[cfg(all(debug_assertions, not(feature = "profiling")))]
                {
                    // Set dark theme visuals for debug UI
                    gui_context.set_visuals(egui::Visuals::dark());
                    
                    dev_console.render(gui_context);
                    if let Some(ref mut panel) = debug_panel {
                        if let Some(ref data) = model_data {
                            panel.render(gui_context, data);
                        }
                    }
                }
            }
        );
        
        let mut screen = frame_input.screen();
        
        // Render presentation layer (which handles its own screen clearing)
        if let Some(ref presenter) = presenter {
            if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                if let Some(ref data) = model_data {
                    presenter_ref.render(&context, &mut screen, data);
                }
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
    
    log!("Starting Intonation Toy application");
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    dev_log!("{}", Platform::get_platform_info());
    
    // Validate critical platform APIs before proceeding
    dev_log!("Checking platform feature support...");
    match Platform::check_feature_support() {
        PlatformValidationResult::MissingCriticalApis(missing_apis) => {
            let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
            error_log!("✗ CRITICAL: Missing required browser APIs: {}", api_list.join(", "));
            error_log!("✗ Application cannot start. Please upgrade your browser or use a supported browser:");
            
            // Display error overlay for unsupported browser
            #[cfg(target_arch = "wasm32")]
            {
                let missing_apis_str = api_list.join(", ");
                crate::web::error_message_box::show_error_with_params(&crate::shared_types::Error::BrowserApiNotSupported, &[&missing_apis_str]);
            }
            return;
        }
        PlatformValidationResult::MobileDevice => {
            error_log!("✗ CRITICAL: Mobile device detected - application not supported on mobile");
            error_log!("✗ Application cannot start on mobile devices. Please use a desktop or laptop computer.");
            
            // Display error overlay for mobile device
            #[cfg(target_arch = "wasm32")]
            {
                crate::web::error_message_box::show_error(&crate::shared_types::Error::MobileDeviceNotSupported);
            }
            return;
        }
        PlatformValidationResult::AllSupported => {
            log!("✓ Platform validation passed - initializing three-layer architecture");
        }
    }
    
    // Initialize theme system
    crate::theme::initialize_theme(crate::app_config::DEFAULT_THEME);
    
    // Apply CSS custom properties for theme switching (static CSS already loaded)
    dev_log!("Applying CSS custom properties for theme initialization...");
    crate::web::styling::apply_color_scheme_styles();
    
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
            let presenter_rc = Rc::new(RefCell::new(presenter));
            
            // Set the self-reference for UI event handling
            presenter_rc.borrow_mut().set_self_reference(presenter_rc.clone());
            
            Some(presenter_rc)
        }
        Err(e) => {
            dev_log!("✗ Presentation layer creation failed: {}", e);
            dev_log!("Application will continue without presentation layer");
            None
        }
    };
    
    // Start three-d application with three-layer architecture
    start_render_loop(
        engine,
        model,
        presenter,
    ).await;
}