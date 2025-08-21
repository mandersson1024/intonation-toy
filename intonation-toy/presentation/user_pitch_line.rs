use three_d::{Blend, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderStates, WriteMask};
use crate::app_config::{INTONATION_ACCURACY_THRESHOLD, USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MIN};
use crate::presentation::audio_analysis::AudioAnalysis;
use crate::shared_types::ColorScheme;
use crate::theme::rgb_to_srgba_with_alpha;

const COLOR_SUCCESS: [f32; 3] = [0.431, 0.905, 0.718];  // Light green/cyan for accurate intonation
const COLOR_WARNING: [f32; 3] = [1.000, 0.722, 0.420];  // Orange for inaccurate intonation

pub struct UserPitchLine {
    mesh: Option<Gm<Line, ColorMaterial>>,
    thickness: f32,
    alpha: f32,
}

impl UserPitchLine {
    pub fn new() -> Self {
        Self {
            mesh: None,
            thickness: USER_PITCH_LINE_THICKNESS_MAX,
            alpha: USER_PITCH_LINE_TRANSPARENCY_MIN,
        }
    }
    
    fn create_material(&self, color_scheme: &ColorScheme, audio_analysis: &AudioAnalysis) -> ColorMaterial {
        let color = if audio_analysis.volume_peak {
            color_scheme.error
        } else if audio_analysis.cents_offset.abs() < INTONATION_ACCURACY_THRESHOLD {
            COLOR_SUCCESS
        } else {
            COLOR_WARNING
        };
        
        ColorMaterial {
            color: rgb_to_srgba_with_alpha(color, self.alpha),
            texture: None,
            is_transparent: self.alpha < 1.0,
            render_states: RenderStates {
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        }
    }
    
    pub fn update_position(
        &mut self,
        context: &Context,
        endpoints: (PhysicalPoint, PhysicalPoint),
        new_thickness: f32,
        new_alpha: f32,
        color_scheme: &ColorScheme,
        audio_analysis: &AudioAnalysis,
    ) {
        if new_thickness != self.thickness || new_alpha != self.alpha || self.mesh.is_none() {
            self.thickness = new_thickness;
            self.alpha = new_alpha;
            let material = self.create_material(color_scheme, audio_analysis);
            let line = Line::new(context, endpoints.0, endpoints.1, new_thickness);
            self.mesh = Some(Gm::new(line, material));
        } else if let Some(ref mut mesh) = self.mesh {
            mesh.set_endpoints(endpoints.0, endpoints.1);
        }
    }
    
    pub fn refresh_colors(&mut self, _color_scheme: &ColorScheme, _audio_analysis: &AudioAnalysis) {
        self.mesh = None;
    }
    
    pub fn mesh(&self) -> Option<&Gm<Line, ColorMaterial>> {
        self.mesh.as_ref()
    }
}