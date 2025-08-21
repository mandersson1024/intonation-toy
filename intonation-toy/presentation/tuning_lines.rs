use three_d::{Blend, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderStates, Srgba, Viewport, WriteMask};
use crate::shared_types::MidiNote;
use crate::theme::get_current_color_scheme;
use crate::app_config::{NOTE_LABEL_FONT_SIZE, NOTE_LABEL_X_OFFSET, NOTE_LABEL_Y_OFFSET, NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN};

struct LineData {
    line: Gm<Line, ColorMaterial>,
    midi_note: MidiNote,
    y_position: f32,
}

pub struct TuningLines {
    line_data: Vec<LineData>,
}

impl TuningLines {
    pub fn new(_context: &Context, _color: Srgba) -> Self {
        Self {
            line_data: Vec::new(),
        }
    }

    pub fn update_lines(&mut self, viewport: Viewport, input_data: &[(f32, MidiNote, f32)], context: &Context, color: Srgba) {
        let width = viewport.width as f32;
        
        let material = ColorMaterial {
            color,
            texture: None,
            is_transparent: false,
            render_states: RenderStates {
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        };
        
        self.line_data.clear();
        self.line_data.reserve(input_data.len());
        
        for &(y, midi_note, thickness) in input_data {
            let line = Line::new(
                context,
                PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y },
                PhysicalPoint { x: width - NOTE_LINE_RIGHT_MARGIN, y },
                thickness
            );
            
            self.line_data.push(LineData {
                line: Gm::new(line, material.clone()),
                midi_note,
                y_position: y,
            });
        }
    }
    
    pub fn lines(&self) -> impl Iterator<Item = &Gm<Line, ColorMaterial>> {
        self.line_data.iter().map(|data| &data.line)
    }
    
    pub fn midi_notes(&self) -> impl Iterator<Item = MidiNote> + '_ {
        self.line_data.iter().map(|data| data.midi_note)
    }
    
    pub fn clear(&mut self) {
        self.line_data.clear();
    }
    
    pub fn get_note_labels(&self) -> Vec<(String, f32, f32, f32, [f32; 4])> {
        let scheme = get_current_color_scheme();
        let text_color = scheme.muted;
        
        self.line_data.iter()
            .map(|data| {
                let note_name = crate::shared_types::midi_note_to_name(data.midi_note);
                let text_y = data.y_position + NOTE_LABEL_Y_OFFSET;
                let text_x = NOTE_LABEL_X_OFFSET;
                
                (note_name, text_x, text_y, NOTE_LABEL_FONT_SIZE, [text_color[0], text_color[1], text_color[2], 1.0])
            })
            .collect()
    }
}