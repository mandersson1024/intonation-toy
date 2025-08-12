#![cfg(target_arch = "wasm32")]

use std::rc::Rc;
use std::cell::RefCell;
use crate::platform::traits::*;

pub struct WebImpl;

impl WebImpl {
    pub fn new() -> Self {
        WebImpl
    }
}

impl WebUI for WebImpl {
    fn setup_main_scene_ui(&self) {
        crate::web::main_scene_ui::setup_main_scene_ui();
    }

    fn cleanup_main_scene_ui(&self) {
        crate::web::main_scene_ui::cleanup_main_scene_ui();
    }

    fn setup_event_listeners(&self, presenter: Rc<RefCell<crate::presentation::Presenter>>) {
        crate::web::main_scene_ui::setup_event_listeners(presenter);
    }

    fn sync_ui_with_presenter_state(&self, model_data: &crate::shared_types::ModelUpdateResult) {
        crate::web::main_scene_ui::sync_ui_with_presenter_state(model_data);
    }

    fn get_current_zoom_factor(&self) -> f32 {
        crate::web::main_scene_ui::get_current_zoom_factor()
    }
}

impl WebError for WebImpl {
    fn show_error(&self, error: &crate::shared_types::Error) {
        crate::web::error_message_box::show_error(error);
    }

    fn show_error_with_params(&self, error: &crate::shared_types::Error, params: &[&str]) {
        crate::web::error_message_box::show_error_with_params(error, params);
    }
}

impl WebPerformance for WebImpl {
    fn sample_memory_usage(&self) -> Option<(f64, f64)> {
        crate::web::performance::sample_memory_usage()
    }
}

impl WebStyling for WebImpl {
    fn apply_color_scheme_styles(&self) {
        crate::web::styling::apply_color_scheme_styles();
    }

    fn reapply_current_theme(&self) {
        crate::web::styling::reapply_current_theme();
    }

    fn get_sidebar_width(&self) -> i32 {
        crate::web::styling::SIDEBAR_WIDTH
    }

    fn get_canvas_margin(&self) -> i32 {
        crate::web::styling::CANVAS_MARGIN
    }
}

impl WebPermission for WebImpl {
    fn setup_first_click_handler(&self, permission_granted: Rc<RefCell<bool>>, engine: &mut Option<crate::engine::AudioEngine>) {
        crate::web::first_click_handler::setup_first_click_handler(permission_granted, engine);
    }
}

impl WebFacade for WebImpl {}