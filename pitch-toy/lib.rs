use three_d::{self, *};
use three_d::egui::Color32;

// Three-layer architecture modules
pub mod engine;
pub mod model;
pub mod presentation;

// Module interfaces
#[path = "module-interfaces/mod.rs"]
pub mod module_interfaces;

// Supporting modules
pub mod common;
pub mod debug;
pub mod live_data;

use common::dev_log;
use wasm_bindgen::prelude::*;
use egui_dev_console::ConsoleCommandRegistry;

use engine::platform::{Platform, PlatformValidationResult};

// Import action types for three-layer action processing
use debug::egui::{EguiMicrophoneButton, HybridEguiLiveDataPanel};


// Import action system
use action::{Action, ActionTrigger, ActionListener};


// UI Control Action Types
#[derive(Debug, Clone)]
pub struct TestSignalAction {
    pub enabled: bool,
    pub waveform: engine::audio::TestWaveform,
    pub frequency: f32,
    pub volume: f32,
}

#[derive(Debug, Clone)]
pub struct BackgroundNoiseAction {
    pub enabled: bool,
    pub level: f32,
    pub noise_type: engine::audio::TestWaveform,
}

#[derive(Debug, Clone)]
pub struct OutputToSpeakersAction {
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct MicrophonePermissionAction {
    pub request_permission: bool,
}

/// UI Control Actions - Central action registry for UI controls
pub struct UIControlActions {
    pub test_signal: Action<TestSignalAction>,
    pub background_noise: Action<BackgroundNoiseAction>,
    pub output_to_speakers: Action<OutputToSpeakersAction>,
    pub microphone_permission: Action<MicrophonePermissionAction>,
}

impl Default for UIControlActions {
    fn default() -> Self {
        Self::new()
    }
}

impl UIControlActions {
    pub fn new() -> Self {
        Self {
            test_signal: Action::new(),
            background_noise: Action::new(),
            output_to_speakers: Action::new(),
            microphone_permission: Action::new(),
        }
    }
    
    /// Get trigger handles for UI components
    pub fn get_triggers(&self) -> UIControlTriggers {
        UIControlTriggers {
            test_signal: self.test_signal.trigger(),
            background_noise: self.background_noise.trigger(),
            output_to_speakers: self.output_to_speakers.trigger(),
            microphone_permission: self.microphone_permission.trigger(),
        }
    }
    
    /// Get listener handles for audio module
    pub fn get_listeners(&self) -> UIControlListeners {
        UIControlListeners {
            test_signal: self.test_signal.listener(),
            background_noise: self.background_noise.listener(),
            output_to_speakers: self.output_to_speakers.listener(),
            microphone_permission: self.microphone_permission.listener(),
        }
    }
}

/// UI Control Triggers - For UI components to fire actions
#[derive(Clone)]
pub struct UIControlTriggers {
    pub test_signal: ActionTrigger<TestSignalAction>,
    pub background_noise: ActionTrigger<BackgroundNoiseAction>,
    pub output_to_speakers: ActionTrigger<OutputToSpeakersAction>,
    pub microphone_permission: ActionTrigger<MicrophonePermissionAction>,
}

/// UI Control Listeners - For audio module to respond to actions
pub struct UIControlListeners {
    pub test_signal: ActionListener<TestSignalAction>,
    pub background_noise: ActionListener<BackgroundNoiseAction>,
    pub output_to_speakers: ActionListener<OutputToSpeakersAction>,
    pub microphone_permission: ActionListener<MicrophonePermissionAction>,
}



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
    mut presenter: Option<presentation::Presenter>,
    debug_actions: module_interfaces::debug_actions::DebugActionsInterface,
    ui_triggers: UIControlTriggers,
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
    
    // Create microphone button
    let microphone_button = EguiMicrophoneButton::new(
        ui_triggers.microphone_permission.clone(),
        ui_triggers.output_to_speakers.clone(),
        ui_triggers.test_signal.clone(),
        ui_triggers.background_noise.clone(),
    );
    
    // Create hybrid live data without legacy interface
    let hybrid_live_data = live_data::HybridLiveData::new();
    
    // Create hybrid debug panel
    let mut hybrid_live_data_panel = HybridEguiLiveDataPanel::new(
        hybrid_live_data,
        debug_actions,
        microphone_button,
    );

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
            crate::module_interfaces::engine_to_model::EngineUpdateResult {
                audio_analysis: None,
                audio_errors: Vec::new(),
                permission_state: crate::module_interfaces::engine_to_model::PermissionState::NotRequested,
            }
        };
        
        // Update model layer with engine data and capture result
        let model_data = if let Some(ref mut model) = model {
            model.update(timestamp, engine_data.clone())
        } else {
            // Provide default model data when model is not available
            crate::module_interfaces::model_to_presentation::ModelUpdateResult {
                volume: crate::module_interfaces::model_to_presentation::Volume { peak: -60.0, rms: -60.0 },
                pitch: crate::module_interfaces::model_to_presentation::Pitch::NotDetected,
                accuracy: crate::module_interfaces::model_to_presentation::Accuracy {
                    closest_note: crate::module_interfaces::model_to_presentation::Note::A,
                    accuracy: 1.0,
                },
                tuning_system: crate::module_interfaces::model_to_presentation::TuningSystem::EqualTemperament,
                errors: Vec::new(),
                permission_state: crate::module_interfaces::model_to_presentation::PermissionState::NotRequested,
            }
        };
        
        // Update debug panel data with engine and model results
        hybrid_live_data_panel.update_data(&engine_data, Some(&model_data));
        
        // Update debug panel data with performance metrics
        let performance_metrics = debug::egui::data_types::PerformanceMetrics {
            fps,
            memory_usage: 0.0, // Placeholder
            audio_latency: 0.0, // Placeholder
            cpu_usage: 0.0, // Placeholder
        };
        hybrid_live_data_panel.update_debug_data(
            None, // audio_devices - not updated in main loop
            Some(performance_metrics),
            None, // audioworklet_status - not updated in main loop
            None, // buffer_pool_stats - not updated in main loop
        );
        
        // Update presentation layer with model data
        if let Some(ref mut presenter) = presenter {
            presenter.update(timestamp, model_data);
            presenter.update_viewport(frame_input.viewport);
        }
        
        // Process user actions through three-layer validation and execution
        if let (Some(presenter), Some(model), Some(engine)) = (&mut presenter, &mut model, &mut engine) {
            // Collect user actions from presentation layer
            let user_actions = presenter.get_user_actions();
            
            // Only process if there are actions to handle
            let has_user_actions = !user_actions.microphone_permission_requests.is_empty() ||
                                  !user_actions.tuning_system_changes.is_empty() ||
                                  !user_actions.root_note_adjustments.is_empty();
            
            if has_user_actions {
                dev_log!("Processing {} user actions", 
                    user_actions.microphone_permission_requests.len() + 
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
                let has_model_actions = !processed_actions.actions.microphone_permission_requests.is_empty() ||
                                       !processed_actions.actions.audio_system_configurations.is_empty() ||
                                       !processed_actions.actions.tuning_configurations.is_empty();
                
                if has_model_actions {
                    dev_log!("Actions ready for execution: {} microphone, {} audio system, {} tuning", 
                        processed_actions.actions.microphone_permission_requests.len(),
                        processed_actions.actions.audio_system_configurations.len(),
                        processed_actions.actions.tuning_configurations.len()
                    );
                    
                    // Microphone permission is handled by first click overlay - no action needed here
                    
                    // Execute non-permission actions synchronously
                    match engine.execute_sync_actions(&processed_actions.actions) {
                        Ok(executed_actions) => {
                            let total_sync = executed_actions.audio_system_configurations.len() + 
                                           executed_actions.tuning_configurations.len();
                            if total_sync > 0 {
                                dev_log!("✓ Executed {} synchronous actions", total_sync);
                            }
                        }
                        Err(e) => {
                            dev_log!("✗ Synchronous action execution failed: {}", e);
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
            if let (Some(presenter), Some(_engine)) = (&mut presenter, &mut engine) {
                // Collect debug actions from presentation layer
                let debug_actions = presenter.get_debug_actions();
                
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
                    
                    // TODO: Execute debug actions synchronously or implement proper async handling
                    // The current engine.execute_debug_actions() is async, which cannot be directly
                    // called in the render loop. This requires the same architectural changes as
                    // regular action execution.
                    
                    dev_log!("[DEBUG] Debug actions ready for execution: {} test signal, {} speaker output, {} background noise", 
                        debug_actions.test_signal_configurations.len(),
                        debug_actions.speaker_output_configurations.len(),
                        debug_actions.background_noise_configurations.len()
                    );
                    
                    // Placeholder: Debug actions would be executed here once async handling is resolved
                    // engine.execute_debug_actions(debug_actions).await
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
                hybrid_live_data_panel.render(gui_context);
            }
        );
        
        let mut screen = frame_input.screen();
        screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));
        
        // Render presentation layer
        if let Some(ref mut presenter) = presenter {
            presenter.render(&context, &mut screen);
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
    
    let debug_actions = module_interfaces::debug_actions::DebugActionsInterface::new();
    
    
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
            Some(presenter)
        }
        Err(e) => {
            dev_log!("✗ Presentation layer creation failed: {}", e);
            dev_log!("Application will continue without presentation layer");
            None
        }
    };
    
    
    // Create UI control actions
    let ui_control_actions = UIControlActions::new();
    let _listeners = ui_control_actions.get_listeners();
    let triggers = ui_control_actions.get_triggers();
    
    // Set up debug action listeners through the engine
    if let Some(ref engine_instance) = engine {
        // Set up debug action listeners
        engine_instance.setup_debug_listeners(&debug_actions);
        
        // UI listeners setup can now be enabled - engine layer observable_data dependencies have been removed
    }
    
    // Start three-d application with three-layer architecture
    run_three_d_with_layers(
        engine,
        model,
        presenter,
        debug_actions,
        triggers,
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


