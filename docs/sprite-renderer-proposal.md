# Sprite Renderer Module Architecture Proposal

## Executive Summary

This proposal outlines the architecture for a **standalone, reusable sprite renderer module** designed as a separate crate (`sprite-renderer`) that integrates seamlessly with the pitch-toy application while maintaining complete independence from application-specific code.

## Core Architecture Principles

### 1. Complete Isolation & Reusability
- **Separate Crate**: `sprite-renderer` as independent library crate
- **Zero Dependencies**: No coupling to pitch-toy application code
- **Framework Agnostic**: Pure Rust + three-d, no Yew or app-specific frameworks
- **Plug-and-Play**: Easy integration into any Rust/WebAssembly project

### 2. Modern Browser Focus
- **Platform Target**: Modern browsers only (Chrome 66+, Firefox 76+, Safari 14.1+)
- **WebGL Backend**: Leverages three-d's WebGL capabilities for 2D rendering
- **WebAssembly Native**: Built for WASM execution environment
- **No Fallbacks**: Fail-fast for unsupported browsers

### 3. Performance-First Design
- **GPU-Accelerated**: All rendering operations on GPU via WebGL
- **Memory Efficient**: Pre-allocated buffers and zero-copy operations
- **60fps Target**: Consistent frame rate under load
- **Batched Operations**: Efficient sprite batching and culling

## Technical Architecture

### Crate Structure

```
sprite-renderer/
├── Cargo.toml                 # Independent crate configuration
├── src/
│   ├── lib.rs                # Public API surface
│   ├── renderer/
│   │   ├── mod.rs            # Renderer core
│   │   ├── context.rs        # Rendering context management
│   │   ├── batch.rs          # Sprite batching system
│   │   └── culling.rs        # Frustum culling
│   ├── sprite/
│   │   ├── mod.rs            # Sprite definitions
│   │   ├── sprite.rs         # Core sprite struct
│   │   ├── atlas.rs          # Texture atlas management
│   │   └── animation.rs      # Animation system
│   ├── shaders/
│   │   ├── mod.rs            # Shader management
│   │   ├── builtin.rs        # Built-in shaders
│   │   ├── solid_color.rs    # Solid color shader
│   │   └── textured.rs       # Texture shader
│   ├── hit_testing/
│   │   ├── mod.rs            # Hit testing system
│   │   ├── bounds.rs         # Rectangular bounds
│   │   └── spatial_index.rs  # Spatial indexing for performance
│   ├── depth/
│   │   ├── mod.rs            # Depth management
│   │   └── layers.rs         # Depth layer system
│   └── utils/
│       ├── mod.rs            # Utilities
│       ├── math.rs           # Math utilities
│       └── color.rs          # Color utilities
├── examples/
│   ├── basic_sprites.rs      # Basic sprite rendering
│   ├── hit_testing.rs        # Hit testing demo
│   └── custom_shaders.rs     # Custom shader example
├── tests/
│   ├── integration/          # Integration tests
│   └── unit/                 # Unit tests
└── assets/
    ├── shaders/              # GLSL shader files
    └── textures/             # Test textures
```

### Core Components

#### 1. SpriteRenderer - Main Entry Point

```rust
pub struct SpriteRenderer {
    context: RenderContext,
    batch_renderer: BatchRenderer,
    shader_manager: ShaderManager,
    texture_atlas: TextureAtlas,
    hit_tester: HitTester,
    depth_manager: DepthManager,
}

impl SpriteRenderer {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, RendererError>;
    pub fn render(&mut self, sprites: &[Sprite], camera: &Camera) -> Result<(), RendererError>;
    pub fn hit_test(&self, position: Vec2, sprites: &[Sprite]) -> Vec<SpriteId>;
    pub fn add_custom_shader(&mut self, shader: CustomShader) -> ShaderId;
    pub fn create_texture_atlas(&mut self, textures: &[Texture]) -> AtlasId;
}
```

#### 2. Sprite - Core Sprite Definition

```rust
#[derive(Clone, Debug)]
pub struct Sprite {
    pub id: SpriteId,
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub color: Color,
    pub texture: Option<TextureId>,
    pub shader: ShaderId,
    pub depth: f32,
    pub visible: bool,
    pub hit_box: Option<HitBox>,
}
```

#### 3. Built-in Shaders

```rust
pub enum BuiltinShader {
    SolidColor,
    Textured,
    TexturedWithColor,
}

pub struct CustomShader {
    pub vertex_source: String,
    pub fragment_source: String,
    pub uniforms: HashMap<String, UniformValue>,
}
```

#### 4. Hit Testing System

```rust
#[derive(Clone, Debug)]
pub struct HitBox {
    pub bounds: Rectangle,
    pub transform: Transform2D,
}

pub struct HitTester {
    spatial_index: SpatialIndex,
}

impl HitTester {
    pub fn test_point(&self, point: Vec2, sprites: &[Sprite]) -> Vec<SpriteId>;
    pub fn test_rectangle(&self, rect: Rectangle, sprites: &[Sprite]) -> Vec<SpriteId>;
    pub fn update_spatial_index(&mut self, sprites: &[Sprite]);
}
```

#### 5. Depth Management

```rust
pub struct DepthManager {
    layers: Vec<DepthLayer>,
}

#[derive(Clone, Debug)]
pub struct DepthLayer {
    pub depth: f32,
    pub sprites: Vec<SpriteId>,
}

impl DepthManager {
    pub fn sort_sprites(&mut self, sprites: &mut [Sprite]);
    pub fn get_render_order(&self) -> Vec<SpriteId>;
}
```

## Integration Architecture

### Integration with Pitch-Toy

The sprite renderer integrates with pitch-toy through a clean adapter pattern:

```rust
// In pitch-toy/src/graphics/sprite_adapter.rs
pub struct SpriteAdapter {
    renderer: SpriteRenderer,
    event_dispatcher: Rc<RefCell<EventDispatcher<GraphicsEvent>>>,
}

impl SpriteAdapter {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, AdapterError>;
    pub fn render_frame(&mut self, sprites: &[Sprite]) -> Result<(), AdapterError>;
    pub fn handle_canvas_event(&mut self, event: CanvasEvent) -> Vec<SpriteId>;
}
```

### Event System Integration

```rust
// Graphics events that pitch-toy can listen to
#[derive(Clone, Debug)]
pub enum GraphicsEvent {
    SpriteClicked(SpriteId),
    SpriteHovered(SpriteId),
    RenderComplete,
    ShaderCompiled(ShaderId),
    TextureLoaded(TextureId),
}
```

## API Design

### Public API Surface

```rust
// Re-exports for easy usage
pub use crate::renderer::SpriteRenderer;
pub use crate::sprite::{Sprite, SpriteId};
pub use crate::shaders::{BuiltinShader, CustomShader, ShaderId};
pub use crate::hit_testing::{HitBox, HitTester};
pub use crate::depth::DepthManager;
pub use crate::utils::{Vec2, Rectangle, Color, Transform2D};

// Error types
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("WebGL context creation failed")]
    WebGLContextFailed,
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    #[error("Texture loading failed: {0}")]
    TextureLoadingFailed(String),
    #[error("Invalid sprite data: {0}")]
    InvalidSpriteData(String),
}

// Configuration
#[derive(Clone, Debug)]
pub struct RendererConfig {
    pub max_sprites: usize,
    pub max_textures: usize,
    pub enable_depth_testing: bool,
    pub enable_hit_testing: bool,
    pub vsync: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            max_sprites: 10000,
            max_textures: 256,
            enable_depth_testing: true,
            enable_hit_testing: true,
            vsync: true,
        }
    }
}
```

### Usage Examples

#### Basic Sprite Rendering

```rust
use sprite_renderer::*;

// Initialize renderer
let canvas = get_canvas_element();
let mut renderer = SpriteRenderer::new(&canvas)?;

// Create sprites
let sprites = vec![
    Sprite {
        id: SpriteId::new(),
        position: Vec2::new(100.0, 100.0),
        size: Vec2::new(50.0, 50.0),
        rotation: 0.0,
        color: Color::RED,
        texture: None,
        shader: BuiltinShader::SolidColor.into(),
        depth: 0.0,
        visible: true,
        hit_box: Some(HitBox::rectangle(Vec2::new(50.0, 50.0))),
    }
];

// Render frame
let camera = Camera::new(Vec2::new(400.0, 300.0), 1.0);
renderer.render(&sprites, &camera)?;
```

#### Hit Testing

```rust
// Test mouse click
let mouse_pos = Vec2::new(125.0, 125.0);
let hit_sprites = renderer.hit_test(mouse_pos, &sprites);

for sprite_id in hit_sprites {
    println!("Sprite {} was clicked!", sprite_id);
}
```

#### Custom Shaders

```rust
// Add custom shader
let custom_shader = CustomShader {
    vertex_source: include_str!("shaders/gradient.vert"),
    fragment_source: include_str!("shaders/gradient.frag"),
    uniforms: HashMap::new(),
};

let shader_id = renderer.add_custom_shader(custom_shader)?;

// Use custom shader
sprite.shader = shader_id;
```

## Performance Considerations

### Memory Management
- **Pre-allocated Buffers**: Vertex buffers allocated once, reused
- **Object Pooling**: Sprite objects pooled to reduce allocations
- **Texture Atlasing**: Multiple textures combined into single atlas
- **Uniform Buffer Objects**: Efficient uniform data transfer

### Rendering Optimizations
- **Instanced Rendering**: Similar sprites rendered in batches
- **Frustum Culling**: Off-screen sprites culled before rendering
- **Depth Pre-pass**: Early depth testing for overdraw reduction
- **Sprite Sorting**: Depth-based sorting for optimal rendering order

### Hit Testing Optimizations
- **Spatial Indexing**: Hierarchical spatial data structure
- **Broad Phase**: Quick elimination of non-intersecting sprites
- **Narrow Phase**: Precise hit box testing only when needed
- **Caching**: Hit test results cached for repeated queries

## Testing Strategy

### Unit Tests
- **Sprite Creation**: Validate sprite initialization and manipulation
- **Shader Compilation**: Test built-in and custom shader compilation
- **Hit Testing**: Verify hit detection accuracy
- **Depth Sorting**: Validate depth-based rendering order

### Integration Tests
- **Rendering Pipeline**: End-to-end rendering tests
- **Performance Tests**: Frame rate and memory usage validation
- **Browser Compatibility**: Multi-browser testing via wasm-pack
- **Canvas Integration**: WebGL context creation and management

### Example Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sprite_creation() {
        let sprite = Sprite::new(
            SpriteId::new(),
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0)
        );
        assert_eq!(sprite.position, Vec2::new(0.0, 0.0));
        assert_eq!(sprite.size, Vec2::new(100.0, 100.0));
    }
    
    #[test]
    fn test_hit_testing() {
        let sprite = create_test_sprite();
        let hit_tester = HitTester::new();
        
        // Test hit
        let hit_point = Vec2::new(50.0, 50.0);
        assert!(hit_tester.test_point(hit_point, &[sprite.clone()]).contains(&sprite.id));
        
        // Test miss
        let miss_point = Vec2::new(200.0, 200.0);
        assert!(!hit_tester.test_point(miss_point, &[sprite]).contains(&sprite.id));
    }
}
```

## Standalone Browser Testing App

### Test Application Structure

```
sprite-renderer-test-app/
├── Cargo.toml
├── index.html
├── src/
│   ├── main.rs               # Test app entry point
│   ├── test_scenes/
│   │   ├── mod.rs
│   │   ├── basic_sprites.rs  # Basic sprite test
│   │   ├── hit_testing.rs    # Hit testing demo
│   │   ├── custom_shaders.rs # Custom shader test
│   │   └── performance.rs    # Performance stress test
│   └── ui/
│       ├── mod.rs
│       └── controls.rs       # Test controls UI
└── static/
    ├── shaders/              # Test shaders
    └── textures/             # Test textures
```

### Test Scenarios

#### 1. Basic Sprite Rendering Test
- **Purpose**: Verify basic sprite rendering functionality
- **Features**: Multiple sprites, different colors, sizes, rotations
- **Validation**: Visual verification of sprite appearance

#### 2. Hit Testing Demo
- **Purpose**: Interactive hit testing validation
- **Features**: Click detection, hover states, multi-sprite selection
- **Validation**: Console output of hit test results

#### 3. Custom Shader Test
- **Purpose**: Verify custom shader compilation and usage
- **Features**: Gradient shaders, animated effects, uniform parameters
- **Validation**: Visual shader effects and parameter changes

#### 4. Performance Stress Test
- **Purpose**: Validate performance under load
- **Features**: Thousands of sprites, animated movement, fps counter
- **Validation**: Consistent 60fps performance metrics

#### 5. Depth Testing
- **Purpose**: Verify depth sorting and rendering order
- **Features**: Overlapping sprites, depth animation, z-fighting prevention
- **Validation**: Correct depth-based rendering order

## Dependencies

### Core Dependencies
```toml
[dependencies]
three-d = "0.17"
wasm-bindgen = "0.2"
web-sys = "0.3"
js-sys = "0.3"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

### Optional Features
```toml
[features]
default = ["hit-testing", "depth-testing"]
hit-testing = []
depth-testing = []
animations = []
debug-rendering = []
```

## Development Workflow

### Build Commands
```bash
# Build the sprite renderer crate
cargo build --release

# Test the sprite renderer
cargo test
wasm-pack test --headless --firefox

# Build test application
cd sprite-renderer-test-app
trunk build --release

# Serve test application
trunk serve
```

### Integration with Pitch-Toy
```bash
# Add as dependency in pitch-toy/Cargo.toml
[dependencies]
sprite-renderer = { path = "../sprite-renderer" }

# Use in pitch-toy graphics module
use sprite_renderer::*;
```

## Future Enhancements

### Phase 1 (Core Implementation)
- [ ] Basic sprite rendering with solid colors
- [ ] Texture loading and rendering
- [ ] Hit testing system
- [ ] Depth management
- [ ] Built-in shaders

### Phase 2 (Advanced Features)
- [ ] Sprite animation system
- [ ] Particle effects
- [ ] Advanced culling algorithms
- [ ] Texture atlasing optimization
- [ ] Performance profiling tools

### Phase 3 (Extended Capabilities)
- [ ] Skeletal animation support
- [ ] Advanced shader effects
- [ ] Multi-threaded rendering
- [ ] WebGPU backend option
- [ ] Audio-reactive sprite effects

## Conclusion

This sprite renderer module provides a comprehensive, performance-focused solution for 2D sprite rendering in modern browsers. The isolated architecture ensures complete reusability while maintaining seamless integration with the pitch-toy application. The modular design allows for incremental development and testing, with clear separation of concerns and well-defined APIs.

The module addresses all specified requirements:
- ✅ **2D renderer using three-d**
- ✅ **Modern browsers only**
- ✅ **Sprite system with textures and solid colors**
- ✅ **Custom shader support**
- ✅ **Built-in shaders**
- ✅ **Depth support**
- ✅ **Rectangle hit boxes**
- ✅ **Hit test support**
- ✅ **Standalone browser testing app**
- ✅ **Complete isolation from application code**
- ✅ **Designed for integration with pitch-toy**

The architecture follows established patterns from the pitch-toy codebase while maintaining independence, ensuring consistency and ease of integration.