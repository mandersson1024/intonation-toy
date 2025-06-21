# Technical Architecture Document
## Real-time Pitch Visualizer

**Version**: 1.0  
**Date**: June 2025  
**Target Audience**: Technical implementers and project planners

---

## Executive Summary

The Real-time Pitch Visualizer is architected as a **high-performance, web-based real-time audio processing application** with custom graphics capabilities. The system uses a **Rust + WebAssembly architecture** with Web Audio API integration to achieve sub-50ms audio latency while maintaining 60 FPS graphics performance in modern browsers.

**Key Design Principles:**
- **Real-time Performance**: Audio processing optimized for WASM with minimal JS boundary crossings
- **Modular Architecture**: Clean separation between WASM audio core, web graphics, and browser APIs
- **Future-Proof Design**: Supports evolution from simple MVP to immersive WebGL graphics
- **Hybrid Language Strategy**: Rust/WASM for performance-critical audio, JavaScript for browser integration

---

## System Architecture Overview

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Browser Environment                       │
│                                                                 │
│  ┌─────────────────┐         ┌─────────────────────────────────┐ │
│  │  AudioWorklet   │         │         Main Thread             │ │
│  │   (WASM Core)   │         │      (JavaScript/WASM)         │ │
│  │                 │         │                                 │ │
│  │ ┌─────────────┐ │         │ ┌─────────────┐ ┌─────────────┐ │ │
│  │ │Audio Engine │ │         │ │   Renderer  │ │Web Controls │ │ │
│  │ │   (Rust)    │ │         │ │(Canvas/WebGL│ │ (HTML/CSS)  │ │ │
│  │ │• Pitch Det. │◄┼────────►│ │             │ │             │ │ │
│  │ │• Intervals  │ │         │ │• Background │ │• UI State   │ │ │
│  │ │• DSP Core   │ │         │ │  Graphics   │ │• User Input │ │ │
│  │ └─────────────┘ │         │ │• 60 FPS     │ │             │ │ │
│  └─────────────────┘         │ └─────────────┘ └─────────────┘ │ │
│           │                   │                                 │ │
│           ▼                   │                                 │ │
│  ┌─────────────────┐         │                                 │ │
│  │  Web Audio API  │         │                                 │ │
│  │  (Browser)      │         │                                 │ │
│  └─────────────────┘         │                                 │ │
│                               └─────────────────────────────────┘ │
│                                           │                       │
│                ┌─────────────────────────────────────────────────┐ │
│                │        Message Passing (JS/WASM Bridge)       │ │
│                │  • Audio Results  (WASM → JS)                  │ │
│                │  • Control Commands (JS → WASM)                │ │
│                └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Core Architecture Principles

#### 1. **Browser Thread Model**
- **AudioWorklet**: Handles all audio I/O and DSP processing (WASM-based)
- **Main Thread**: Manages DOM, graphics rendering, and user interaction (JS/WASM hybrid)
- **Minimal State Sharing**: All communication via structured message passing

#### 2. **WASM/JS Bridge Optimization**
- **Audio → UI**: Latest audio analysis results (efficient serialization)
- **UI → Audio**: Control commands (batched for performance)
- **Boundary Minimization**: Reduce expensive WASM/JS crossings

#### 3. **Web-Constrained Real-Time Safety**
- **Limited Allocations**: Memory management within WASM linear memory
- **Browser Scheduling**: Work within browser's audio scheduling constraints
- **Predictable Performance**: Audio worklet processing optimized for web constraints

---

## Technology Stack & Rationale

### Core Dependencies

| Component | Technology | Version | Rationale |
|-----------|------------|---------|-----------|
| **Audio I/O** | Web Audio API | Browser Native | Real-time audio processing in browsers |
| **WASM Compilation** | `wasm-pack` | 0.12 | Rust to WebAssembly toolchain |
| **Pitch Detection** | `pitch_detection` | 0.4 | Autocorrelation algorithms (YIN, McLeod) - Rust compiled to WASM |
| **Graphics** | Canvas API / WebGL | Browser Native | Hardware-accelerated browser graphics |
| **GUI** | HTML/CSS/JS | Browser Native | Native web UI components |
| **WASM Runtime** | `wasm-bindgen` | 0.2 | Rust/WASM ↔ JavaScript bridge |

### Technology Decision Matrix

| Requirement | Options Considered | Chosen | Why |
|-------------|-------------------|--------|-----|
| **Audio Processing** | Native extensions, Web Audio API | `Web Audio API` | Standard browser API, good performance |
| **WASM Compilation** | Emscripten, wasm-pack | `wasm-pack` | Rust-native toolchain, excellent browser integration |
| **Graphics** | Canvas 2D, WebGL, CSS animations | `Canvas/WebGL` | Hardware acceleration, 60 FPS capability |
| **GUI Framework** | Web frameworks, WASM UI libs | `HTML/CSS/JS` | Native browser performance, accessibility |
| **Language Bridge** | Direct WASM, wasm-bindgen | `wasm-bindgen` | Type-safe, efficient Rust ↔ JS communication |

---

## Component Architecture

### 1. Audio Processing Module (WASM Core)

#### Core Components

```rust
// Primary audio engine (compiled to WASM)
#[wasm_bindgen]
pub struct AudioEngine {
    pitch_detector: YinDetector<f32>,        // Autocorrelation-based
    interval_calculator: IntervalCalculator,
    
    // Configuration
    reference_frequency: f32,
    tuning_system: TuningSystem,
    enabled: bool,
    
    // Buffers (WASM linear memory)
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
}
```

#### Key Responsibilities
- **WASM Audio Processing**: Efficient DSP algorithms in WebAssembly
- **Real-time Processing**: Pitch detection and interval analysis
- **Message Serialization**: Efficient data exchange with JavaScript
- **Memory Management**: Optimal use of WASM linear memory

#### Performance Requirements
- **Latency Target**: <50ms total (web audio constraints)
- **Buffer Size**: 1024-2048 samples (larger buffers for web stability)
- **Processing Budget**: <70% of AudioWorklet quantum for processing
- **Memory**: Minimal allocations, leverage WASM memory efficiency

### 2. Graphics & UI Module (Browser Frontend)

#### Core Components

```javascript
// Main application controller (JavaScript)
class PitchVisualizerApp {
    constructor() {
        this.wasmModule = null;
        this.audioContext = null;
        this.renderer = new WebGLRenderer();
        this.ui = new WebUIController();
    }
}

// Graphics renderer (Canvas/WebGL)
class WebGLRenderer {
    constructor() {
        this.canvas = document.getElementById('visualizer');
        this.gl = this.canvas.getContext('webgl2');
        this.shaderPrograms = new Map();
    }
}
```

#### Rendering Pipeline

1. **Background Rendering** (WebGL shaders)
   - Audio-reactive animations
   - Real-time pitch visualization
   - Interval feedback graphics

2. **UI Overlay** (HTML/CSS)
   - Control panels
   - Settings interface
   - Debug information

3. **Composition**
   - Canvas + DOM overlay at 60 FPS
   - Handle responsive design
   - Manage browser input events

#### MVP vs. Future Architecture

**MVP Approach:**
```html
<!-- Simple HTML/CSS interface -->
<div id="pitch-display">
    <div class="frequency">Frequency: <span id="freq-value">440.0</span> Hz</div>
    <div class="note">Note: <span id="note-value">A4</span></div>
    <div class="cents">Cents: <span id="cents-value">+0</span></div>
</div>
```

**Future Approach:**
```javascript
// Custom WebGL-accelerated graphics
function renderImmersiveGraphics(renderer, audioData) {
    // Custom shader-based visualization
    renderer.renderPitchVisualization(audioData);
    renderer.renderIntervalDisplay(audioData);
    
    // Minimal UI overlay
    renderer.renderMinimalControls();
}
```

### 3. Communication Layer (WASM/JS Bridge)

#### Message Types

```rust
// Audio analysis results (WASM → JS)
#[wasm_bindgen]
pub struct AudioMessage {
    pub frequency: f32,
    pub confidence: f32,
    pub cents_deviation: f32,
    pub note_name: String,
    pub interval_cents: i16,
    pub interval_name: String,
    pub timestamp_ms: f64,
}

// Control commands (JS → WASM)
#[wasm_bindgen]
impl AudioEngine {
    pub fn set_reference(&mut self, freq: f32);
    pub fn set_reference_note(&mut self, note: &str);
    pub fn set_enabled(&mut self, enabled: bool);
    pub fn set_tuning_system(&mut self, system: &str);
}
```

#### Communication Patterns

1. **Audio Results** (AudioWorklet → Main Thread)
   - **Pattern**: Event-driven messaging
   - **Implementation**: MessagePort postMessage
   - **Behavior**: Efficient serialization, latest values prioritized

2. **Control Commands** (Main Thread → AudioWorklet)
   - **Pattern**: Reliable delivery
   - **Implementation**: MessagePort postMessage
   - **Behavior**: Commands queued and processed in order

#### Browser Message Strategy

```javascript
// AudioWorklet: Send results to main thread
const result = wasmEngine.process_audio(audioBuffer);
this.port.postMessage(result);

// Main thread: Receive audio results
audioWorkletNode.port.onmessage = (event) => {
    updateVisualization(event.data);
};
```

---

## Implementation Phases

### Phase 1: MVP Foundation (P0)

**Objective**: Functional web-based audio processing with basic visual feedback

**Deliverables:**
1. **WASM Audio Pipeline**
   ```rust
   // WASM-compiled audio engine using autocorrelation
   #[wasm_bindgen]
   pub fn process_audio_buffer(buffer: &[f32]) -> AudioMessage {
       let detector = YinDetector::new(sample_rate, buffer_size);
       detector.get_pitch(buffer, sample_rate, 0.5, None)
   }
   ```

2. **Web Interface**
   ```html
   <!-- Simple HTML interface -->
   <div id="pitch-display">
       <div class="frequency">Frequency: <span id="freq">440.0</span> Hz</div>
   </div>
   ```

3. **Web Audio Integration**
   ```javascript
   // AudioWorklet + WASM integration
   const audioContext = new AudioContext();
   const wasmModule = await import('./pkg/pitch_visualizer.js');
   ```

**Success Criteria:**
- Audio latency <50ms (web constraints)
- Pitch detection accuracy ±5 cents
- 60 FPS visual updates
- Cross-browser compatibility

### Phase 2: Educational Features (P1)

**Objective**: Interval analysis and reference pitch selection

**Deliverables:**
1. **WASM Interval Calculator**
   ```rust
   #[wasm_bindgen]
   pub fn calculate_interval(freq: f32, reference: f32) -> IntervalResult {
       // Returns interval name and cent deviation
   }
   ```

2. **Web Control Interface**
   ```html
   <!-- HTML controls for reference selection -->
   <input type="range" id="ref-freq" min="220" max="880" value="440">
   <label for="ref-freq">Reference Frequency: <span id="freq-display">440</span> Hz</label>
   ```

3. **Performance Optimization**
   - Target <50ms total latency (browser constraints)
   - Optimize WASM/JS boundary crossings
   - Minimize audio buffer copying

**Success Criteria:**
- Accurate interval identification
- Sub-50ms audio latency
- Smooth cross-browser experience

### Phase 3: Custom Graphics (P2)

**Objective**: Immersive visual feedback with WebGL shaders

**Deliverables:**
1. **WebGL Rendering Pipeline**
   ```javascript
   // Custom WebGL shaders for visualization
   class WebGLRenderer {
       constructor() {
           this.gl = canvas.getContext('webgl2');
           this.shaderProgram = this.createShaderProgram();
       }
   }
   ```

2. **Audio-Reactive Graphics**
   - Real-time pitch visualization with WebGL
   - Interval feedback animations
   - Configurable visual themes

3. **Hybrid Web Architecture**
   - Canvas-based custom graphics background
   - HTML/CSS controls overlay
   - Responsive design integration

**Success Criteria:**
- Compelling visual feedback
- Maintained 60 FPS across browsers
- Intuitive responsive user experience

---

## Development Guidelines

### Code Organization

```
project/
├── src/                       # Rust/WASM core
│   ├── lib.rs                 # WASM entry point with wasm_bindgen
│   ├── audio/                 # Audio processing (compiled to WASM)
│   │   ├── mod.rs             # Audio module exports
│   │   ├── engine.rs          # Core audio engine
│   │   ├── pitch_detector.rs  # Pitch detection algorithms
│   │   └── interval_calc.rs   # Musical interval calculations
│   └── utils.rs               # WASM utilities and helpers
├── web/                       # Web frontend
│   ├── index.html             # Main HTML page
│   ├── style.css              # Styling
│   ├── app.js                 # Main application controller
│   ├── audio-worklet.js       # AudioWorklet processor
│   └── renderer.js            # WebGL graphics renderer
├── pkg/                       # Generated WASM output (wasm-pack)
└── Cargo.toml                 # Rust dependencies
```

### Testing Strategy

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_pitch_detection_accuracy() {
        // Test with known frequencies
    }
    
    #[test]
    fn test_interval_calculation() {
        // Test interval math
    }
    
    #[test]
    fn test_message_passing() {
        // Test communication reliability
    }
}
```

#### Integration Tests
- **WASM Audio Pipeline**: End-to-end audio processing tests
- **Web Audio Integration**: AudioWorklet and Web Audio API tests
- **Cross-browser Compatibility**: Performance across browsers
- **Performance**: Latency and throughput benchmarks

#### Manual Testing
- **Real-world Usage**: Testing with actual musical instruments via browser
- **Child Usability**: Testing with target age group on tablets/computers
- **Cross-browser Testing**: Chrome, Firefox, Safari, Edge compatibility
- **Device Testing**: Different devices and audio interfaces

### Performance Monitoring

#### Key Metrics
1. **Audio Latency**: Microphone → WASM Processing → Visual Output
2. **Frame Rate**: Maintain 60 FPS across different browsers
3. **WASM Performance**: Processing time within AudioWorklet quantum
4. **Memory Usage**: WASM linear memory and JS heap usage
5. **Browser Compatibility**: Performance consistency across browsers

#### Profiling Tools
- **Audio**: Web Audio API performance timeline
- **WASM**: Browser DevTools and wasm-pack profiling
- **Graphics**: Browser DevTools performance tab
- **Cross-browser**: BrowserStack or similar testing platforms

---

## Risk Mitigation

### Technical Risks

| Risk | Impact | Likelihood | Mitigation Strategy |
|------|--------|------------|-------------------|
| **Browser Audio Latency** | High | Medium | Optimize WASM/JS boundaries, use AudioWorklets |
| **Cross-browser Compatibility** | High | High | Test early on all major browsers, use Web Standards |
| **WASM Performance** | Medium | Medium | Profile and optimize hot paths, minimize boundary crossings |
| **Microphone Permissions** | High | Medium | Clear UX for permission requests, fallback options |
| **Pitch Detection Accuracy** | High | Low | Use proven algorithms, extensive testing |

### Development Risks

| Risk | Impact | Likelihood | Mitigation Strategy |
|------|--------|------------|-------------------|
| **Scope Creep** | High | High | Strict MVP definition, phased development |
| **Technology Learning Curve** | Medium | Medium | Prototype critical components first |
| **Integration Complexity** | Medium | Medium | Incremental integration, extensive testing |

---

## Future Architecture Considerations

### Scalability

1. **Multi-channel Audio**: Support for multiple simultaneous inputs
2. **Network Features**: Remote monitoring and collaboration
3. **Plugin Architecture**: Extensible DSP pipeline

### Platform Expansion

1. **Progressive Web App**: PWA capabilities for offline use and app-like experience
2. **Mobile Optimization**: Responsive design for tablets and phones
3. **Native App Wrappers**: Electron or Tauri for desktop app versions

### Advanced Features

1. **Machine Learning**: AI-enhanced pitch detection
2. **Advanced Visualizations**: 3D graphics and immersive displays (may use FFT for spectral visualization)
3. **Educational Content**: Built-in lessons and exercises

---

## Conclusion

This architecture provides a solid foundation for the Real-time Pitch Visualizer while maintaining flexibility for future expansion. The key design decisions prioritize:

1. **Performance**: Real-time audio processing with sub-50ms latency (web-optimized)
2. **Maintainability**: Clean separation between WASM core and web frontend
3. **Extensibility**: Architecture supports evolution from MVP to advanced WebGL features
4. **Cross-platform Reach**: Web-based deployment for universal accessibility

The phased development approach allows for iterative refinement and user feedback integration while maintaining focus on the core educational mission.

---

## Appendices

### A. Build and Deployment

```bash
# Install wasm-pack (if not already installed)
cargo install wasm-pack

# Build WASM module
wasm-pack build --target web --out-dir pkg

# Development server (simple HTTP server)
python -m http.server 8000
# or
npx http-server web/

# Production build (optimized)
wasm-pack build --target web --release --out-dir pkg

# Run Rust tests
cargo test

# Integration testing
# Open http://localhost:8000 in browser for manual testing
```

### B. Dependencies Summary

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

### C. Performance Targets

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| **Audio Latency** | <20ms | Input-to-output timing |
| **GUI Frame Rate** | 60 FPS | Frame timing statistics |
| **Pitch Accuracy** | ±5 cents | Reference tone testing |
| **CPU Usage** | <50% single core | System monitor |
| **Memory Usage** | <100MB | Process memory tracking | 