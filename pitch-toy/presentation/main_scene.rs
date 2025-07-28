use three_d::{Context, Viewport, Camera, ColorMaterial, Mesh, CpuMesh, Positions, Indices, 
                AmbientLight, Srgba, ClearState, Gm, RenderTarget, vec3};

pub struct MainScene {
    camera: Camera,
    line_mesh: Gm<Mesh, ColorMaterial>,
    ambient: AmbientLight,
}

impl MainScene {
    pub fn new(context: &Context, viewport: Viewport) -> Self {
        // Create orthographic camera
        let camera = Camera::new_orthographic(
            viewport,
            vec3(0.0, 0.0, 2.0),  // eye position
            vec3(0.0, 0.0, 0.0),  // target position
            vec3(0.0, 1.0, 0.0),  // up direction
            2.0,                   // height
            0.1,                   // z_near
            10.0,                  // z_far
        );

        // Create line geometry spanning the full width at center
        // Use a thin rectangle as a workaround for line rendering
        let positions = vec![
            vec3(-1.0, -0.005, 0.0),  // bottom left
            vec3(1.0, -0.005, 0.0),   // bottom right
            vec3(1.0, 0.005, 0.0),    // top right
            vec3(-1.0, 0.005, 0.0),   // top left
        ];
        
        let indices = vec![0, 1, 2, 0, 2, 3];
        
        let mesh = Mesh::new(context, &CpuMesh {
            positions: Positions::F32(positions),
            indices: Indices::U32(indices),
            ..Default::default()
        });
        
        // Create material with visible color (white line)
        let material = ColorMaterial {
            color: Srgba::WHITE,
            ..Default::default()
        };
        
        let line_mesh = Gm::new(mesh, material);
        
        // Create ambient light
        let ambient = AmbientLight::new(context, 1.0, Srgba::WHITE);
        
        Self {
            camera,
            line_mesh,
            ambient,
        }
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }
    
    pub fn render(&self, screen: &mut RenderTarget) {
        // Clear the screen with black background
        screen.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));
        
        // Render the horizontal line
        screen.render(
            &self.camera,
            &self.line_mesh,
            &[&self.ambient],
        );
    }
}