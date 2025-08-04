use three_d::{Context, ClearState, RenderTarget, Viewport};

#[derive(Debug)]
pub struct StartupScene;

impl StartupScene {
    pub fn new() -> Self {
        Self
    }
    
    pub fn update_viewport(&mut self, _viewport: Viewport) {
        // No-op for startup scene - we don't need to update anything
    }
    
    pub fn render(&self, screen: &mut RenderTarget) {
        screen.clear(ClearState::color_and_depth(0.5, 0.2, 0.0, 1.0, 1.0));
    }
}