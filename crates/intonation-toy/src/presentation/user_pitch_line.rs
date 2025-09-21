#![cfg(target_arch = "wasm32")]

use three_d::{Blend, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderStates, WriteMask};
use crate::app_config::{INTONATION_ACCURACY_THRESHOLD, USER_PITCH_LINE_THICKNESS};
use crate::presentation::audio_analysis::AudioAnalysis;
use crate::common::shared_types::ColorScheme;
use crate::common::theme::rgb_to_srgba_with_alpha;

const COLOR_SUCCESS: [f32; 3] = [0.431, 0.905, 0.718];
const COLOR_WARNING: [f32; 3] = [1.000, 0.722, 0.420];

pub struct UserPitchLine {
    mesh: Gm<Line, ColorMaterial>,
    thickness: f32,
}

impl UserPitchLine {
    pub fn new(context: &Context) -> Self {
        let endpoints = (PhysicalPoint{x: 0.0, y: 0.0}, PhysicalPoint{x: 100.0, y: 0.0});
        let line = Line::new(context, endpoints.0, endpoints.1, USER_PITCH_LINE_THICKNESS);
        let material = ColorMaterial {
            color: rgb_to_srgba_with_alpha([1.0, 1.0, 1.0], 0.0),  // Transparent white initially
            texture: None,
            is_transparent: true,
            render_states: RenderStates {
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        };

        Self {
            mesh: Gm::new(line, material),
            thickness: USER_PITCH_LINE_THICKNESS,
        }
    }
}

impl UserPitchLine {
    fn create_material(&self, color_scheme: &ColorScheme, audio_analysis: &AudioAnalysis) -> ColorMaterial {
        let color = if audio_analysis.volume_peak {
            color_scheme.error
        } else if audio_analysis.cents_offset.abs() < INTONATION_ACCURACY_THRESHOLD {
            COLOR_SUCCESS
        } else {
            COLOR_WARNING
        };
        
        ColorMaterial {
            color: rgb_to_srgba_with_alpha(color, 1.0),
            texture: None,
            is_transparent: false,
            render_states: RenderStates {
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        }
    }
    
    pub fn update_position(
        &mut self,
        _context: &Context,
        endpoints: (PhysicalPoint, PhysicalPoint),
        color_scheme: &ColorScheme,
        audio_analysis: &AudioAnalysis,
    ) {
        self.mesh.set_endpoints(endpoints.0, endpoints.1);
        self.mesh.material = self.create_material(color_scheme, audio_analysis);
    }
    
    pub fn refresh_colors(&mut self, color_scheme: &ColorScheme, audio_analysis: &AudioAnalysis) {
        self.mesh.material = self.create_material(color_scheme, audio_analysis);
    }
    
    pub fn mesh(&self) -> &Gm<Line, ColorMaterial> {
        &self.mesh
    }
}