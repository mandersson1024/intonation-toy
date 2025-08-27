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

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use engine::platform::{Platform, PlatformValidationResult};
#[cfg(all(debug_assertions, not(feature = "profiling")))]
use egui_dev_console::ConsoleCommandRegistry;


#[cfg(all(debug_assertions, not(feature = "profiling")))]
use debug::debug_panel::DebugPanel;

#[cfg(target_arch = "wasm32")]
pub async fn start_render_loop(
    mut engine: engine::AudioEngine,
    mut model: model::DataModel,
    presenter: Rc<RefCell<presentation::Presenter>>,
) {
    let dpr = web_sys::window().unwrap().device_pixel_ratio();
    let render_size: u32 = if dpr <= 1.0 { app_config::VIEWPORT_RENDER_SIZE } else { app_config::VIEWPORT_RENDER_SIZE_RETINA };

    let window = Window::new(WindowSettings {
        title: app_config::WINDOW_TITLE.to_string(),
        max_size: Some((render_size, render_size)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);
    
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut command_registry = ConsoleCommandRegistry::default();
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    crate::engine::platform::commands::register_platform_commands(&mut command_registry);

    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut dev_console = egui_dev_console::DevConsole::new(command_registry);
    
    #[cfg(all(debug_assertions, not(feature = "profiling")))]
    let mut debug_panel = Some(DebugPanel::new(
            debug::debug_data::DebugData::new(),
            presenter.clone(),
        ));

    
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
        
        let engine_data = {
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
        };
        
        {
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
        
        let model_data = Some({
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
            
            let (audioworklet_status, buffer_pool_stats) = {
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
                (status, stats)
            };
            
            panel.update_debug_data(
                Some(performance_metrics),
                audioworklet_status,
                buffer_pool_stats,
            );
        }
        
        if let Some(data) = &model_data {
            if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                presenter_ref.process_data(timestamp, data.clone());
                presenter_ref.update_graphics(frame_input.viewport, data);
            }
        }
        
        #[cfg(debug_assertions)]
        {
            let debug_actions = presenter.try_borrow_mut()
                .map(|mut p| p.get_debug_actions())
                .unwrap_or_else(|_| presentation::DebugLayerActions::new());
            
            if !debug_actions.test_signal_configurations.is_empty() {
                if let Err(e) = engine.execute_debug_actions_sync(debug_actions) {
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
        
        if let Some(data) = &model_data {
            if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                presenter_ref.render(&context, &mut screen, data);
            }
        }
        
        let _ = gui.render();
        FrameOutput::default()
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    crate::common::theme::initialize_theme(crate::app_config::DEFAULT_THEME);
    crate::web::styling::apply_theme();

    {
        // Bail out if any required API is missing

        let result = Platform::check_feature_support();
        if result != PlatformValidationResult::AllSupported {
            crate::common::error_handling::handle_platform_validation_error(result);
            return;
        }
    }

    {
        // Canvas resizing
        
        let resize_canvas_callback = Closure::wrap(Box::new(move || {
            web::utils::resize_canvas();
        }) as Box<dyn FnMut()>);
        
        web_sys::window().unwrap().add_event_listener_with_callback("resize", resize_canvas_callback.as_ref().unchecked_ref()).unwrap();
    }

    web::utils::resize_canvas();
    web::utils::show_first_click_overlay();
    web::utils::hide_preloader();

    let media_stream = match ask_for_media_stream_permission().await {
        Ok(stream) => stream,
        Err(_) => {
            crate::web::error_message_box::show_error(&crate::common::shared_types::Error::MicrophonePermissionDenied);
            return;
        }
    };

    web::utils::hide_first_click_overlay();

    let engine = match engine::AudioEngine::create(media_stream).await {
        Ok(engine) => engine,
        Err(err) => {
            crate::common::error_log!("Failed to create AudioEngine: {:?}", err);
            return;
        }
    };

    let model = match model::DataModel::create() {
        Ok(model) => model,
        Err(err) => {
            crate::common::error_log!("Failed to create DataModel: {:?}", err);
            return;
        }
    };

    let presenter = match presentation::Presenter::create() {
        Ok(presenter) => presenter,
        Err(err) => {
            crate::common::error_log!("Failed to create Presenter: {:?}", err);
            return;
        }
    };
    
    start_render_loop(engine, model, presenter).await;
}

#[cfg(target_arch = "wasm32")]
async fn ask_for_media_stream_permission() -> Result<web_sys::MediaStream, String> {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    
    // Wait for user click on the overlay
    let document = web_sys::window().unwrap().document().unwrap();
    let overlay = document.query_selector(".first-click-overlay").unwrap().unwrap();
    
    // Create a promise that resolves with the MediaStream
    let (promise, resolve) = {
        let mut resolve_holder = None;
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            resolve_holder = Some(resolve);
        });
        (promise, resolve_holder.unwrap())
    };
    
    // Set up click handler that calls getUserMedia INSIDE the callback
    let resolve_clone = resolve.clone();
    let click_closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
        // Request media access INSIDE the click callback - critical for security!
        let constraints = web_sys::MediaStreamConstraints::new();
        constraints.set_audio(&true.into());
        constraints.set_video(&false.into());
        
        let navigator = web_sys::window().and_then(|w| w.navigator().media_devices().ok()).unwrap();
        let media_promise = navigator.get_user_media_with_constraints(&constraints).unwrap();
        
        // Resolve our promise with the media promise
        resolve_clone.call1(&wasm_bindgen::JsValue::NULL, &media_promise).unwrap();
    });
    
    overlay.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref()).unwrap();
    
    // Wait for click - this will resolve with the getUserMedia promise
    let media_promise_js = match wasm_bindgen_futures::JsFuture::from(promise).await {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("Microphone access denied or failed: {:?}", e));
        }
    };
    
    // Clean up the event listener
    overlay.remove_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref()).unwrap();
    click_closure.forget();
    
    // Check if it's already a MediaStream or if it's a Promise we need to await
    if media_promise_js.has_type::<web_sys::MediaStream>() {
        // It's already a MediaStream
        Ok(media_promise_js.dyn_into::<web_sys::MediaStream>().unwrap())
    } else {
        // It's a Promise that resolves to a MediaStream
        let media_promise = media_promise_js.dyn_into::<js_sys::Promise>().unwrap();
        let media_stream_js = wasm_bindgen_futures::JsFuture::from(media_promise).await.unwrap();
        Ok(media_stream_js.dyn_into::<web_sys::MediaStream>().unwrap())
    }
}

