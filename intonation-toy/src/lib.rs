
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


#[cfg(target_arch = "wasm32")]
pub async fn start_render_loop(
    mut engine: engine::AudioEngine,
    mut model: model::DataModel,
    presenter: std::rc::Rc<std::cell::RefCell<presentation::Presenter>>,
) {
    use crate::common::fps_counter::FpsCounter;
    #[cfg(all(debug_assertions))]
use crate::debug::debug_panel::DebugPanel;

    let dpr = web_sys::window().unwrap().device_pixel_ratio();
    let render_size: u32 = if dpr <= 1.0 { app_config::VIEWPORT_RENDER_SIZE } else { app_config::VIEWPORT_RENDER_SIZE_RETINA };

    let window = three_d::Window::new(three_d::WindowSettings {
        title: app_config::WINDOW_TITLE.to_string(),
        max_size: Some((render_size, render_size)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);
    
    #[cfg(all(debug_assertions))]
    let mut dev_console = {
        use egui_dev_console::ConsoleCommandRegistry;

        let mut command_registry = ConsoleCommandRegistry::default();
        crate::engine::platform::commands::register_platform_commands(&mut command_registry);
        egui_dev_console::DevConsole::new(command_registry)
    };
    
    #[cfg(all(debug_assertions))]
    let mut debug_panel = DebugPanel::new(presenter.clone());
    
    let mut fps_counter = FpsCounter::new(30);
    
    window.render_loop(move |mut frame_input| {
        let fps = fps_counter.update(frame_input.accumulated_time);
        let engine_data = profile!("engine_update", engine.update());
        
        {
            let mut process_user_actions = || {
                let user_actions = if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
                    presenter_ref.get_user_actions()
                } else {
                    debug_assert!(false, "Failed to borrow presenter for user actions");
                    return;
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
        
        let model_data = profile!("model_update", model.update(engine_data.clone()));
        
        #[cfg(all(debug_assertions))]
        debug_panel.update_all_data(
            &engine_data,
            Some(&model_data),
            web::performance::get_performance_metrics(fps),
            engine.get_debug_audioworklet_status(),
            engine.get_debug_buffer_pool_stats(),
        );
        
        if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
            presenter_ref.update(frame_input.viewport, &model_data);
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
                    gui_context.set_visuals(three_d::egui::Visuals::dark());
                    
                    dev_console.render(gui_context);
                    debug_panel.render(gui_context, &model_data);
                }
            }
        );
        
        let mut screen = frame_input.screen();
        
        if let Ok(mut presenter_ref) = presenter.try_borrow_mut() {
            presenter_ref.render(&context, &mut screen, &model_data);
        }
        
        let _ = gui.render();
        three_d::FrameOutput::default()
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

    let engine = match engine::AudioEngine::new(media_stream).await {
        Ok(engine) => engine,
        Err(err) => {
            crate::common::error_log!("Failed to create AudioEngine: {:?}", err);
            return;
        }
    };

    let model = match model::DataModel::new() {
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


