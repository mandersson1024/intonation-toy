use three_d::{egui::{collapsing_header::HeaderResponse, viewport}, AmbientLight, Camera, ClearState, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderTarget, Srgba, Viewport};

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

        let white_material = ColorMaterial {
            color: Srgba::WHITE,
            ..Default::default()
        };
        
        let green_material = ColorMaterial {
            color: Srgba::GREEN,
            ..Default::default()
        };
        
        Self {
            camera: Camera::new_2d(viewport),
            center_line: Gm::new(center_line, white_material.clone()),
            user_pitch_line: Gm::new(user_pitch_line, green_material.clone()),
            light: AmbientLight::new(context, 1.0, Srgba::GREEN),
        }
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.center_line.set_endpoints(PhysicalPoint{x:0.0, y:viewport.height as f32 * 0.5}, PhysicalPoint{x:viewport.width as f32, y:viewport.height as f32 * 0.5});
        self.camera.set_viewport(viewport);
    }
    
    pub fn render(&self, screen: &mut RenderTarget) {
        screen.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));

        screen.render(
            &self.camera,
            (&self.user_pitch_line).into_iter().chain(&self.center_line),
            &[&self.light],
        );
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, normalized_interval: f32) {
        let h: f32 = viewport.height as f32;
        let y: f32 = h * normalized_interval + h * 0.5;
        self.user_pitch_line.set_endpoints(PhysicalPoint{x:0.0, y:y}, PhysicalPoint{x:viewport.width as f32, y:y});
    }
}