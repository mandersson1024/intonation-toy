use three_d::{Context, Viewport, Camera, ColorMaterial, Line,
                AmbientLight, Srgba, ClearState, Gm, RenderTarget, PhysicalPoint};

pub struct MainScene {
    camera: Camera,
    center_line: Gm<Line, ColorMaterial>,
    user_pitch_line: Gm<Line, ColorMaterial>,
    light: AmbientLight,
}

impl MainScene {
    pub fn new(context: &Context, viewport: Viewport) -> Self {
        let center_line = Line::new(context, PhysicalPoint{x:0.0, y:0.0}, PhysicalPoint{x:0.0, y:0.0}, 2.0);
        let user_pitch_line = Line::new(context, PhysicalPoint{x:0.0, y:0.0}, PhysicalPoint{x:0.0, y:0.0}, 2.0);

        let material = ColorMaterial {
            color: Srgba::WHITE,
            ..Default::default()
        };
        
        Self {
            camera: Camera::new_2d(viewport),
            center_line: Gm::new(center_line, material.clone()),
            user_pitch_line: Gm::new(user_pitch_line, material.clone()),
            light: AmbientLight::new(context, 1.0, Srgba::GREEN),
        }
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.center_line.set_endpoints(PhysicalPoint{x:0.0, y:viewport.height as f32 * 0.5}, PhysicalPoint{x:viewport.width as f32, y:viewport.height as f32 * 0.5});
        self.user_pitch_line.set_endpoints(PhysicalPoint{x:0.0, y:viewport.height as f32 * 0.4}, PhysicalPoint{x:viewport.width as f32, y:viewport.height as f32 * 0.45});
        self.camera.set_viewport(viewport);
    }
    
    pub fn render(&self, screen: &mut RenderTarget) {
        screen.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));

        screen.render(
            &self.camera,
            (&self.center_line).into_iter().chain(&self.user_pitch_line),
            &[&self.light],
        );
    }
}