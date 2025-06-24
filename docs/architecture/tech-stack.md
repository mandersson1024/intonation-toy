# Technology Stack

## Overview

Pitch-Toy is built using a modern web technology stack optimized for real-time audio processing and WebAssembly performance. This document details the technology choices, rationale, and integration patterns.

## Core Technologies

### Primary Language: Rust 🦀

**Version**: 1.70+  
**Rationale**: Memory safety, zero-cost abstractions, excellent WebAssembly support, and performance characteristics ideal for real-time audio processing.

**Key Features Used**:
- Ownership system for memory safety
- `no_std` compatible crates for WASM optimization
- Cargo feature flags for conditional compilation
- Strong type system for audio processing reliability

### WebAssembly (WASM)

**Target**: `wasm32-unknown-unknown`  
**Purpose**: Brings near-native performance to browser-based audio processing

**Benefits**:
- Predictable performance for real-time audio
- Memory safety without garbage collection
- Efficient binary format for web delivery
- Seamless JavaScript interoperability

### Frontend Framework: Yew

**Version**: 0.21  
**Rationale**: React-like component model compiled to WebAssembly, providing familiar development patterns with Rust's performance benefits.

**Architecture Pattern**: Component-based with hooks for state management
```rust
// Yew component example
#[function_component(AudioInterface)]
pub fn audio_interface() -> Html {
    let audio_state = use_state(|| AudioState::new());
    
    html! {
        <div class="audio-interface">
            <AudioControlPanel state={audio_state.clone()} />
            <DebugPanel state={audio_state.clone()} />
        </div>
    }
}
```

## Audio Processing Stack

### Web Audio API

**Components Used**:
- `AudioContext`: Audio processing context
- `AudioWorklet`: Low-latency audio processing thread
- `getUserMedia`: Microphone access
- `AnalyserNode`: Real-time audio analysis

**Performance Characteristics**:
- Target latency: <50ms
- Buffer sizes: 1024-2048 samples
- Processing budget: <70% of AudioWorklet time

### Pitch Detection: `pitch-detection` Crate

**Version**: 0.3  
**Algorithms**:
- **YIN Algorithm**: Primary pitch detection method
- **McLeod Pitch Method**: Alternative algorithm for validation

**Integration**:
```rust
use pitch_detection::detector::yin::YinDetector;

let mut detector = YinDetector::new(sample_rate, buffer_size);
let pitch = detector.detect_pitch(&audio_buffer)?;
```

## Build and Development Tools

### Trunk

**Purpose**: WebAssembly build tool and development server  
**Features**:
- Hot reload for development
- Automatic WASM compilation
- Asset bundling and optimization
- Development server with proxy support

**Commands**:
```bash
# Development with hot reload
trunk serve

# Production build
trunk build --release
```

### Cargo Features

**Feature Flag Strategy**:
```toml
[features]
default = ["basic-features"]
basic-features = ["audio-processing", "pitch-detection"]
debug-features = ["debug-logging", "performance-profiling"]
full-features = ["basic-features", "debug-features", "advanced-features"]
```

**Build Profiles**:
- **Development**: Fast compilation, full debugging
- **Release**: Maximum optimization, minimal debug info

## JavaScript Interoperability

### wasm-bindgen

**Purpose**: Seamless Rust-JavaScript interoperability  
**Usage Patterns**:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct AudioProcessor {
    // Rust implementation
}

#[wasm_bindgen]
impl AudioProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AudioProcessor {
        AudioProcessor { /* ... */ }
    }
    
    #[wasm_bindgen]
    pub fn process_audio(&mut self, buffer: &[f32]) -> f32 {
        // Audio processing logic
    }
}
```

### js-sys and web-sys

**js-sys**: JavaScript standard library bindings  
**web-sys**: Web API bindings for browser functionality

**Key Web APIs Used**:
```rust
use web_sys::{
    AudioContext, AudioWorklet, AudioWorkletNode,
    MediaDevices, MediaStream, Performance
};
```

## Data Management

### State Management

**Pattern**: Centralized services with `Rc<RefCell<T>>`
```rust
// Shared state pattern
type SharedAudioEngine = Rc<RefCell<AudioEngineService>>;
type SharedErrorManager = Rc<RefCell<ErrorManager>>;

// Component usage
#[function_component(App)]
fn app() -> Html {
    let audio_engine = use_state(|| Some(Rc::new(RefCell::new(AudioEngineService::new()))));
    let error_manager = use_state(|| Some(Rc::new(RefCell::new(ErrorManager::new()))));
    // ...
}
```

### Serialization: Serde

**Purpose**: Configuration persistence and data exchange  
**Usage**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub detection_algorithm: PitchAlgorithm,
}
```

## Performance and Monitoring

### Performance Monitoring

**Built-in Metrics**:
- Audio processing latency
- Memory usage tracking
- CPU utilization monitoring
- Error rate tracking

**Implementation**:
```rust
pub struct PerformanceMonitor {
    pub audio_latency: f64,
    pub memory_usage: usize,
    pub cpu_utilization: f32,
    pub error_count: u32,
}
```

### Profiling Tools

**Development Features**:
- Real-time performance visualization
- Memory allocation tracking
- Audio pipeline debugging
- Stress testing capabilities

## Browser Compatibility

### Supported Browsers

| Browser | Minimum Version | Key Features |
|---------|----------------|--------------|
| Chrome | 69+ | AudioWorklet, WASM, getUserMedia |
| Firefox | 76+ | AudioWorklet, WASM, getUserMedia |
| Safari | 14.1+ | AudioWorklet, WASM, getUserMedia |
| Edge | 79+ | AudioWorklet, WASM, getUserMedia |

### Feature Detection

```rust
// Browser capability detection
pub fn check_browser_support() -> BrowserSupport {
    BrowserSupport {
        webassembly: check_wasm_support(),
        audio_worklet: check_audio_worklet_support(),
        media_devices: check_media_devices_support(),
        performance_api: check_performance_api_support(),
    }
}
```

### Compatibility Strategy

- **No Fallbacks**: Direct users to upgrade unsupported browsers
- **Feature Detection**: Graceful degradation where possible
- **Error Messaging**: Clear guidance for compatibility issues

## Dependencies

### Core Dependencies

```toml
[dependencies]
# WebAssembly and JavaScript interop
wasm-bindgen = "0.2"
js-sys = "0.3"
wasm-bindgen-futures = "0.4"

# Frontend framework
yew = { version = "0.21", features = ["csr"] }
yew-hooks = "0.3"
yew-router = "0.18"

# Audio processing
pitch-detection = "0.3"

# Utilities
gloo = "0.10"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Web APIs
web-sys = { version = "0.3", features = [
    "AudioContext", "AudioWorklet", "MediaDevices", 
    "Performance", "console", # ... extensive feature list
]}
```

### Development Dependencies

```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
web-sys = { version = "0.3", features = ["console"] }
```

## Architecture Patterns

### Modular Design

**Module Organization**:
```
src/
├── audio/           # Core audio processing
├── components/      # Yew UI components  
├── services/        # Business logic services
├── hooks/           # Custom Yew hooks
├── types/           # Shared type definitions
└── browser_compat/  # Browser compatibility layer
```

### Service-Oriented Architecture

**Key Services**:
- `AudioEngineService`: Audio processing coordination
- `ErrorManager`: Centralized error handling
- `PerformanceMonitor`: System monitoring
- `BrowserCompatService`: Cross-browser compatibility

### Component Architecture

**Patterns**:
- **Container/Presenter**: Separate logic from presentation
- **Custom Hooks**: Reusable component logic
- **Error Boundaries**: Graceful error handling in UI
- **Performance Optimization**: Memoization and efficient updates

## Security Considerations

### WebAssembly Security

- **Memory Safety**: Rust's ownership system prevents common vulnerabilities
- **Sandboxing**: WASM runs in browser sandbox
- **Input Validation**: All audio data validated before processing

### Web Security

- **HTTPS Required**: getUserMedia requires secure context
- **Permissions**: Explicit microphone permission handling
- **Content Security Policy**: Compatible with strict CSP rules

## Development Workflow

### Local Development

```bash
# Install dependencies
cargo install trunk

# Development server with hot reload
trunk serve

# Run tests
cargo test

# Build for production
trunk build --release
```

### Testing Strategy

- **Unit Tests**: Cargo test framework
- **Integration Tests**: WASM testing with `wasm-bindgen-test`
- **Manual Testing**: Browser compatibility validation
- **Performance Testing**: Automated latency and throughput tests

## Future Technology Considerations

### Potential Additions

- **WebGL**: For advanced visualizations
- **WebGPU**: For GPU-accelerated audio processing
- **Web Workers**: For heavy computational tasks
- **IndexedDB**: For persistent audio data storage

### Scaling Considerations

- **Module Federation**: For larger applications
- **Service Workers**: For offline functionality
- **WebRTC**: For real-time audio streaming
- **WebAssembly SIMD**: For vectorized audio processing

## Conclusion

This technology stack provides:

✅ **High Performance**: Near-native audio processing speeds  
✅ **Memory Safety**: Rust's ownership system prevents common bugs  
✅ **Modern Architecture**: Component-based design with strong typing  
✅ **Browser Compatibility**: Support for modern web standards  
✅ **Developer Experience**: Hot reload, strong tooling, and clear abstractions  
✅ **Maintainability**: Modular design with clear separation of concerns  

The stack is designed to evolve with the project's needs while maintaining performance and reliability for real-time audio applications.