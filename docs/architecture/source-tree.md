# Source Tree Architecture

## Project Structure Overview

```
pitch-toy/
├── 📁 .bmad-core/              # BMAD methodology configuration
│   ├── core-config.yml         # Project configuration
│   ├── tasks/                  # Development tasks
│   ├── templates/              # Document templates
│   └── checklists/            # Quality assurance checklists
├── 📁 .cursor/                 # Cursor AI configuration
│   └── rules/                  # AI agent rules
├── 📁 build-configs/           # Build configuration files
│   ├── dev.toml               # Development build settings
│   └── release.toml           # Production build settings
├── 📁 dist/                    # Build output directory
│   ├── development/           # Development builds
│   └── production/            # Production builds
├── 📁 docs/                    # Project documentation
│   ├── architecture/          # Architecture documentation
│   ├── epics/                 # Epic specifications
│   ├── stories/               # User stories
│   ├── prd.md                 # Product requirements
│   └── tech-spec.md           # Technical specification
├── 📁 src/                     # Source code
│   ├── lib.rs                 # Application entry point
│   └── modules/               # Modular components
├── 📁 target/                  # Rust build artifacts
├── Cargo.toml                  # Rust project configuration
├── index.html                  # HTML entry point
├── README.md                   # Project documentation
└── CLAUDE.md                   # AI assistant instructions
```

## Source Code Architecture (`src/`)

### Current Structure
```
src/
├── lib.rs                      # Main application entry point
└── modules/                    # Module directory (planned)
```

### Planned Modular Structure

```
src/
├── lib.rs                      # Application bootstrap and Yew app
├── 📁 modules/
│   ├── 📁 audio/              # Audio processing modules
│   │   ├── mod.rs             # Audio module exports
│   │   ├── microphone.rs      # Microphone input handling
│   │   ├── processor.rs       # Audio processing pipeline
│   │   ├── pitch_detector.rs  # YIN algorithm implementation
│   │   ├── volume_detector.rs # Volume level analysis
│   │   ├── buffer.rs          # Audio buffer management
│   │   └── test_signals.rs    # Test signal generation
│   ├── 📁 graphics/           # GPU rendering modules
│   │   ├── mod.rs             # Graphics module exports
│   │   ├── renderer.rs        # wgpu rendering pipeline
│   │   ├── shaders/           # WGSL shader files
│   │   │   ├── vertex.wgsl    # Vertex shaders
│   │   │   └── fragment.wgsl  # Fragment shaders
│   │   ├── pipeline.rs        # Render pipeline management
│   │   ├── buffers.rs         # GPU buffer management
│   │   └── textures.rs        # Texture management
│   ├── 📁 presentation/       # Visual presentation layer
│   │   ├── mod.rs             # Presentation module exports
│   │   ├── layer.rs           # Main presentation controller
│   │   ├── visualizations.rs  # Visualization logic
│   │   ├── animations.rs      # Animation systems
│   │   └── commands.rs        # Render command generation
│   ├── 📁 events/             # Event system
│   │   ├── mod.rs             # Event module exports
│   │   ├── dispatcher.rs      # Central event dispatcher
│   │   ├── types.rs           # Event type definitions
│   │   └── handlers.rs        # Event handler traits
│   ├── 📁 theme/              # Theme management
│   │   ├── mod.rs             # Theme module exports
│   │   ├── manager.rs         # Theme state management
│   │   ├── themes.rs          # Built-in theme definitions
│   │   └── colors.rs          # Color palette management
│   ├── 📁 console/            # Development console
│   │   ├── mod.rs             # Console module exports
│   │   ├── component.rs       # Yew console component
│   │   ├── commands.rs        # Console command system
│   │   ├── history.rs         # Command history management
│   │   └── output.rs          # Console output formatting
│   ├── 📁 debug/              # Debug and monitoring
│   │   ├── mod.rs             # Debug module exports
│   │   ├── overlay.rs         # Debug overlay component
│   │   ├── metrics.rs         # Performance metrics
│   │   ├── profiler.rs        # Performance profiling
│   │   └── logger.rs          # Structured logging
│   └── 📁 common/             # Shared utilities
│       ├── mod.rs             # Common module exports
│       ├── types.rs           # Common type definitions
│       ├── math.rs            # Mathematical utilities
│       ├── time.rs            # Timing utilities
│       └── config.rs          # Configuration management
```

## Module Dependency Graph

```
┌─────────────────┐    ┌─────────────────┐
│   lib.rs        │    │   console/      │
│   (App Entry)   │◄───┤   (DevConsole)  │
└─────────────────┘    └─────────────────┘
          │                       │
          ▼                       ▼
┌─────────────────┐    ┌─────────────────┐
│   events/       │◄───┤   debug/        │
│   (Dispatcher)  │    │   (Metrics)     │
└─────────────────┘    └─────────────────┘
          │
          ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   audio/        │    │   presentation/ │    │   graphics/     │
│   (Processing)  │───▶│   (Controller)  │───▶│   (Renderer)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
          │                       │                       │
          ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   common/       │◄───┤   theme/        │    │   shaders/      │
│   (Utilities)   │    │   (Manager)     │    │   (WGSL)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Build System Architecture

### Development Build Pipeline

```
Source Files (src/) 
    │
    ▼
Rust Compiler (rustc)
    │
    ▼
WebAssembly Module (.wasm)
    │
    ▼
Trunk Bundler
    │
    ├─── HTML Template (index.html)
    ├─── CSS Styles
    ├─── Static Assets
    └─── JavaScript Bindings
    │
    ▼
Development Server (localhost:8080)
    │
    ├─── Hot Reload
    ├─── Source Maps
    └─── Debug Symbols
```

### Production Build Pipeline

```
Source Files (src/)
    │
    ▼
Cargo Build (--release)
    │
    ├─── Optimization Level 3
    ├─── Link Time Optimization
    └─── Code Generation Units = 1
    │
    ▼
WebAssembly Module (.wasm)
    │
    ▼
wasm-opt Optimization
    │
    ├─── Size Optimization (-Oz)
    ├─── Dead Code Elimination
    └─── Function Inlining
    │
    ▼
Trunk Build (--release)
    │
    ├─── Asset Minification
    ├─── Bundle Compression
    └─── Cache Busting
    │
    ▼
Production Bundle (dist/production/)
    │
    ├─── index.html
    ├─── pitch-toy.wasm
    ├─── pitch-toy.js
    └─── static/
```

## Asset Organization

### Static Assets
```
static/
├── fonts/                  # Typography assets
│   ├── inter.woff2        # Primary UI font
│   └── mono.woff2         # Monospace console font
├── icons/                 # UI iconography
│   ├── microphone.svg     # Microphone status
│   ├── settings.svg       # Configuration
│   └── debug.svg          # Debug toggle
└── textures/              # GPU textures
    ├── noise.png          # Procedural noise
    └── gradients.png      # Color gradients
```

### Generated Assets
```
dist/
├── development/
│   ├── index.html         # Development HTML
│   ├── pitch-toy.wasm     # Debug WASM binary
│   ├── pitch-toy.js       # JS bindings
│   └── static/            # Asset copies
└── production/
    ├── index.html         # Optimized HTML
    ├── pitch-toy.wasm     # Optimized WASM
    ├── pitch-toy.js       # Minified JS
    └── static/            # Compressed assets
```

## Configuration Architecture

### Cargo.toml Structure
```toml
[package]                   # Project metadata
name = "pitch-toy"
version = "0.1.0"
edition = "2021"

[dependencies]              # Runtime dependencies
yew = "0.21"               # UI framework
web-sys = "0.3"            # Web API bindings
pitch-detection = "0.3"    # Audio processing
rustfft = "6.0"            # FFT implementation
wgpu = "0.17"              # GPU graphics

[dependencies.wasm-bindgen] # WASM interop
version = "0.2"
features = ["serde-serialize"]

[lib]                       # Library configuration
crate-type = ["cdylib"]     # Dynamic library for WASM

[profile.release]           # Production optimization
opt-level = 3               # Maximum optimization
lto = true                  # Link-time optimization
codegen-units = 1           # Single code generation unit
```

### Trunk Configuration
```toml
# build-configs/dev.toml
[build]
target = "index.html"
dist = "dist/development"
public-url = "/"

[watch]
watch = ["src", "static"]
ignore = ["target", "dist"]

[serve]
address = "127.0.0.1"
port = 8080

# build-configs/release.toml
[build]
target = "index.html"
dist = "dist/production"
release = true
minify = true

[clean]
dist = true
cargo = false
```

## Development Workflow

### File Organization Principles

1. **Modular Design**: Each module is self-contained with clear interfaces
2. **Dependency Injection**: Modules depend on traits, not concrete types
3. **Event-Driven**: Loose coupling through event dispatcher
4. **Performance First**: Critical path optimization in module design
5. **Testability**: Each module can be unit tested independently

### Hot Reload Architecture

```
File Change Detection
    │
    ▼
Rust Incremental Compilation
    │
    ▼
WebAssembly Module Update
    │
    ▼
Browser Hot Reload
    │
    ├─── Preserve Application State
    ├─── Update UI Components
    └─── Maintain Audio Context
```

### Testing Structure

#### Phased Testing Architecture

**Phase 1 (Current) - Native Tests:**
```
src/lib.rs                  # Native tests embedded in source
└── test_build_configuration # 1 meaningful test for build detection
```

**Phase 2 (Current) - WASM Tests:**
```
tests/                     # WASM test structure
└── wasm.rs               # WASM integration tests
    ├── test_wasm_build_configuration # Build configuration detection
    ├── test_wasm_basic_functionality # Basic WASM functionality
    └── TODO: Future WASM-specific tests:
        ├── audio_algorithms   # Audio processing in WASM
        ├── math_utilities     # Mathematical functions
        ├── data_structures    # Serialization/boundaries
        ├── module_communication # Inter-module data flow
        └── performance_benchmarks # WASM performance tests
```

**Phase 3 (Later) - Browser Integration:**
```
End-to-end testing via Cypress/Playwright:
- Canvas/WebGPU integration
- Web Audio API functionality  
- User interaction workflows
- Cross-browser compatibility
```

#### Testing Commands
- **Phase 1**: `cargo test` → 1 native test, instant feedback
- **Phase 2**: `wasm-pack test --headless --firefox` → WASM functionality validation (now available)
- **Phase 3**: Cypress/Playwright → browser integration testing (when needed)

#### Current Implementation Status
- **Entry Point Isolation**: `#[cfg(not(test))]` prevents main conflicts
- **YAGNI Compliance**: Only test what exists, plan for what's coming
- **Clear Documentation**: tests-wasm/README.md defines future implementation criteria

## Performance Architecture

### Memory Layout
- **Linear Memory**: Single WebAssembly memory space
- **Stack Allocation**: Preferred for small, temporary data
- **Heap Management**: Minimal heap usage for performance
- **Buffer Reuse**: Circular buffers for audio streaming

### Threading Model
- **Main Thread**: UI updates and event coordination
- **Audio Thread**: AudioWorklet processing (isolated)
- **GPU Thread**: Graphics rendering (browser-managed)
- **Background Tasks**: Non-critical processing

### Optimization Strategies
- **Zero-Copy**: Minimize data copying between modules
- **Pre-allocation**: Buffer pools for frequent allocations
- **Batch Processing**: Group operations for efficiency
- **Profile-Guided**: Optimization based on runtime profiling

This source tree architecture ensures modular development, clear separation of concerns, and optimal performance for the real-time pitch detection application.