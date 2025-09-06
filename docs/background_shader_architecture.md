# Background Quad Custom Shader Architecture

This document outlines the architecture for rendering the background quad using a custom shader material with the three_d API.

## Architecture Overview

### 1. Shader Program Creation

```rust
// Define vertex shader
let vertex_shader = "
    in vec3 position;
    uniform mat4 viewProjectionMatrix;
    out vec2 uv;
    
    void main() {
        uv = position.xy * 0.5 + 0.5;
        gl_Position = viewProjectionMatrix * vec4(position, 1.0);
    }
";

// Define fragment shader
let fragment_shader = "
    in vec2 uv;
    out vec4 fragColor;
    uniform float time;
    
    void main() {
        // Custom shader logic here
        fragColor = vec4(uv, sin(time), 1.0);
    }
";

// Create program
let program = Program::from_source(
    &context,
    vertex_shader,
    fragment_shader
)?;
```

### 2. Custom Material Implementation

```rust
struct BackgroundMaterial {
    program: Program,
    time: f32,
}

impl Material for BackgroundMaterial {
    fn fragment_shader_source(&self, _lights: &[&dyn Light]) -> String {
        fragment_shader.to_string()
    }
    
    fn vertex_shader_source(&self) -> String {
        vertex_shader.to_string()
    }
    
    fn use_uniforms(&self, camera: &Camera, _lights: &[&dyn Light]) -> Result<(), three_d::Error> {
        self.program.use_uniform("viewProjectionMatrix", camera.projection() * camera.view())?;
        self.program.use_uniform("time", self.time)?;
        Ok(())
    }
    
    fn render_states(&self) -> RenderStates {
        RenderStates::default()
    }
}
```

### 3. Create Geometric Model with Custom Material

```rust
let quad_geometry = Mesh::new(&context, &CpuMesh::square());
let background_material = BackgroundMaterial {
    program,
    time: 0.0,
};
let background_quad = Gm::new(quad_geometry, background_material);
```

### 4. Render with Custom Shader

```rust
screen.render(&self.camera, [&background_quad], &[]);
```

## Alternative Simpler Approach

If you don't need full `Material` trait implementation:

```rust
// Create program
let program = Program::from_source(&context, vertex_source, fragment_source)?;

// Create geometry
let positions = vec![
    Vec3::new(-1.0, -1.0, 0.0),
    Vec3::new(1.0, -1.0, 0.0),
    Vec3::new(1.0, 1.0, 0.0),
    Vec3::new(-1.0, 1.0, 0.0),
];
let indices = vec![0u32, 1, 2, 0, 2, 3];

// Render directly with program
program.use_vertex_attribute("position", &positions)?;
program.use_uniform("viewProjectionMatrix", camera.projection() * camera.view())?;
program.use_uniform("time", time_value)?;
program.draw_elements(RenderStates::default(), context.viewport(), &indices);
```

## Trade-offs

- **First approach**: Integrates better with three_d's rendering pipeline, easier to manage with other objects
- **Second approach**: More direct control over the rendering process, potentially more performant

## Implementation Location

The background shader should be implemented in the presentation layer renderer, specifically in:
- `intonation-toy/src/presentation/renderer.rs` around line 148-149 where the background quad is currently rendered