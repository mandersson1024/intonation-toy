use three_d::{AmbientLight, Blend, Camera, ClearState, ColorMaterial, Context, Deg, Gm, Line, Object, PhysicalPoint, RenderStates, RenderTarget, Srgba, Texture2DRef, Viewport, WriteMask};
use three_d::core::{Texture2D, DepthTexture2D, Interpolation, Wrapping};
use three_d::renderer::geometry::Rectangle;
use crate::shared_types::{MidiNote, ColorScheme, TuningSystem, Scale};
use crate::theme::{get_current_color_scheme, rgb_to_srgba, rgb_to_srgba_with_alpha};
use crate::app_config::{USER_PITCH_LINE_THICKNESS_MIN, USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MIN, USER_PITCH_LINE_TRANSPARENCY_MAX, CLARITY_THRESHOLD, INTONATION_ACCURACY_THRESHOLD};

// Left margin to reserve space for note names
const NOTE_NAME_X_OFFSET: f32 = 14.0;
const NOTE_NAME_Y_OFFSET: f32 = 2.0;
const NOTE_LINE_LEFT_MARGIN: f32 = 30.0;
const NOTE_LINE_RIGHT_MARGIN: f32 = 10.0;

// Font size for note labels
const NOTE_LABEL_FONT_SIZE: f32 = 16.0;

// Line thickness values
pub const OCTAVE_LINE_THICKNESS: f32 = 4.0;
pub const REGULAR_LINE_THICKNESS: f32 = 2.0;
const DEFAULT_LINE_THICKNESS: f32 = 1.0;

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

pub fn interval_to_screen_y_position(interval: f32, viewport_height: f32, zoom_factor: f32) -> f32 {
    // interval of [0.5, 2.0] means [-1, +1] octaves
    let y: f32 = viewport_height * (0.5 + interval * zoom_factor * 0.5);
    y
}

/// Create a ColorMaterial with the given color and optional transparency
fn create_color_material(color: Srgba, is_transparent: bool) -> ColorMaterial {
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
    
    /// Render note labels above each tuning line
    pub fn render_note_labels(&self, text_renderer: &mut TextRenderer, volume_peak: bool, cents_offset: f32) {
        for (i, &midi_note) in self.midi_notes.iter().enumerate() {
            let y_position = self.y_positions[i];
            
            // Convert MIDI note to name
            let note_name = crate::shared_types::midi_note_to_name(midi_note);
            
            // Position text aligned with the line (same Y position)
            let text_y = y_position + NOTE_NAME_Y_OFFSET;
            let text_x = NOTE_NAME_X_OFFSET;
            
            // Determine color based on whether this is the closest note
            let scheme = get_current_color_scheme();
            let text_color = scheme.muted;

            text_renderer.queue_text(&note_name, text_x, text_y, NOTE_LABEL_FONT_SIZE, [text_color[0], text_color[1], text_color[2], 1.0]);
        }
    }
}

pub struct TextRenderer {
    text_builder: three_d_text_builder::TextBuilder,
    queued_texts: Vec<QueuedText>,
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
    pub fn new(_context: &Context) -> Result<Self, String> {
        // Use the actual Roboto Regular font file
        let roboto_font = include_bytes!("../static/fonts/Roboto-Regular.ttf");
        
        let text_builder = three_d_text_builder::TextBuilder::new(
            roboto_font,
            three_d_text_builder::TextBuilderSettings::default()
        ).map_err(|e| format!("Failed to create TextBuilder with Roboto font: {:?}", e))?;
            
        Ok(Self {
            text_builder,
            queued_texts: Vec::new(),
        })
    }
    
    /// Queue text for rendering at the specified position
    pub fn queue_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.queued_texts.push(QueuedText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }
    
    /// Clear all queued text (called each frame)
    pub fn clear_queue(&mut self) {
        self.queued_texts.clear();
    }
    
    /// Create text models using the actual Roboto font
    pub fn create_text_models(&mut self, context: &Context, viewport: Viewport) -> Vec<three_d::Gm<three_d_text_builder::TextMesh, three_d_text_builder::TextMaterial>> {
        let mut text_refs = Vec::new();
        
        // Set viewport for proper text positioning
        self.text_builder.set_viewport(viewport);
        
        // Create TextRef objects for each queued text
        for queued_text in &self.queued_texts {
            let text_ref = three_d_text_builder::TextRef {
                text: &queued_text.text,
                size: queued_text.size,
                color: three_d::Srgba::new(
                    (queued_text.color[0] * 255.0) as u8,
                    (queued_text.color[1] * 255.0) as u8,
                    (queued_text.color[2] * 255.0) as u8,
                    (queued_text.color[3] * 255.0) as u8,
                ),
                position: three_d_text_builder::TextPosition::Pixels(three_d::vec2(queued_text.x, queued_text.y)),
                ..Default::default()
            };
            text_refs.push(text_ref);
        }
        
        // Build text models using the proper API
        if !text_refs.is_empty() {
            self.text_builder.build(context, &text_refs).collect()
        } else {
            Vec::new()
        }
    }
}

pub struct MainScene {
    camera: Camera,
    user_pitch_line: Gm<Line, ColorMaterial>,
    user_pitch_line_material: ColorMaterial,
    light: AmbientLight,
    pub tuning_lines: TuningLines,
    text_renderer: TextRenderer,
    context: Context,
    pitch_detected: bool,
    current_scheme: ColorScheme,
    user_pitch_line_thickness: f32,
    user_pitch_line_alpha: f32,
    volume_peak: bool,
    cents_offset: f32,
    // Background texture system: pre-rendered texture with dark green background
    // These resources are automatically cleaned up by three-d's RAII when replaced or dropped
    background_texture: Texture2DRef,
    background_depth_texture: DepthTexture2D,
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
    
    fn create_background_texture(context: &Context, viewport: Viewport, text_renderer: &mut TextRenderer) -> Result<(Texture2DRef, DepthTexture2D, Gm<Rectangle, ColorMaterial>), String> {
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
        
        // Create depth texture for the render target
        let mut depth_texture = DepthTexture2D::new::<f32>(
            context,
            viewport.width as u32,
            viewport.height as u32,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Create render target with the texture and render the "A" character
        {
            let render_target = RenderTarget::new(
                background_texture.as_color_target(None),
                depth_texture.as_depth_target(),
            );
            
            // Clear with current theme background color
            let scheme = get_current_color_scheme();
            let [r, g, b] = scheme.background;
            render_target.clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0));
            
            // Set up camera for off-screen rendering
            let camera = Camera::new_2d(viewport);
            
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
        let background_material = ColorMaterial {
            color: Srgba::WHITE, // White tint to show texture as-is
            texture: Some(background_texture_ref.clone()),
            is_transparent: false,
            render_states: RenderStates::default(),
        };
        
        let background_quad = Gm::new(background_rectangle, background_material);
        
        Ok((background_texture_ref, depth_texture, background_quad))
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
            viewport.width as u32,
            viewport.height as u32,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Create new depth texture for the render target
        let mut depth_texture = DepthTexture2D::new::<f32>(
            &self.context,
            viewport.width as u32,
            viewport.height as u32,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Create render target and render content to the texture
        {
            let render_target = RenderTarget::new(
                background_texture.as_color_target(None),
                depth_texture.as_depth_target(),
            );
            
            // Clear with current theme background color
            let scheme = get_current_color_scheme();
            let [r, g, b] = scheme.background;
            render_target.clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0));
            
            // Set up camera for off-screen rendering
            let camera = Camera::new_2d(viewport);
            
            // Render tuning lines to background texture
            let tuning_lines_vec: Vec<&dyn Object> = self.tuning_lines.lines()
                .map(|line| line as &dyn Object)
                .collect();
            
            if !tuning_lines_vec.is_empty() {
                render_target.render(&camera, tuning_lines_vec, &[&self.light]);
            }
            
            // Clear and queue note labels
            self.text_renderer.clear_queue();
            self.tuning_lines.render_note_labels(
                &mut self.text_renderer,
                self.volume_peak,
                self.cents_offset
            );
            
            // Render text models to background texture
            let text_models = self.text_renderer.create_text_models(&self.context, viewport);
            if !text_models.is_empty() {
                let text_objects: Vec<&dyn Object> = text_models.iter()
                    .map(|model| model as &dyn Object)
                    .collect();
                render_target.render(&camera, text_objects, &[&self.light]);
            }
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
        
        // Wrap texture in Texture2DRef for shared ownership
        let background_texture_ref = Texture2DRef::from_texture(background_texture);
        
        // Create material with the background texture
        let background_material = ColorMaterial {
            color: Srgba::WHITE,
            texture: Some(background_texture_ref.clone()),
            is_transparent: false,
            render_states: RenderStates::default(),
        };
        
        let background_quad = Gm::new(background_rectangle, background_material);
        
        // Replace the old background resources with the new ones
        self.background_texture = background_texture_ref;
        self.background_depth_texture = depth_texture;
        self.background_quad = background_quad;
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
        let mut text_renderer = TextRenderer::new(context)?;
        
        // Create background texture with dark green background and background quad
        let (background_texture, background_depth_texture, background_quad) = Self::create_background_texture(context, viewport, &mut text_renderer)?;
        
        Ok(Self {
            camera: Camera::new_2d(viewport),
            user_pitch_line: Gm::new(user_pitch_line, user_pitch_line_material.clone()),
            user_pitch_line_material,
            light: AmbientLight::new(context, 1.0, rgb_to_srgba(scheme.secondary)),
            tuning_lines,
            text_renderer,
            context: context.clone(),
            pitch_detected: false,
            current_scheme: scheme,
            user_pitch_line_thickness: initial_thickness,
            user_pitch_line_alpha: initial_alpha,
            volume_peak: initial_volume_peak,
            cents_offset: initial_cents_offset,
            background_texture,
            background_depth_texture,
            background_quad,
            presentation_context: None,
        })
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        let current_viewport = self.camera.viewport();
        
        // Check if viewport size actually changed
        let size_changed = current_viewport.width != viewport.width || current_viewport.height != viewport.height;
        
        // Always update camera viewport
        self.camera.set_viewport(viewport);
        
        // Only recreate background texture if size changed
        if size_changed {
            // Recreate background texture with new viewport size
            match Self::create_background_texture(&self.context, viewport, &mut self.text_renderer) {
                Ok((new_background_texture, new_background_depth_texture, new_background_quad)) => {
                    // Replace old resources with new ones
                    // The old textures and render targets will be automatically cleaned up
                    // when they go out of scope due to RAII in three-d
                    self.background_texture = new_background_texture;
                    self.background_depth_texture = new_background_depth_texture;
                    self.background_quad = new_background_quad;
                },
                Err(e) => {
                    // Log error but continue with old background texture
                    crate::common::dev_log!("Failed to recreate background texture on viewport change: {}", e);
                }
            }
        }
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
        self.tuning_lines.regular_material = create_color_material(rgb_to_srgba(scheme.muted), false);
        self.tuning_lines.octave_material = create_color_material(rgb_to_srgba(scheme.muted), false);
        
        // Clear and recreate all tuning lines with new material
        // They will be recreated with correct positions and thickness on next update_lines call
        self.tuning_lines.lines.clear();
        self.tuning_lines.midi_notes.clear();
        self.tuning_lines.y_positions.clear();
        self.tuning_lines.thicknesses.clear();
        
        // Update ambient light
        self.light = AmbientLight::new(&self.context, 1.0, rgb_to_srgba(scheme.secondary));
    }
    
    pub fn render(&mut self, screen: &mut RenderTarget) {
        // Check for theme changes
        let scheme = get_current_color_scheme();
        if scheme != self.current_scheme {
            self.current_scheme = scheme.clone();
            self.refresh_colors();
        }
        
        let bg = scheme.background;
        screen.clear(ClearState::color_and_depth(bg[0], bg[1], bg[2], 1.0, 1.0));

        // Render background quad with pre-rendered tuning lines and note labels when presentation context is active
        if self.presentation_context.is_some() {
            screen.render(
                &self.camera,
                &[&self.background_quad],
                &[&self.light],
            );
        }

        // Render only the user pitch line for real-time updates
        screen.render(
            &self.camera,
            &[&self.user_pitch_line],
            &[&self.light],
        );
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32, pitch_detected: bool, clarity: Option<f32>, cents_offset: f32) {
        self.pitch_detected = pitch_detected;
        self.cents_offset = cents_offset;
        if pitch_detected {
            let y = interval_to_screen_y_position(interval, viewport.height as f32, crate::web::main_scene_ui::get_current_zoom_factor());
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
    
    /// Update the presentation context with new root note, tuning system, and scale.
    /// Also re-renders the background texture when the presentation context changes.
    pub fn update_presentation_context(&mut self, context: &crate::shared_types::PresentationContext, viewport: Viewport) {
        if self.presentation_context.as_ref() == Some(context) {
            return;
        }
        
        self.presentation_context = Some(context.clone());
        
        // Calculate tuning line positions and update them
        let tuning_line_data = self.get_tuning_line_positions(
            context.root_note,
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
        root_note_midi: MidiNote,
        tuning_system: TuningSystem,
        scale: &Scale,
        viewport: Viewport,
    ) -> Vec<(f32, MidiNote, f32)> {
        let root_frequency = crate::music_theory::midi_note_to_standard_frequency(root_note_midi);
        
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
        
        // Add center line (root note, 0 semitones)
        if crate::shared_types::semitone_in_scale(*scale, 0) {
            // Root frequency stays at interval 0.0 (log2(1) = 0)
            let interval = 0.0;
            let y_position = interval_to_screen_y_position(
                interval,
                viewport.height as f32,
                crate::web::main_scene_ui::get_current_zoom_factor(),
            );
            let thickness = get_thickness(0);
            line_data.push((y_position, root_note_midi, thickness));
        }
        
        // Add intervals above root: +1 to +12 semitones
        for semitone in 1..=12 {
            // Only show intervals that are in the current scale
            if crate::shared_types::semitone_in_scale(*scale, semitone) {
                let frequency = crate::music_theory::interval_frequency(
                    tuning_system,
                    root_frequency,
                    semitone,
                );
                let interval = (frequency / root_frequency).log2();
                let y_position = interval_to_screen_y_position(
                    interval,
                    viewport.height as f32,
                    crate::web::main_scene_ui::get_current_zoom_factor(),
                );
                let midi_note = (root_note_midi as i32 + semitone).clamp(0, 127) as MidiNote;
                let thickness = get_thickness(semitone);
                line_data.push((y_position, midi_note, thickness));
            }
        }
        
        // Add intervals below root: -12 to -1 semitones
        for semitone in -12..=-1 {
            // Only show intervals that are in the current scale
            if crate::shared_types::semitone_in_scale(*scale, semitone) {
                let frequency = crate::music_theory::interval_frequency(
                    tuning_system,
                    root_frequency,
                    semitone,
                );
                let interval = (frequency / root_frequency).log2();
                let y_position = interval_to_screen_y_position(
                    interval,
                    viewport.height as f32,
                    crate::web::main_scene_ui::get_current_zoom_factor(),
                );
                let midi_note = (root_note_midi as i32 + semitone).clamp(0, 127) as MidiNote;
                let thickness = get_thickness(semitone);
                line_data.push((y_position, midi_note, thickness));
            }
        }
        
        line_data
    }
    
}

