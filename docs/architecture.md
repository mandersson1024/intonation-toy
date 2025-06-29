# Pitch Toy Architecture Documentation

## Overview

Pitch Toy is a high-performance, browser-based real-time pitch detection and visualization application built with Rust and WebAssembly. This document provides a comprehensive architectural overview of the system design, component interactions, and technical implementation strategy.

## Executive Summary

### Core Architecture Principles

1. **Event-Driven Design**: All components communicate via typed events through a central Event Dispatcher
2. **Separation of Concerns**: Clear boundaries between audio processing, visualization, and UI management
3. **Performance Isolation**: GPU rendering isolated from audio processing to prevent interference
4. **Modular Development**: Each component can be developed and tested independently
5. **Configuration-Driven**: Build profiles and feature flags control development vs. production behavior
6. **YAGNI Compliance**: Follow "You Aren't Gonna Need It" principle - implement only what's needed now (see [Coding Standards](architecture/coding-standards.md#yagni-you-arent-gonna-need-it))

### Key Performance Targets

- **Audio Latency**: ≤30ms (production), ≤50ms (development)
- **Graphics Performance**: Consistent 60fps rendering
- **Memory Usage**: ≤50MB GPU memory, ≤100KB audio buffers
- **CPU Usage**: ≤5% for audio processing on modern devices

## System Architecture

### High-Level Component Diagram

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Browser   │    │   Microphone    │    │   GPU Hardware  │
│                 │    │     Input       │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    WebAssembly Runtime                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 Pitch Toy Application                   │   │
│  │                                                         │   │
│  │  ┌─────────────┐    ┌─────────────────┐               │   │
│  │  │ Dev Console │    │ Event Dispatcher│               │   │
│  │  │   (Yew)     │◄───┤   (Core Event   │               │   │
│  │  └─────────────┘    │     System)     │               │   │
│  │         │            └─────────────────┘               │   │
│  │         ▼                       │                      │   │
│  │  ┌─────────────┐               │                      │   │
│  │  │Debug Overlay│               │                      │   │
│  │  │   (Yew)     │               │                      │   │
│  │  └─────────────┘               │                      │   │
│  │                                │                      │   │
│  │         Audio Processing        │    Visual Rendering   │   │
│  │  ┌─────────────────────────────┐│┌─────────────────────┐│   │
│  │  │  ┌─────────────────────────┐││                     ││   │
│  │  │  │   Microphone Input      │││   Presentation      ││   │
│  │  │  │     (AudioWorklet)      │││     Layer           ││   │
│  │  │  └─────────────────────────┘││                     ││   │
│  │  │              │               ││                     ││   │
│  │  │              ▼               ││                     ││   │
│  │  │  ┌─────────────────────────┐││                     ││   │
│  │  │  │   Audio Processor       │││                     ││   │
│  │  │  │  • YIN Pitch Detection  │││                     ││   │
│  │  │  │  • FFT Spectral Analysis│││                     ││   │
│  │  │  │  • Volume Detection     │││                     ││   │
│  │  │  └─────────────────────────┘││                     ││   │
│  │  │              │               ││                     ││   │
│  │  │              ▼               ││                     ││   │
│  │  │  ┌─────────────────────────┐││                     ││   │
│  │  │  │   Test Signal Generator │││                     ││   │
│  │  │  │  (Development/Testing)  │││                     ││   │
│  │  │  └─────────────────────────┘││                     ││   │
│  │  └─────────────────────────────┘│└─────────────────────┘│   │
│  │                                │           │           │   │
│  │                                ▼           ▼           │   │
│  │                        ┌─────────────────────────────┐ │   │
│  │                        │      Theme Manager          │ │   │
│  │                        │   • Color Palettes         │ │   │
│  │                        │   • Visual Themes          │ │   │
│  │                        │   • Theme Transitions      │ │   │
│  │                        └─────────────────────────────┘ │   │
│  │                                     │                   │   │
│  │                                     ▼                   │   │
│  │                        ┌─────────────────────────────┐ │   │
│  │                        │   Graphics Renderer (wgpu)  │ │   │
│  │                        │   • GPU Pipeline           │ │   │
│  │                        │   • Immersive UI           │ │   │
│  │                        │   • Real-time Visualization│ │   │
│  │                        └─────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow Architecture

```
Microphone Input
    │
    ▼ (128-sample chunks @ 44.1/48kHz)
AudioWorklet Processing
    │
    ▼ (Buffered audio data)
Audio Processor
    │
    ├─── YIN Pitch Detection ──────┐
    │                              │
    ├─── FFT Spectral Analysis ────┤
    │                              │
    └─── Volume Level Detection ───┤
                                   │
                                   ▼
                            Event Dispatcher
                                   │
                    ┌──────────────┼──────────────┐
                    │              │              │
                    ▼              ▼              ▼
            Dev Console    Presentation    Debug Overlay
                           Layer
                              │
                              ▼
                        Render Commands
                              │
                              ▼
                        Theme Manager
                              │
                              ▼
                        Graphics Renderer
                              │
                              ▼
                        GPU Visualization
```

## Component Architecture

### Core Components

#### 1. Event Dispatcher (Central Nervous System)
- **Purpose**: Type-safe event routing between all system components
- **Key Features**:
  - Event queuing and prioritization for performance-critical paths
  - Subscription management with automatic cleanup
  - Event logging and debugging support
  - Zero-allocation event publishing for hot paths

#### 2. Audio Processing Pipeline

##### Microphone Input Module
- **Responsibilities**:
  - getUserMedia permission management
  - AudioWorklet-based real-time audio processing
  - Stream reconnection and error handling
- **Performance**: Fixed 128-sample chunks, <3ms processing interval

##### Audio Processor Module
- **Responsibilities**:
  - YIN algorithm pitch detection (primary)
  - FFT spectral analysis (visualization + confidence)
  - Volume level detection and RMS calculation
  - Musical note mapping with tuning system support
- **Performance**: ≤5ms analysis latency, zero-allocation processing

##### Test Signal Generator
- **Purpose**: Deterministic signal generation for development and testing
- **Capabilities**:
  - Sine waves, harmonic content, frequency sweeps
  - Musical intervals with multiple tuning systems
  - Noise injection for algorithm robustness testing

#### 3. Visual Rendering System

##### Presentation Layer
- **Responsibilities**:
  - Audio event coordination and state management
  - Render command generation for GPU pipeline
  - Animation timing and visual effect coordination
- **Integration**: Bridges audio processing and GPU rendering

##### Theme Manager
- **Responsibilities**:
  - Theme configuration and runtime switching
  - Color palette and visual style management
  - Smooth theme transitions without performance impact
- **Built-in Themes**: "Kids" (playful), "Nerds" (scientific)

##### Graphics Renderer (wgpu) - PRIMARY USER INTERFACE
- **Responsibilities**:
  - **ALL end-user interface rendering** - No HTML/CSS for production UI
  - GPU-accelerated rendering pipeline for complete user experience
  - Immersive full-screen visualizations
  - Interactive GPU-rendered controls (buttons, sliders, theme selection)
  - 60fps performance with adaptive resolution
- **Capabilities**: WebGPU/wgpu cross-platform graphics
- **Critical Constraint**: HTML/CSS forbidden for end-user interface elements

#### 4. Development Infrastructure

##### Development Console (Yew) - HTML/CSS ALLOWED
- **Purpose**: Interactive debugging and development interface
- **Features**:
  - Command execution system with extensible DevCommand trait
  - Real-time system control and configuration
  - Audio processing debugging and testing
- **Implementation**: HTML/CSS/Yew components (development tools exception)
- **Availability**: Development builds only

##### Debug Overlay (Yew) - HTML/CSS ALLOWED
- **Purpose**: Non-intrusive performance monitoring
- **Metrics**: FPS, audio latency, memory usage, CPU utilization
- **Implementation**: HTML/CSS/Yew components (development tools exception)
- **Integration**: Real-time metrics from all system components

## Technical Implementation

### Technology Stack

#### Core Framework
- **Rust**: Systems programming language for performance and safety
- **WebAssembly**: High-performance execution environment
- **Yew 0.21**: React-like framework for component-based UI
- **Trunk**: WebAssembly application bundler with hot reload

#### Audio Processing
- **Web Audio API**: Browser audio processing framework
- **AudioWorklet**: Real-time audio thread processing
- **pitch-detection 0.3**: YIN algorithm implementation
- **rustfft 6.0**: Fast Fourier Transform library

#### Graphics Rendering
- **wgpu 0.17**: Cross-platform GPU abstraction
- **WebGPU**: Modern browser graphics API
- **WGSL**: WebGPU Shading Language for GPU shaders

#### Browser Integration
- **web-sys 0.3**: Web API bindings for Rust
- **wasm-bindgen 0.2**: Rust-JavaScript interop layer

### Performance Architecture

#### Memory Management Strategy
- **Pre-allocated Buffers**: Audio and graphics buffers allocated once, reused
- **Zero-Copy Design**: Arc<T> for large data sharing, minimize copying
- **Circular Buffers**: Efficient audio streaming with in-place operations
- **Memory Pools**: Reusable buffer allocation for real-time processing

#### Optimization Techniques
- **Sliding Window Analysis**: 50% overlap for smooth pitch detection
- **Parallel Processing**: Time-domain and frequency-domain analysis
- **GPU Offloading**: Compute-intensive visualization on GPU
- **Bundle Optimization**: Code splitting, dead code elimination, compression

#### Browser Compatibility
- **Minimum Requirements**: Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+
- **Required APIs**: WebAssembly, Web Audio API, AudioWorklet, WebGPU
- **Mobile Support**: iOS Safari 14.5+, Chrome Android 66+
- **Feature Detection**: Graceful degradation for unsupported features

## Development Workflow

### Build Configurations

#### Development Build
- **Features**: Full debugging symbols, hot reload, development console
- **Performance**: Relaxed latency targets for debugging
- **Logging**: Comprehensive structured logging

#### Production Build
- **Features**: Maximum optimization
- **Performance**: Strict latency and FPS targets
- **Logging**: Error reporting only

### Testing Strategy

#### Unit Testing
- **Audio Algorithms**: Known signal inputs for validation
- **Mathematical Functions**: FFT, autocorrelation, note mapping
- **Event System**: Message routing and subscription management
- **Theme System**: Color calculations and transitions

#### Integration Testing
- **End-to-End Pipeline**: Microphone to visualization
- **Performance Testing**: Real-time performance under load
- **Cross-Browser Testing**: Automated compatibility validation
- **Mobile Testing**: Different screen sizes and capabilities

#### Performance Testing
- **Latency Measurement**: Audio processing and rendering latency
- **Frame Rate Consistency**: 60fps maintenance under load
- **Memory Profiling**: Leak detection and usage optimization
- **Bundle Analysis**: Load time measurement

## Deployment Architecture

### Static Hosting Strategy
- **Primary**: GitHub Pages, Netlify, Vercel
- **CDN Integration**: Global content distribution
- **Asset Optimization**: Texture compression, shader minification
- **Cache Strategy**: Optimized browser caching

### Progressive Web App Features
- **Service Worker**: Offline capability (future enhancement)
- **Web App Manifest**: Installable web application
- **Background Processing**: Audio worker persistence
- **Performance Monitoring**: Real-time metrics and error reporting

## Security Considerations

### WebAssembly Security
- **Sandboxed Execution**: Isolated WASM runtime environment
- **Memory Safety**: Rust memory guarantees prevent buffer overflows
- **No File System Access**: Browser security model enforcement
- **CORS Compliance**: Cross-origin resource sharing policies

### Audio Privacy
- **Permission-Based Access**: Explicit microphone consent required
- **Local Processing**: No audio data transmission or storage
- **Session Isolation**: No persistent audio data retention
- **Secure Context**: HTTPS requirement for getUserMedia

## Future Enhancements

### Planned Features
- **Additional Tuning Systems**: Historical temperaments, microtonal scales
- **Advanced Visualizations**: 3D spectrum analysis, harmonic visualization
- **Audio Export**: Recording and analysis export capabilities
- **MIDI Integration**: MIDI input support for instrument integration
- **Plugin Architecture**: Extensible audio processing plugins

### Scalability Considerations
- **Multi-Threading**: Audio processing worker thread isolation
- **GPU Compute**: GPGPU acceleration for complex audio algorithms
- **WebCodecs**: Hardware-accelerated audio processing
- **WebXR**: Virtual/Augmented reality visualization modes

## Monitoring and Observability

### Performance Metrics
- **Real-Time Monitoring**: FPS, latency, memory usage
- **Browser Telemetry**: Performance API timing measurements
- **Error Reporting**: Structured exception tracking
- **Usage Analytics**: Feature utilization and performance data

### Development Telemetry
- **Debug Logging**: Structured development logs
- **Performance Profiling**: CPU and memory analysis
- **Network Monitoring**: Asset loading performance
- **Compatibility Reporting**: Browser feature detection results

This architecture provides a robust foundation for high-performance real-time audio processing and visualization in the browser, with comprehensive development tools and extensible design for future enhancements.