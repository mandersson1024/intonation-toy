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

use three_d::{Blend, Camera, ClearState, ColorMaterial, Context, Deg, Gm, Line, Object, PhysicalPoint, RenderStates, RenderTarget, Srgba, Texture2DRef, Viewport, WriteMask};
use three_d::core::{Texture2D, DepthTexture2D, Interpolation, Wrapping};
use three_d::renderer::geometry::Rectangle;
use crate::shared_types::{MidiNote, ColorScheme, TuningSystem, Scale};
use crate::theme::{get_current_color_scheme, rgb_to_srgba, rgb_to_srgba_with_alpha};
use crate::app_config::{USER_PITCH_LINE_THICKNESS_MIN, USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MIN, USER_PITCH_LINE_TRANSPARENCY_MAX, CLARITY_THRESHOLD, INTONATION_ACCURACY_THRESHOLD, NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN, OCTAVE_LINE_THICKNESS, REGULAR_LINE_THICKNESS};
use crate::presentation::tuning_lines::{TuningLines, create_color_material};
use crate::presentation::egui_composite_backend::EguiCompositeBackend;


// User pitch line colors
const COLOR_SUCCESS: [f32; 3] = [0.431, 0.905, 0.718];  // Light green/cyan for accurate intonation
const COLOR_WARNING: [f32; 3] = [1.000, 0.722, 0.420];  // Orange for inaccurate intonation


// Helper function to get the user pitch line color from the color scheme
// Returns error color when volume peak flag is true, more saturated accent color when within configured threshold, otherwise regular accent color
fn get_user_pitch_line_color(scheme: &ColorScheme, volume_peak: bool, cents_offset: f32) -> [f32; 3] {
    if volume_peak {
        scheme.error
    } else if cents_offset.abs() < INTONATION_ACCURACY_THRESHOLD {
        COLOR_SUCCESS
    } else {
        COLOR_WARNING
    }
}

pub fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    // interval of [0.5, 2.0] means [-1, +1] octaves
    // Using fixed zoom factor of 0.92
    const ZOOM_FACTOR: f32 = 0.92;
    let y: f32 = viewport_height * (0.5 + interval * ZOOM_FACTOR * 0.5);
    y
}

pub struct MainScene {
    camera: Camera,
    user_pitch_line: Gm<Line, ColorMaterial>,
    user_pitch_line_material: ColorMaterial,
    pub tuning_lines: TuningLines,
    text_backend: EguiCompositeBackend,
    context: Context,
    pitch_detected: bool,
    current_scheme: ColorScheme,
    user_pitch_line_thickness: f32,
    user_pitch_line_alpha: f32,
    volume_peak: bool,
    cents_offset: f32,
    // Background texture system: pre-rendered texture with background
    // These resources are automatically cleaned up by three-d's RAII when replaced or dropped
    background_quad: Gm<Rectangle, ColorMaterial>,
    presentation_context: Option<crate::shared_types::PresentationContext>,
}

impl MainScene {
    /// Creates the material for the user pitch line based on current state
    fn create_user_pitch_line_material(&self) -> ColorMaterial {
        let color = get_user_pitch_line_color(&self.current_scheme, self.volume_peak, self.cents_offset);
        let has_transparency = self.user_pitch_line_alpha < 1.0;
        create_color_material(
            rgb_to_srgba_with_alpha(color, self.user_pitch_line_alpha),
            has_transparency
        )
    }
    
    fn create_background_texture(context: &Context, viewport: Viewport) -> Result<Gm<Rectangle, ColorMaterial>, String> {
        // Validate viewport dimensions
        if viewport.width == 0 || viewport.height == 0 {
            return Err("Invalid viewport dimensions: width and height must be greater than 0".to_string());
        }
        
        // Create a Texture2D for the background
        let mut background_texture = Texture2D::new_empty::<[u8; 4]>(
            context,
            viewport.width as u32,
            viewport.height as u32,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Create minimal depth texture (not used for 2D rendering)
        let mut depth_texture = DepthTexture2D::new::<f32>(
            context,
            viewport.width as u32,
            viewport.height as u32,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Create render target
        {
            let render_target = RenderTarget::new(
                background_texture.as_color_target(None),
                depth_texture.as_depth_target(),
            );
            
            // Clear with current theme background color
            let scheme = get_current_color_scheme();
            let [r, g, b] = scheme.background;
            render_target.clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0));
        } // render_target goes out of scope here, automatically cleaned up
        
        // Create background quad that fills the entire viewport
        let quad_width = viewport.width as f32;
        let quad_height = viewport.height as f32;
        let quad_center_x = quad_width * 0.5;
        let quad_center_y = quad_height * 0.5;
        
        let background_rectangle = Rectangle::new(
            context,
            (quad_center_x, quad_center_y), // center position as tuple
            Deg(0.0),                       // no rotation
            quad_width,                     // full viewport width
            quad_height,                    // full viewport height
        );
        
        // Wrap texture in Texture2DRef (which is Arc<Texture2D>) for shared ownership
        let background_texture_ref = Texture2DRef::from_texture(background_texture);
        
        // Create material with the background texture
        // Enable transparency and disable depth testing so it doesn't block the user pitch line
        let background_material = ColorMaterial {
            color: Srgba::WHITE, // White tint to show texture as-is
            texture: Some(background_texture_ref.clone()),
            is_transparent: true,
            render_states: RenderStates {
                depth_test: three_d::DepthTest::Always,
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        };
        
        let background_quad = Gm::new(background_rectangle, background_material);
        
        Ok(background_quad)
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
        
        // Create new background quad with the updated texture
        let quad_width = viewport.width as f32;
        let quad_height = viewport.height as f32;
        let quad_center_x = quad_width * 0.5;
        let quad_center_y = quad_height * 0.5;
        
        let background_rectangle = Rectangle::new(
            &self.context,
            (quad_center_x, quad_center_y),
            Deg(0.0),
            quad_width,
            quad_height,
        );

        let background_material = ColorMaterial {
            color: Srgba::default(),
            texture: Some(Texture2DRef::from_texture(background_texture)),
            is_transparent: true,
            render_states: RenderStates {
                depth_test: three_d::DepthTest::Always,
                write_mask: WriteMask::COLOR,
                ..Default::default()
            },
        };
        
        self.background_quad = Gm::new(background_rectangle, background_material);
    }
    
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
        let scheme = get_current_color_scheme();
        let initial_thickness = USER_PITCH_LINE_THICKNESS_MAX;
        let user_pitch_line = Line::new(context, PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y:0.0}, PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y:0.0}, initial_thickness);

        let initial_volume_peak = false;
        let initial_cents_offset = 0.0;
        let initial_alpha = USER_PITCH_LINE_TRANSPARENCY_MAX;
        
        // Create initial material
        let color = get_user_pitch_line_color(&scheme, initial_volume_peak, initial_cents_offset);
        let user_pitch_line_material = create_color_material(
            rgb_to_srgba_with_alpha(color, initial_alpha), 
            initial_alpha < 1.0
        );
        
        let tuning_lines = TuningLines::new(context, rgb_to_srgba(scheme.muted), rgb_to_srgba(scheme.muted));
        
        // Create text backend using EguiComposite
        let text_backend = EguiCompositeBackend::new(context)?;

        // Create background texture and background quad with fallback handling
        let background_quad = match Self::create_background_texture(context, viewport) {
            Ok(quad) => quad,
            Err(e) => {
                crate::common::dev_log!("Warning: Failed to create background texture: {}, using fallback", e);
                // Create a simple colored quad as fallback
                let quad_width = viewport.width as f32;
                let quad_height = viewport.height as f32;
                let quad_center_x = quad_width * 0.5;
                let quad_center_y = quad_height * 0.5;
                
                let background_rectangle = Rectangle::new(
                    context,
                    (quad_center_x, quad_center_y),
                    Deg(0.0),
                    quad_width,
                    quad_height,
                );
                
                let [r, g, b] = scheme.background;
                let background_material = ColorMaterial {
                    color: Srgba::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255),
                    texture: None,
                    is_transparent: false,
                    render_states: RenderStates::default(),
                };
                
                Gm::new(background_rectangle, background_material)
            }
        };
        
        Ok(Self {
            camera: Camera::new_2d(viewport),
            user_pitch_line: Gm::new(user_pitch_line, user_pitch_line_material.clone()),
            user_pitch_line_material,
            tuning_lines,
            text_backend,
            context: context.clone(),
            pitch_detected: false,
            current_scheme: scheme,
            user_pitch_line_thickness: initial_thickness,
            user_pitch_line_alpha: initial_alpha,
            volume_peak: initial_volume_peak,
            cents_offset: initial_cents_offset,
            background_quad,
            presentation_context: None,
        })
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }
    
    fn refresh_colors(&mut self) {
        let scheme = self.current_scheme.clone();
        
        // Update user pitch line material with new color
        self.user_pitch_line_material = self.create_user_pitch_line_material();
        
        // Recreate user pitch line with updated material
        let line = Line::new(&self.context, 
            PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y:0.0}, 
            PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y:0.0}, 
            self.user_pitch_line_thickness);
        self.user_pitch_line = Gm::new(line, self.user_pitch_line_material.clone());
        
        // Update tuning lines materials
        self.tuning_lines.update_materials(rgb_to_srgba(scheme.muted), rgb_to_srgba(scheme.muted));
        
        // Clear and recreate all tuning lines with new material
        // They will be recreated with correct positions and thickness on next update_lines call
        self.tuning_lines.clear();
    }
    
    pub fn render(&mut self, screen: &mut RenderTarget) {
        let scheme = get_current_color_scheme();
        if scheme != self.current_scheme {
            self.current_scheme = scheme.clone();
            self.refresh_colors();
        }

        // Always render the background quad to provide a consistent baseline
        self.camera.disable_tone_and_color_mapping();
        screen.render(
            &self.camera,
            &[&self.background_quad],
            &[],
        );
        self.camera.set_default_tone_and_color_mapping();

        // Only render user pitch line if pitch is detected
        if self.pitch_detected {
            screen.render(&self.camera, &[&self.user_pitch_line], &[]);
        }
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32, pitch_detected: bool, clarity: Option<f32>, cents_offset: f32) {
        self.pitch_detected = pitch_detected;
        self.cents_offset = cents_offset;
        
        // Validate viewport dimensions before proceeding
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for pitch position update");
            return;
        }
        
        if pitch_detected {
            let y = interval_to_screen_y_position(interval, viewport.height as f32);
            let endpoints = (PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y}, PhysicalPoint{x:viewport.width as f32 - NOTE_LINE_RIGHT_MARGIN, y});
            
            // Calculate thickness and alpha based on clarity
            let (new_thickness, new_alpha) = if let Some(clarity_value) = clarity {
                // Map clarity from [CLARITY_THRESHOLD, 1.0] to [USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_THICKNESS_MIN]
                let clamped_clarity = clarity_value.clamp(CLARITY_THRESHOLD, 1.0);
                let normalized_clarity = (clamped_clarity - CLARITY_THRESHOLD) / (1.0 - CLARITY_THRESHOLD);
                let thickness = USER_PITCH_LINE_THICKNESS_MAX + normalized_clarity * (USER_PITCH_LINE_THICKNESS_MIN - USER_PITCH_LINE_THICKNESS_MAX);
                
                // Map clarity to alpha using configured transparency range
                // At CLARITY_THRESHOLD: alpha = USER_PITCH_LINE_TRANSPARENCY_MIN
                // At 1.0 clarity: alpha = USER_PITCH_LINE_TRANSPARENCY_MAX
                let alpha = USER_PITCH_LINE_TRANSPARENCY_MIN + normalized_clarity * (USER_PITCH_LINE_TRANSPARENCY_MAX - USER_PITCH_LINE_TRANSPARENCY_MIN);
                (thickness, alpha)
            } else {
                (USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MAX) // Default values when no clarity provided
            };
            
            // Check if thickness or alpha changed - if so, recreate the line
            let thickness_changed = (new_thickness - self.user_pitch_line_thickness).abs() > f32::EPSILON;
            let alpha_changed = (new_alpha - self.user_pitch_line_alpha).abs() > f32::EPSILON;
            
            if thickness_changed || alpha_changed {
                // Update thickness and alpha first so the material creation uses new values
                self.user_pitch_line_thickness = new_thickness;
                self.user_pitch_line_alpha = new_alpha;
                
                // Update the user pitch line material
                self.user_pitch_line_material = self.create_user_pitch_line_material();
                
                let line = Line::new(&self.context, endpoints.0, endpoints.1, new_thickness);
                self.user_pitch_line = Gm::new(line, self.user_pitch_line_material.clone());
            } else {
                // Only position changed, use existing line
                self.user_pitch_line.set_endpoints(endpoints.0, endpoints.1);
            }
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
        self.volume_peak = volume_peak;
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
        
        // Show intervals from -12 to +12 semitones including root (0)
        let mut line_data = Vec::new();
        
        // Add center line (tuning fork, 0 semitones)
        if crate::shared_types::semitone_in_scale(*scale, 0) {
            // Tuning fork frequency stays at interval 0.0 (log2(1) = 0)
            let interval = 0.0;
            let y_position = interval_to_screen_y_position(
                interval,
                viewport.height as f32
            );
            let thickness = get_thickness(0);
            line_data.push((y_position, tuning_fork_midi, thickness));
        }
        
        // Add intervals above tuning fork: +1 to +12 semitones
        for semitone in 1..=12 {
            // Only show intervals that are in the current scale
            if crate::shared_types::semitone_in_scale(*scale, semitone) {
                let frequency = crate::music_theory::interval_frequency(
                    tuning_system,
                    tuning_fork_frequency,
                    semitone,
                );
                let interval = (frequency / tuning_fork_frequency).log2();
                let y_position = interval_to_screen_y_position(
                    interval,
                    viewport.height as f32
                );
                let midi_note = (tuning_fork_midi as i32 + semitone).clamp(0, 127) as MidiNote;
                let thickness = get_thickness(semitone);
                line_data.push((y_position, midi_note, thickness));
            }
        }
        
        // Add intervals below tuning fork: -12 to -1 semitones
        for semitone in -12..=-1 {
            // Only show intervals that are in the current scale
            if crate::shared_types::semitone_in_scale(*scale, semitone) {
                let frequency = crate::music_theory::interval_frequency(
                    tuning_system,
                    tuning_fork_frequency,
                    semitone,
                );
                let interval = (frequency / tuning_fork_frequency).log2();
                let y_position = interval_to_screen_y_position(
                    interval,
                    viewport.height as f32
                );
                let midi_note = (tuning_fork_midi as i32 + semitone).clamp(0, 127) as MidiNote;
                let thickness = get_thickness(semitone);
                line_data.push((y_position, midi_note, thickness));
            }
        }
        
        line_data
    }
    
}

