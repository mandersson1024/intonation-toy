use std::rc::Rc;
use std::cell::RefCell;

pub trait WebUI {
    fn setup_main_scene_ui(&self);
    fn cleanup_main_scene_ui(&self);
    fn setup_event_listeners(&self, presenter: Rc<RefCell<crate::presentation::Presenter>>);
    fn sync_ui_with_presenter_state(&self, model_data: &crate::shared_types::ModelUpdateResult);
    fn get_current_zoom_factor(&self) -> f32;
}

pub trait WebError {
    fn show_error(&self, error: &crate::shared_types::Error);
    fn show_error_with_params(&self, error: &crate::shared_types::Error, params: &[&str]);
}

pub trait WebPerformance {
    fn sample_memory_usage(&self) -> Option<(f64, f64)>;
}

pub trait WebStyling {
    fn apply_color_scheme_styles(&self);
    fn reapply_current_theme(&self);
    fn get_sidebar_width(&self) -> i32;
    fn get_canvas_margin(&self) -> i32;
}

pub trait WebPermission {
    fn setup_first_click_handler(&self, permission_granted: Rc<RefCell<bool>>, engine: &mut Option<crate::engine::AudioEngine>);
}

pub trait WebFacade: WebUI + WebError + WebPerformance + WebStyling + WebPermission {}