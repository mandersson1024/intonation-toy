/*!
 * Text Rendering for MainScene
 * 
 * This module implements egui-based text rendering using a composite approach:
 * 
 * **EguiCompositeBackend** - Uses egui for composite text rendering with two-stage approach
 * 
 * The backend leverages egui's font rendering capabilities and provides
 * high-quality text rendering, proper Unicode support, and integrated glyph atlas management.
 */

// Standard library imports
// None needed

// External crate imports
use three_d::{Blend, Camera, ClearState, ColorMaterial, Context, Deg, Gm, Line, Object, PhysicalPoint, RenderStates, RenderTarget, Srgba, Texture2DRef, Viewport, WriteMask};
use three_d::core::{DepthTexture2D, Interpolation, Texture2D, Wrapping};
use three_d::renderer::geometry::Rectangle;

// Internal crate imports
use crate::app_config::{CLARITY_THRESHOLD, INTONATION_ACCURACY_THRESHOLD, NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN, OCTAVE_LINE_THICKNESS, REGULAR_LINE_THICKNESS, USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_THICKNESS_MIN, USER_PITCH_LINE_TRANSPARENCY_MAX, USER_PITCH_LINE_TRANSPARENCY_MIN};
use crate::presentation::egui_composite_backend::EguiCompositeBackend;
use crate::presentation::tuning_lines::{TuningLines, create_color_material};
use crate::shared_types::{ColorScheme, MidiNote, Scale, TuningSystem};
use crate::theme::{get_current_color_scheme, rgb_to_srgba, rgb_to_srgba_with_alpha};

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

/// Converts musical interval to screen Y position
/// interval of [0.5, 2.0] means [-1, +1] octaves
fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    // Using fixed zoom factor of 0.92
    const ZOOM_FACTOR: f32 = 0.92;
    viewport_height * (0.5 + interval * ZOOM_FACTOR * 0.5)
}

/// Creates a textured quad for background rendering
fn create_background_quad(
    context: &Context,
    width: u32,
    height: u32,
    texture: Texture2DRef,
) -> Gm<Rectangle, ColorMaterial> {
    assert!(width > 0, "Width must be greater than 0, got: {}", width);
    assert!(height > 0, "Height must be greater than 0, got: {}", height);
    
    let quad_width = width as f32;
    let quad_height = height as f32;
    let quad_center_x = quad_width * 0.5;
    let quad_center_y = quad_height * 0.5;
    
    let background_rectangle = Rectangle::new(
        context,
        (quad_center_x, quad_center_y),
        Deg(0.0),
        quad_width,
        quad_height,
    );
    
    let background_material = ColorMaterial {
        color: Srgba::WHITE,
        texture: Some(texture),
        is_transparent: true,
        render_states: RenderStates {
            depth_test: three_d::DepthTest::Always,
            write_mask: WriteMask::COLOR,
            blend: Blend::TRANSPARENCY,
            ..Default::default()
        },
    };
    
    Gm::new(background_rectangle, background_material)
}

/// Calculates line thickness and alpha based on clarity value
fn calculate_pitch_line_appearance(clarity: Option<f32>) -> (f32, f32) {
    if let Some(clarity_value) = clarity {
        let clamped_clarity = clarity_value.clamp(CLARITY_THRESHOLD, 1.0);
        let normalized_clarity = (clamped_clarity - CLARITY_THRESHOLD) / (1.0 - CLARITY_THRESHOLD);
        
        let thickness = USER_PITCH_LINE_THICKNESS_MAX + 
            normalized_clarity * (USER_PITCH_LINE_THICKNESS_MIN - USER_PITCH_LINE_THICKNESS_MAX);
        let alpha = USER_PITCH_LINE_TRANSPARENCY_MIN + 
            normalized_clarity * (USER_PITCH_LINE_TRANSPARENCY_MAX - USER_PITCH_LINE_TRANSPARENCY_MIN);
        
        (thickness, alpha)
    } else {
        (USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MAX)
    }
}

/// Encapsulates audio analysis state from the engine
struct AudioAnalysis {
    pitch_detected: bool,
    cents_offset: f32,
    volume_peak: bool,
}

impl AudioAnalysis {
    /// Creates a new AudioAnalysis with default values
    fn new() -> Self {
        Self {
            pitch_detected: false,
            cents_offset: 0.0,
            volume_peak: false,
        }
    }
    
    /// Updates pitch detection state
    fn update_pitch(&mut self, pitch_detected: bool, cents_offset: f32) {
        self.pitch_detected = pitch_detected;
        self.cents_offset = cents_offset;
    }
    
    /// Updates volume peak state
    fn update_volume_peak(&mut self, volume_peak: bool) {
        self.volume_peak = volume_peak;
    }
}

/// Encapsulates all user pitch line related data and rendering state
struct UserPitchLine {
    mesh: Option<Gm<Line, ColorMaterial>>,
    material: ColorMaterial,
    thickness: f32,
    alpha: f32,
}

impl UserPitchLine {
    /// Creates a new UserPitchLine with default values
    fn new() -> Self {
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
        create_color_material(
            rgb_to_srgba_with_alpha(color, self.alpha),
            has_transparency
        )
    }
    
    /// Updates the pitch line position and properties
    fn update_position(
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
    fn refresh_colors(&mut self, color_scheme: &ColorScheme, audio_analysis: &AudioAnalysis) {
        self.material = self.create_material(color_scheme, audio_analysis);
        self.mesh = None; // Will be recreated with new material when needed
    }
    
    /// Returns a reference to the mesh if it exists
    fn mesh(&self) -> Option<&Gm<Line, ColorMaterial>> {
        self.mesh.as_ref()
    }
}

pub struct MainScene {
    camera: Camera,
    user_pitch_line: UserPitchLine,
    audio_analysis: AudioAnalysis,
    tuning_lines: TuningLines,
    text_backend: EguiCompositeBackend,
    context: Context,
    color_scheme: ColorScheme,
    // Background texture system: pre-rendered texture with background
    // These resources are automatically cleaned up by three-d's RAII when replaced or dropped
    background_quad: Option<Gm<Rectangle, ColorMaterial>>,
    presentation_context: Option<crate::shared_types::PresentationContext>,
}

impl MainScene {
    // Associated functions
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
        let scheme = get_current_color_scheme();
        let tuning_lines = TuningLines::new(context, rgb_to_srgba(scheme.muted), rgb_to_srgba(scheme.muted));
        let text_backend = EguiCompositeBackend::new(context)?;

        Ok(Self {
            camera: Camera::new_2d(viewport),
            user_pitch_line: UserPitchLine::new(),
            audio_analysis: AudioAnalysis::new(),
            tuning_lines,
            text_backend,
            context: context.clone(),
            color_scheme: scheme,
            background_quad: None,
            presentation_context: None,
        })
    }

    // Private helper methods
    
    
    
    fn refresh_colors(&mut self) {
        let scheme = self.color_scheme.clone();
        
        // Update user pitch line colors
        self.user_pitch_line.refresh_colors(&scheme, &self.audio_analysis);
        
        // Update tuning lines materials
        self.tuning_lines.update_materials(rgb_to_srgba(scheme.muted), rgb_to_srgba(scheme.muted));
        
        // Clear and recreate all tuning lines with new material
        // They will be recreated with correct positions and thickness on next update_lines call
        self.tuning_lines.clear();
    }
    
    /// Get tuning line positions for the active tuning system
    /// Returns tuning line data with positions, MIDI notes, and thickness
    fn get_tuning_line_positions(
        &self,
        tuning_fork_midi: MidiNote,
        tuning_system: TuningSystem,
        scale: &Scale,
        viewport: Viewport,
    ) -> Vec<(f32, MidiNote, f32)> {
        let tuning_fork_frequency = crate::music_theory::midi_note_to_standard_frequency(tuning_fork_midi);
        
        // Helper function to determine line thickness based on semitone offset
        let get_thickness = |semitone: i32| -> f32 {
            // Octave lines (multiples of 12 semitones) get configurable thickness, others get regular thickness
            if semitone % 12 == 0 {
                OCTAVE_LINE_THICKNESS
            } else {
                REGULAR_LINE_THICKNESS
            }
        };
        
        // Helper function to calculate y position for a semitone interval
        let calculate_y_position = |semitone: i32| -> f32 {
            let frequency = crate::music_theory::interval_frequency(
                tuning_system,
                tuning_fork_frequency,
                semitone,
            );
            let interval = (frequency / tuning_fork_frequency).log2();
            interval_to_screen_y_position(interval, viewport.height as f32)
        };
        
        // Show intervals from -12 to +12 semitones including root (0)
        let mut line_data = Vec::new();
        
        // Add center line (tuning fork, 0 semitones)
        if crate::shared_types::semitone_in_scale(*scale, 0) {
            let y_position = interval_to_screen_y_position(0.0, viewport.height as f32);
            let thickness = get_thickness(0);
            line_data.push((y_position, tuning_fork_midi, thickness));
        }
        
        // Add intervals above tuning fork: +1 to +12 semitones
        for semitone in 1..=12 {
            if crate::shared_types::semitone_in_scale(*scale, semitone) {
                let y_position = calculate_y_position(semitone);
                let midi_note = (tuning_fork_midi as i32 + semitone).clamp(0, 127) as MidiNote;
                let thickness = get_thickness(semitone);
                line_data.push((y_position, midi_note, thickness));
            }
        }
        
        // Add intervals below tuning fork: -12 to -1 semitones
        for semitone in -12..=-1 {
            if crate::shared_types::semitone_in_scale(*scale, semitone) {
                let y_position = calculate_y_position(semitone);
                let midi_note = (tuning_fork_midi as i32 + semitone).clamp(0, 127) as MidiNote;
                let thickness = get_thickness(semitone);
                line_data.push((y_position, midi_note, thickness));
            }
        }
        
        line_data
    }

    // Public API methods
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }
    
    pub fn render(&mut self, screen: &mut RenderTarget) {
        let scheme = get_current_color_scheme();
        if scheme != self.color_scheme {
            self.color_scheme = scheme.clone();
            self.refresh_colors();
        }

        // Render the background quad if available
        if let Some(ref background_quad) = self.background_quad {
            self.camera.disable_tone_and_color_mapping();
            screen.render(
                &self.camera,
                [background_quad],
                &[],
            );
            self.camera.set_default_tone_and_color_mapping();
        }

        // Only render user pitch line if pitch is detected and line exists
        if self.audio_analysis.pitch_detected {
            if let Some(ref mesh) = self.user_pitch_line.mesh() {
                screen.render(&self.camera, [mesh], &[]);
            }
        }
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32, pitch_detected: bool, clarity: Option<f32>, cents_offset: f32) {
        self.audio_analysis.update_pitch(pitch_detected, cents_offset);
        
        // Validate viewport dimensions before proceeding
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for pitch position update");
            return;
        }
        
        if pitch_detected {
            let y = interval_to_screen_y_position(interval, viewport.height as f32);
            let endpoints = (
                PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y}, 
                PhysicalPoint{x:viewport.width as f32 - NOTE_LINE_RIGHT_MARGIN, y}
            );
            
            let (new_thickness, new_alpha) = calculate_pitch_line_appearance(clarity);
            
            self.user_pitch_line.update_position(
                &self.context,
                endpoints,
                new_thickness,
                new_alpha,
                &self.color_scheme,
                &self.audio_analysis,
            );
        }
    }
    
    /// Update tuning lines with position, MIDI note, and thickness data provided by the presenter
    /// MainScene doesn't know about music theory - it just positions lines where told
    pub fn update_tuning_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote, f32)]) {
        // Validate viewport and line data before proceeding
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for tuning lines update");
            return;
        }
        
        // Handle empty line data gracefully
        if line_data.is_empty() {
            crate::common::dev_log!("Warning: No tuning line data provided, clearing existing lines");
            self.tuning_lines.clear();
            return;
        }
        
        // Use the new thickness-aware method
        self.tuning_lines.update_lines(viewport, line_data);
    }
    
    pub fn update_closest_note(&mut self, note: Option<MidiNote>) {
        self.tuning_lines.set_closest_note(note);
    }
    
    /// Update the volume peak state for color determination
    pub fn update_volume_peak(&mut self, volume_peak: bool) {
        self.audio_analysis.update_volume_peak(volume_peak);
    }
    
    /// Renders tuning lines and note labels to the background texture by recreating it.
    /// This method recreates the background texture with the tuning lines and note labels
    /// rendered to it, replacing the existing background texture.
    pub fn render_to_background_texture(&mut self, viewport: Viewport) {
        // Validate viewport dimensions
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for background texture");
            return;
        }
        
        // Create a new Texture2D for the background
        let mut background_texture = Texture2D::new_empty::<[u8; 4]>(
            &self.context,
            viewport.width,
            viewport.height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Create new depth texture for the render target
        let mut depth_texture = DepthTexture2D::new::<f32>(
            &self.context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Create render target and render content to the texture
        {
            let camera = Camera::new_2d(viewport);
            let [r, g, b] = get_current_color_scheme().background;

            let tuning_lines_vec: Vec<&dyn Object> = 
                self.tuning_lines
                .lines()
                .map(|line| line as &dyn Object)
                .collect();            

            self.text_backend.clear_queue();
            self.tuning_lines.render_note_labels(&mut self.text_backend);
            let text_models = self.text_backend.create_text_models(&self.context, viewport);
            crate::common::dev_log!("TEXT_DEBUG: Got {} text models from backend", text_models.len());
            let text_objects: Vec<&dyn Object> = 
                text_models
                .iter()
                .map(|model| model.as_ref() as &dyn Object)
                .collect();

            RenderTarget::new(
                background_texture.as_color_target(None),
                depth_texture.as_depth_target(),
            )
            .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
            .render(&camera, tuning_lines_vec, &[])
            .render(&camera, text_objects, &[]);
        } // render_target goes out of scope here
        
        let background_texture_ref = Texture2DRef::from_texture(background_texture);
        self.background_quad = Some(create_background_quad(&self.context, viewport.width, viewport.height, background_texture_ref));
    }
    
    /// Update the presentation context with new tuning fork, tuning system, and scale.
    /// Also re-renders the background texture when the presentation context changes.
    pub fn update_presentation_context(&mut self, context: &crate::shared_types::PresentationContext, viewport: Viewport) {
        if self.presentation_context.as_ref() == Some(context) {
            return;
        }

        self.presentation_context = Some(context.clone());
        
        // Validate viewport dimensions before proceeding
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for presentation context update");
            return;
        }
        
        // Calculate tuning line positions and update them
        let tuning_line_data = self.get_tuning_line_positions(
            context.tuning_fork_note,
            context.tuning_system,
            context.current_scale.as_ref().unwrap_or(&Scale::Chromatic),
            viewport,
        );
        
        // Update tuning lines with calculated data
        self.update_tuning_lines(viewport, &tuning_line_data);
        
        // Re-render background texture with new tuning lines
        self.render_to_background_texture(viewport);
    }
    
}

