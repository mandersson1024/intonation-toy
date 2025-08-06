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
            context: context.clone(),
            material,
        }
    }

    /// Update the number of tuning lines, their positions, and MIDI note numbers
    /// The presenter calls this method with position and MIDI note data for the active tuning system
    pub fn update_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote)]) {
        let width = viewport.width as f32;
        let needed_lines = line_data.len();
        
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
        
        // Remove excess lines, midi notes, and y_positions if we have too many
        self.lines.truncate(needed_lines);
        self.midi_notes.truncate(needed_lines);
        self.y_positions.truncate(needed_lines);
        
        // Resize midi_notes and y_positions vectors if needed
        while self.midi_notes.len() < needed_lines {
            self.midi_notes.push(0); // Temporary value, will be set below
        }
        while self.y_positions.len() < needed_lines {
            self.y_positions.push(0.0); // Temporary value, will be set below
        }
        
        // Set positions and MIDI notes for all lines
        for (i, &(y, midi_note)) in line_data.iter().enumerate() {
            self.lines[i].set_endpoints(
                PhysicalPoint { x: 0.0, y },
                PhysicalPoint { x: width, y }
            );
            self.midi_notes[i] = midi_note;
            self.y_positions[i] = y;
            //crate::common::dev_log!("TUNING_DEBUG: Line {}: y={:.1}, midi_note={}, width={:.1}", i, y, midi_note, width);
        }
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
            
            // Queue the text for rendering (white color, 16px size)
            text_renderer.queue_text(&note_name, text_x, text_y, 16.0, [1.0, 1.0, 1.0, 1.0]);
        }
    }
}

pub struct TextRenderer {
    // For now, just a placeholder that doesn't actually render text
    // Real implementation would require resolving three-d version compatibility
    _queued_texts: Vec<QueuedText>,
    context: Context,
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
    pub fn new(context: &Context) -> Result<Self, String> {
        Ok(Self {
            _queued_texts: Vec::new(),
            context: context.clone(),
        })
    }
    
    /// Queue text for rendering at the specified position
    pub fn queue_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self._queued_texts.push(QueuedText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
        // For now, just store the text data - actual rendering would be implemented later
        // Real implementation would create text meshes and add them to a render queue
    }
    
    /// Clear all queued text (called each frame)
    pub fn clear_queue(&mut self) {
        self._queued_texts.clear();
    }
    
    /// Create simple letter shapes using multiple lines
    /// Returns an iterator of text meshes for rendering
    pub fn render_meshes(&self) -> impl Iterator<Item = Gm<Line, ColorMaterial>> {
        let mut text_meshes = Vec::new();
        
        for queued_text in &self._queued_texts {
            let char_width = queued_text.size * 0.8;
            let char_height = queued_text.size;
            let mut x_offset = 0.0;
            
            let material = ColorMaterial {
                color: Srgba::new(
                    (queued_text.color[0] * 255.0) as u8,
                    (queued_text.color[1] * 255.0) as u8,
                    (queued_text.color[2] * 255.0) as u8,
                    (queued_text.color[3] * 255.0) as u8,
                ),
                ..Default::default()
            };
            
            for ch in queued_text.text.chars() {
                let x = queued_text.x + x_offset;
                let y = queued_text.y;
                
                // Create simple letter shapes using line segments
                let letter_lines = self.create_letter_lines(ch, x, y, char_width, char_height);
                
                for (start, end) in letter_lines {
                    let line = Line::new(
                        &self.context,
                        PhysicalPoint { x: start.0, y: start.1 },
                        PhysicalPoint { x: end.0, y: end.1 },
                        2.0
                    );
                    text_meshes.push(Gm::new(line, material.clone()));
                }
                
                x_offset += char_width;
            }
        }
        
        text_meshes.into_iter()
    }
    
    /// Create line segments for simple letter shapes
    fn create_letter_lines(&self, ch: char, x: f32, y: f32, width: f32, height: f32) -> Vec<((f32, f32), (f32, f32))> {
        let mut lines = Vec::new();
        let half_width = width * 0.4;
        let half_height = height * 0.4;
        
        match ch {
            'A' | 'a' => {
                // Left diagonal
                lines.push(((x, y + half_height), (x + half_width, y - half_height)));
                // Right diagonal  
                lines.push(((x + half_width, y - half_height), (x + width, y + half_height)));
                // Cross bar
                lines.push(((x + width * 0.25, y), (x + width * 0.75, y)));
            },
            'B' => {
                // Vertical line
                lines.push(((x, y - half_height), (x, y + half_height)));
                // Top horizontal
                lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                // Middle horizontal
                lines.push(((x, y), (x + half_width, y)));
                // Bottom horizontal
                lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                // Top right vertical
                lines.push(((x + half_width, y - half_height), (x + half_width, y)));
                // Bottom right vertical
                lines.push(((x + half_width, y), (x + half_width, y + half_height)));
            },
            'C' | 'c' => {
                // Left vertical
                lines.push(((x, y - half_height), (x, y + half_height)));
                // Top horizontal
                lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                // Bottom horizontal
                lines.push(((x, y + half_height), (x + half_width, y + half_height)));
            },
            'D' | 'd' => {
                // Left vertical
                lines.push(((x, y - half_height), (x, y + half_height)));
                // Top horizontal
                lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                // Bottom horizontal
                lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                // Right vertical
                lines.push(((x + half_width, y - half_height), (x + half_width, y + half_height)));
            },
            'E' | 'e' => {
                // Left vertical
                lines.push(((x, y - half_height), (x, y + half_height)));
                // Top horizontal
                lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                // Middle horizontal
                lines.push(((x, y), (x + half_width * 0.7, y)));
                // Bottom horizontal
                lines.push(((x, y + half_height), (x + half_width, y + half_height)));
            },
            'F' | 'f' => {
                // Left vertical
                lines.push(((x, y - half_height), (x, y + half_height)));
                // Top horizontal
                lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                // Middle horizontal
                lines.push(((x, y), (x + half_width * 0.7, y)));
            },
            'G' | 'g' => {
                // Left vertical
                lines.push(((x, y - half_height), (x, y + half_height)));
                // Top horizontal
                lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                // Bottom horizontal
                lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                // Right middle
                lines.push(((x + half_width * 0.5, y), (x + half_width, y)));
                // Right bottom
                lines.push(((x + half_width, y), (x + half_width, y + half_height)));
            },
            '0'..='9' => {
                let digit = ch as u8 - b'0';
                match digit {
                    0 => {
                        // Oval shape
                        lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                        lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                        lines.push(((x, y - half_height), (x, y + half_height)));
                        lines.push(((x + half_width, y - half_height), (x + half_width, y + half_height)));
                    },
                    1 => {
                        // Vertical line
                        lines.push(((x + half_width * 0.5, y - half_height), (x + half_width * 0.5, y + half_height)));
                    },
                    2 => {
                        // Top horizontal
                        lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                        // Middle horizontal
                        lines.push(((x, y), (x + half_width, y)));
                        // Bottom horizontal
                        lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                        // Right top vertical
                        lines.push(((x + half_width, y - half_height), (x + half_width, y)));
                        // Left bottom vertical
                        lines.push(((x, y), (x, y + half_height)));
                    },
                    3 => {
                        // Top horizontal
                        lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                        // Middle horizontal
                        lines.push(((x, y), (x + half_width, y)));
                        // Bottom horizontal
                        lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                        // Right vertical
                        lines.push(((x + half_width, y - half_height), (x + half_width, y + half_height)));
                    },
                    4 => {
                        // Left vertical top
                        lines.push(((x, y - half_height), (x, y)));
                        // Right vertical
                        lines.push(((x + half_width, y - half_height), (x + half_width, y + half_height)));
                        // Middle horizontal
                        lines.push(((x, y), (x + half_width, y)));
                    },
                    5 => {
                        // Top horizontal
                        lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                        // Middle horizontal
                        lines.push(((x, y), (x + half_width, y)));
                        // Bottom horizontal
                        lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                        // Left top vertical
                        lines.push(((x, y - half_height), (x, y)));
                        // Right bottom vertical
                        lines.push(((x + half_width, y), (x + half_width, y + half_height)));
                    },
                    6 => {
                        // Left vertical
                        lines.push(((x, y - half_height), (x, y + half_height)));
                        // Top horizontal
                        lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                        // Middle horizontal
                        lines.push(((x, y), (x + half_width, y)));
                        // Bottom horizontal
                        lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                        // Right bottom vertical
                        lines.push(((x + half_width, y), (x + half_width, y + half_height)));
                    },
                    7 => {
                        // Top horizontal
                        lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                        // Diagonal
                        lines.push(((x + half_width, y - half_height), (x, y + half_height)));
                    },
                    8 => {
                        // Rectangle outline
                        lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                        lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                        lines.push(((x, y - half_height), (x, y + half_height)));
                        lines.push(((x + half_width, y - half_height), (x + half_width, y + half_height)));
                        // Middle horizontal
                        lines.push(((x, y), (x + half_width, y)));
                    },
                    9 => {
                        // Top rectangle
                        lines.push(((x, y - half_height), (x + half_width, y - half_height)));
                        lines.push(((x, y - half_height), (x, y)));
                        lines.push(((x + half_width, y - half_height), (x + half_width, y + half_height)));
                        lines.push(((x, y), (x + half_width, y)));
                        // Bottom horizontal
                        lines.push(((x, y + half_height), (x + half_width, y + half_height)));
                    },
                    _ => {}
                }
            },
            '#' => {
                // Sharp symbol - two vertical lines and two horizontal lines
                lines.push(((x + width * 0.25, y - half_height), (x + width * 0.25, y + half_height)));
                lines.push(((x + width * 0.75, y - half_height), (x + width * 0.75, y + half_height)));
                lines.push(((x, y - half_height * 0.5), (x + width, y - half_height * 0.5)));
                lines.push(((x, y + half_height * 0.5), (x + width, y + half_height * 0.5)));
            },
            'b' => {
                // Flat symbol - vertical line and curved part
                lines.push(((x, y - half_height), (x, y + half_height)));
                lines.push(((x, y - half_height * 0.5), (x + half_width * 0.7, y)));
                lines.push(((x + half_width * 0.7, y), (x, y + half_height * 0.5)));
            },
            _ => {
                // Default: just a small horizontal line for unknown characters
                lines.push(((x, y), (x + half_width, y)));
            }
        }
        
        lines
    }
}

pub struct MainScene {
    camera: Camera,
    center_line: Gm<Line, ColorMaterial>,
    user_pitch_line: Gm<Line, ColorMaterial>,
    light: AmbientLight,
    pub tuning_lines: TuningLines,
    text_renderer: TextRenderer,
}

impl MainScene {
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
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
        let text_renderer = TextRenderer::new(context)?;
        
        Ok(Self {
            camera: Camera::new_2d(viewport),
            center_line: Gm::new(center_line, white_material.clone()),
            user_pitch_line: Gm::new(user_pitch_line, green_material.clone()),
            light: AmbientLight::new(context, 1.0, Srgba::GREEN),
            tuning_lines,
            text_renderer,
        })
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        let center_y = viewport.height as f32 * 0.5;
        
        
        // Update center line position
        self.center_line.set_endpoints(PhysicalPoint{x:0.0, y:center_y}, PhysicalPoint{x:viewport.width as f32, y:center_y});
        
        self.camera.set_viewport(viewport);
    }
    
    
    pub fn render(&mut self, screen: &mut RenderTarget) {
        screen.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));

        // Collect all lines to render: semitone lines, center line, and user pitch line
        let mut renderable_lines: Vec<&Gm<Line, ColorMaterial>> = Vec::new();
        
        renderable_lines.push(&self.user_pitch_line); // first in list is on top
        renderable_lines.push(&self.center_line);

        // Add all semitone lines
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
        
        // Render text meshes as a second pass
        let text_meshes: Vec<_> = self.text_renderer.render_meshes().collect();
        screen.render(
            &self.camera,
            text_meshes.iter(),
            &[&self.light],
        );
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32) {
        let y = interval_to_screen_y_position(interval, viewport.height as f32);
        self.user_pitch_line.set_endpoints(PhysicalPoint{x:0.0, y}, PhysicalPoint{x:viewport.width as f32, y});
    }
    
    /// Update tuning lines with position and MIDI note data provided by the presenter
    /// MainScene doesn't know about music theory - it just positions lines where told
    pub fn update_tuning_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote)]) {
        self.tuning_lines.update_lines(viewport, line_data);
    }
    
}