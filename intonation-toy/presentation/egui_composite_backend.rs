use std::collections::HashMap;
use three_d::*;
use crate::app_config::NOTE_LABEL_FONT_SIZE;

/// Queue item for text to be rendered
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
    crate::common::dev_log!("TEXT_DEBUG: Converting mesh with {} vertices, {} indices", mesh.vertices.len(), mesh.indices.len());
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        crate::common::dev_log!("TEXT_DEBUG: Skipping empty mesh");
        return None;
    }

    // Convert vertices from egui format to three-d format
    let positions: Vec<Vec3> = mesh.vertices.iter().enumerate().map(|(i, v)| {
        let converted_pos = Vec3::new(
            v.pos.x * pixels_per_point,
            (viewport.height as f32 / pixels_per_point - v.pos.y) * pixels_per_point, // Flip Y coordinate
            0.0
        );
        if i < 3 { // Log first 3 vertices for debugging
            crate::common::dev_log!("TEXT_DEBUG: Vertex {}: egui({}, {}) -> three_d({}, {})", i, v.pos.x, v.pos.y, converted_pos.x, converted_pos.y);
        }
        converted_pos
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
    
    crate::common::dev_log!("TEXT_DEBUG: Creating GPU mesh...");
    let gpu_mesh = Mesh::new(context, &cpu_mesh);
    crate::common::dev_log!("TEXT_DEBUG: GPU mesh created successfully");
    
    // Create material with transparency for text rendering
    let material = ColorMaterial {
        color: Srgba::WHITE, // White to show texture as-is
        texture: Some(texture.clone()),
        is_transparent: true,
        render_states: RenderStates {
            blend: Blend::TRANSPARENCY,
            write_mask: WriteMask::COLOR,
            ..Default::default()
        },
    };
    
    crate::common::dev_log!("TEXT_DEBUG: Created mesh with {} vertices", mesh.vertices.len());
    Some(Box::new(Gm::new(gpu_mesh, material)) as Box<dyn Object>)
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
    /// Flag to track if glyph cache has been pre-loaded
    glyph_cache_preloaded: bool,
}

impl EguiCompositeBackend {
    /// Create a new EguiCompositeBackend instance
    /// 
    /// # Returns
    /// A Result containing the backend instance or an error string
    pub fn new() -> Result<Self, String> {
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
            glyph_cache_preloaded: false,
        })
    }
    
    /// Update texture atlas with new textures from egui
    fn update_texture_atlas(&mut self, context: &Context, textures_delta: egui::TexturesDelta) {
        crate::common::dev_log!("ATLAS_DEBUG: Updating texture atlas with {} textures to set, {} to free", 
                                textures_delta.set.len(), textures_delta.free.len());
        
        // Handle texture updates
        for (id, image_delta) in textures_delta.set {
            crate::common::dev_log!("ATLAS_DEBUG: Updating texture {:?}, pos: {:?}, image size: {}x{}", 
                                    id, image_delta.pos, 
                                    image_delta.image.width(), image_delta.image.height());
            
            if image_delta.pos.is_some() {
                // Partial update - skip it to avoid replacing the full texture with just the new glyph
                crate::common::dev_log!("ATLAS_DEBUG: Skipping partial update - would lose existing glyphs");
                continue;
            }
            
            // Only process full texture updates (pos: None)
            crate::common::dev_log!("ATLAS_DEBUG: Processing full texture update");
            let texture = self.create_texture_from_image(context, &image_delta.image);
            self.texture_atlas.insert(id, texture);
            crate::common::dev_log!("ATLAS_DEBUG: Texture {:?} inserted successfully", id);
        }
        
        // Handle texture removals
        for id in textures_delta.free {
            crate::common::dev_log!("ATLAS_DEBUG: Removing texture {:?}", id);
            self.texture_atlas.remove(&id);
        }
        
        crate::common::dev_log!("ATLAS_DEBUG: Atlas now contains {} textures", self.texture_atlas.len());
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

impl EguiCompositeBackend {
    pub fn queue_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.queued_texts.push(QueuedText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }
    
    pub fn clear_queue(&mut self) {
        self.queued_texts.clear();
    }
    
    pub fn create_text_models(&mut self, context: &Context, viewport: Viewport) -> Vec<Box<dyn Object>> {
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
        
        // Pre-load glyph cache with all possible note characters (one-time setup)
        // Must be done after begin_frame so fonts are available
        if !self.glyph_cache_preloaded {
            crate::common::dev_log!("ATLAS_DEBUG: Pre-loading glyph cache with all note characters");
            
            // Create invisible shapes to force glyph loading
            let mut preload_shapes = Vec::new();
            
            // Pre-render all possible note characters using current context
            self.egui_ctx.fonts(|f| {
                let color = egui::Color32::WHITE;
                
                // Pre-load characters at the exact size we use for note labels
                let sizes = [NOTE_LABEL_FONT_SIZE];
                let chars_to_preload = "CDEFGABb0123456789#";
                
                for &size in &sizes {
                    let font_id = egui::FontId::new(size, egui::FontFamily::Proportional);
                    for ch in chars_to_preload.chars() {
                        let galley = f.layout_no_wrap(ch.to_string(), font_id.clone(), color);
                        
                        // Create a text shape to force glyph atlas creation
                        preload_shapes.push(egui::Shape::Text(egui::epaint::TextShape {
                            pos: egui::Pos2::new(-1000.0, -1000.0), // Position off-screen
                            galley,
                            underline: egui::Stroke::NONE,
                            fallback_color: color,
                            override_text_color: Some(color),
                            opacity_factor: 1.0,
                            angle: 0.0,
                        }));
                    }
                }
                
                crate::common::dev_log!("ATLAS_DEBUG: Pre-loaded {} characters at {} sizes", 
                                        chars_to_preload.len(), sizes.len());
            });
            
            // Convert preload shapes to clipped shapes and tessellate to force atlas creation
            let preload_clipped_shapes: Vec<egui::epaint::ClippedShape> = preload_shapes.into_iter()
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
            
            // Force tessellation to create the font atlas
            let _preload_primitives = self.egui_ctx.tessellate(preload_clipped_shapes, self.pixels_per_point);
            
            // Mark cache as preloaded
            self.glyph_cache_preloaded = true;
            
            crate::common::dev_log!("ATLAS_DEBUG: Glyph cache pre-loading completed - atlas should now contain all characters");
        }
        
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
        
        
        // Convert egui primitives directly to three-d renderables
        // These will be rendered directly into the background texture alongside tuning lines
        let mut render_objects: Vec<Box<dyn Object>> = Vec::new();
        
        crate::common::dev_log!("ATLAS_DEBUG: Processing {} clipped primitives", clipped_primitives.len());
        
        for (i, primitive) in clipped_primitives.iter().enumerate() {
            if let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive {
                crate::common::dev_log!("ATLAS_DEBUG: Primitive {}: Mesh with texture_id {:?}, {} vertices", 
                                        i, mesh.texture_id, mesh.vertices.len());
                
                // Get texture for this mesh
                match self.texture_atlas.get(&mesh.texture_id) {
                    Some(texture) => {
                        crate::common::dev_log!("ATLAS_DEBUG: Found texture for mesh {}", i);
                        let texture = texture.clone();
                        
                        // Convert egui mesh to three-d format using helper function
                        match convert_egui_mesh_to_three_d(
                            context,
                            &mesh,
                            texture,
                            viewport,
                            self.pixels_per_point,
                        ) {
                            Some(render_object) => {
                                render_objects.push(render_object);
                                crate::common::dev_log!("ATLAS_DEBUG: Successfully converted mesh {} to render object", i);
                            }
                            None => {
                                crate::common::dev_log!("ATLAS_DEBUG: Failed to convert mesh {} to render object", i);
                            }
                        }
                    }
                    None => {
                        crate::common::dev_log!("ATLAS_DEBUG: No texture found for mesh {} with texture_id {:?}", i, mesh.texture_id);
                        crate::common::dev_log!("ATLAS_DEBUG: Available texture IDs: {:?}", 
                                                self.texture_atlas.keys().collect::<Vec<_>>());
                    }
                }
            }
        }
        
        render_objects
    }
}