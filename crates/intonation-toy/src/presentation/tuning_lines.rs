#![cfg(target_arch = "wasm32")]

use three_d::{Blend, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderStates, Srgba, Viewport, WriteMask};
use crate::common::shared_types::MidiNote;
use crate::common::theme::get_current_color_scheme;
use crate::app_config::{NOTE_LABEL_FONT_SIZE, NOTE_LABEL_X_OFFSET, NOTE_LABEL_Y_OFFSET, INTERVAL_LABEL_X_OFFSET, NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN};


struct LineData {
    line: Gm<Line, ColorMaterial>,
    midi_note: MidiNote,
    y_position: f32,
    semitone_offset: i32,
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

    pub fn update_lines(&mut self, viewport: Viewport, input_data: &[(f32, MidiNote, f32, i32)], context: &Context, regular_color: Srgba, octave_color: Srgba) {
        let width = viewport.width as f32;

        let regular_material = ColorMaterial {
            color: regular_color,
            texture: None,
            is_transparent: false,
            render_states: RenderStates {
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        };

        let octave_material = ColorMaterial {
            color: octave_color,
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

        for &(y, midi_note, thickness, semitone_offset) in input_data {
            let line = Line::new(
                context,
                PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y },
                PhysicalPoint { x: width - NOTE_LINE_RIGHT_MARGIN, y },
                thickness
            );

            // Use accent color for octave lines (semitone_offset % 12 == 0)
            let material = if semitone_offset % 12 == 0 {
                octave_material.clone()
            } else {
                regular_material.clone()
            };

            self.line_data.push(LineData {
                line: Gm::new(line, material),
                midi_note,
                y_position: y,
                semitone_offset,
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
        let muted_color = scheme.muted;
        let accent_color = scheme.accent;

        self.line_data.iter()
            .map(|data| {
                let note_name = crate::common::shared_types::midi_note_to_name(data.midi_note);
                let text_y = data.y_position + NOTE_LABEL_Y_OFFSET;
                let text_x = NOTE_LABEL_X_OFFSET;

                // Use accent color for octave lines
                let text_color = if data.semitone_offset % 12 == 0 {
                    accent_color
                } else {
                    muted_color
                };

                (note_name, text_x, text_y, NOTE_LABEL_FONT_SIZE, [text_color[0], text_color[1], text_color[2], 1.0])
            })
            .collect()
    }

    pub fn get_interval_labels(&self, viewport_width: f32) -> Vec<(String, f32, f32, f32, [f32; 4])> {
        let scheme = get_current_color_scheme();
        let muted_color = scheme.muted;
        let accent_color = scheme.accent;

        self.line_data.iter()
            .map(|data| {
                let interval_name = crate::common::music_theory::semitone_to_interval_name(data.semitone_offset);
                let text_y = data.y_position + NOTE_LABEL_Y_OFFSET;
                let text_x = viewport_width - INTERVAL_LABEL_X_OFFSET;

                // Use accent color for octave lines
                let text_color = if data.semitone_offset % 12 == 0 {
                    accent_color
                } else {
                    muted_color
                };

                (interval_name, text_x, text_y, NOTE_LABEL_FONT_SIZE, [text_color[0], text_color[1], text_color[2], 1.0])
            })
            .collect()
    }
}