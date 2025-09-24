#![cfg(target_arch = "wasm32")]

use three_d::{Blend, ColorMaterial, Context, Gm, Line, Object, PhysicalPoint, RenderStates, Srgba, Viewport, WriteMask};
use crate::common::shared_types::MidiNote;
use crate::common::theme::{get_current_color_scheme, rgb_to_rgba, rgb_to_srgba_with_alpha};
use crate::app_config::{NOTE_LABEL_FONT_SIZE, NOTE_LABEL_X_OFFSET, NOTE_LABEL_Y_OFFSET, INTERVAL_LABEL_X_OFFSET, NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN};

pub enum ColorMode {
    Normal,
    Highlight,
}

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
    

    pub fn get_note_labels(&self, color_mode: ColorMode) -> Vec<(String, f32, f32, f32, [f32; 4], bool)> {
        let scheme = get_current_color_scheme();

        self.line_data.iter()
            .map(|data| {
                let note_name = crate::common::shared_types::midi_note_to_name(data.midi_note);
                let text_y = data.y_position + NOTE_LABEL_Y_OFFSET;
                let text_x = NOTE_LABEL_X_OFFSET;
                let is_bold = data.semitone_offset % 12 == 0;

                let text_color = match color_mode {
                    ColorMode::Highlight => rgb_to_rgba(scheme.accent),
                    ColorMode::Normal => if is_bold { rgb_to_rgba(scheme.primary) } else { rgb_to_rgba(scheme.muted) },
                };

                (note_name, text_x, text_y, NOTE_LABEL_FONT_SIZE, text_color, is_bold)
            })
            .collect()
    }

    pub fn get_interval_labels(&self, viewport_width: f32, color_mode: ColorMode) -> Vec<(String, f32, f32, f32, [f32; 4], bool)> {
        let scheme = get_current_color_scheme();

        self.line_data.iter()
            .map(|data| {
                let interval_name = crate::common::music_theory::semitone_to_interval_name(data.semitone_offset);
                let text_y = data.y_position + NOTE_LABEL_Y_OFFSET;
                let text_x = viewport_width - INTERVAL_LABEL_X_OFFSET;
                let is_bold = data.semitone_offset % 12 == 0;

                let text_color = match color_mode {
                    ColorMode::Highlight => rgb_to_rgba(scheme.accent),
                    ColorMode::Normal => if is_bold { rgb_to_rgba(scheme.primary) } else { rgb_to_rgba(scheme.muted) },
                };

                (interval_name, text_x, text_y, NOTE_LABEL_FONT_SIZE, text_color, is_bold)
            })
            .collect()
    }

    pub fn get_lines(&self, context: &Context, viewport_width: f32, color_mode: ColorMode) -> Vec<Box<dyn Object>> {
        let scheme = get_current_color_scheme();

        let color = match color_mode {
            ColorMode::Highlight => rgb_to_srgba_with_alpha(scheme.accent, 1.0),
            ColorMode::Normal => rgb_to_srgba_with_alpha(scheme.muted, 1.0),
        };

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

        self.line_data.iter()
            .map(|data| {
                let thickness = if data.semitone_offset % 12 == 0 {
                    crate::app_config::OCTAVE_LINE_THICKNESS
                } else {
                    crate::app_config::REGULAR_LINE_THICKNESS
                };

                let line = Line::new(
                    context,
                    PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y: data.y_position },
                    PhysicalPoint { x: viewport_width - NOTE_LINE_RIGHT_MARGIN, y: data.y_position },
                    thickness
                );
                Box::new(Gm::new(line, material.clone())) as Box<dyn Object>
            })
            .collect()
    }
}