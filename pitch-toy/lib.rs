use three_d::{self, Window, WindowSettings, GUI, ClearState, FrameOutput, egui, egui::Color32};
use std::rc::Rc;
use std::cell::RefCell;

// Three-layer architecture modules
pub mod engine;
pub mod model;
pub mod presentation;

// Platform-specific modules
pub mod web;

// Module interfaces
pub mod shared_types;

// Re-export types for test usage
#[cfg(test)]
pub use shared_types::{MidiNote, TuningSystem};
#[cfg(test)]
pub use presentation::{
    ChangeTuningSystem, AdjustRootNote,
    PresentationLayerActions,
};
#[cfg(all(debug_assertions, test))]
pub use presentation::{
    ConfigureTestSignal, ConfigureOutputToSpeakers,
    DebugLayerActions,
};
#[cfg(test)]
pub use model::{ProcessedActions, ModelLayerActions};
#[cfg(test)]
pub use presentation::{PresentationLayerActionsBuilder};
#[cfg(all(debug_assertions, test))]
pub use presentation::{DebugLayerActionsBuilder};

// Supporting modules
pub(crate) mod common;
pub(crate) mod debug;

use common::{dev_log, trace_log, log, error_log};
use wasm_bindgen::prelude::*;
use egui_dev_console::ConsoleCommandRegistry;

use engine::platform::{Platform, PlatformValidationResult};

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
    
    let debug_data = debug::debug_data::DebugData::new();
    
    // Create debug panel
    let mut debug_panel = if let Some(ref presenter_ref) = presenter {
        Some(DebugPanel::new(
            debug_data,
            presenter_ref.clone(),
        ))
    } else {
        None
    };

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
            let result = engine.update(timestamp);
            
            result
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
            let result = model.update(timestamp, engine_data.clone());
            
            result
        } else {
            // Provide default model data when model is not available
            crate::shared_types::ModelUpdateResult {
                volume: crate::shared_types::Volume { peak_amplitude: -60.0, rms_amplitude: -60.0 },
                pitch: crate::shared_types::Pitch::NotDetected,
                accuracy: crate::shared_types::IntonationData {
                    closest_midi_note: 69,
                    cents_offset: 0.0,
                },
                tuning_system: crate::shared_types::TuningSystem::EqualTemperament,
                errors: Vec::new(),
                permission_state: crate::shared_types::PermissionState::NotRequested,
                // New flattened fields with default values
                closest_midi_note: 69,
                cents_offset: 0.0,
                interval_semitones: 0,
                root_note: 53,
            }
        };
        
        // Update debug panel data with engine and model results
        if let Some(ref mut panel) = debug_panel {
            panel.update_data(&engine_data, Some(&model_data));
        }
        
        // Update debug panel data with performance metrics
        let (memory_usage_mb, memory_usage_percent) = web::performance::sample_memory_usage().unwrap_or((0.0, 0.0));
        let audio_latency = if let Some(ref engine) = engine {
            engine.get_pitch_analyzer_metrics()
                .map(|metrics| metrics.average_latency_ms)
                .unwrap_or(0.0)
        } else {
            0.0
        };
        
        let performance_metrics = debug::debug_panel::data_types::PerformanceMetrics {
            fps,
            memory_usage_mb,
            memory_usage_percent,
            audio_latency,
        };
        if let Some(ref mut panel) = debug_panel {
            // Collect real debug data from the engine
            #[cfg(debug_assertions)]
            let (audio_devices, audioworklet_status, buffer_pool_stats) = if let Some(ref engine) = engine {
                let devices = engine.get_debug_audio_devices();
                let status = engine.get_debug_audioworklet_status().map(|s| {
                    // Convert from engine AudioWorkletStatus to debug AudioWorkletStatus
                    debug::debug_panel::data_types::AudioWorkletStatus {
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
            
            #[cfg(not(debug_assertions))]
            let (audio_devices, audioworklet_status, buffer_pool_stats) = (None, None, None);
            
            // Update debug-specific data
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
                presenter_ref.process_data(timestamp, model_data);
                presenter_ref.update_graphics(frame_input.viewport);
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
                trace_log!("Processing {} user actions", 
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
                    trace_log!("Actions ready for execution: {} audio system, {} tuning", 
                        processed_actions.actions.audio_system_configurations.len(),
                        processed_actions.actions.tuning_configurations.len()
                    );
                    
                    // Execute actions synchronously
                    let total_sync = processed_actions.actions.audio_system_configurations.len() + 
                                   processed_actions.actions.tuning_configurations.len();
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
                                       !debug_actions.root_note_audio_configurations.is_empty();
                
                if has_debug_actions {
                    trace_log!("[DEBUG] Processing {} debug actions", 
                        debug_actions.test_signal_configurations.len() + 
                        debug_actions.speaker_output_configurations.len() + 
                        debug_actions.root_note_audio_configurations.len()
                    );
                    
                    // Execute debug actions synchronously
                    match _engine.execute_debug_actions_sync(debug_actions) {
                        Ok(executed_debug_actions) => {
                            let total_debug = executed_debug_actions.test_signal_executions.len() + 
                                            executed_debug_actions.speaker_output_executions.len();
                            if total_debug > 0 {
                                trace_log!("[DEBUG] ✓ Executed {} debug actions", total_debug);
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
                        trace_log!("[DEBUG] Skipping debug action processing - missing layers: {}", missing);
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
                if let Some(ref mut panel) = debug_panel {
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
    
    log!("Starting Pitch Toy application");
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    dev_log!("{}", Platform::get_platform_info());
    
    // Validate critical platform APIs before proceeding
    dev_log!("Checking platform feature support...");
    if let PlatformValidationResult::MissingCriticalApis(missing_apis) = Platform::check_feature_support() {
        let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
        error_log!("✗ CRITICAL: Missing required browser APIs: {}", api_list.join(", "));
        error_log!("✗ Application cannot start. Please upgrade your browser or use a supported browser:");
        // TODO: Add error screen rendering in future story when UI requirements are defined
        return;
    }

    log!("✓ Platform validation passed - initializing three-layer architecture");
    
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
    start_render_loop(
        engine,
        model,
        presenter,
    ).await;
}