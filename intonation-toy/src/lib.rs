use three_d::{Window, WindowSettings, FrameOutput, egui};
use std::rc::Rc;
use std::cell::RefCell;
use crate::common::fps_counter::FpsCounter;

pub mod app_config;
pub mod engine;
pub mod model;
pub mod presentation;
pub mod web;
pub mod common;
#[cfg(debug_assertions)]
pub(crate) mod debug;

#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::JsCast,
    wasm_bindgen::closure::Closure,
    wasm_bindgen::prelude::wasm_bindgen,
    engine::platform::{Platform, PlatformValidationResult},
};

#[cfg(all(debug_assertions))]
use egui_dev_console::ConsoleCommandRegistry;


#[cfg(all(debug_assertions))]
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
    
    #[cfg(all(debug_assertions))]
    let mut dev_console = {
        let mut command_registry = ConsoleCommandRegistry::default();
        crate::engine::platform::commands::register_platform_commands(&mut command_registry);
        egui_dev_console::DevConsole::new(command_registry)
    };
    
    #[cfg(all(debug_assertions))]
    let mut debug_panel = Some(DebugPanel::new(presenter.clone()));
    
    let mut fps_counter = FpsCounter::new(30);
    
    window.render_loop(move |mut frame_input| {
        let fps = fps_counter.update(frame_input.accumulated_time);
        let engine_data = profile!("engine_update", engine.update());
        
        {
            let mut process_user_actions = || {
                let user_actions = match presenter.try_borrow_mut() {
                    Ok(mut p) => p.get_user_actions(),
                    Err(e) => {
                        debug_assert!(false, "Failed to borrow presenter for user actions: {}", e);
                        return;
                    }
                };
                
                if !user_actions.has_actions() {
                    return;
                }
                
                let processed_actions = model.process_user_actions(user_actions);         
                if processed_actions.actions.has_actions() {
                    if let Err(e) = engine.execute_actions(processed_actions.actions) {
                        debug_assert!(false, "✗ Action execution failed: {}", e);
                    }
                }
            };

            profile!("process_user_actions", process_user_actions());
        }
        
        let model_data = Some(profile!("model_update", model.update(engine_data.clone())));
        
        #[cfg(all(debug_assertions))]
        if let Some(ref mut panel) = debug_panel {
            panel.update_data(&engine_data, model_data.as_ref());
        }
        
        #[cfg(all(debug_assertions))]
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
                presenter_ref.process_data(data.clone());
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
                #[cfg(all(debug_assertions))]
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

        let support = Platform::check_feature_support();
        if support != PlatformValidationResult::AllSupported {
            crate::common::error_handling::handle_platform_validation_error(support);
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

    let media_stream = match web::user_media_permission::ask_for_permission().await {
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


