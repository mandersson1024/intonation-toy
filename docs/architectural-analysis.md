# Pitch-Toy Architectural Analysis

## Executive Summary

Pitch-toy is a real-time pitch detection and visualization application built with Rust/WebAssembly (WASM) and modern Web Audio APIs. The architecture follows a modular, performance-oriented design with emphasis on real-time audio processing, zero-allocation patterns, and comprehensive state management.

**Key Architectural Features:**
- **Hybrid Architecture**: Rust/WASM core with JavaScript Web Audio integration
- **Real-Time Processing**: AudioWorklet-based audio processing with <50ms latency
- **Modular Design**: Clear separation between audio, graphics, debug, and platform modules
- **Zero-Allocation**: Buffer pooling and transferable objects minimize GC pressure
- **Dependency Injection**: Modern AudioSystemContext pattern replacing global state
- **Observable Data**: Reactive data flow with decoupled components

## 1. Overall System Architecture

### 1.1 Technology Stack

**Core Technologies:**
- **Rust/WASM**: Core audio processing and application logic
- **Web Audio API**: Real-time audio processing with AudioWorklet
- **three-d**: WebGL/WebGPU rendering engine
- **egui**: Immediate mode GUI for debug interfaces
- **Trunk**: WebAssembly application bundler

**Development Tools:**
- **wasm-pack**: WebAssembly packaging and testing
- **wasm-bindgen**: Rust/JavaScript FFI bindings
- **Node.js**: Testing environment for WASM modules

### 1.2 Project Structure

```
pitch-toy/
├── Cargo.toml              # Main workspace configuration
├── pitch-toy/              # Main application crate
│   ├── audio/               # Audio processing modules
│   ├── debug/               # Debug and monitoring components
│   ├── graphics/            # WebGL rendering
│   ├── platform/            # Platform detection and validation
│   ├── static/              # Static assets (JS, CSS)
│   └── lib.rs               # Application entry point
├── egui-dev-console/        # Debug console crate
├── observable-data/         # Reactive data patterns
├── action/                  # Action system for UI controls
└── docs/                    # Documentation
```

### 1.3 Module Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Entry Point                  │
│                        (lib.rs)                            │
└─────────────────────┬───────────────────────────────────────┘
                      │
    ┌─────────────────┼─────────────────┐
    │                 │                 │
    ▼                 ▼                 ▼
┌─────────┐    ┌─────────────┐    ┌─────────┐
│  Audio  │    │  Graphics   │    │  Debug  │
│ Module  │    │   Module    │    │ Module  │
└─────────┘    └─────────────┘    └─────────┘
    │                 │                 │
    ▼                 ▼                 ▼
┌─────────┐    ┌─────────────┐    ┌─────────┐
│Platform │    │   Action    │    │Observable│
│ Module  │    │   System    │    │  Data   │
└─────────┘    └─────────────┘    └─────────┘
```

## 2. Audio Processing Architecture

### 2.1 Audio Data Flow

```
Microphone → MediaStream → AudioContext → AudioWorklet → Batch Processing → Pitch Detection → UI Updates
```

**Processing Pipeline:**
1. **Audio Capture**: Microphone access via `getUserMedia()`
2. **Stream Processing**: MediaStreamAudioSourceNode feeds AudioWorklet
3. **Real-Time Processing**: AudioWorklet processes fixed 128-sample chunks
4. **Batch Accumulation**: Chunks batched (default 1024 samples) for efficient transfer
5. **Pitch Analysis**: YIN algorithm processes batched audio data
6. **Volume Analysis**: Parallel RMS/peak analysis for signal quality
7. **Data Publishing**: Results published via observable data pattern

### 2.2 Core Audio Components

**AudioSystemContext (Dependency Injection Pattern):**
- **Centralized Management**: All audio components managed through context
- **Lifecycle Control**: Initialize, start, stop, cleanup operations
- **Component Access**: Type-safe access to audio managers and analyzers
- **Data Flow Integration**: Setter injection for reactive updates

**AudioWorklet Integration:**
- **Modern API**: AudioWorklet-only implementation (no legacy fallbacks)
- **Fixed Processing**: 128-sample chunks per Web Audio API specification
- **Transferable Buffers**: Zero-copy buffer passing with ArrayBuffer transfer
- **Buffer Pool**: Ping-pong recycling pattern with 16-buffer pool

**Pitch Detection Pipeline:**
- **YIN Algorithm**: Primary pitch detection with configurable thresholds
- **Window Sizes**: 256-4096 samples, optimized for different latency/accuracy trade-offs
- **Frequency Range**: 80Hz-2000Hz (configurable for vocal/instrumental ranges)
- **Confidence Scoring**: Multi-factor confidence calculation with volume weighting

### 2.3 Buffer Management Strategy

**Transferable Buffer Pool:**
- **Pool Size**: 16 buffers of 1024 samples each (~64KB total)
- **Lifecycle States**: Available → In-Flight → Processing → Returned
- **Timeout Handling**: 5-second timeout with automatic buffer reclaim
- **Zero-Allocation**: Pre-allocated buffers avoid GC pressure

**Performance Characteristics:**
- **Pool Hit Rate**: >95% under normal load conditions
- **GC Pause Detection**: Monitors processing time spikes
- **Memory Efficiency**: <1KB per second dynamic allocation
- **Latency**: ≤50ms end-to-end processing latency

## 3. Graphics and Rendering Architecture

### 3.1 Rendering Pipeline

**three-d Engine Integration:**
- **WebGL/WebGPU**: Modern graphics API with fallback support
- **Immediate Mode**: Simple sprite-based rendering
- **Canvas Target**: Fixed 1280x720 canvas with responsive scaling
- **egui Integration**: Debug UI overlays on 3D scene

**Rendering Components:**
- **SpriteScene**: Basic 2D sprite rendering system
- **Debug Overlays**: egui-based debug panels and controls
- **Performance Monitoring**: Real-time FPS and memory usage display

### 3.2 User Interface Architecture

**egui Immediate Mode GUI:**
- **Live Data Panel**: Real-time audio metrics and visualization
- **Microphone Controls**: Permission management and device selection
- **Debug Console**: Command-line interface for system control
- **Performance Metrics**: FPS, memory usage, audio latency display

**Action System:**
- **Decoupled Commands**: UI actions separated from audio logic
- **Type-Safe Messages**: Structured action types with validation
- **Reactive Updates**: Immediate UI response to state changes

## 4. State Management and Data Flow

### 4.1 Observable Data Pattern

**Data Sources and Observers:**
- **DataSource**: Thread-safe data publishing
- **DataObserver**: Reactive data consumption
- **Setter/Observer Pairs**: Decoupled data updates

**Live Data Integration:**
```rust
struct LiveData {
    microphone_permission: DataObserver<AudioPermission>,
    audio_devices: DataObserver<AudioDevices>,
    performance_metrics: DataObserver<PerformanceMetrics>,
    volume_level: DataObserver<VolumeLevel>,
    pitch_data: DataObserver<PitchData>,
    audioworklet_status: DataObserver<AudioWorkletStatus>,
    buffer_pool_stats: DataObserver<BufferPoolStats>,
}
```

### 4.2 Dependency Injection Migration

**Legacy Global State (Being Phased Out):**
- **Global Variables**: Thread-local storage for audio components
- **Coupling Issues**: Tight coupling between modules
- **Testing Challenges**: Difficult to mock and test

**New AudioSystemContext Pattern:**
- **Constructor Injection**: Dependencies passed at construction
- **Interface Segregation**: Minimal interfaces for each component
- **Testability**: Easy to mock and unit test
- **Lifecycle Management**: Clear initialization and cleanup

## 5. Inter-Module Communication

### 5.1 Message Passing Architecture

**AudioWorklet Communication:**
- **Type-Safe Protocol**: Structured message types with validation
- **Bidirectional**: Commands to worklet, data/status from worklet
- **Transferable Support**: Automatic transferable object detection
- **Error Handling**: Comprehensive error propagation and recovery

**Action System:**
- **UI Actions**: Test signal, background noise, microphone controls
- **Audio Responses**: Configuration updates, status changes
- **Decoupled Design**: UI components don't directly access audio system

### 5.2 Module Boundaries

**Strict Separation:**
- **Audio Module**: No dependencies on debug or graphics modules
- **Debug Module**: Can observe audio data but cannot control it
- **Graphics Module**: Renders UI but doesn't process audio
- **Platform Module**: Provides system information and validation

**Communication Patterns:**
- **Observable Data**: Publish-subscribe for state updates
- **Action System**: Command pattern for UI interactions
- **Message Protocol**: Structured messages for cross-thread communication

## 6. Performance Characteristics

### 6.1 Memory Management

**Static Allocation:**
- **Audio Components**: ~200KB for all processing components
- **Buffer Pool**: 64KB working set with 95%+ reuse rate
- **Graphics**: Minimal VRAM usage for sprite rendering

**Dynamic Allocation:**
- **Steady State**: <1KB per second during normal operation
- **Zero-Allocation**: Pre-allocated buffers avoid GC pressure
- **Memory Validation**: Buffer size and integrity checks

### 6.2 Real-Time Performance

**Latency Requirements:**
- **Target**: ≤50ms end-to-end processing latency
- **AudioWorklet**: ~5-15ms per batch processing
- **Pitch Detection**: ~10-30ms depending on window size
- **Buffer Transfer**: ~1-3ms via transferables
- **UI Updates**: ~2-5ms via observable data

**Throughput:**
- **Sample Rate**: 48kHz standard, 44.1kHz supported
- **Batch Size**: 1024 samples default (configurable)
- **Update Rate**: 47Hz @ 1024 samples, 48kHz sample rate
- **Processing Rate**: >1M samples/second sustained

## 7. Testing Strategy

### 7.1 Test Architecture

**WASM Testing:**
- **wasm-pack test --node**: All tests run in Node.js environment
- **Unit Tests**: Individual component testing
- **Integration Tests**: Cross-module communication testing
- **Performance Tests**: Buffer pool and processing benchmarks

**Test Coverage:**
- **Audio Processing**: Pitch detection accuracy, volume analysis
- **Buffer Management**: Pool exhaustion, timeout handling
- **Message Protocol**: Serialization, validation, error handling
- **State Management**: Observable data, action system

### 7.2 Browser Compatibility

**Requirements:**
- **AudioWorklet**: Chrome 66+, Firefox 76+, Safari 14.1+
- **WebAssembly**: Universal modern browser support
- **Transferable Objects**: Universal modern browser support
- **MediaStream API**: Universal modern browser support

**Validation:**
- **Platform Detection**: Automatic feature detection
- **Error Handling**: Graceful degradation on missing features
- **User Feedback**: Clear error messages for unsupported browsers

## 8. Security and Privacy

### 8.1 Microphone Access

**Permission Model:**
- **Explicit Consent**: User must grant microphone permission
- **HTTPS Required**: Secure context for production deployment
- **Stream Management**: Proper cleanup on component unmount
- **Privacy Indicators**: Clear visual feedback for active recording

### 8.2 Memory Safety

**Rust Implementation:**
- **Memory Safety**: No buffer overflows or use-after-free
- **Bounds Checking**: All buffer operations validated
- **Resource Cleanup**: Automatic cleanup on context destruction
- **Type Safety**: Compile-time prevention of common errors

## 9. Development Workflow

### 9.1 Build System

**Trunk Integration:**
- **Development Server**: `trunk serve` for hot reloading
- **Release Builds**: `trunk build --release` for production
- **Asset Processing**: Automatic CSS and JS bundling
- **WASM Optimization**: Configurable optimization levels

**Testing Workflow:**
- **Unit Tests**: `./scripts/test-all.sh` runs all package tests
- **Integration Tests**: Cross-module communication testing
- **Performance Tests**: Buffer pool and latency benchmarks
- **Browser Tests**: Manual testing across supported browsers

### 9.2 Code Quality

**Linting and Formatting:**
- **Rust**: `cargo fmt` and `cargo clippy`
- **TypeScript**: ESLint and Prettier (if added)
- **Documentation**: Comprehensive inline documentation
- **Error Handling**: Comprehensive error propagation

## 10. Architectural Strengths

1. **Modern Web Audio API**: Leverages latest browser capabilities
2. **Zero-Allocation Design**: Minimizes garbage collection impact
3. **Modular Architecture**: Clear separation of concerns
4. **Performance Monitoring**: Comprehensive metrics and debugging
5. **Type Safety**: Rust + TypeScript for robust implementation
6. **Real-Time Capable**: Meets strict latency requirements
7. **Testable Design**: Dependency injection enables comprehensive testing
8. **Memory Safety**: Rust prevents common memory-related bugs

## 11. Current Limitations and Technical Debt

### 11.1 Global State Migration

**Current State:**
- **Mixed Patterns**: Both global state and dependency injection present
- **Legacy Compatibility**: Global accessors maintained for backward compatibility
- **Thread Safety**: Some components use thread-local storage

**Migration Strategy:**
- **Gradual Transition**: AudioSystemContext pattern being adopted
- **Backward Compatibility**: Legacy APIs maintained during transition
- **Documentation**: Clear migration path documented

### 11.2 Browser Compatibility

**AudioWorklet Requirement:**
- **No Legacy Fallbacks**: Requires modern browser support
- **Feature Detection**: Comprehensive platform validation
- **Error Handling**: Clear error messages for unsupported browsers

## 12. Future Optimization Opportunities

1. **SIMD Instructions**: Vectorized audio processing for higher throughput
2. **Worker Threads**: Offload pitch analysis to separate worker
3. **Adaptive Windowing**: Dynamic window size based on signal characteristics
4. **Multi-Channel Support**: Stereo processing with channel correlation
5. **WebAssembly Optimization**: Core algorithms in WASM for maximum performance
6. **Memory Pool Tuning**: Dynamic pool sizing based on workload
7. **Latency Optimization**: Sub-20ms end-to-end processing
8. **Mobile Optimization**: Touch-friendly UI and reduced memory usage

## 13. Conclusion

The pitch-toy architecture represents a well-engineered, modern approach to real-time audio processing in web browsers. The combination of Rust's memory safety, WebAssembly's performance, and modern Web Audio APIs creates a robust foundation for pitch detection applications.

The architecture successfully balances performance requirements with maintainability, providing a clean separation of concerns while maintaining the real-time constraints necessary for audio processing. The ongoing migration from global state to dependency injection demonstrates architectural evolution toward more testable and maintainable code.

The comprehensive testing strategy, performance monitoring, and modular design make this codebase suitable for both development and production deployment, with clear paths for future optimization and feature expansion.