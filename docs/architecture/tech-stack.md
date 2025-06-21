# Technology Stack & Dependencies
## Real-time Pitch Visualizer

**Version**: 1.0  
**Source**: Technical Architecture Document  
**Purpose**: Define technology choices, dependencies, and rationale

---

## Core Dependencies

| Component | Technology | Version | Rationale |
|-----------|------------|---------|-----------|
| **Audio I/O** | Web Audio API | Browser Native | Real-time audio processing in browsers |
| **WASM Compilation** | `wasm-pack` | 0.12 | Rust to WebAssembly toolchain |
| **Pitch Detection** | `pitch_detection` | 0.4 | Autocorrelation algorithms (YIN, McLeod) - Rust compiled to WASM |
| **Graphics** | Canvas API / WebGL | Browser Native | Hardware-accelerated browser graphics |
| **GUI** | HTML/CSS/JS | Browser Native | Native web UI components |
| **WASM Runtime** | `wasm-bindgen` | 0.2 | Rust/WASM ↔ JavaScript bridge |

## Technology Decision Matrix

| Requirement | Options Considered | Chosen | Why |
|-------------|-------------------|--------|-----|
| **Audio Processing** | Native extensions, Web Audio API | `Web Audio API` | Standard browser API, good performance |
| **WASM Compilation** | Emscripten, wasm-pack | `wasm-pack` | Rust-native toolchain, excellent browser integration |
| **Graphics** | Canvas 2D, WebGL, CSS animations | `Canvas/WebGL` | Hardware acceleration, 60 FPS capability |
| **GUI Framework** | Web frameworks, WASM UI libs | `HTML/CSS/JS` | Native browser performance, accessibility |
| **Language Bridge** | Direct WASM, wasm-bindgen | `wasm-bindgen` | Type-safe, efficient Rust ↔ JS communication |

## Build Dependencies

**Core Dependencies:**
- `cpal` - Cross-platform audio I/O
- `wgpu` - Graphics API abstraction  
- `egui` - Immediate mode GUI
- `crossbeam` - Lock-free communication

**Audio Processing:**
- `pitch_detection` - Autocorrelation-based pitch detection algorithms

**Utilities:**
- `anyhow`/`thiserror` - Error handling
- `log`/`env_logger` - Logging

## Browser Compatibility

**Target Browsers:**
- Chrome (latest)
- Firefox (latest) 
- Safari (latest)
- Edge (latest)

**Required Browser Features:**
- Web Audio API
- WebAssembly support
- Canvas API / WebGL
- getUserMedia API

## Performance Constraints

- **Audio Latency**: <50ms total (web audio constraints)
- **Buffer Size**: 1024-2048 samples (larger buffers for web stability)
- **Processing Budget**: <70% of AudioWorklet quantum for processing
- **Memory**: Minimal allocations, leverage WASM memory efficiency 