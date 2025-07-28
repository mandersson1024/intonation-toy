use three_d::{Context, Viewport, Camera, ColorMaterial, Line,
                AmbientLight, Srgba, ClearState, Gm, RenderTarget, PhysicalPoint};

pub struct MainScene {
    camera: Camera,
    center_line: Gm<Line, ColorMaterial>,
    light: AmbientLight,
}

impl MainScene {
    pub fn new(context: &Context, viewport: Viewport) -> Self {
        let line = Line::new(context, PhysicalPoint{x:0.0, y:0.0}, PhysicalPoint{x:0.0, y:0.0}, 1.0);

        let material = ColorMaterial {
            color: Srgba::WHITE,
            ..Default::default()
        };
        
        Self {
            camera: Camera::new_2d(viewport),
            center_line: Gm::new(line, material),
            light: AmbientLight::new(context, 1.0, Srgba::GREEN),
        }
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.center_line.set_endpoints(PhysicalPoint{x:0.0, y:viewport.height as f32 / 2.0}, PhysicalPoint{x:viewport.width as f32, y:viewport.height as f32 / 2.0});
        self.camera.set_viewport(viewport);
    }
    
    pub fn render(&self, screen: &mut RenderTarget) {
        // Clear the screen with black background
        screen.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));
        
        // Render the horizontal line
        screen.render(
            &self.camera,
            &self.center_line,
            &[&self.light],
        );
    }
}