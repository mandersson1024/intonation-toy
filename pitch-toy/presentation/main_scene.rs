use three_d::{AmbientLight, Camera, ClearState, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderTarget, Srgba, Viewport};
use std::collections::HashMap;
use crate::shared_types::TuningSystem;

fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    // interval of [0.5, 2.0] means [-1, +1] octaves
    let scale_factor = 0.8;
    let y: f32 = viewport_height * (0.5 + interval * scale_factor * 0.5);
    y
}

pub struct SemitoneLines {
    lines: Vec<Gm<Line, ColorMaterial>>,
    material_color: Srgba,
}

impl SemitoneLines {
    pub fn new(context: &Context, color: Srgba) -> Self {
        let material = ColorMaterial {
            color,
            ..Default::default()
        };
        
        let mut lines = Vec::with_capacity(24);
        
        for _ in 0..24 {
            let line = Line::new(
                context,
                PhysicalPoint { x: 0.0, y: 0.0 },
                PhysicalPoint { x: 0.0, y: 0.0 },
                1.0
            );
            lines.push(Gm::new(line, material.clone()));
        }
        
        Self {
            lines,
            material_color: color,
        }
    }

    pub fn set_line_positions(&mut self, viewport: Viewport, center_y: f32) {
        let width = viewport.width as f32;
        let height = viewport.height as f32;
        let scale_factor = height * 0.4 / 2.0;
        
        // Center frequency is 1.0 Hz
        let center_freq = 1.0;
        
        // Lines 0-11: semitones +1 to +12 above center
        for i in 0..12 {
            let semitone = (i + 1) as f32;
            let frequency = center_freq * 2.0_f32.powf(semitone / 12.0);
            let y = interval_to_screen_y_position(frequency.log2(), viewport.height as f32);
            
            self.lines[i].set_endpoints(
                PhysicalPoint { x: 0.0, y },
                PhysicalPoint { x: width, y }
            );
        }
        
        // Lines 12-23: semitones -1 to -12 below center
        for i in 0..12 {
            let semitone = -((i + 1) as f32);
            let frequency = center_freq * 2.0_f32.powf(semitone / 12.0);
            let y = interval_to_screen_y_position(frequency.log2(), viewport.height as f32);
            
            self.lines[i + 12].set_endpoints(
                PhysicalPoint { x: 0.0, y },
                PhysicalPoint { x: width, y }
            );
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
    tuning_lines: HashMap<TuningSystem, SemitoneLines>,
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
        
        // Create HashMap with SemitoneLines for each TuningSystem
        let mut tuning_lines = HashMap::new();
        
        // Equal Temperament with white color
        tuning_lines.insert(
            TuningSystem::EqualTemperament, 
            SemitoneLines::new(context, Srgba::WHITE)
        );
        
        // Just Intonation with light blue color
        tuning_lines.insert(
            TuningSystem::JustIntonation,
            SemitoneLines::new(context, Srgba::new(128, 179, 255, 255))
        );
        
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
        
        // Update all semitone lines for each tuning system
        for (_, semitone_lines) in self.tuning_lines.iter_mut() {
            semitone_lines.set_line_positions(viewport, center_y);
        }
        
        self.camera.set_viewport(viewport);
    }
    
    pub fn render(&self, screen: &mut RenderTarget) {
        screen.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));

        // Collect all lines to render: semitone lines, center line, and user pitch line
        let mut renderable_lines: Vec<&Gm<Line, ColorMaterial>> = Vec::new();
        
        renderable_lines.push(&self.user_pitch_line); // first in list is on top
        renderable_lines.push(&self.center_line);

        // Add all semitone lines from all tuning systems
        for (_, semitone_lines) in &self.tuning_lines {
            for line in semitone_lines.lines() {
                renderable_lines.push(line);
            }
        }
        
        
        screen.render(
            &self.camera,
            renderable_lines.into_iter(),
            &[&self.light],
        );
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32) {
        let y = interval_to_screen_y_position(interval, viewport.height as f32);
        self.user_pitch_line.set_endpoints(PhysicalPoint{x:0.0, y:y}, PhysicalPoint{x:viewport.width as f32, y:y});
    }
}