#![cfg(target_arch = "wasm32")]

use three_d::*;

/// Width of the data texture used for historical data
pub const DATA_TEXTURE_WIDTH: f32 = 512.0;

// Simple material that uses our custom shader
pub struct BackgroundShaderMaterial {
    pub texture: Option<Texture2DRef>,
    pub data_texture: Option<Texture2DRef>,
    pub data_index: i32,
    pub data_texture_width: f32,
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
            uniform int dataIndex;
            uniform float dataTextureWidth;

            in vec2 uvs;
            out vec4 fragColor;

            void main() {
                vec4 texColor = texture(backgroundTexture, uvs);

                // Map screen x coordinate to texture coordinate with offset so latest entry appears at right
                // uvs.x goes from 0 to 1, we want to map it so current dataIndex appears at x=1
                float normalizedIndex = float(dataIndex) / dataTextureWidth;
                float u = mod(uvs.x + normalizedIndex, 1.0);

                vec4 data = texture(dataTexture, vec2(u, 0.5));
                float detected = data.r;
                float pitch = data.g;

                // Magenta tint when detected, only below the pitch line
                float magentaTint = 0.3 * detected * step(uvs.y, pitch);
                fragColor = texColor + vec4(magentaTint, 0.0, magentaTint, 0.0);
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
        program.use_uniform("dataIndex", self.data_index);
        program.use_uniform("dataTextureWidth", self.data_texture_width);
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
    data_values: [f32; 2], // Store detected, pitch
}

impl BackgroundShader {
    pub fn new(context: &Context) -> Result<Self, three_d::CoreError> {
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

        // Create initial 1x1 data texture with default values
        let data = vec![[0.0_f32, 0.5_f32]]; // detected, pitch
        let data_texture = Texture2D::new(
            context,
            &CpuTexture {
                data: TextureData::RgF32(data),
                width: 1,
                height: 1,
                ..Default::default()
            },
        );

        let material = BackgroundShaderMaterial {
            texture: None,
            data_texture: Some(data_texture.into()),
            data_index: 0,
            data_texture_width: DATA_TEXTURE_WIDTH,
        };

        Ok(Self {
            mesh: Gm::new(mesh, material),
            context: context.clone(),
            data_values: [0.0, 0.5],
        })
    }

    pub fn update(&mut self, _delta_time: f32) {
        // No longer needed since we removed time-based animation
    }

    pub fn set_background_texture(&mut self, texture: Texture2DRef) {
        self.mesh.material.texture = Some(texture);
    }

    pub fn set_detected(&mut self, detected: f32) {
        self.data_values[0] = detected;
        self.update_data_texture();
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.data_values[1] = pitch;
        self.update_data_texture();
    }


    fn update_data_texture(&mut self) {
        let data = vec![[self.data_values[0], self.data_values[1]]];
        let data_texture = Texture2D::new(
            &self.context,
            &CpuTexture {
                data: TextureData::RgF32(data),
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
        self.mesh.material.data_texture = Some(data_texture.into());
    }

    pub fn mesh(&self) -> &Gm<Mesh, BackgroundShaderMaterial> {
        &self.mesh
    }
}