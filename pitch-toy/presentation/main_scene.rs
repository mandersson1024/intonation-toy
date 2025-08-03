use three_d::{AmbientLight, Camera, ClearState, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderTarget, Srgba, Viewport};

pub fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    // interval of [0.5, 2.0] means [-1, +1] octaves
    let scale_factor = 1.0;
    let y: f32 = viewport_height * (0.5 + interval * scale_factor * 0.5);
    y
}

pub struct TuningLines {
    lines: Vec<Gm<Line, ColorMaterial>>,
    context: Context,
    material: ColorMaterial,
}

impl TuningLines {
    pub fn new(context: &Context, color: Srgba) -> Self {
        let material = ColorMaterial {
            color,
            ..Default::default()
        };
        
        Self {
            lines: Vec::new(),
            context: context.clone(),
            material,
        }
    }

    /// Update the number of tuning lines and their positions
    /// The presenter calls this method with the positions for the active tuning system
    pub fn update_lines(&mut self, viewport: Viewport, line_y_positions: &[f32]) {
        let width = viewport.width as f32;
        let needed_lines = line_y_positions.len();
        
        //crate::common::dev_log!("TUNING_DEBUG: Updating {} tuning lines, had {} before", needed_lines, self.lines.len());
        
        // Resize lines vector if needed
        while self.lines.len() < needed_lines {
            let line = Line::new(
                &self.context,
                PhysicalPoint { x: 0.0, y: 0.0 },
                PhysicalPoint { x: 0.0, y: 0.0 },
                1.0  // Make lines thicker to match center line
            );
            self.lines.push(Gm::new(line, self.material.clone()));
        }
        
        // Remove excess lines if we have too many
        self.lines.truncate(needed_lines);
        
        // Set positions for all lines
        for (i, &y) in line_y_positions.iter().enumerate() {
            self.lines[i].set_endpoints(
                PhysicalPoint { x: 0.0, y },
                PhysicalPoint { x: width, y }
            );
            //crate::common::dev_log!("TUNING_DEBUG: Line {}: y={:.1}, width={:.1}", i, y, width);
        }
    }
    
    pub fn lines(&self) -> impl Iterator<Item = &Gm<Line, ColorMaterial>> {
        self.lines.iter()
    }
}

pub struct MainScene {
    camera: Camera,
    center_line: Gm<Line, ColorMaterial>,
    user_pitch_line: Gm<Line, ColorMaterial>,
    light: AmbientLight,
    pub tuning_lines: TuningLines,
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
        
        let tuning_lines = TuningLines::new(context, Srgba::WHITE);
        
        Self {
            camera: Camera::new_2d(viewport),
            center_line: Gm::new(center_line, white_material.clone()),
            user_pitch_line: Gm::new(user_pitch_line, green_material.clone()),
            light: AmbientLight::new(context, 1.0, Srgba::GREEN),
            tuning_lines,
        }
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        let center_y = viewport.height as f32 * 0.5;
        
        
        // Update center line position
        self.center_line.set_endpoints(PhysicalPoint{x:0.0, y:center_y}, PhysicalPoint{x:viewport.width as f32, y:center_y});
        
        self.camera.set_viewport(viewport);
    }
    
    
    pub fn render(&self, screen: &mut RenderTarget) {
        screen.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));

        // Collect all lines to render: semitone lines, center line, and user pitch line
        let mut renderable_lines: Vec<&Gm<Line, ColorMaterial>> = Vec::new();
        
        renderable_lines.push(&self.user_pitch_line); // first in list is on top
        renderable_lines.push(&self.center_line);

        // Add all semitone lines
        for line in self.tuning_lines.lines() {
            renderable_lines.push(line);
        }
        
        
        screen.render(
            &self.camera,
            renderable_lines.into_iter(),
            &[&self.light],
        );
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32) {
        let y = interval_to_screen_y_position(interval, viewport.height as f32);
        self.user_pitch_line.set_endpoints(PhysicalPoint{x:0.0, y}, PhysicalPoint{x:viewport.width as f32, y});
    }
    
    /// Update tuning lines with positions provided by the presenter
    /// MainScene doesn't know about music theory - it just positions lines where told
    pub fn update_tuning_lines(&mut self, viewport: Viewport, line_positions: &[f32]) {
        self.tuning_lines.update_lines(viewport, line_positions);
    }
}