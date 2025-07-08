# Sprite Renderer Examples

This directory contains example implementations demonstrating various features of the sprite-renderer library.

## Overview

This directory contains two types of examples:

- **📝 Simple Examples**: Single Rust files demonstrating specific API features
- **🎮 Interactive Demos**: Complete WebAssembly applications with web interfaces

## Available Examples

### 📝 Simple Examples (Code-focused)

#### Basic Rendering (`basic_rendering.rs`)
Demonstrates fundamental sprite rendering concepts:
- Creating a canvas element
- Initializing the sprite renderer
- Loading textures
- Creating and rendering sprites
- Setting up a 2D camera

#### Batch Rendering (`batch_rendering.rs`)
Shows efficient rendering of multiple sprites:
- Creating multiple sprites
- Grouping sprites by texture
- Using batch rendering for performance
- Texture atlas optimization

#### Custom Shaders (`custom_shaders.rs`)
Demonstrates custom shader usage:
- Loading and compiling shaders
- Using custom uniforms
- Creating specialized effects
- Error handling for shader compilation

#### Hit Testing (`hit_testing.rs`)
Shows sprite interaction capabilities:
- Configuring hit testing
- Using spatial indexing
- Handling mouse/touch events
- Collision detection

### 🎮 Interactive Demos (WebAssembly Applications)

#### Basic Demo (`basic_demo/`)
A foundational WebAssembly application showing:
- WebAssembly module integration
- Canvas setup and configuration  
- Mouse/keyboard interaction framework
- Error handling and debugging infrastructure
- **Note**: Visual rendering pending sprite-renderer implementation


## Running Examples

### Simple Examples (Rust Code)

To run a simple example, use:

```bash
cargo run --example basic_rendering
```

For examples that require specific features:

```bash
cargo run --features hit-testing --example hit_testing
```

### 🎮 Interactive Demos (WebAssembly)

To run interactive demos:

1. **Install Trunk** (one-time setup):
   ```bash
   cargo install trunk
   ```

2. Navigate to the demo directory:
   ```bash
   cd basic_demo
   ```

3. **Serve with hot reload** (recommended):
   ```bash
   ./build.sh serve
   # or directly: trunk serve
   ```
   This automatically opens `http://localhost:8080` with hot reload!
   
   **Benefits of Trunk:**
   - 🔄 **Hot Reload**: Instant updates when you change code
   - 🚀 **Auto Open**: Automatically opens browser
   - 📦 **Asset Processing**: Handles CSS, images, etc.
   - ⚡ **Fast Builds**: Optimized WebAssembly compilation

4. **Or build only**:
   ```bash
   ./build.sh build    # Development build
   ./build.sh release  # Optimized build
   ```

## Example Structure

### Simple Examples
```
basic_rendering.rs          # Single file with main() function
batch_rendering.rs          # Demonstrates batching techniques
custom_shaders.rs           # Custom shader usage
hit_testing.rs             # Collision detection
```

### Interactive Demos
```
basic_demo/                # Complete WebAssembly application
├── src/lib.rs             # Rust WebAssembly code
├── Cargo.toml             # Dependencies and config
├── Trunk.toml             # Trunk build configuration
├── index.html             # Web interface
├── build.sh               # Build/serve script
├── dist/                  # Built output (generated)
└── assets/                # Images, textures, etc.
```

## Development Tips

### Adding Simple Examples
Create a new `.rs` file with a `main()` function:
```rust
fn main() {
    // Your example code here
}
```

### Adding Interactive Demos
1. Create a new directory under `examples/`
2. Set up the standard structure (src/, Cargo.toml, index.html)
3. Use `#[wasm_bindgen(start)]` for the entry point
4. Export functions with `#[wasm_bindgen]` for JavaScript interaction

## Current Status

**Note**: These examples are currently placeholders and will be implemented as the corresponding functionality is developed in the library. Each example file contains detailed comments about what will be implemented.

## Development Notes

- Examples are designed to be self-contained and runnable
- Each example focuses on a specific aspect of the library
- Code is documented with step-by-step explanations
- Error handling is demonstrated throughout
- Performance considerations are highlighted where relevant