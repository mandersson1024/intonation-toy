# Sprite Renderer

A high-performance 2D sprite rendering library for WebAssembly applications, built with Rust and designed for GPU-accelerated rendering.

## Overview

The sprite-renderer crate provides a complete solution for rendering 2D sprites in web browsers using WebGL through the three-d engine. It offers a type-safe, zero-dependency API for creating, managing, and rendering sprites with support for advanced features like hit testing, depth sorting, and custom shaders.

## Key Features

- **GPU-Accelerated Rendering**: Leverages WebGL for high-performance sprite rendering
- **Type-Safe API**: Built with Rust's type system for compile-time safety
- **WebAssembly Ready**: Optimized for WebAssembly deployment
- **Modular Architecture**: Optional features for hit testing and depth sorting
- **Custom Shaders**: Support for custom shader programs
- **Batch Rendering**: Efficient rendering of multiple sprites
- **Zero-Dependency Interface**: Standalone library with minimal external dependencies

## Quick Start

### Prerequisites

- Rust 1.70 or later
- `wasm-pack` for WebAssembly builds

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sprite-renderer = "0.1.0"

# Optional features
sprite-renderer = { version = "0.1.0", features = ["hit-testing", "depth-testing"] }
```

### Basic Usage

```rust
use sprite_renderer::*;
use wasm_bindgen::JsCast;

// Get canvas element from DOM
let document = web_sys::window().unwrap().document().unwrap();
let canvas = document.get_element_by_id("canvas").unwrap()
    .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

// Initialize renderer
let mut renderer = SpriteRenderer::new(&canvas)?;

// Create a sprite
let sprite = Sprite::builder()
    .position(100.0, 100.0)
    .size(64.0, 64.0)
    .color(Color::RED)
    .build();

// Set up camera
let camera = Camera::default_2d(800, 600);

// Render the sprite
renderer.render(&[sprite], &camera)?;
```

## Architecture

The library is organized into several modules:

- **`renderer/`**: Core rendering engine and WebGL context management
- **`sprite/`**: Sprite definitions, builders, and management
- **`shaders/`**: Shader compilation and management system
- **`hit_testing/`**: Spatial indexing and collision detection (optional)
- **`depth/`**: Depth sorting and layer management (optional)
- **`utils/`**: Common utilities and helper functions

## Features

### Default Features

- `hit-testing`: Spatial indexing and collision detection
- `depth-testing`: Depth sorting and layer management

### Feature Flags

Enable specific features in your `Cargo.toml`:

```toml
[dependencies]
sprite-renderer = { version = "0.1.0", features = ["hit-testing"] }
```

Or disable all default features:

```toml
[dependencies]
sprite-renderer = { version = "0.1.0", default-features = false }
```

## Building

### Development Build

```bash
cargo build
```

### WebAssembly Build

```bash
wasm-pack build --target web
```

### Documentation

```bash
cargo doc --open
```

### Testing

```bash
cargo test
```

## Error Handling

The library provides comprehensive error handling through the `RendererError` enum:

```rust
use sprite_renderer::{RendererError, Result};

match renderer.render(&sprites, &camera) {
    Ok(()) => println!("Rendering successful"),
    Err(RendererError::WebGLContextFailed) => {
        eprintln!("Failed to create WebGL context");
    }
    Err(RendererError::ShaderCompilationFailed) => {
        eprintln!("Shader compilation failed");
    }
    Err(e) => eprintln!("Rendering error: {}", e),
}
```

## Performance Considerations

- Use batch rendering for multiple sprites
- Minimize texture switches by using sprite atlases
- Enable depth testing only when needed
- Reuse sprite objects when possible

## Browser Compatibility

- Chrome/Edge 66+
- Firefox 76+
- Safari 14.1+

Requires WebAssembly and WebGL support.

## Contributing

This library follows standard Rust development practices:

1. Run tests: `cargo test`
2. Check formatting: `cargo fmt`
3. Run lints: `cargo clippy`
4. Build documentation: `cargo doc`

## License

This project is licensed under the MIT License.