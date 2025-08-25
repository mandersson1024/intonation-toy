use three_d::{Window, WindowSettings, FrameOutput, egui};
use std::rc::Rc;
use std::cell::RefCell;

pub mod app_config;
pub mod engine;
pub mod model;
pub mod presentation;
pub mod web;
pub mod common;
#[cfg(debug_assertions)]
pub(crate) mod debug;

use wasm_bindgen::prelude::*;
#[cfg(all(debug_assertions, not(feature = "profiling")))]
use egui_dev_console::ConsoleCommandRegistry;

use engine::platform::{Platform, PlatformValidationResult};

#[cfg(target_arch = "wasm32")]
fn resize_canvas(canvas: &web_sys::HtmlCanvasElement) {
    let window_obj = web_sys::window().unwrap();
    let document = window_obj.document().unwrap();
    
    let sidebar_width = crate::web::styling::SIDEBAR_WIDTH;
    let margin = crate::web::styling::CANVAS_MARGIN;
    
    let available_width = window_obj.inner_width().unwrap().as_f64().unwrap() as i32 - sidebar_width - (margin * 2);
    let available_height = window_obj.inner_height().unwrap().as_f64().unwrap() as i32 - (margin * 2);
    
    let canvas_size = std::cmp::min(available_width, available_height)
        .min(crate::app_config::CANVAS_MAX_SIZE)
        .max(crate::app_config::CANVAS_MIN_SIZE);
    
    let scene_wrapper = document.get_element_by_id("scene-wrapper").unwrap();
    
    scene_wrapper.set_attribute("style", &format!(
        "position: absolute; top: {}px; left: {}px; width: {}px; height: {}px;",
        margin, margin, canvas_size, canvas_size
    )).unwrap();
    
    canvas.style().set_property("width", &format!("{}px", canvas_size)).unwrap();
    canvas.style().set_property("height", &format!("{}px", canvas_size)).unwrap();
}

#[cfg(all(debug_assertions, not(feature = "profiling")))]
use debug::debug_panel::DebugPanel;

/// Run three-d with three-layer architecture (engine → model → presenter)
pub async fn start_render_loop(
    mut engine: Option<engine::AudioEngine>,
    mut model: Option<model::DataModel>,
    presenter: Option<Rc<RefCell<presentation::Presenter>>>,
) {
    dev_log!("Starting three-d with three-layer architecture");
    
    #[cfg(target_arch = "wasm32")]
    let canvas = {
        let window_obj = web_sys::window().unwrap();
        let document = window_obj.document().unwrap();
        
        let canvas = document.get_element_by_id("three-d-canvas").unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        
        let canvas_clone = canvas.clone();
        let resize_callback = Closure::wrap(Box::new(move || {
            resize_canvas(&canvas_clone);
        }) as Box<dyn FnMut()>);
        
        window_obj.add_event_listener_with_callback("resize", resize_callback.as_ref().unchecked_ref()).unwrap();
        resize_callback.forget();
        
        Some(canvas)
    };

    let dpr = web_sys::window().unwrap().device_pixel_ratio();
    let render_size: u32 = if dpr <= 1.0 { app_config::VIEWPORT_RENDER_SIZE } else { app_config::VIEWPORT_RENDER_SIZE_RETINA };

    let window = Window::new(WindowSettings {
        title: app_config::WINDOW_TITLE.to_string(),
        max_size: Some((render_size, render_size)),
        ..Default::default()
    })
    .unwrap();
    
    #[cfg(target_arch = "wasm32")]
    if let Some(ref canvas_element) = canvas {
        resize_canvas(canvas_element);
    }
    
    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);
    
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut command_registry = ConsoleCommandRegistry::default();
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    crate::engine::platform::commands::register_platform_commands(&mut command_registry);

    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut dev_console = egui_dev_console::DevConsole::new(command_registry);
    
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut debug_panel = presenter.as_ref().map(|presenter_ref| DebugPanel::new(
            debug::debug_data::DebugData::new(),
            presenter_ref.clone(),
        ));

    let permission_granted = std::rc::Rc::new(std::cell::RefCell::new(false));
    web::first_click_handler::setup_first_click_handler(permission_granted.clone(), &mut engine);
    
    let mut frame_count = 0u32;
    let mut last_fps_update = 0.0;
    let mut fps = 0.0;
    
    window.render_loop(move |mut frame_input| {
        frame_count += 1;
        let current_time = frame_input.accumulated_time;
        
        if current_time - last_fps_update >= 1000.0 {
            fps = (frame_count as f64) / ((current_time - last_fps_update) / 1000.0);
            frame_count = 0;
            last_fps_update = current_time;
        }
        
        let timestamp = current_time / 1000.0;
        
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
            crate::common::shared_types::EngineUpdateResult {
                audio_analysis: None,
                audio_errors: Vec::new(),
                permission_state: crate::common::shared_types::PermissionState::NotRequested,
            }
        };
        
        if let (Some(presenter), Some(model), Some(engine)) = (&presenter, &mut model, &mut engine) {
            let mut user_action_processing = || {
                let user_actions = presenter.try_borrow_mut()
                    .map(|mut p| p.get_user_actions())
                    .unwrap_or_default();
                
                let has_user_actions = !user_actions.tuning_system_changes.is_empty() ||
                                      !user_actions.tuning_fork_adjustments.is_empty() ||
                                      !user_actions.scale_changes.is_empty() ||
                                      !user_actions.tuning_fork_configurations.is_empty();
                
                if has_user_actions {
                    let processed_actions = model.process_user_actions(user_actions);
                    
                    for error in &processed_actions.validation_errors {
                        dev_log!("Action validation error: {:?}", error);
                    }
                    
                    let has_model_actions = !processed_actions.actions.tuning_system_changes.is_empty() ||
                                           !processed_actions.actions.tuning_fork_note_changes.is_empty() ||
                                           !processed_actions.actions.tuning_fork_configurations.is_empty();
                    
                    if has_model_actions {
                        if let Err(e) = engine.execute_actions(processed_actions.actions) {
                            dev_log!("✗ Action execution failed: {}", e);
                        }
                    }
                }
            };

            #[cfg(feature = "profiling")]
            crate::web::profiling::profiled("user_action_processing", user_action_processing);
            #[cfg(not(feature = "profiling"))]
            user_action_processing();
        }
        
        let model_data = model.as_mut().map(|model| {
            #[cfg(feature = "profiling")]
            {
                crate::web::profiling::profiled("model_update", || {
                    model.update(timestamp, engine_data.clone())
                })
            }
            #[cfg(not(feature = "profiling"))]
            {
                model.update(timestamp, engine_data.clone())
            }
        });
        
        #[cfg(all(debug_assertions, not(feature = "profiling")))]
        if let Some(ref mut panel) = debug_panel {
            panel.update_data(&engine_data, model_data.as_ref());
        }
        
        #[cfg(all(debug_assertions, not(feature = "profiling")))]
        if let Some(ref mut panel) = debug_panel {
            let (memory_usage_mb, memory_usage_percent) = web::performance::sample_memory_usage().unwrap_or((0.0, 0.0));
            
            let performance_metrics = debug::data_types::PerformanceMetrics {
                fps,
                memory_usage_mb,
                memory_usage_percent,
            };
            
            let (audio_devices, audioworklet_status, buffer_pool_stats) = if let Some(ref engine) = engine {
                let devices = engine.get_debug_audio_devices();
                let status = engine.get_debug_audioworklet_status().map(|s| {
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
            
            panel.update_debug_data(
                audio_devices,
                Some(performance_metrics),
                audioworklet_status,
                buffer_pool_stats,
            );
        }
        
        if let (Some(presenter), Some(data)) = (&presenter, &model_data) {
            if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                presenter_ref.process_data(timestamp, data.clone());
                presenter_ref.update_graphics(frame_input.viewport, data);
            }
        }
        
        #[cfg(debug_assertions)]
        if let (Some(presenter), Some(_engine)) = (&presenter, &mut engine) {
            let debug_actions = presenter.try_borrow_mut()
                .map(|mut p| p.get_debug_actions())
                .unwrap_or_else(|_| presentation::DebugLayerActions::new());
            
            if !debug_actions.test_signal_configurations.is_empty() {
                if let Err(e) = _engine.execute_debug_actions_sync(debug_actions) {
                    dev_log!("[DEBUG] ✗ Debug action execution failed: {}", e);
                }
            }
        }
        
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                #[cfg(all(debug_assertions, not(feature = "profiling")))]
                {
                    gui_context.set_visuals(egui::Visuals::dark());
                    
                    dev_console.render(gui_context);
                    if let (Some(panel), Some(data)) = (&mut debug_panel, &model_data) {
                        panel.render(gui_context, data);
                    }
                }
            }
        );
        
        let mut screen = frame_input.screen();
        
        if let (Some(presenter), Some(data)) = (&presenter, &model_data) {
            if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                presenter_ref.render(&context, &mut screen, data);
            }
        }
        
        let _ = gui.render();
        FrameOutput::default()
    });
}

#[wasm_bindgen(start)]
pub async fn start() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    
    log!("Starting Intonation Toy application");
    
    crate::common::theme::initialize_theme(crate::app_config::DEFAULT_THEME);
    crate::web::styling::apply_color_scheme_styles();
    
    match Platform::check_feature_support() {
        PlatformValidationResult::MissingCriticalApis(missing_apis) => {
            let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
            error_log!("✗ CRITICAL: Missing required browser APIs: {}", api_list.join(", "));
            
            #[cfg(target_arch = "wasm32")]
            {
                let missing_apis_str = api_list.join(", ");
                crate::web::error_message_box::show_error_with_params(&crate::common::shared_types::Error::BrowserApiNotSupported, &[&missing_apis_str]);
            }
            return;
        }
        PlatformValidationResult::MobileDevice => {
            error_log!("✗ CRITICAL: Mobile device detected - application not supported on mobile");
            
            #[cfg(target_arch = "wasm32")]
            {
                crate::web::error_message_box::show_error(&crate::common::shared_types::Error::MobileDeviceNotSupported);
            }
            return;
        }
        PlatformValidationResult::AllSupported => {
            log!("✓ Platform validation passed - initializing three-layer architecture");
        }
    }
    
    let engine = engine::AudioEngine::create().await.ok();
    let model = model::DataModel::create().ok();
    let presenter = presentation::Presenter::create().ok().map(|presenter| {
        let presenter_rc = Rc::new(RefCell::new(presenter));
        presenter_rc.borrow_mut().set_self_reference(presenter_rc.clone());
        presenter_rc
    });
    
    start_render_loop(engine, model, presenter).await;
}