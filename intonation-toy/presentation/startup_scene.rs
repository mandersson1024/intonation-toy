use three_d::{ClearState, RenderTarget, Viewport};
use crate::theme::{get_current_color_scheme};
use crate::shared_types::ColorScheme;

#[derive(Debug)]
pub struct StartupScene {
    current_scheme: ColorScheme,
}

impl Default for StartupScene {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupScene {
    pub fn new() -> Self {
        Self {
            current_scheme: get_current_color_scheme(),
        }
    }
    
    pub fn update_viewport(&mut self, _viewport: Viewport) {
        // No-op for startup scene - we don't need to update anything
    }
    
    fn refresh_colors(&mut self) {
        // Colors will be applied during render
    }
    
    pub fn render(&mut self, screen: &mut RenderTarget) {
        let scheme = get_current_color_scheme();
        if scheme != self.current_scheme {
            self.refresh_colors();
            self.current_scheme = scheme.clone();
        }
        
        let bg = scheme.background;
        screen.clear(ClearState::color_and_depth(bg[0], bg[1], bg[2], 1.0, 1.0));
    }
}