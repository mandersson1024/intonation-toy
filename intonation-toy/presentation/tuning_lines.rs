use three_d::{Blend, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderStates, Srgba, Viewport, WriteMask};
use crate::shared_types::MidiNote;
use crate::theme::get_current_color_scheme;

// Constants specific to tuning lines
pub const NOTE_LABEL_FONT_SIZE: f32 = 24.0;
pub const NOTE_LABEL_X_OFFSET: f32 = 10.0;
pub const NOTE_LABEL_Y_OFFSET: f32 = 13.0;
pub const NOTE_LINE_LEFT_MARGIN: f32 = 64.0;
pub const NOTE_LINE_RIGHT_MARGIN: f32 = 30.0;

// Line thickness values
pub const OCTAVE_LINE_THICKNESS: f32 = 8.0;
pub const REGULAR_LINE_THICKNESS: f32 = 4.0;
const DEFAULT_LINE_THICKNESS: f32 = 1.0;

/// Create a ColorMaterial with the given color and optional transparency
pub fn create_color_material(color: Srgba, is_transparent: bool) -> ColorMaterial {
    ColorMaterial {
        color,
        texture: None,
        is_transparent,
        render_states: RenderStates {
            write_mask: WriteMask::COLOR,
            blend: Blend::TRANSPARENCY,
            ..Default::default()
        },
    }
}

pub struct TuningLines {
    lines: Vec<Gm<Line, ColorMaterial>>,
    midi_notes: Vec<MidiNote>,
    y_positions: Vec<f32>,
    thicknesses: Vec<f32>,
    context: Context,
    regular_material: ColorMaterial,
    octave_material: ColorMaterial,
    closest_midi_note: Option<MidiNote>,
}

impl TuningLines {
    pub fn new(context: &Context, regular_color: Srgba, octave_color: Srgba) -> Self {
        let regular_material = create_color_material(regular_color, false);
        let octave_material = create_color_material(octave_color, false);
        
        Self {
            lines: Vec::new(),
            midi_notes: Vec::new(),
            y_positions: Vec::new(),
            thicknesses: Vec::new(),
            context: context.clone(),
            regular_material,
            octave_material,
            closest_midi_note: None,
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
                PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y: 0.0 },
                PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y: 0.0 },
                DEFAULT_LINE_THICKNESS  // Default thickness, will be updated
            );
            // Use regular material as default, will be updated if needed
            self.lines.push(Gm::new(line, self.regular_material.clone()));
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
            self.thicknesses.push(DEFAULT_LINE_THICKNESS); // Temporary value, will be set below
        }
        
        // Set positions, MIDI notes, and thickness for all lines
        for (i, &(y, midi_note, thickness)) in line_data.iter().enumerate() {
            // Determine material priority: accent > octave > regular
            let is_octave = thickness == OCTAVE_LINE_THICKNESS;
            let material = if is_octave { 
                &self.octave_material 
            } else { 
                &self.regular_material 
            };
            
            // If thickness changed, recreate the line
            if i < self.thicknesses.len() && self.thicknesses[i] != thickness {
                let line = Line::new(
                    &self.context,
                    PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y },
                    PhysicalPoint { x: width - NOTE_LINE_RIGHT_MARGIN, y },
                    thickness
                );
                self.lines[i] = Gm::new(line, material.clone());
            } else {
                // Always recreate the line to ensure material is up to date
                // This is simpler and ensures accent highlighting works correctly
                let line = Line::new(
                    &self.context,
                    PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y },
                    PhysicalPoint { x: width - NOTE_LINE_RIGHT_MARGIN, y },
                    thickness
                );
                self.lines[i] = Gm::new(line, material.clone());
            }
            self.midi_notes[i] = midi_note;
            self.y_positions[i] = y;
            self.thicknesses[i] = thickness;
        }
    }
    
    pub fn lines(&self) -> impl Iterator<Item = &Gm<Line, ColorMaterial>> {
        self.lines.iter()
    }
    
    /// Returns an iterator over the MIDI notes corresponding to each tuning line
    pub fn midi_notes(&self) -> impl Iterator<Item = MidiNote> + '_ {
        self.midi_notes.iter().copied()
    }
    
    /// Set the closest MIDI note that should be highlighted with accent color
    pub fn set_closest_note(&mut self, note: Option<MidiNote>) {
        self.closest_midi_note = note;
    }
    
    /// Update the materials used for rendering tuning lines
    pub fn update_materials(&mut self, regular_color: Srgba, octave_color: Srgba) {
        self.regular_material = create_color_material(regular_color, false);
        self.octave_material = create_color_material(octave_color, false);
    }
    
    /// Clear all tuning lines data
    pub fn clear(&mut self) {
        self.lines.clear();
        self.midi_notes.clear();
        self.y_positions.clear();
        self.thicknesses.clear();
    }
    
    /// Render note labels above each tuning line
    pub fn render_note_labels(&self, text_backend: &mut crate::presentation::main_scene::EguiCompositeBackend) {
        crate::common::dev_log!("TEXT_DEBUG: Rendering {} note labels", self.midi_notes.len());
        for (i, &midi_note) in self.midi_notes.iter().enumerate() {
            let y_position = self.y_positions[i];
            
            // Convert MIDI note to name
            let note_name = crate::shared_types::midi_note_to_name(midi_note);
            
            // Position text aligned with the line (same Y position)
            let text_y = y_position + NOTE_LABEL_Y_OFFSET;
            let text_x = NOTE_LABEL_X_OFFSET;
            
            // Determine color based on whether this is the closest note
            let scheme = get_current_color_scheme();
            let text_color = scheme.muted;

            crate::common::dev_log!("TEXT_DEBUG: Queuing text '{}' at ({}, {}) with size {}", note_name, text_x, text_y, NOTE_LABEL_FONT_SIZE);
            if note_name.contains('b') {
                crate::common::dev_log!("TEXT_DEBUG: Note '{}' contains 'b' character", note_name);
            }
            text_backend.queue_text(&note_name, text_x, text_y, NOTE_LABEL_FONT_SIZE, [text_color[0], text_color[1], text_color[2], 1.0]);
        }
    }
}