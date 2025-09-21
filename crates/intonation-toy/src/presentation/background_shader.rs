#![cfg(target_arch = "wasm32")]

use three_d::*;
use crate::app_config::{NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN};

/// Width of the data texture used for historical data
pub const DATA_TEXTURE_WIDTH: usize = 512;

// Simple material that uses our custom shader
pub struct BackgroundShaderMaterial {
    pub texture: Option<Texture2DRef>,
    pub highlight_texture: Option<Texture2DRef>,
    pub data_texture: Option<Texture2DRef>,
    pub left_margin: f32,
    pub right_margin: f32,
    pub tint_color: Vec3,
    pub current_pitch_color: Vec3,
    pub latest_cents_offset: f32,
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
            uniform sampler2D highlightTexture;
            uniform sampler2D dataTexture;
            uniform float leftMargin;
            uniform float rightMargin;
            uniform vec3 tintColor;
            uniform vec3 currentPitchColor;
            uniform float latestCentsOffset;

            in vec2 uvs;
            out vec4 fragColor;

            void main() {
                vec4 texColor = texture(backgroundTexture, uvs);
                vec4 highlightColor = texture(highlightTexture, uvs);

                // Check for accuracy using the uniform
                vec4 latestData = texture(dataTexture, vec2(1.0, 0.5));
                float latestDetected = latestData.r;
                float latestPitch = latestData.g;
                bool isAccurate = abs(latestCentsOffset) < 15.0;

                // Create a band around the latest pitch line (extends to full width including margins)
                float bandThickness = 0.02; // Adjust band thickness as needed
                float distanceFromLatestPitch = abs(uvs.y - latestPitch);
                bool isInPitchBand = distanceFromLatestPitch < bandThickness;

                // Choose base texture: highlight when latest data is accurate and in pitch band
                vec4 baseTexture = (latestDetected > 0.0 && isAccurate && isInPitchBand) ? highlightColor : texColor;

                // Check if we're within the margins for tinting
                float isWithinMargins = step(leftMargin, uvs.x) * step(uvs.x, 1.0 - rightMargin);

                if (isWithinMargins > 0.0) {
                    // Remap x coordinate to account for margins
                    // Map [leftMargin, 1-rightMargin] to [0, 1]
                    float mappedX = (uvs.x - leftMargin) / (1.0 - leftMargin - rightMargin);

                    // Sample the data texture for tinting logic
                    vec4 data = texture(dataTexture, vec2(mappedX, 0.5));
                    float detected = data.r;
                    float pitch = data.g;

                    // Apply tint when detected, only below the pitch line (using historical data)
                    float tintStrength = 0.3 * detected * step(uvs.y, pitch);
                    vec4 tintedBackground = baseTexture + vec4(tintColor * tintStrength, 0.0);

                    fragColor = tintedBackground;
                } else if (uvs.x > 1.0 - rightMargin) {
                    // Right margin area - check for current pitch
                    // Sample the rightmost data point to get the latest pitch
                    vec4 data = texture(dataTexture, vec2(1.0, 0.5));
                    float detected = data.r;
                    float pitch = data.g;
                    float centsOffset = data.b;

                    // Draw horizontal line at pitch level when detected
                    float lineThickness = 0.004; // Adjust thickness as needed
                    float isOnLine = detected * step(abs(uvs.y - pitch), lineThickness);

                    if (isOnLine > 0.0) {
                        // Colored line
                        float lineStrength = 0.5;
                        fragColor = baseTexture + vec4(currentPitchColor * lineStrength, 0.0);
                    } else {
                        fragColor = baseTexture;
                    }
                } else {
                    // Outside margins, use base texture (includes highlight band)
                    fragColor = baseTexture;
                }
            }
        "#.to_string()
    }

    fn use_uniforms(&self, program: &Program, _camera: &dyn Viewer, _lights: &[&dyn Light]) {
        if let Some(ref texture) = self.texture {
            program.use_texture("backgroundTexture", texture);
        }
        if let Some(ref highlight_texture) = self.highlight_texture {
            program.use_texture("highlightTexture", highlight_texture);
        }
        if let Some(ref data_texture) = self.data_texture {
            program.use_texture("dataTexture", data_texture);
        }
        program.use_uniform("leftMargin", self.left_margin);
        program.use_uniform("rightMargin", self.right_margin);
        program.use_uniform("tintColor", self.tint_color);
        program.use_uniform("currentPitchColor", self.current_pitch_color);
        program.use_uniform("latestCentsOffset", self.latest_cents_offset);
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
    data_buffer: Vec<[f32; 3]>, // Buffer of [detected, pitch, cents_offset] values, newest at end
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

        let data_buffer = vec![[0.0_f32, 0.5_f32, 0.0_f32]; DATA_TEXTURE_WIDTH];

        // Create initial data texture with the buffer
        let data_texture = Texture2D::new(
            context,
            &CpuTexture {
                data: TextureData::RgbF32(data_buffer.clone()),
                width: DATA_TEXTURE_WIDTH as u32,
                height: 1,
                wrap_s: Wrapping::ClampToEdge,
                wrap_t: Wrapping::ClampToEdge,
                ..Default::default()
            },
        );

        let material = BackgroundShaderMaterial {
            texture: None,
            highlight_texture: None,
            data_texture: Some(data_texture.into()),
            left_margin: NOTE_LINE_LEFT_MARGIN / viewport_width,
            right_margin: NOTE_LINE_RIGHT_MARGIN / viewport_width,
            tint_color: Vec3::new(1.0, 0.0, 1.0), // Default magenta
            current_pitch_color: Vec3::new(0.88, 0.80, 0.62), // Default accent (sand)
            latest_cents_offset: 0.0,
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

    pub fn set_highlight_texture(&mut self, texture: Texture2DRef) {
        self.mesh.material.highlight_texture = Some(texture);
    }

    pub fn add_data_point(&mut self, detected: f32, pitch: f32, cents_offset: f32) {
        // Shift all data left by removing first element
        self.data_buffer.remove(0);
        // Add new data point at the end
        self.data_buffer.push([detected, pitch, cents_offset]);
        self.update_data_texture();
    }

    pub fn set_margins(&mut self, left: f32, right: f32) {
        self.mesh.material.left_margin = left;
        self.mesh.material.right_margin = right;
    }

    pub fn set_tint_color(&mut self, color: Vec3) {
        self.mesh.material.tint_color = color;
    }

    pub fn set_extension_color(&mut self, color: Vec3) {
        self.mesh.material.current_pitch_color = color;
    }

    fn update_data_texture(&mut self) {
        // Data buffer already has oldest on left, newest on right
        // Just upload it directly to the texture
        let data_texture = Texture2D::new(
            &self.context,
            &CpuTexture {
                data: TextureData::RgbF32(self.data_buffer.clone()),
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