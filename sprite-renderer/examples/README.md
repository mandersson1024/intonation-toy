# Sprite Renderer Examples

This directory contains example implementations demonstrating various features of the sprite-renderer library.

## Available Examples

### Basic Rendering (`basic_rendering.rs`)
Demonstrates fundamental sprite rendering concepts:
- Creating a canvas element
- Initializing the sprite renderer
- Loading textures
- Creating and rendering sprites
- Setting up a 2D camera

### Batch Rendering (`batch_rendering.rs`)
Shows efficient rendering of multiple sprites:
- Creating multiple sprites
- Grouping sprites by texture
- Using batch rendering for performance
- Texture atlas optimization

### Custom Shaders (`custom_shaders.rs`)
Demonstrates custom shader usage:
- Loading and compiling shaders
- Using custom uniforms
- Creating specialized effects
- Error handling for shader compilation

### Hit Testing (`hit_testing.rs`)
Shows sprite interaction capabilities:
- Configuring hit testing
- Using spatial indexing
- Handling mouse/touch events
- Collision detection

## Running Examples

To run an example, use:

```bash
cargo run --example basic_rendering
```

For examples that require specific features:

```bash
cargo run --features hit-testing --example hit_testing
```

## Current Status

**Note**: These examples are currently placeholders and will be implemented as the corresponding functionality is developed in the library. Each example file contains detailed comments about what will be implemented.

## Development Notes

- Examples are designed to be self-contained and runnable
- Each example focuses on a specific aspect of the library
- Code is documented with step-by-step explanations
- Error handling is demonstrated throughout
- Performance considerations are highlighted where relevant