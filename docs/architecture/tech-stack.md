# Technology Stack & Dependencies
## Real-time Pitch Visualizer

**Version**: 1.0  
**Source**: Technical Architecture Document  
**Purpose**: Define technology choices, dependencies, and rationale

---

## Core Dependencies

| Component | Technology | Version | Rationale |
|-----------|------------|---------|-----------|
| **Frontend Framework** | Yew | 0.21 | Rust-native web framework, component-based architecture |
| **Audio I/O** | Web Audio API | Browser Native | Real-time audio processing in browsers |
| **WASM Compilation** | `wasm-pack` / `trunk` | 0.12 / Latest | Rust to WebAssembly toolchain + Yew build system |
| **Pitch Detection** | `pitch_detection` | 0.4 | Autocorrelation algorithms (YIN, McLeod) - Rust compiled to WASM |
| **Graphics** | Canvas API / WebGL | Browser Native | Hardware-accelerated browser graphics |
| **UI Components** | Yew Components | 0.21 | Type-safe Rust UI with HTML-like syntax |
| **WASM Runtime** | `wasm-bindgen` | 0.2 | Rust/WASM ↔ JavaScript bridge |
| **Browser APIs** | `web-sys` | 0.3 | Rust bindings to browser APIs |

## Technology Decision Matrix

| Requirement | Options Considered | Chosen | Why |
|-------------|-------------------|--------|-----|
| **Frontend Framework** | React, Vue, Vanilla JS, Yew | `Yew` | Unified Rust codebase, type safety, performance |
| **Audio Processing** | Native extensions, Web Audio API | `Web Audio API` | Standard browser API, good performance |
| **WASM Compilation** | Emscripten, wasm-pack, trunk | `trunk` | Yew-optimized build system with hot reload |
| **Graphics** | Canvas 2D, WebGL, CSS animations | `Canvas/WebGL` | Hardware acceleration, 60 FPS capability |
| **UI Architecture** | JS components, WASM UI, Yew | `Yew Components` | React-like DX with Rust benefits |
| **Language Bridge** | Direct WASM, wasm-bindgen | `wasm-bindgen` | Type-safe, efficient Rust ↔ JS communication |

## Build Dependencies

**Frontend Dependencies:**
- `yew` - Modern Rust framework for web applications
- `web-sys` - Rust bindings to Web APIs
- `wasm-bindgen` - Rust/WebAssembly ↔ JavaScript bridge
- `wasm-bindgen-futures` - Async support for WASM
- `gloo` - Toolkit for Rust/WASM web development

**Audio Processing:**
- `pitch_detection` - Autocorrelation-based pitch detection algorithms
- Web Audio API integration via `web-sys`

**Build Tools:**
- `trunk` - Build tool and dev server for Yew applications
- `wasm-pack` - WebAssembly build tool

**Utilities:**
- `anyhow`/`thiserror` - Error handling
- `log`/`tracing` - Logging and diagnostics
- `serde` - Serialization framework

## Browser Compatibility

### Mandatory Requirements
- **WebAssembly Support**: Required for audio processing core - no JavaScript fallbacks provided
- **Web Audio API**: Required for real-time audio input and processing
- **AudioWorklet Support**: Required for low-latency audio processing
- **getUserMedia**: Required for microphone access
- **Canvas API / WebGL**: Required for graphics rendering

### Supported Browsers
- **Chrome**: 69+ (AudioWorklet + WASM)
- **Firefox**: 76+ (AudioWorklet + WASM) 
- **Safari**: 14.1+ (AudioWorklet + WASM)
- **Edge**: 79+ (AudioWorklet + WASM)

### Unsupported Browsers
- Internet Explorer (no WASM support)
- Chrome < 69 (no AudioWorklet)
- Firefox < 76 (no AudioWorklet)
- Safari < 14.1 (no AudioWorklet)

### Design Decision
**No fallbacks provided for unsupported browsers.** Users are directed to upgrade to supported browsers with clear messaging about modern web audio requirements.

## Performance Constraints

- **Audio Latency**: <50ms total (web audio constraints)
- **Buffer Size**: 1024-2048 samples (larger buffers for web stability)
- **Processing Budget**: <70% of AudioWorklet quantum for processing
- **Memory**: Minimal allocations, leverage WASM memory efficiency 