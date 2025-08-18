/*!
 * Text Rendering Backends for MainScene
 * 
 * This module implements two egui-based text rendering backends:
 * 
 * 1. **EguiDirectBackend** - Uses egui's tessellation system for direct text rendering (default)
 * 2. **EguiCompositeBackend** - Uses egui for composite text rendering with two-stage approach
 * 
 * The backend can be switched at compile time by changing DEFAULT_TEXT_RENDERING_MODE
 * in app_config.rs, or at runtime using MainScene::set_text_rendering_mode().
 * 
 * Both backends leverage egui's font rendering capabilities and provide
 * high-quality text rendering, proper Unicode support, and integrated glyph atlas management.
 */

use three_d::{Blend, Camera, ClearState, ColorMaterial, Context, CpuMesh, Deg, Gm, Indices, Line, Mesh, Object, PhysicalPoint, Positions, RenderStates, RenderTarget, Srgba, Texture2DRef, Vec2, Vec3, Viewport, WriteMask};
use three_d::core::{Texture2D, DepthTexture2D, Interpolation, Wrapping};
use three_d::renderer::geometry::Rectangle;
use three_d::egui;
use std::collections::HashMap;
use crate::shared_types::{MidiNote, ColorScheme, TuningSystem, Scale};
use crate::theme::{get_current_color_scheme, rgb_to_srgba, rgb_to_srgba_with_alpha};
use crate::app_config::{USER_PITCH_LINE_THICKNESS_MIN, USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MIN, USER_PITCH_LINE_TRANSPARENCY_MAX, CLARITY_THRESHOLD, INTONATION_ACCURACY_THRESHOLD};

// Left margin to reserve space for note names
const NOTE_NAME_X_OFFSET: f32 = 30.0;
const NOTE_NAME_Y_OFFSET: f32 = 2.0;
const NOTE_LINE_LEFT_MARGIN: f32 = 64.0;
const NOTE_LINE_RIGHT_MARGIN: f32 = 20.0;

// Font size for note labels
const NOTE_LABEL_FONT_SIZE: f32 = 26.0;

// Line thickness values
pub const OCTAVE_LINE_THICKNESS: f32 = 8.0;
pub const REGULAR_LINE_THICKNESS: f32 = 4.0;
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

pub fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    // interval of [0.5, 2.0] means [-1, +1] octaves
    // Using fixed zoom factor of 0.92
    const ZOOM_FACTOR: f32 = 0.92;
    let y: f32 = viewport_height * (0.5 + interval * ZOOM_FACTOR * 0.5);
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
    pub fn render_note_labels(&self, text_backend: &mut dyn TextRenderingBackend) {
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

            text_backend.queue_text(&note_name, text_x, text_y, NOTE_LABEL_FONT_SIZE, [text_color[0], text_color[1], text_color[2], 1.0]);
        }
    }
}

/// Trait for abstracting text rendering functionality
/// Allows different text rendering backends to be used interchangeably
pub trait TextRenderingBackend {
    /// Queue text for rendering at the specified position
    /// 
    /// # Arguments
    /// * `text` - The text string to render
    /// * `x` - X position in screen coordinates
    /// * `y` - Y position in screen coordinates
    /// * `size` - Font size
    /// * `color` - RGBA color values in range [0.0, 1.0]
    fn queue_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]);
    
    /// Clear all queued text (typically called each frame)
    fn clear_queue(&mut self);
    
    /// Create renderable objects from queued text
    /// 
    /// # Arguments
    /// * `context` - The three-d Context for GPU resource creation
    /// * `viewport` - The current viewport for proper text positioning
    /// 
    /// # Returns
    /// A vector of renderable objects that can be rendered by the three-d renderer
    fn create_text_models(&mut self, context: &Context, viewport: Viewport) -> Vec<Box<dyn Object>>;
}


#[derive(Debug, Clone)]
struct QueuedText {
    text: String,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 4],
}

/// Helper function to convert egui mesh to three-d format
/// 
/// # Arguments
/// * `context` - The three-d Context for GPU resource creation
/// * `mesh` - The egui mesh to convert
/// * `texture` - The texture to apply to the mesh
/// * `viewport` - The viewport for coordinate conversion
/// * `pixels_per_point` - Scaling factor for coordinate conversion
/// 
/// # Returns
/// A three-d renderable object or None if conversion fails
fn convert_egui_mesh_to_three_d(
    context: &Context,
    mesh: &egui::epaint::Mesh,
    texture: Texture2DRef,
    viewport: Viewport,
    pixels_per_point: f32,
) -> Option<Box<dyn Object>> {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return None;
    }

    // Convert vertices from egui format to three-d format
    let positions: Vec<Vec3> = mesh.vertices.iter().map(|v| {
        Vec3::new(
            v.pos.x * pixels_per_point,
            (viewport.height as f32 / pixels_per_point - v.pos.y) * pixels_per_point, // Flip Y coordinate
            0.0
        )
    }).collect();
    
    let uvs: Vec<Vec2> = mesh.vertices.iter().map(|v| {
        Vec2::new(v.uv.x, v.uv.y)
    }).collect();
    
    let colors: Vec<Srgba> = mesh.vertices.iter().map(|v| {
        Srgba::new(
            v.color.r(),
            v.color.g(),
            v.color.b(),
            v.color.a(),
        )
    }).collect();
    
    // Convert indices
    let indices: Vec<u32> = mesh.indices.iter().copied().collect();
    
    // Create mesh
    let cpu_mesh = CpuMesh {
        positions: Positions::F32(positions),
        uvs: Some(uvs),
        colors: Some(colors),
        indices: Indices::U32(indices),
        ..Default::default()
    };
    
    let gpu_mesh = Mesh::new(context, &cpu_mesh);
    
    // Create material with transparency for text rendering
    let material = create_text_material(texture);
    
    Some(Box::new(Gm::new(gpu_mesh, material)) as Box<dyn Object>)
}

/// Create a ColorMaterial with appropriate settings for text rendering
/// 
/// # Arguments
/// * `texture` - The texture to apply to the text
/// 
/// # Returns
/// A ColorMaterial configured for transparent text rendering
fn create_text_material(texture: Texture2DRef) -> ColorMaterial {
    ColorMaterial {
        color: Srgba::WHITE,
        texture: Some(texture),
        is_transparent: true,
        render_states: RenderStates {
            blend: Blend::TRANSPARENCY,
            write_mask: WriteMask::COLOR,
            ..Default::default()
        },
    }
}

/// Validate viewport dimensions
/// 
/// # Arguments
/// * `viewport` - The viewport to validate
/// 
/// # Returns
/// true if viewport is valid, false otherwise
fn validate_viewport(viewport: Viewport) -> bool {
    viewport.width > 0 && viewport.height > 0
}

/// Backend implementation using egui's tessellation system for direct text rendering
/// 
/// This backend creates an off-screen egui context for text rendering and tessellation.
/// It uses egui's text layout and rendering capabilities to generate text shapes,
/// which are then tessellated into triangular meshes. These meshes are rendered
/// to an off-screen texture that is returned as a textured quad.
/// 
/// # Advantages over ThreeDTextBuilder
/// - Better font rendering quality with proper anti-aliasing
/// - Support for complex text layouts and Unicode
/// - Integrated glyph atlas management
/// - More efficient memory usage for text-heavy scenes
/// 
/// # Current Limitations
/// - The mesh rendering implementation is simplified
/// - Full egui mesh conversion to three-d format is not yet implemented
/// - Currently returns a single textured quad for all text
pub struct EguiDirectBackend {
    /// Off-screen egui context for text rendering
    egui_ctx: egui::Context,
    /// Queue of text items to render
    queued_texts: Vec<QueuedText>,
    /// Texture atlas cache for egui textures
    texture_atlas: HashMap<egui::TextureId, Texture2DRef>,
    /// Pixels per point for the display
    pixels_per_point: f32,
}

impl EguiDirectBackend {
    /// Create a new EguiDirectBackend instance
    /// 
    /// # Arguments
    /// * `context` - The three-d Context for GPU resource creation
    /// 
    /// # Returns
    /// A Result containing the backend instance or an error string
    pub fn new(_context: &Context) -> Result<Self, String> {
        // Create off-screen egui context
        let egui_ctx = egui::Context::default();
        
        // Load and configure Roboto font
        let mut fonts = egui::FontDefinitions::default();
        
        // Load the Roboto font from the static directory
        let roboto_font_data = include_bytes!("../static/fonts/Roboto-Regular.ttf");
        fonts.font_data.insert(
            "Roboto".to_owned(),
            egui::FontData::from_static(roboto_font_data)
        );
        
        // Configure font families to use Roboto
        fonts.families.entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Roboto".to_owned());
        
        fonts.families.entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "Roboto".to_owned());
        
        // Set the fonts in the context
        egui_ctx.set_fonts(fonts);
        
        Ok(Self {
            egui_ctx,
            queued_texts: Vec::new(),
            texture_atlas: HashMap::new(),
            pixels_per_point: 1.0,
        })
    }
    
    /// Update texture atlas with new textures from egui
    fn update_texture_atlas(&mut self, context: &Context, textures_delta: egui::TexturesDelta) {
        // Handle texture updates
        for (id, image_delta) in textures_delta.set {
            let texture = if let Some(_pos) = image_delta.pos {
                // Partial update - get existing texture and update region
                if let Some(existing) = self.texture_atlas.get(&id) {
                    existing.clone()
                } else {
                    // Create new texture if it doesn't exist
                    self.create_texture_from_image(context, &image_delta.image)
                }
            } else {
                // Full texture replacement
                self.create_texture_from_image(context, &image_delta.image)
            };
            
            self.texture_atlas.insert(id, texture);
        }
        
        // Handle texture removals
        for id in textures_delta.free {
            self.texture_atlas.remove(&id);
        }
    }
    
    /// Create a three-d texture from an egui image
    fn create_texture_from_image(&self, context: &Context, image: &egui::ImageData) -> Texture2DRef {
        use three_d::CpuTexture;
        match image {
            egui::ImageData::Color(color_image) => {
                // Convert egui Color32 image to RGBA bytes
                let pixels: Vec<[u8; 4]> = color_image.pixels.iter()
                    .map(|c| [c.r(), c.g(), c.b(), c.a()])
                    .collect();
                
                let cpu_texture = CpuTexture {
                    data: three_d::TextureData::RgbaU8(pixels),
                    width: color_image.width() as u32,
                    height: color_image.height() as u32,
                    ..Default::default()
                };
                
                let texture = Texture2D::new(context, &cpu_texture);
                Texture2DRef::from_texture(texture)
            },
            egui::ImageData::Font(font_image) => {
                // Convert font image (single channel) to RGBA
                let pixels: Vec<[u8; 4]> = font_image.pixels.iter()
                    .map(|coverage| {
                        let alpha = (coverage * 255.0) as u8;
                        [255, 255, 255, alpha]
                    })
                    .collect();
                
                let cpu_texture = CpuTexture {
                    data: three_d::TextureData::RgbaU8(pixels),
                    width: font_image.width() as u32,
                    height: font_image.height() as u32,
                    ..Default::default()
                };
                
                let texture = Texture2D::new(context, &cpu_texture);
                Texture2DRef::from_texture(texture)
            },
        }
    }
    
    /// Convert screen coordinates to egui coordinates
    fn screen_to_egui_pos(&self, x: f32, y: f32, viewport: Viewport) -> egui::Pos2 {
        // egui uses top-left origin, our screen coords use bottom-left
        // Also account for pixels_per_point scaling
        egui::Pos2 {
            x: x / self.pixels_per_point,
            y: (viewport.height as f32 - y) / self.pixels_per_point,
        }
    }
}

impl TextRenderingBackend for EguiDirectBackend {
    fn queue_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.queued_texts.push(QueuedText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }
    
    fn clear_queue(&mut self) {
        self.queued_texts.clear();
    }
    
    fn create_text_models(&mut self, context: &Context, viewport: Viewport) -> Vec<Box<dyn Object>> {
        if self.queued_texts.is_empty() {
            return Vec::new();
        }
        
        // Validate viewport dimensions
        if !validate_viewport(viewport) {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for text rendering");
            return Vec::new();
        }
        
        // Calculate pixels per point for the current display
        self.pixels_per_point = 1.0; // Could be adjusted for HiDPI displays
        
        // Begin egui frame
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(
                    viewport.width as f32 / self.pixels_per_point,
                    viewport.height as f32 / self.pixels_per_point,
                ),
            )),
            ..Default::default()
        };
        
        self.egui_ctx.begin_frame(raw_input);
        
        // Create text shapes for each queued text
        let mut shapes = Vec::new();
        for queued in &self.queued_texts {
            let pos = self.screen_to_egui_pos(queued.x, queued.y, viewport);
            
            // Convert color from float [0,1] to Color32
            let color = egui::Color32::from_rgba_premultiplied(
                (queued.color[0] * 255.0) as u8,
                (queued.color[1] * 255.0) as u8,
                (queued.color[2] * 255.0) as u8,
                (queued.color[3] * 255.0) as u8,
            );
            
            // Create text shape
            let galley = self.egui_ctx.fonts(|f| {
                f.layout_no_wrap(
                    queued.text.clone(),
                    egui::FontId::new(queued.size, egui::FontFamily::Proportional),
                    color,
                )
            });
            
            shapes.push(egui::Shape::Text(egui::epaint::TextShape {
                pos,
                galley,
                underline: egui::Stroke::NONE,
                fallback_color: color,
                override_text_color: Some(color),
                opacity_factor: 1.0,
                angle: 0.0,
            }));
        }
        
        // End frame and get output
        let output = self.egui_ctx.end_frame();
        
        // Convert shapes to clipped shapes for tessellation
        let clipped_shapes: Vec<egui::epaint::ClippedShape> = shapes.into_iter()
            .map(|shape| egui::epaint::ClippedShape {
                clip_rect: egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(
                        viewport.width as f32 / self.pixels_per_point,
                        viewport.height as f32 / self.pixels_per_point,
                    ),
                ),
                shape,
            })
            .collect();
        
        // Tessellate the shapes
        let clipped_primitives = self.egui_ctx.tessellate(clipped_shapes, self.pixels_per_point);
        
        // Update texture atlas with any new textures
        self.update_texture_atlas(context, output.textures_delta);
        
        // If no primitives were generated, return empty
        if clipped_primitives.is_empty() {
            return Vec::new();
        }
        
        // Create off-screen render target
        let mut render_texture = Texture2D::new_empty::<[u8; 4]>(
            context,
            viewport.width,
            viewport.height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        let mut depth_texture = DepthTexture2D::new::<f32>(
            context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Render tessellated text to texture
        {
            let render_target = RenderTarget::new(
                render_texture.as_color_target(None),
                depth_texture.as_depth_target(),
            );
            
            // Clear with transparent background
            render_target.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0));
            
            let camera = Camera::new_2d(viewport);
            
            // Convert egui primitives to three-d renderables and render them
            let mut render_objects: Vec<Box<dyn Object>> = Vec::new();
            
            for primitive in clipped_primitives {
                if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                    // Get texture for this mesh
                    if let Some(texture) = self.texture_atlas.get(&mesh.texture_id).cloned() {
                        // Convert egui mesh to three-d format using helper function
                        if let Some(render_object) = convert_egui_mesh_to_three_d(
                            context,
                            &mesh,
                            texture,
                            viewport,
                            self.pixels_per_point,
                        ) {
                            render_objects.push(render_object);
                        }
                    }
                }
            }
            
            // Render all text meshes to the off-screen texture
            if !render_objects.is_empty() {
                let render_refs: Vec<&dyn Object> = render_objects.iter()
                    .map(|obj| obj.as_ref())
                    .collect();
                render_target.render(&camera, render_refs, &[]);
            }
        }
        
        // Create final textured quad with the rendered text
        let texture_ref = Texture2DRef::from_texture(render_texture);
        
        let quad = Rectangle::new(
            context,
            (viewport.width as f32 * 0.5, viewport.height as f32 * 0.5),
            Deg(0.0),
            viewport.width as f32,
            viewport.height as f32,
        );
        
        let material = ColorMaterial {
            color: Srgba::WHITE,
            texture: Some(texture_ref),
            is_transparent: true,
            render_states: RenderStates {
                depth_test: three_d::DepthTest::Always,
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        };
        
        vec![Box::new(Gm::new(quad, material)) as Box<dyn Object>]
    }
}

/// Backend implementation using egui's composite text rendering with two-stage approach
/// 
/// This backend implements a two-stage rendering process:
/// 1. First stage: Render text to a separate texture using egui's off-screen rendering
/// 2. Second stage: Composite/blend the text texture onto a background texture
/// 
/// # Advantages over EguiDirect
/// - Better text quality through separate text rendering stage
/// - Improved compositing control for blending text with backgrounds
/// - More flexible texture management for complex layouts
/// - Ability to apply post-processing effects to text layers
/// 
/// # Architecture
/// - Maintains separate egui context for off-screen text rendering
/// - Uses texture composition techniques for blending
/// - Returns composited result as renderable objects
pub struct EguiCompositeBackend {
    /// Off-screen egui context for text rendering
    egui_ctx: egui::Context,
    /// Queue of text items to render
    queued_texts: Vec<QueuedText>,
    /// Texture atlas cache for egui textures
    texture_atlas: HashMap<egui::TextureId, Texture2DRef>,
    /// Pixels per point for the display
    pixels_per_point: f32,
}

impl EguiCompositeBackend {
    /// Create a new EguiCompositeBackend instance
    /// 
    /// # Arguments
    /// * `context` - The three-d Context for GPU resource creation
    /// 
    /// # Returns
    /// A Result containing the backend instance or an error string
    pub fn new(_context: &Context) -> Result<Self, String> {
        // Create off-screen egui context
        let egui_ctx = egui::Context::default();
        
        // Load and configure Roboto font
        let mut fonts = egui::FontDefinitions::default();
        
        // Load the Roboto font from the static directory
        let roboto_font_data = include_bytes!("../static/fonts/Roboto-Regular.ttf");
        fonts.font_data.insert(
            "Roboto".to_owned(),
            egui::FontData::from_static(roboto_font_data)
        );
        
        // Configure font families to use Roboto
        fonts.families.entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Roboto".to_owned());
        
        fonts.families.entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "Roboto".to_owned());
        
        // Set the fonts in the context
        egui_ctx.set_fonts(fonts);
        
        Ok(Self {
            egui_ctx,
            queued_texts: Vec::new(),
            texture_atlas: HashMap::new(),
            pixels_per_point: 1.0,
        })
    }
    
    /// Update texture atlas with new textures from egui
    fn update_texture_atlas(&mut self, context: &Context, textures_delta: egui::TexturesDelta) {
        // Handle texture updates
        for (id, image_delta) in textures_delta.set {
            let texture = if let Some(_pos) = image_delta.pos {
                // Partial update - get existing texture and update region
                if let Some(existing) = self.texture_atlas.get(&id) {
                    existing.clone()
                } else {
                    // Create new texture if it doesn't exist
                    self.create_texture_from_image(context, &image_delta.image)
                }
            } else {
                // Full texture replacement
                self.create_texture_from_image(context, &image_delta.image)
            };
            
            self.texture_atlas.insert(id, texture);
        }
        
        // Handle texture removals
        for id in textures_delta.free {
            self.texture_atlas.remove(&id);
        }
    }
    
    /// Create a three-d texture from an egui image
    fn create_texture_from_image(&self, context: &Context, image: &egui::ImageData) -> Texture2DRef {
        use three_d::CpuTexture;
        match image {
            egui::ImageData::Color(color_image) => {
                // Convert egui Color32 image to RGBA bytes
                let pixels: Vec<[u8; 4]> = color_image.pixels.iter()
                    .map(|c| [c.r(), c.g(), c.b(), c.a()])
                    .collect();
                
                let cpu_texture = CpuTexture {
                    data: three_d::TextureData::RgbaU8(pixels),
                    width: color_image.width() as u32,
                    height: color_image.height() as u32,
                    ..Default::default()
                };
                
                let texture = Texture2D::new(context, &cpu_texture);
                Texture2DRef::from_texture(texture)
            },
            egui::ImageData::Font(font_image) => {
                // Convert font image (single channel) to RGBA
                let pixels: Vec<[u8; 4]> = font_image.pixels.iter()
                    .map(|coverage| {
                        let alpha = (coverage * 255.0) as u8;
                        [255, 255, 255, alpha]
                    })
                    .collect();
                
                let cpu_texture = CpuTexture {
                    data: three_d::TextureData::RgbaU8(pixels),
                    width: font_image.width() as u32,
                    height: font_image.height() as u32,
                    ..Default::default()
                };
                
                let texture = Texture2D::new(context, &cpu_texture);
                Texture2DRef::from_texture(texture)
            },
        }
    }
    
    /// Convert screen coordinates to egui coordinates
    fn screen_to_egui_pos(&self, x: f32, y: f32, viewport: Viewport) -> egui::Pos2 {
        // egui uses top-left origin, our screen coords use bottom-left
        // Also account for pixels_per_point scaling
        egui::Pos2 {
            x: x / self.pixels_per_point,
            y: (viewport.height as f32 - y) / self.pixels_per_point,
        }
    }
    
    /// Create background texture for compositing
    fn create_background_texture(&self, context: &Context, viewport: Viewport) -> Texture2DRef {
        use three_d::CpuTexture;
        
        // Create a simple background texture (transparent for now)
        let pixels = vec![[0u8, 0, 0, 0]; (viewport.width * viewport.height) as usize];
        
        let cpu_texture = CpuTexture {
            data: three_d::TextureData::RgbaU8(pixels),
            width: viewport.width,
            height: viewport.height,
            ..Default::default()
        };
        
        let texture = Texture2D::new(context, &cpu_texture);
        Texture2DRef::from_texture(texture)
    }
    
    /// Composite text texture onto background texture
    fn composite_textures(&self, context: &Context, viewport: Viewport, text_texture: Texture2DRef, background_texture: Texture2DRef) -> Texture2DRef {
        // Create a new texture for the composited result
        let mut composite_texture = Texture2D::new_empty::<[u8; 4]>(
            context,
            viewport.width,
            viewport.height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        let mut composite_depth_texture = DepthTexture2D::new::<f32>(
            context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Render both textures onto the composite texture
        {
            let composite_render_target = RenderTarget::new(
                composite_texture.as_color_target(None),
                composite_depth_texture.as_depth_target(),
            );
            
            // Clear with transparent background
            composite_render_target.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0));
            
            let camera = Camera::new_2d(viewport);
            
            // Create quads for both textures
            let quad_width = viewport.width as f32;
            let quad_height = viewport.height as f32;
            let quad_center = (quad_width * 0.5, quad_height * 0.5);
            
            // Background quad
            let background_quad = Rectangle::new(
                context,
                quad_center,
                Deg(0.0),
                quad_width,
                quad_height,
            );
            
            let background_material = ColorMaterial {
                color: Srgba::WHITE,
                texture: Some(background_texture),
                is_transparent: false,
                render_states: RenderStates::default(),
            };
            
            // Text quad (rendered on top with transparency)
            let text_quad = Rectangle::new(
                context,
                quad_center,
                Deg(0.0),
                quad_width,
                quad_height,
            );
            
            let text_material = ColorMaterial {
                color: Srgba::WHITE,
                texture: Some(text_texture),
                is_transparent: true,
                render_states: RenderStates {
                    blend: Blend::TRANSPARENCY,
                    write_mask: WriteMask::COLOR,
                    ..Default::default()
                },
            };
            
            let background_gm = Gm::new(background_quad, background_material);
            let text_gm = Gm::new(text_quad, text_material);
            
            // Render background first, then text on top
            composite_render_target
                .render(&camera, &[&background_gm], &[])
                .render(&camera, &[&text_gm], &[]);
        }
        
        Texture2DRef::from_texture(composite_texture)
    }
}

impl TextRenderingBackend for EguiCompositeBackend {
    fn queue_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.queued_texts.push(QueuedText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }
    
    fn clear_queue(&mut self) {
        self.queued_texts.clear();
    }
    
    fn create_text_models(&mut self, context: &Context, viewport: Viewport) -> Vec<Box<dyn Object>> {
        if self.queued_texts.is_empty() {
            return Vec::new();
        }
        
        // Validate viewport dimensions
        if !validate_viewport(viewport) {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for composite text rendering");
            return Vec::new();
        }
        
        // Stage 1: Render text to separate texture using egui
        
        // Calculate pixels per point for the current display
        self.pixels_per_point = 1.0; // Could be adjusted for HiDPI displays
        
        // Begin egui frame
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(
                    viewport.width as f32 / self.pixels_per_point,
                    viewport.height as f32 / self.pixels_per_point,
                ),
            )),
            ..Default::default()
        };
        
        self.egui_ctx.begin_frame(raw_input);
        
        // Create text shapes for each queued text
        let mut shapes = Vec::new();
        for queued in &self.queued_texts {
            let pos = self.screen_to_egui_pos(queued.x, queued.y, viewport);
            
            // Convert color from float [0,1] to Color32
            let color = egui::Color32::from_rgba_premultiplied(
                (queued.color[0] * 255.0) as u8,
                (queued.color[1] * 255.0) as u8,
                (queued.color[2] * 255.0) as u8,
                (queued.color[3] * 255.0) as u8,
            );
            
            // Create text shape
            let galley = self.egui_ctx.fonts(|f| {
                f.layout_no_wrap(
                    queued.text.clone(),
                    egui::FontId::new(queued.size, egui::FontFamily::Proportional),
                    color,
                )
            });
            
            shapes.push(egui::Shape::Text(egui::epaint::TextShape {
                pos,
                galley,
                underline: egui::Stroke::NONE,
                fallback_color: color,
                override_text_color: Some(color),
                opacity_factor: 1.0,
                angle: 0.0,
            }));
        }
        
        // End frame and get output
        let output = self.egui_ctx.end_frame();
        
        // Convert shapes to clipped shapes for tessellation
        let clipped_shapes: Vec<egui::epaint::ClippedShape> = shapes.into_iter()
            .map(|shape| egui::epaint::ClippedShape {
                clip_rect: egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(
                        viewport.width as f32 / self.pixels_per_point,
                        viewport.height as f32 / self.pixels_per_point,
                    ),
                ),
                shape,
            })
            .collect();
        
        // Tessellate the shapes
        let clipped_primitives = self.egui_ctx.tessellate(clipped_shapes, self.pixels_per_point);
        
        // Update texture atlas with any new textures
        self.update_texture_atlas(context, output.textures_delta);
        
        // Create text texture and render tessellated text to it
        let mut text_texture = Texture2D::new_empty::<[u8; 4]>(
            context,
            viewport.width,
            viewport.height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        let mut text_depth_texture = DepthTexture2D::new::<f32>(
            context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        // Stage 1: Render text to separate texture
        {
            let text_render_target = RenderTarget::new(
                text_texture.as_color_target(None),
                text_depth_texture.as_depth_target(),
            );
            
            // Clear with transparent background
            text_render_target.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0));
            
            let camera = Camera::new_2d(viewport);
            
            // Convert egui primitives to three-d renderables and render them
            let mut text_render_objects: Vec<Box<dyn Object>> = Vec::new();
            
            for primitive in clipped_primitives {
                if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                    // Get texture for this mesh
                    if let Some(texture) = self.texture_atlas.get(&mesh.texture_id).cloned() {
                        // Convert egui mesh to three-d format using helper function
                        if let Some(render_object) = convert_egui_mesh_to_three_d(
                            context,
                            &mesh,
                            texture,
                            viewport,
                            self.pixels_per_point,
                        ) {
                            text_render_objects.push(render_object);
                        }
                    }
                }
            }
            
            // Render all text meshes to the text texture
            if !text_render_objects.is_empty() {
                let render_refs: Vec<&dyn Object> = text_render_objects.iter()
                    .map(|obj| obj.as_ref())
                    .collect();
                text_render_target.render(&camera, render_refs, &[]);
            }
        }
        
        let text_texture_ref = Texture2DRef::from_texture(text_texture);
        
        // Stage 2: Create background texture and composite
        let background_texture = self.create_background_texture(context, viewport);
        let final_texture = self.composite_textures(context, viewport, text_texture_ref, background_texture);
        
        // Create final textured quad with the composited result
        let quad = Rectangle::new(
            context,
            (viewport.width as f32 * 0.5, viewport.height as f32 * 0.5),
            Deg(0.0),
            viewport.width as f32,
            viewport.height as f32,
        );
        
        let material = ColorMaterial {
            color: Srgba::WHITE,
            texture: Some(final_texture),
            is_transparent: true,
            render_states: RenderStates {
                depth_test: three_d::DepthTest::Always,
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        };
        
        vec![Box::new(Gm::new(quad, material)) as Box<dyn Object>]
    }
}



pub struct MainScene {
    camera: Camera,
    user_pitch_line: Gm<Line, ColorMaterial>,
    user_pitch_line_material: ColorMaterial,
    pub tuning_lines: TuningLines,
    text_backend: Box<dyn TextRenderingBackend>,
    current_text_rendering_mode: crate::app_config::TextRenderingMode,
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
            self.tuning_lines.render_note_labels(&mut *self.text_backend);
            let text_models = self.text_backend.create_text_models(&self.context, viewport);
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
        
        // Create text backend using the configured mode
        let text_backend: Box<dyn TextRenderingBackend> = match crate::app_config::DEFAULT_TEXT_RENDERING_MODE {
            crate::app_config::TextRenderingMode::EguiDirect => {
                Box::new(EguiDirectBackend::new(context)?)
            }
            crate::app_config::TextRenderingMode::EguiComposite => {
                Box::new(EguiCompositeBackend::new(context)?)
            }
        };
        let current_text_rendering_mode = crate::app_config::DEFAULT_TEXT_RENDERING_MODE;

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
            current_text_rendering_mode,
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
        self.tuning_lines.regular_material = create_color_material(rgb_to_srgba(scheme.muted), false);
        self.tuning_lines.octave_material = create_color_material(rgb_to_srgba(scheme.muted), false);
        
        // Clear and recreate all tuning lines with new material
        // They will be recreated with correct positions and thickness on next update_lines call
        self.tuning_lines.lines.clear();
        self.tuning_lines.midi_notes.clear();
        self.tuning_lines.y_positions.clear();
        self.tuning_lines.thicknesses.clear();
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
            self.tuning_lines.lines.clear();
            self.tuning_lines.midi_notes.clear();
            self.tuning_lines.y_positions.clear();
            self.tuning_lines.thicknesses.clear();
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
    
    /// Switch to a different text rendering backend
    pub fn set_text_rendering_mode(&mut self, mode: crate::app_config::TextRenderingMode) -> Result<(), String> {
        self.text_backend = match mode {
            crate::app_config::TextRenderingMode::EguiDirect => {
                Box::new(EguiDirectBackend::new(&self.context)?)
            }
            crate::app_config::TextRenderingMode::EguiComposite => {
                Box::new(EguiCompositeBackend::new(&self.context)?)
            }
        };
        self.current_text_rendering_mode = mode;
        Ok(())
    }
    
    /// Get the current text rendering mode
    pub fn get_text_rendering_mode(&self) -> crate::app_config::TextRenderingMode {
        self.current_text_rendering_mode
    }
}

