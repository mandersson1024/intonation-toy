use three_d::*;

pub struct EguiTextBackend {
    egui_ctx: egui::Context,
    font_texture: Option<Texture2DRef>,
    glyphs_preloaded: bool,
}

impl EguiTextBackend {
    pub fn new() -> Result<Self, String> {
        let egui_ctx = egui::Context::default();
        
        let mut fonts = egui::FontDefinitions::default();
        let roboto_data = include_bytes!("../../static/fonts/Roboto-Regular.ttf");
        fonts.font_data.insert(
            "Roboto".to_owned(),
            egui::FontData::from_static(roboto_data)
        );
        
        fonts.families.entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Roboto".to_owned());
        
        fonts.families.entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "Roboto".to_owned());
        
        egui_ctx.set_fonts(fonts);
        
        Ok(Self {
            egui_ctx,
            font_texture: None,
            glyphs_preloaded: false,
        })
    }
    
    pub fn render_texts(&mut self, 
                       context: &Context,
                       viewport: Viewport,
                       texts: &[(String, f32, f32, f32, [f32; 4])]) -> Vec<Box<dyn Object>> {
        if texts.is_empty() || viewport.width == 0 || viewport.height == 0 {
            return Vec::new();
        }
        
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(viewport.width as f32, viewport.height as f32),
            )),
            ..Default::default()
        };
        
        self.egui_ctx.begin_pass(raw_input);
        
        if !self.glyphs_preloaded {
            self.preload_glyphs();
            self.glyphs_preloaded = true;
        }
        
        let mut shapes = Vec::new();
        for (text, x, y, size, color) in texts {
            let pos = egui::Pos2 {
                x: *x,
                y: viewport.height as f32 - y,
            };
            
            let egui_color = egui::Color32::from_rgba_premultiplied(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                (color[3] * 255.0) as u8,
            );
            
            let galley = self.egui_ctx.fonts(|f| {
                f.layout_no_wrap(
                    text.clone(),
                    egui::FontId::new(*size, egui::FontFamily::Proportional),
                    egui_color,
                )
            });
            
            shapes.push(egui::Shape::Text(egui::epaint::TextShape {
                pos,
                galley,
                underline: egui::Stroke::NONE,
                fallback_color: egui_color,
                override_text_color: Some(egui_color),
                opacity_factor: 1.0,
                angle: 0.0,
            }));
        }
        
        let output = self.egui_ctx.end_pass();
        
        let clipped_shapes: Vec<egui::epaint::ClippedShape> = shapes.into_iter()
            .map(|shape| egui::epaint::ClippedShape {
                clip_rect: egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(viewport.width as f32, viewport.height as f32),
                ),
                shape,
            })
            .collect();
        
        let primitives = self.egui_ctx.tessellate(clipped_shapes, 1.0);
        
        for (id, image_delta) in output.textures_delta.set {
            if matches!(id, egui::TextureId::Managed(_)) && image_delta.pos.is_none() {
                self.font_texture = Some(self.create_texture(context, &image_delta.image));
            }
        }
        
        let mut render_objects = Vec::new();
        
        for primitive in primitives {
            if let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive {
                if let Some(texture) = &self.font_texture {
                    if let Some(obj) = self.create_render_object(context, mesh, texture.clone(), viewport) {
                        render_objects.push(obj);
                    }
                }
            }
        }
        
        render_objects
    }
    
    fn create_texture(&self, context: &Context, image: &egui::ImageData) -> Texture2DRef {
        let (width, height, pixels) = match image {
            egui::ImageData::Color(img) => {
                let pixels: Vec<[u8; 4]> = img.pixels.iter()
                    .map(|c| [c.r(), c.g(), c.b(), c.a()])
                    .collect();
                (img.width() as u32, img.height() as u32, pixels)
            },
            egui::ImageData::Font(img) => {
                let pixels: Vec<[u8; 4]> = img.pixels.iter()
                    .map(|coverage| {
                        let alpha = (coverage * 255.0) as u8;
                        [255, 255, 255, alpha]
                    })
                    .collect();
                (img.width() as u32, img.height() as u32, pixels)
            },
        };
        
        let cpu_texture = CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width,
            height,
            ..Default::default()
        };
        
        Texture2DRef::from_texture(Texture2D::new(context, &cpu_texture))
    }
    
    fn create_render_object(&self,
                           context: &Context,
                           mesh: &egui::epaint::Mesh,
                           texture: Texture2DRef,
                           viewport: Viewport) -> Option<Box<dyn Object>> {
        if mesh.vertices.is_empty() || mesh.indices.is_empty() {
            return None;
        }
        
        let positions: Vec<Vec3> = mesh.vertices.iter()
            .map(|v| Vec3::new(
                v.pos.x,
                viewport.height as f32 - v.pos.y,
                0.0
            ))
            .collect();
        
        let uvs: Vec<Vec2> = mesh.vertices.iter()
            .map(|v| Vec2::new(v.uv.x, v.uv.y))
            .collect();
        
        let colors: Vec<Srgba> = mesh.vertices.iter()
            .map(|v| Srgba::new(v.color.r(), v.color.g(), v.color.b(), v.color.a()))
            .collect();
        
        let indices: Vec<u32> = mesh.indices.to_vec();
        
        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            uvs: Some(uvs),
            colors: Some(colors),
            indices: Indices::U32(indices),
            ..Default::default()
        };
        
        let gpu_mesh = Mesh::new(context, &cpu_mesh);
        
        let material = ColorMaterial {
            color: Srgba::WHITE,
            texture: Some(texture),
            is_transparent: true,
            render_states: RenderStates {
                blend: Blend::TRANSPARENCY,
                write_mask: WriteMask::COLOR,
                ..Default::default()
            },
        };
        
        Some(Box::new(Gm::new(gpu_mesh, material)))
    }
    
    fn preload_glyphs(&mut self) {
        let chars_to_preload = "ABCDEFGb#0123456789";
        
        use crate::app_config::NOTE_LABEL_FONT_SIZE;
        
        let mut preload_shapes = Vec::new();
        
        self.egui_ctx.fonts(|f| {
            let font_id = egui::FontId::new(NOTE_LABEL_FONT_SIZE, egui::FontFamily::Proportional);
            let color = egui::Color32::WHITE;
            
            for ch in chars_to_preload.chars() {
                let galley = f.layout_no_wrap(ch.to_string(), font_id.clone(), color);
                
                preload_shapes.push(egui::Shape::Text(egui::epaint::TextShape {
                    pos: egui::Pos2::new(-1000.0, -1000.0),
                    galley,
                    underline: egui::Stroke::NONE,
                    fallback_color: color,
                    override_text_color: Some(color),
                    opacity_factor: 1.0,
                    angle: 0.0,
                }));
            }
        });
        
        let clipped_shapes: Vec<egui::epaint::ClippedShape> = preload_shapes.into_iter()
            .map(|shape| egui::epaint::ClippedShape {
                clip_rect: egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(1000.0, 1000.0),
                ),
                shape,
            })
            .collect();
        
        let _ = self.egui_ctx.tessellate(clipped_shapes, 1.0);
    }
}

