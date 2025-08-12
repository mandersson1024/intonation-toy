#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;
use std::cell::RefCell;
use crate::platform::traits::*;

pub struct StubImpl;

impl StubImpl {
    pub fn new() -> Self {
        StubImpl
    }
}

impl WebUI for StubImpl {
    fn setup_main_scene_ui(&self) {
    }

    fn cleanup_main_scene_ui(&self) {
    }

    fn setup_event_listeners(&self, _presenter: Rc<RefCell<crate::presentation::Presenter>>) {
    }

    fn sync_ui_with_presenter_state(&self, _model_data: &crate::shared_types::ModelUpdateResult) {
    }

    fn get_current_zoom_factor(&self) -> f32 {
        1.0
    }
}

impl WebError for StubImpl {
    fn show_error(&self, _error: &crate::shared_types::Error) {
    }

    fn show_error_with_params(&self, _error: &crate::shared_types::Error, _params: &[&str]) {
    }
}

impl WebPerformance for StubImpl {
    fn sample_memory_usage(&self) -> Option<(f64, f64)> {
        None
    }
}

impl WebStyling for StubImpl {
    fn apply_color_scheme_styles(&self) {
    }

    fn reapply_current_theme(&self) {
    }

    fn get_sidebar_width(&self) -> i32 {
        300
    }

    fn get_canvas_margin(&self) -> i32 {
        100
    }
}

impl WebPermission for StubImpl {
    fn setup_first_click_handler(&self, _permission_granted: Rc<RefCell<bool>>, _engine: &mut Option<crate::engine::AudioEngine>) {
    }
}

impl WebFacade for StubImpl {}