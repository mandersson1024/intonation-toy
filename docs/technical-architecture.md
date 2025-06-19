# Technical Architecture Document
## Real-time Pitch Visualizer

**Version**: 1.0  
**Date**: June 2025  
**Target Audience**: Technical implementers and project planners

---

## Executive Summary

The Real-time Pitch Visualizer is architected as a **high-performance, real-time audio processing application** with custom graphics capabilities. The system uses a **multi-threaded, lock-free architecture** to achieve sub-20ms audio latency while maintaining 60 FPS graphics performance.

**Key Design Principles:**
- **Real-time Performance**: Audio processing never blocks or allocates memory
- **Modular Architecture**: Clean separation between audio, graphics, and communication layers
- **Future-Proof Design**: Supports evolution from simple MVP to immersive custom graphics
- **Single Language**: Pure Rust implementation for simplicity and performance

---

## System Architecture Overview

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Main Application Process                      │
│                                                                 │
│  ┌─────────────────┐         ┌─────────────────────────────────┐ │
│  │   Audio Thread  │         │         Main Thread             │ │
│  │  (High Priority)│         │        (GUI Thread)             │ │
│  │                 │         │                                 │ │
│  │ ┌─────────────┐ │         │ ┌─────────────┐ ┌─────────────┐ │ │
│  │ │Audio Engine │ │         │ │   Renderer  │ │    egui     │ │ │
│  │ │             │ │         │ │   (wgpu)    │ │  Controls   │ │ │
│  │ │• Pitch Det. │◄┼────────►│ │             │ │             │ │ │
│  │ │• Intervals  │ │         │ │• Background │ │• UI State   │ │ │
│  │ │• Core Audio │ │         │ │  Shaders    │ │• User Input │ │ │
│  │ └─────────────┘ │         │ │• 60 FPS     │ │             │ │ │
│  └─────────────────┘         │ └─────────────┘ └─────────────┘ │ │
│           │                   │                                 │ │
│           ▼                   │                                 │ │
│  ┌─────────────────┐         │                                 │ │
│  │   Core Audio    │         │                                 │ │
│  │   Integration   │         │                                 │ │
│  └─────────────────┘         │                                 │ │
│                               └─────────────────────────────────┘ │
│                                           │                       │
│                ┌─────────────────────────────────────────────────┐ │
│                │           Message Bus (Lock-Free)              │ │
│                │  • Audio Results  (Audio → GUI)                │ │
│                │  • Control Commands (GUI → Audio)              │ │
│                └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Core Architecture Principles

#### 1. **Thread Isolation**
- **Audio Thread**: Handles all audio I/O and DSP processing
- **Main Thread**: Manages GUI, graphics rendering, and user interaction
- **No Shared Mutable State**: All communication via message passing

#### 2. **Lock-Free Communication**
- **Audio → GUI**: Latest audio analysis results (bounded channel, size=1)
- **GUI → Audio**: Control commands (unbounded channel for reliability)
- **Non-blocking**: Audio thread never waits for GUI operations

#### 3. **Real-Time Safety**
- **No Allocations**: All memory pre-allocated in audio thread
- **No Locks**: Message passing uses lock-free data structures
- **Predictable Timing**: Audio callback completes within buffer period

---

## Technology Stack & Rationale

### Core Dependencies

| Component | Technology | Version | Rationale |
|-----------|------------|---------|-----------|
| **Audio I/O** | `cpal` | 0.15 | Cross-platform, low-latency audio |
| **Pitch Detection** | `pitch_detection` | 0.4 | Autocorrelation-based algorithms (YIN, McLeod) |
| **Graphics** | `wgpu` | 0.18 | Modern GPU API, future-proof |
| **GUI** | `egui` | 0.24 | Immediate mode, integrates with wgpu |
| **Window Management** | `winit` | 0.29 | Cross-platform windowing |
| **Communication** | `crossbeam` | 0.8 | Lock-free channels |

### Technology Decision Matrix

| Requirement | Options Considered | Chosen | Why |
|-------------|-------------------|--------|-----|
| **Audio Processing** | JACK, PortAudio, cpal | `cpal` | Rust-native, good macOS support |
| **Graphics** | OpenGL, Vulkan, Metal, wgpu | `wgpu` | Cross-platform, modern, integrates with egui |
| **GUI Framework** | Native macOS, Dear ImGui, egui | `egui` | Rust-native, immediate mode, wgpu integration |
| **Communication** | std::sync, crossbeam, rtrb | `crossbeam` | Mature, well-tested, good performance |

---

## Component Architecture

### 1. Audio Processing Module (`src/audio/`)

#### Core Components

```rust
// Primary audio engine  
pub struct AudioEngine {
    input_stream: cpal::Stream,
    output_stream: Option<cpal::Stream>,
    pitch_detector: YinDetector<f32>,        // Autocorrelation-based
    interval_calculator: IntervalCalculator,
    
    // Configuration
    reference_frequency: f32,
    tuning_system: TuningSystem,
    enabled: bool,
    
    // Buffers (pre-allocated)
    input_buffer: Vec<f32>,
}
```

#### Key Responsibilities
- **Audio I/O Management**: Setup and manage Core Audio streams
- **Real-time Processing**: Pitch detection and interval analysis
- **Control Message Handling**: Respond to GUI commands
- **Result Broadcasting**: Send analysis results to GUI

#### Performance Requirements
- **Latency Target**: <20ms total (audio input → visual output)
- **Buffer Size**: 512-1024 samples (autocorrelation needs larger windows than FFT)
- **Processing Budget**: <50% of buffer period for audio processing
- **Memory**: No allocations in audio callback

### 2. Graphics & UI Module (`src/gui/`)

#### Core Components

```rust
// Main application controller
pub struct PitchVisualizerApp {
    window: Window,
    renderer: Renderer,
    gui_handle: GuiHandle,
    state: AppState,
}

// Graphics renderer
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    egui_renderer: egui_wgpu::Renderer,
    
    // Custom rendering pipeline
    background_pipeline: wgpu::RenderPipeline,
    shader_uniforms: ShaderUniforms,
}
```

#### Rendering Pipeline

1. **Background Rendering** (Custom wgpu shaders)
   - Audio-reactive animations
   - Real-time pitch visualization
   - Interval feedback graphics

2. **UI Overlay** (egui)
   - Control panels
   - Settings interface
   - Debug information

3. **Composition**
   - Composite background + UI at 60 FPS
   - Handle window resizing
   - Manage input events

#### MVP vs. Future Architecture

**MVP Approach:**
```rust
// Simple egui-only interface
fn render_mvp_ui(ui: &mut egui::Ui, state: &AppState) {
    ui.label(format!("Frequency: {:.1} Hz", state.frequency));
    ui.label(format!("Note: {}", state.note_name));
    ui.label(format!("Cents: {:+.0}", state.cents_deviation));
}
```

**Future Approach:**
```rust
// Custom GPU-accelerated graphics
fn render_immersive_graphics(renderer: &Renderer, audio_data: &AudioMessage) {
    // Custom shader-based visualization
    renderer.render_pitch_visualization(audio_data);
    renderer.render_interval_display(audio_data);
    
    // Minimal UI overlay
    renderer.render_minimal_controls();
}
```

### 3. Communication Layer (`src/bridge/`)

#### Message Types

```rust
// Audio analysis results (Audio → GUI)
pub struct AudioMessage {
    pub frequency: f32,
    pub confidence: f32,
    pub cents_deviation: f32,
    pub note_name: [char; 4],
    pub interval_cents: i16,
    pub interval_name: [char; 16],
    pub timestamp_us: u64,
}

// Control commands (GUI → Audio)
pub enum ControlMessage {
    SetReference(f32),
    SetReferenceNote(String),
    SetEnabled(bool),
    SetTuningSystem(TuningSystem),
}
```

#### Communication Patterns

1. **Audio Results** (Audio Thread → GUI Thread)
   - **Pattern**: Latest-value semantics
   - **Implementation**: Bounded channel (capacity=1)
   - **Behavior**: Old values are discarded, GUI always gets latest

2. **Control Commands** (GUI Thread → Audio Thread)
   - **Pattern**: Reliable delivery
   - **Implementation**: Unbounded channel
   - **Behavior**: All commands are processed in order

#### Thread Safety Strategy

```rust
// Audio thread: Non-blocking sends
let result = AudioMessage { /* ... */ };
let _ = audio_sender.try_send(result); // Never blocks

// GUI thread: Non-blocking receives  
while let Ok(latest) = gui_receiver.try_recv() {
    current_audio_data = latest; // Get most recent
}
```

---

## Implementation Phases

### Phase 1: MVP Foundation (P0)

**Objective**: Functional audio processing with basic visual feedback

**Deliverables:**
1. **Basic Audio Pipeline**
   ```rust
   // Minimal working audio engine using autocorrelation
   let detector = YinDetector::new(sample_rate, buffer_size);
   let pitch = detector.get_pitch(&audio_samples, sample_rate, 0.5, None);
   ```

2. **Simple GUI**
   ```rust
   // egui-only interface
   egui::Window::new("Pitch Info")
       .show(ctx, |ui| {
           ui.label(format!("Frequency: {:.1} Hz", freq));
       });
   ```

3. **Communication Working**
   ```rust
   // Verified message passing
   audio_thread.send(result) → gui_thread.receive(result)
   ```

**Success Criteria:**
- Audio latency <50ms
- Pitch detection accuracy ±5 cents
- 60 FPS GUI updates
- No crashes or audio dropouts

### Phase 2: Educational Features (P1)

**Objective**: Interval analysis and reference pitch selection

**Deliverables:**
1. **Interval Calculator**
   ```rust
   pub fn calculate_interval(freq: f32, reference: f32) -> Interval {
       // Returns interval name and cent deviation
   }
   ```

2. **Reference Pitch Controls**
   ```rust
   // GUI controls for reference selection
   ui.add(egui::Slider::new(&mut ref_freq, 220.0..=880.0));
   ```

3. **Performance Optimization**
   - Target <20ms latency
   - Optimize autocorrelation window size
   - Reduce allocation overhead

**Success Criteria:**
- Accurate interval identification
- Sub-20ms audio latency
- Smooth user experience

### Phase 3: Custom Graphics (P2)

**Objective**: Immersive visual feedback with custom shaders

**Deliverables:**
1. **Background Shader Pipeline**
   ```rust
   // Custom wgpu shaders for visualization
   struct BackgroundRenderer {
       pipeline: wgpu::RenderPipeline,
       uniforms: AudioUniforms,
   }
   ```

2. **Audio-Reactive Graphics**
   - Real-time pitch visualization
   - Interval feedback animations
   - Configurable visual themes

3. **Hybrid UI Architecture**
   - Custom graphics background
   - egui controls overlay
   - Seamless integration

**Success Criteria:**
- Compelling visual feedback
- Maintained 60 FPS performance
- Intuitive user experience

---

## Development Guidelines

### Code Organization

```
src/
├── main.rs                    # Application entry point
├── lib.rs                     # Public API exports
├── audio/                     # Audio processing
│   ├── mod.rs                 # Module definitions
│   ├── engine.rs              # Core audio engine
│   ├── pitch_detector.rs      # Pitch detection algorithms
│   └── interval_calc.rs       # Musical interval calculations
├── gui/                       # Graphics and UI
│   ├── mod.rs                 # GUI module exports
│   ├── app.rs                 # Application controller
│   ├── renderer.rs            # Graphics rendering
│   └── widgets.rs             # Custom UI components
└── bridge/                    # Inter-thread communication
    ├── mod.rs                 # Message types
    └── message_bus.rs         # Communication implementation
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
- **Audio Pipeline**: End-to-end audio processing tests
- **GUI Responsiveness**: UI interaction tests
- **Performance**: Latency and throughput benchmarks

#### Manual Testing
- **Real-world Usage**: Testing with actual musical instruments
- **Child Usability**: Testing with target age group
- **Cross-device Compatibility**: Different audio interfaces

### Performance Monitoring

#### Key Metrics
1. **Audio Latency**: Input → Processing → Output
2. **GUI Frame Rate**: Maintain 60 FPS under load
3. **CPU Usage**: Audio thread priority and utilization
4. **Memory Usage**: Allocation patterns and peak usage

#### Profiling Tools
- **Audio**: `cpal` built-in latency measurement
- **Graphics**: `wgpu` performance counters
- **General**: Rust's built-in profiler and `cargo flamegraph`

---

## Risk Mitigation

### Technical Risks

| Risk | Impact | Likelihood | Mitigation Strategy |
|------|--------|------------|-------------------|
| **Audio Latency** | High | Medium | Conservative buffer sizes, real-time thread priority |
| **GUI Performance** | Medium | Low | Separate graphics thread, efficient rendering |
| **Cross-platform Issues** | Medium | Medium | Focus on macOS first, abstract platform-specific code |
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

1. **Cross-platform GUI**: Maintain native look and feel
2. **Mobile Support**: iOS/Android versions with adapted UI
3. **Web Version**: WebAssembly compilation for browser use

### Advanced Features

1. **Machine Learning**: AI-enhanced pitch detection
2. **Advanced Visualizations**: 3D graphics and immersive displays (may use FFT for spectral visualization)
3. **Educational Content**: Built-in lessons and exercises

---

## Conclusion

This architecture provides a solid foundation for the Real-time Pitch Visualizer while maintaining flexibility for future expansion. The key design decisions prioritize:

1. **Performance**: Real-time audio processing with sub-20ms latency
2. **Maintainability**: Clean separation of concerns, modular design
3. **Extensibility**: Architecture supports evolution from MVP to advanced features
4. **Simplicity**: Single-language implementation reduces complexity

The phased development approach allows for iterative refinement and user feedback integration while maintaining focus on the core educational mission.

---

## Appendices

### A. Build and Deployment

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Run with debug logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Performance profiling
cargo build --release
cargo run --release --bin pitch-visualizer
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