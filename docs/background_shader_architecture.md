# Background Quad Custom Shader Architecture

This document outlines the architecture for rendering the background quad using a custom shader with direct program control in three_d.

## Implementation

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

// Define fragment shader with texture buffer support
let fragment_shader = "
    in vec2 uv;
    out vec4 fragColor;
    uniform float time;
    uniform sampler2D bufferTexture;
    uniform int currentPixel;

    void main() {
        // Read from texture buffer
        vec4 historicalData = texture(bufferTexture, vec2(float(gl_FragCoord.x) / 1024.0, 0.5));

        // Custom shader logic here
        fragColor = vec4(uv, sin(time), 1.0);
    }
";

// Create program
let program = Program::from_source(&context, vertex_source, fragment_source)?;
```

### 2. Texture Buffer Setup

```rust
// Create texture buffer (e.g., 1024x1 for 1024 frames of history)
let mut texture_data = vec![0u8; 1024 * 4]; // RGBA
let texture = Texture2D::new(
    &context,
    &CpuTexture {
        data: TextureData::RgbaU8(texture_data),
        width: 1024,
        height: 1,
        ..Default::default()
    }
)?;
```

### 3. Per-Frame Update

```rust
// Update one pixel per frame
let pixel_index = frame_counter % 1024;
texture.update_part(
    pixel_index, 0,  // x, y position
    1, 1,            // width, height
    &[r, g, b, a]    // RGBA values
);
```

### 4. Rendering

```rust
// Create geometry
let positions = vec![
    Vec3::new(-1.0, -1.0, 0.0),
    Vec3::new(1.0, -1.0, 0.0),
    Vec3::new(1.0, 1.0, 0.0),
    Vec3::new(-1.0, 1.0, 0.0),
];
let indices = vec![0u32, 1, 2, 0, 2, 3];

// Bind uniforms and textures
program.use_vertex_attribute("position", &positions)?;
program.use_uniform("viewProjectionMatrix", camera.projection() * camera.view())?;
program.use_uniform("time", time_value)?;
program.use_texture("bufferTexture", &texture)?;
program.use_uniform("currentPixel", pixel_index as i32)?;

// Draw
program.draw_elements(RenderStates::default(), context.viewport(), &indices);
```

## Implementation Location

The background shader should be implemented in the presentation layer renderer, specifically in:
- `intonation-toy/src/presentation/renderer.rs` around line 148-149 where the background quad is currently rendered