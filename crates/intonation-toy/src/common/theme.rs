use crate::common::shared_types::{Theme, ColorScheme};
use std::sync::{Mutex, OnceLock};

static CURRENT_THEME: OnceLock<Mutex<Theme>> = OnceLock::new();

pub fn initialize_theme(theme: Theme) {
    CURRENT_THEME
        .set(Mutex::new(theme))
        .expect("Theme already initialized");
}

pub fn get_current_theme() -> Theme {
    *CURRENT_THEME
        .get()
        .expect("Theme not initialized")
        .lock()
        .unwrap()
}

pub fn get_current_color_scheme() -> ColorScheme {
    get_current_theme().color_scheme()
}

pub fn set_current_theme(theme: Theme) {
    if let Some(theme_mutex) = CURRENT_THEME.get() {
        *theme_mutex.lock().unwrap() = theme;
    }
}

pub fn rgb_to_srgba_with_alpha(rgb: [f32; 3], alpha: f32) -> three_d::Srgba {
    three_d::Srgba::new(
        (rgb[0] * 255.0) as u8,
        (rgb[1] * 255.0) as u8,
        (rgb[2] * 255.0) as u8,
        (alpha.clamp(0.0, 1.0) * 255.0) as u8,
    )
}