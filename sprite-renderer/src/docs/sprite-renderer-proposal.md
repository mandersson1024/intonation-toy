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

### three-d Renderer Integration Strategy

The sprite renderer is built as a **specialized layer on top of three-d's renderer module**, leveraging its high-level abstractions for maximum efficiency and minimal WebGL complexity:

#### Core three-d Integration Points
- **`three_d::renderer::Renderer`**: Main rendering engine - handles all WebGL operations
- **`three_d::renderer::Geometry`**: Sprite quad geometries with efficient vertex layouts
- **`three_d::renderer::Material`**: Texture and shader material management
- **`three_d::renderer::Object`**: Individual sprite render objects
- **`three_d::renderer::RenderTarget`**: Canvas and off-screen render targets
- **`three_d::renderer::Camera2D`**: 2D camera transformations

#### Architecture Philosophy
Instead of reimplementing WebGL primitives, we **compose three-d renderer components** to create sprite-specific functionality:

```rust
// Rather than raw WebGL:
// gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

// We use three-d renderer abstractions:
let geometry = three_d::renderer::Geometry::new(&context, vertices, indices)?;
let material = three_d::renderer::PhysicalMaterial::new(&context, &texture)?;
let sprite_object = three_d::renderer::Object::new(&context, &geometry, &material)?;
```

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

#### 1. SpriteRenderer - Main Entry Point (three-d Integration)

```rust
pub struct SpriteRenderer {
    // Core three-d components
    renderer: three_d::renderer::Renderer,
    camera: three_d::renderer::Camera2D,
    render_target: three_d::renderer::RenderTarget,
    
    // Sprite-specific systems built on three-d
    sprite_batch_system: SpriteBatchSystem,
    material_cache: MaterialCache,
    geometry_cache: GeometryCache,
    hit_tester: HitTester,
    depth_manager: DepthManager,
}

impl SpriteRenderer {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, RendererError>;
    pub fn render(&mut self, sprites: &[Sprite]) -> Result<(), RendererError>;
    pub fn hit_test(&self, position: Vec2, sprites: &[Sprite]) -> Vec<SpriteId>;
    pub fn add_custom_material(&mut self, material: CustomMaterial) -> MaterialId;
    pub fn create_texture_atlas(&mut self, textures: &[Texture]) -> AtlasId;
    
    // Direct three-d access for advanced usage
    pub fn three_d_renderer(&self) -> &three_d::renderer::Renderer;
    pub fn three_d_camera(&mut self) -> &mut three_d::renderer::Camera2D;
}
```

#### 2. SpriteBatchSystem - three-d Optimized Batching

```rust
pub struct SpriteBatchSystem {
    batches: Vec<SpriteBatch>,
    quad_geometry: three_d::renderer::Geometry,
    instance_buffer: three_d::renderer::InstanceBuffer,
}

pub struct SpriteBatch {
    material: three_d::renderer::Material,
    render_objects: Vec<three_d::renderer::Object>,
    instances: Vec<SpriteInstance>,
}

impl SpriteBatchSystem {
    pub fn batch_sprites(&mut self, sprites: &[Sprite]) -> Result<(), RendererError>;
    pub fn render_batches(&self, renderer: &three_d::renderer::Renderer) -> Result<(), RendererError>;
    pub fn clear_batches(&mut self);
}
```

#### 3. Sprite - Core Sprite Definition

```rust
#[derive(Clone, Debug)]
pub struct Sprite {
    pub id: SpriteId,
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub color: Color,
    pub texture: Option<TextureId>,
    pub material: MaterialId,  // Changed from shader to material
    pub depth: f32,
    pub visible: bool,
    pub hit_box: Option<HitBox>,
}

#[derive(Clone, Debug)]
pub struct SpriteInstance {
    pub transform: three_d::renderer::Mat4,
    pub color: three_d::renderer::Color,
    pub uv_transform: three_d::renderer::Mat3,
}
```

#### 4. Built-in Materials (three-d Integration)

```rust
pub enum BuiltinMaterial {
    SolidColor,
    Textured,
    TexturedWithColor,
}

pub struct MaterialCache {
    solid_color_material: three_d::renderer::ColorMaterial,
    textured_material: three_d::renderer::PhysicalMaterial,
    custom_materials: HashMap<MaterialId, Box<dyn three_d::renderer::Material>>,
}

pub struct CustomMaterial {
    pub vertex_source: String,
    pub fragment_source: String,
    pub uniforms: HashMap<String, three_d::renderer::UniformValue>,
}

impl MaterialCache {
    pub fn get_builtin_material(&self, material: BuiltinMaterial) -> &dyn three_d::renderer::Material;
    pub fn add_custom_material(&mut self, material: CustomMaterial) -> MaterialId;
    pub fn get_material(&self, id: MaterialId) -> Option<&dyn three_d::renderer::Material>;
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

#### Basic Sprite Rendering (three-d Integration)

```rust
use sprite_renderer::*;

// Initialize renderer - three-d handles WebGL context creation
let canvas = get_canvas_element();
let mut renderer = SpriteRenderer::new(&canvas)?;

// Create sprites with three-d materials
let sprites = vec![
    Sprite {
        id: SpriteId::new(),
        position: Vec2::new(100.0, 100.0),
        size: Vec2::new(50.0, 50.0),
        rotation: 0.0,
        color: Color::RED,
        texture: None,
        material: BuiltinMaterial::SolidColor.into(),
        depth: 0.0,
        visible: true,
        hit_box: Some(HitBox::rectangle(Vec2::new(50.0, 50.0))),
    }
];

// Render frame - three-d handles all WebGL operations
renderer.render(&sprites)?;

// Optional: Access three-d camera for advanced transformations
let camera = renderer.three_d_camera();
camera.set_viewport(three_d::renderer::Viewport::new_at_origo(800, 600));
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

#### Custom Materials (three-d Integration)

```rust
// Add custom material using three-d shader system
let custom_material = CustomMaterial {
    vertex_source: include_str!("shaders/gradient.vert"),
    fragment_source: include_str!("shaders/gradient.frag"),
    uniforms: HashMap::from([
        ("time".to_string(), three_d::renderer::UniformValue::Float(0.0)),
        ("resolution".to_string(), three_d::renderer::UniformValue::Vec2([800.0, 600.0])),
    ]),
};

let material_id = renderer.add_custom_material(custom_material)?;

// Use custom material
sprite.material = material_id;

// Advanced: Direct three-d material access
let three_d_renderer = renderer.three_d_renderer();
let material = three_d::renderer::PhysicalMaterial::new(
    three_d_renderer.context(),
    &three_d::renderer::CpuMaterial {
        albedo: three_d::renderer::Color::RED,
        ..Default::default()
    }
)?;
```

## Performance Considerations

### three-d Renderer Optimizations

#### Memory Management (three-d Integration)
- **three-d Geometry Caching**: Reuse `three_d::renderer::Geometry` objects for identical sprite quads
- **Material Sharing**: Single `three_d::renderer::Material` instances shared across sprites
- **Instance Buffer Optimization**: Use `three_d::renderer::InstanceBuffer` for efficient sprite batching
- **Texture Atlas Integration**: Leverage `three_d::renderer::Texture2DArray` for atlas management

#### Rendering Optimizations (three-d Powered)
- **three-d Instanced Rendering**: Use `three_d::renderer::InstancedMesh` for sprite batches
- **Automatic Frustum Culling**: Leverage three-d's built-in culling with `three_d::renderer::Camera2D`
- **Depth Buffer Optimization**: Use three-d's depth testing with `three_d::renderer::RenderTarget`
- **Material Sorting**: Group sprites by `three_d::renderer::Material` to minimize state changes
- **Geometry Instancing**: Single quad geometry instanced for all sprites via three-d

#### three-d Specific Optimizations
```rust
// Leverage three-d's built-in optimizations
pub struct OptimizedSpriteRenderer {
    // Pre-allocated three-d resources
    quad_geometry: three_d::renderer::Geometry,
    instance_buffer: three_d::renderer::InstanceBuffer,
    material_cache: HashMap<MaterialId, three_d::renderer::Material>,
    
    // three-d render state optimization
    render_states: three_d::renderer::RenderStates,
}

impl OptimizedSpriteRenderer {
    pub fn render_optimized(&mut self, sprites: &[Sprite]) -> Result<(), RendererError> {
        // Sort sprites by material to minimize state changes
        let sorted_sprites = self.sort_by_material(sprites);
        
        // Use three-d's instanced rendering
        for (material, sprite_batch) in sorted_sprites {
            self.instance_buffer.update_instances(&sprite_batch);
            self.renderer.render_instanced(
                &self.quad_geometry,
                &material,
                &self.instance_buffer,
                &self.render_states
            )?;
        }
        Ok(())
    }
}
```

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

### Core Dependencies (three-d Renderer Focus)
```toml
[dependencies]
# Core three-d renderer - our primary rendering backend
three-d = { version = "0.18", features = ["canvas", "renderer"] }

# WebAssembly bindings for browser integration
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["HtmlCanvasElement", "WebGl2RenderingContext"] }
js-sys = "0.3"

# Serialization and error handling
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
anyhow = "1.0"

# Math and utilities that complement three-d
nalgebra = "0.32"  # Compatible with three-d's math types

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