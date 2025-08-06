use three_d::{AmbientLight, Camera, ClearState, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderTarget, Srgba, Viewport};
use crate::shared_types::MidiNote;

pub fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    // interval of [0.5, 2.0] means [-1, +1] octaves
    let scale_factor = 1.0;
    let y: f32 = viewport_height * (0.5 + interval * scale_factor * 0.5);
    y
}

pub struct TuningLines {
    lines: Vec<Gm<Line, ColorMaterial>>,
    midi_notes: Vec<MidiNote>,
    y_positions: Vec<f32>,
    thicknesses: Vec<f32>,
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
            midi_notes: Vec::new(),
            y_positions: Vec::new(),
            thicknesses: Vec::new(),
            context: context.clone(),
            material,
        }
    }

    /// Update the number of tuning lines, their positions, MIDI note numbers, and thickness
    /// The presenter calls this method with position, MIDI note, and thickness data for the active tuning system
    pub fn update_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote, f32)]) {
        let width = viewport.width as f32;
        let needed_lines = line_data.len();
        
        //crate::common::dev_log!("TUNING_DEBUG: Updating {} tuning lines, had {} before", needed_lines, self.lines.len());
        
        // Resize lines vector if needed (thickness will be set later)
        while self.lines.len() < needed_lines {
            let line = Line::new(
                &self.context,
                PhysicalPoint { x: 0.0, y: 0.0 },
                PhysicalPoint { x: 0.0, y: 0.0 },
                1.0  // Default thickness, will be updated
            );
            self.lines.push(Gm::new(line, self.material.clone()));
        }
        
        // Remove excess lines, midi notes, y_positions, and thicknesses if we have too many
        self.lines.truncate(needed_lines);
        self.midi_notes.truncate(needed_lines);
        self.y_positions.truncate(needed_lines);
        self.thicknesses.truncate(needed_lines);
        
        // Resize midi_notes, y_positions, and thicknesses vectors if needed
        while self.midi_notes.len() < needed_lines {
            self.midi_notes.push(0); // Temporary value, will be set below
        }
        while self.y_positions.len() < needed_lines {
            self.y_positions.push(0.0); // Temporary value, will be set below
        }
        while self.thicknesses.len() < needed_lines {
            self.thicknesses.push(1.0); // Temporary value, will be set below
        }
        
        // Set positions, MIDI notes, and thickness for all lines
        for (i, &(y, midi_note, thickness)) in line_data.iter().enumerate() {
            // If thickness changed, we need to recreate the line
            if i < self.thicknesses.len() && self.thicknesses[i] != thickness {
                let line = Line::new(
                    &self.context,
                    PhysicalPoint { x: 0.0, y },
                    PhysicalPoint { x: width, y },
                    thickness
                );
                self.lines[i] = Gm::new(line, self.material.clone());
            } else {
                // Just update endpoints if thickness hasn't changed
                self.lines[i].set_endpoints(
                    PhysicalPoint { x: 0.0, y },
                    PhysicalPoint { x: width, y }
                );
            }
            self.midi_notes[i] = midi_note;
            self.y_positions[i] = y;
            self.thicknesses[i] = thickness;
            //crate::common::dev_log!("TUNING_DEBUG: Line {}: y={:.1}, midi_note={}, thickness={:.1}, width={:.1}", i, y, midi_note, thickness, width);
        }
    }
    
    /// Backward compatibility method that accepts old format without thickness
    /// Calls the new update_lines method with default thickness of 1.0
    pub fn update_lines_legacy(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote)]) {
        // Convert old format to new format with default thickness
        let with_thickness: Vec<(f32, MidiNote, f32)> = line_data
            .iter()
            .map(|&(y, midi_note)| (y, midi_note, 1.0))
            .collect();
        self.update_lines(viewport, &with_thickness);
    }
    
    pub fn lines(&self) -> impl Iterator<Item = &Gm<Line, ColorMaterial>> {
        self.lines.iter()
    }
    
    /// Returns an iterator over the MIDI notes corresponding to each tuning line
    pub fn midi_notes(&self) -> impl Iterator<Item = MidiNote> + '_ {
        self.midi_notes.iter().copied()
    }
    
    /// Render note labels above each tuning line
    pub fn render_note_labels(&self, text_renderer: &mut TextRenderer) {
        for (i, &midi_note) in self.midi_notes.iter().enumerate() {
            let y_position = self.y_positions[i];
            
            // Convert MIDI note to name
            let note_name = crate::shared_types::midi_note_to_name(midi_note);
            
            // Position text slightly above the line (20 pixels up)
            let text_y = y_position - 20.0;
            let text_x = 10.0; // Small offset from left edge
            
            // Queue the text for rendering (white color, small size)
            text_renderer.queue_text(&note_name, text_x, text_y, 8.0, [1.0, 1.0, 1.0, 1.0]);
        }
    }
}

pub struct TextRenderer {
    text_builder: three_d_text_builder::TextBuilder,
    queued_texts: Vec<QueuedText>,
}

#[derive(Debug, Clone)]
struct QueuedText {
    text: String,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 4],
}

impl TextRenderer {
    pub fn new(_context: &Context) -> Result<Self, String> {
        // Use the actual Roboto Regular font file
        let roboto_font = include_bytes!("../static/fonts/Roboto-Regular.ttf");
        
        let text_builder = three_d_text_builder::TextBuilder::new(
            roboto_font,
            three_d_text_builder::TextBuilderSettings::default()
        ).map_err(|e| format!("Failed to create TextBuilder with Roboto font: {:?}", e))?;
            
        Ok(Self {
            text_builder,
            queued_texts: Vec::new(),
        })
    }
    
    /// Queue text for rendering at the specified position
    pub fn queue_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.queued_texts.push(QueuedText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }
    
    /// Clear all queued text (called each frame)
    pub fn clear_queue(&mut self) {
        self.queued_texts.clear();
    }
    
    /// Create text models using the actual Roboto font
    pub fn create_text_models(&mut self, context: &Context, viewport: Viewport) -> Vec<three_d::Gm<three_d_text_builder::TextMesh, three_d_text_builder::TextMaterial>> {
        let mut text_refs = Vec::new();
        
        // Set viewport for proper text positioning
        self.text_builder.set_viewport(viewport);
        
        // Create TextRef objects for each queued text
        for queued_text in &self.queued_texts {
            let text_ref = three_d_text_builder::TextRef {
                text: &queued_text.text,
                size: queued_text.size,
                color: three_d::Srgba::new(
                    (queued_text.color[0] * 255.0) as u8,
                    (queued_text.color[1] * 255.0) as u8,
                    (queued_text.color[2] * 255.0) as u8,
                    (queued_text.color[3] * 255.0) as u8,
                ),
                position: three_d_text_builder::TextPosition::Pixels(three_d::vec2(queued_text.x, queued_text.y)),
                ..Default::default()
            };
            text_refs.push(text_ref);
        }
        
        // Build text models using the proper API
        if !text_refs.is_empty() {
            self.text_builder.build(context, &text_refs).collect()
        } else {
            Vec::new()
        }
    }
}

pub struct MainScene {
    camera: Camera,
    user_pitch_line: Gm<Line, ColorMaterial>,
    light: AmbientLight,
    pub tuning_lines: TuningLines,
    text_renderer: TextRenderer,
    context: Context,
}

impl MainScene {
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
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
        let text_renderer = TextRenderer::new(context)?;
        
        Ok(Self {
            camera: Camera::new_2d(viewport),
            user_pitch_line: Gm::new(user_pitch_line, green_material.clone()),
            light: AmbientLight::new(context, 1.0, Srgba::GREEN),
            tuning_lines,
            text_renderer,
            context: context.clone(),
        })
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }
    
    
    pub fn render(&mut self, screen: &mut RenderTarget) {
        screen.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));

        // Collect all lines to render: tuning lines and user pitch line
        let mut renderable_lines: Vec<&Gm<Line, ColorMaterial>> = Vec::new();
        
        renderable_lines.push(&self.user_pitch_line); // first in list is on top

        // Add all tuning lines
        for line in self.tuning_lines.lines() {
            renderable_lines.push(line);
        }
        
        // Render lines
        screen.render(
            &self.camera,
            renderable_lines.into_iter(),
            &[&self.light],
        );
        
        // Clear previous frame's text queue
        self.text_renderer.clear_queue();
        
        // Render note labels above tuning lines
        self.tuning_lines.render_note_labels(&mut self.text_renderer);
        
        // Render text models using actual Roboto font  
        let viewport = self.camera.viewport();
        let text_models = self.text_renderer.create_text_models(&self.context, viewport);
        if !text_models.is_empty() {
            screen.render(
                &self.camera,
                &text_models,
                &[&self.light],
            );
        }
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32) {
        let y = interval_to_screen_y_position(interval, viewport.height as f32);
        self.user_pitch_line.set_endpoints(PhysicalPoint{x:0.0, y}, PhysicalPoint{x:viewport.width as f32, y});
    }
    
    /// Update tuning lines with position, MIDI note, and thickness data provided by the presenter
    /// MainScene doesn't know about music theory - it just positions lines where told
    pub fn update_tuning_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote, f32)]) {
        // Use the new thickness-aware method
        self.tuning_lines.update_lines(viewport, line_data);
    }
    
}