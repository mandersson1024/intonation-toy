#![cfg(target_arch = "wasm32")]

use three_d::*;
use crate::app_config::{NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN};

/// Width of the data texture used for historical data
pub const DATA_TEXTURE_WIDTH: usize = 512;

// Simple material that uses our custom shader
pub struct BackgroundShaderMaterial {
    pub texture: Option<Texture2DRef>,
    pub data_texture: Option<Texture2DRef>,
    pub left_margin: f32,
    pub right_margin: f32,
}

impl Material for BackgroundShaderMaterial {
    fn id(&self) -> EffectMaterialId {
        // Return a unique ID for this material type
        EffectMaterialId(0x1234)
    }

    fn fragment_shader_source(&self, _lights: &[&dyn Light]) -> String {
        // Return our custom fragment shader with magenta tint when detected
        r#"
            uniform sampler2D backgroundTexture;
            uniform sampler2D dataTexture;
            uniform float leftMargin;
            uniform float rightMargin;

            in vec2 uvs;
            out vec4 fragColor;

            void main() {
                vec4 texColor = texture(backgroundTexture, uvs);

                // Check if we're within the margins for tinting
                float isWithinMargins = step(leftMargin, uvs.x) * step(uvs.x, 1.0 - rightMargin);

                if (isWithinMargins > 0.0) {
                    // Remap x coordinate to account for margins
                    // Map [leftMargin, 1-rightMargin] to [0, 1]
                    float mappedX = (uvs.x - leftMargin) / (1.0 - leftMargin - rightMargin);

                    // Sample the data texture
                    vec4 data = texture(dataTexture, vec2(mappedX, 0.5));
                    float detected = data.r;
                    float pitch = data.g;

                    // Magenta tint when detected, only below the pitch line
                    float magentaTint = 0.3 * detected * step(uvs.y, pitch);

                    vec4 tintedBackground = texColor + vec4(magentaTint, 0.0, magentaTint, 0.0);

                    // Blend the yellow line over the background
                    fragColor = tintedBackground;
                } else {
                    // Outside margins, just show the background texture
                    fragColor = texColor;
                }
            }
        "#.to_string()
    }

    fn use_uniforms(&self, program: &Program, _camera: &dyn Viewer, _lights: &[&dyn Light]) {
        if let Some(ref texture) = self.texture {
            program.use_texture("backgroundTexture", texture);
        }
        if let Some(ref data_texture) = self.data_texture {
            program.use_texture("dataTexture", data_texture);
        }
        program.use_uniform("leftMargin", self.left_margin);
        program.use_uniform("rightMargin", self.right_margin);
    }

    fn render_states(&self) -> RenderStates {
        RenderStates {
            write_mask: WriteMask::COLOR,
            depth_test: DepthTest::LessOrEqual,
            ..Default::default()
        }
    }

    fn material_type(&self) -> MaterialType {
        MaterialType::Opaque
    }
}

pub struct BackgroundShader {
    mesh: Gm<Mesh, BackgroundShaderMaterial>,
    context: Context,
    data_buffer: Vec<[f32; 2]>, // Buffer of [detected, pitch] values, newest at end
}

impl BackgroundShader {
    pub fn new(context: &Context, viewport_width: f32) -> Result<Self, three_d::CoreError> {
        // Create a fullscreen quad mesh
        let positions = vec![
            Vec3::new(-1.0, -1.0, -0.999), // Closer to camera
            Vec3::new( 1.0, -1.0, -0.999),
            Vec3::new( 1.0,  1.0, -0.999),
            Vec3::new(-1.0,  1.0, -0.999),
        ];

        let uvs = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];

        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            uvs: Some(uvs),
            indices: Indices::U32(vec![0, 1, 2, 0, 2, 3]),
            ..Default::default()
        };

        let mesh = Mesh::new(context, &cpu_mesh);

        let data_buffer = vec![[0.0_f32, 0.5_f32]; DATA_TEXTURE_WIDTH];

        // Create initial data texture with the buffer
        let data_texture = Texture2D::new(
            context,
            &CpuTexture {
                data: TextureData::RgF32(data_buffer.clone()),
                width: DATA_TEXTURE_WIDTH as u32,
                height: 1,
                wrap_s: Wrapping::ClampToEdge,
                wrap_t: Wrapping::ClampToEdge,
                ..Default::default()
            },
        );

        let material = BackgroundShaderMaterial {
            texture: None,
            data_texture: Some(data_texture.into()),
            left_margin: NOTE_LINE_LEFT_MARGIN / viewport_width,
            right_margin: NOTE_LINE_RIGHT_MARGIN / viewport_width,
        };

        Ok(Self {
            mesh: Gm::new(mesh, material),
            context: context.clone(),
            data_buffer,
        })
    }

    pub fn update(&mut self, _delta_time: f32) {
        // No longer needed since we removed time-based animation
    }

    pub fn set_background_texture(&mut self, texture: Texture2DRef) {
        self.mesh.material.texture = Some(texture);
    }

    pub fn add_data_point(&mut self, detected: f32, pitch: f32) {
        // Shift all data left by removing first element
        self.data_buffer.remove(0);
        // Add new data point at the end
        self.data_buffer.push([detected, pitch]);
        self.update_data_texture();
    }

    pub fn set_margins(&mut self, left: f32, right: f32) {
        self.mesh.material.left_margin = left;
        self.mesh.material.right_margin = right;
    }

    fn update_data_texture(&mut self) {
        // Data buffer already has oldest on left, newest on right
        // Just upload it directly to the texture
        let data_texture = Texture2D::new(
            &self.context,
            &CpuTexture {
                data: TextureData::RgF32(self.data_buffer.clone()),
                width: self.data_buffer.len() as u32,
                height: 1,
                wrap_s: Wrapping::ClampToEdge,
                wrap_t: Wrapping::ClampToEdge,
                ..Default::default()
            },
        );
        self.mesh.material.data_texture = Some(data_texture.into());
    }

    pub fn mesh(&self) -> &Gm<Mesh, BackgroundShaderMaterial> {
        &self.mesh
    }
}