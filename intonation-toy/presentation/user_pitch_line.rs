// External crate imports
use three_d::{Blend, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderStates, WriteMask};

// Internal crate imports
use crate::app_config::{INTONATION_ACCURACY_THRESHOLD, USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MIN};
use crate::presentation::audio_analysis::AudioAnalysis;
use crate::shared_types::ColorScheme;
use crate::theme::rgb_to_srgba_with_alpha;

// Constants
const COLOR_SUCCESS: [f32; 3] = [0.431, 0.905, 0.718];  // Light green/cyan for accurate intonation
const COLOR_WARNING: [f32; 3] = [1.000, 0.722, 0.420];  // Orange for inaccurate intonation

// Helper functions

/// Helper function to get the user pitch line color from the color scheme
/// Returns error color when volume peak flag is true, more saturated accent color when within configured threshold, otherwise regular accent color
fn get_user_pitch_line_color(scheme: &ColorScheme, volume_peak: bool, cents_offset: f32) -> [f32; 3] {
    if volume_peak {
        scheme.error
    } else if cents_offset.abs() < INTONATION_ACCURACY_THRESHOLD {
        COLOR_SUCCESS
    } else {
        COLOR_WARNING
    }
}

/// Encapsulates all user pitch line related data and rendering state
pub struct UserPitchLine {
    mesh: Option<Gm<Line, ColorMaterial>>,
    material: ColorMaterial,
    thickness: f32,
    alpha: f32,
}

impl UserPitchLine {
    /// Creates a new UserPitchLine with default values
    pub fn new() -> Self {
        Self {
            mesh: None,
            material: ColorMaterial::default(),
            thickness: USER_PITCH_LINE_THICKNESS_MAX,
            alpha: USER_PITCH_LINE_TRANSPARENCY_MIN,
        }
    }
    
    /// Creates the material for the user pitch line based on current state
    fn create_material(&self, color_scheme: &ColorScheme, audio_analysis: &AudioAnalysis) -> ColorMaterial {
        let color = get_user_pitch_line_color(color_scheme, audio_analysis.volume_peak, audio_analysis.cents_offset);
        let has_transparency = self.alpha < 1.0;
        ColorMaterial {
            color: rgb_to_srgba_with_alpha(color, self.alpha),
            texture: None,
            is_transparent: has_transparency,
            render_states: RenderStates {
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        }
    }
    
    /// Updates the pitch line position and properties
    pub fn update_position(
        &mut self,
        context: &Context,
        endpoints: (PhysicalPoint, PhysicalPoint),
        new_thickness: f32,
        new_alpha: f32,
        color_scheme: &ColorScheme,
        audio_analysis: &AudioAnalysis,
    ) {
        let thickness_changed = (new_thickness - self.thickness).abs() > f32::EPSILON;
        let alpha_changed = (new_alpha - self.alpha).abs() > f32::EPSILON;
        
        if thickness_changed || alpha_changed || self.mesh.is_none() {
            // Update properties first
            self.thickness = new_thickness;
            self.alpha = new_alpha;
            
            // Update material
            self.material = self.create_material(color_scheme, audio_analysis);
            
            // Create new mesh
            let line = Line::new(context, endpoints.0, endpoints.1, new_thickness);
            self.mesh = Some(Gm::new(line, self.material.clone()));
        } else {
            // Only position changed, update existing mesh
            if let Some(ref mut mesh) = self.mesh {
                mesh.set_endpoints(endpoints.0, endpoints.1);
            }
        }
    }
    
    /// Refreshes colors by recreating the material and clearing the mesh
    pub fn refresh_colors(&mut self, color_scheme: &ColorScheme, audio_analysis: &AudioAnalysis) {
        self.material = self.create_material(color_scheme, audio_analysis);
        self.mesh = None; // Will be recreated with new material when needed
    }
    
    /// Returns a reference to the mesh if it exists
    pub fn mesh(&self) -> Option<&Gm<Line, ColorMaterial>> {
        self.mesh.as_ref()
    }
}