# Pitch Toy Architecture Documentation

## Overview

Pitch Toy is a high-performance, browser-based real-time pitch detection and visualization application built with Rust and WebAssembly. This document provides a comprehensive architectural overview of the system design, component interactions, and technical implementation strategy.

## Executive Summary

### Core Architecture Principles

1. **Event-Driven Design**: All components communicate via typed events through a central Event Dispatcher for maximum decoupling
2. **Separation of Concerns**: Clear boundaries between audio processing, visualization, and UI management maintained through event interfaces
3. **Performance Isolation**: GPU rendering isolated from audio processing to prevent interference
4. **Modular Development**: Each component can be developed and tested independently
5. **Configuration-Driven**: Build profiles and feature flags control development vs. production behavior
6. **YAGNI Compliance**: Follow "You Aren't Gonna Need It" principle - implement only what's needed now (see [Coding Standards](architecture/coding-standards.md#yagni-you-arent-gonna-need-it))

### Key Performance Targets

- **Audio Latency**: ≤30ms (production), ≤50ms (development)
- **Graphics Performance**: Consistent 60fps rendering
- **Memory Usage**: ≤50MB GPU memory, ≤100KB audio buffers

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
│  │  │Debug Console│    │ Event Dispatcher│               │   │
│  │  │   (Yew)     │    │   (Core Event   │               │   │
│  │  └─────────────┘    │     System)     │               │   │
│  │         │            └─────────────────┘               │   │
│  │         ▼                       ▲                      │   │
│  │  ┌─────────────┐               │                      │   │
│  │  │Debug Live   │               │                      │   │
│  │  │   Panel     │               │                      │   │
│  │  │   (Yew)     │               │                      │   │
│  │  └─────────────┘               │                      │   │
│  │         │                       │                      │   │
│  │         ▼                       │                      │   │
│  │  ┌─────────────┐               │                      │   │
│  │  │Debug Permit │               │                      │   │
│  │  │   Button    │               │                      │   │
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
│  │  │              │               ││                     ││   │
│  │  │              ▼               ││                     ││   │
│  │  │  ┌─────────────────────────┐││                     ││   │
│  │  │  │  ConsoleAudioService    │││                     ││   │
│  │  │  │  • Permission Interface │││                     ││   │
│  │  │  │  • Device Management    │││                     ││   │
│  │  │  │  • Event Publishing     │││                     ││   │
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
│  │                        │   Graphics Renderer (three-d)│ │   │
│  │                        │   • WebGL Pipeline         │ │   │
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
                         ConsoleAudioService
                                   │
                                   ▼
                            Event Dispatcher
                                   │
                    ┌──────────────┼──────────────┐
                    │              │              │
                    ▼              ▼              ▼
            Debug Console      Presentation    Debug Live Panel
          (Service Interface)     Layer       & Permission Button
                                  │
                                  ▼
                            Render Commands
                                  │
                                  ▼
                            Theme Manager
                                  │
                                  ▼
                        Graphics Renderer (three-d)
                                  │
                                  ▼
                            GPU Visualization
```

## Crate Architecture

### Multi-Crate Structure

The application follows a modular multi-crate architecture to promote code reusability and maintain clear separation of concerns:

#### 1. `pitch-toy` (Main Application Crate)
- **Purpose**: Core application logic and domain-specific functionality
- **Dependencies**: `dev-console`, standard audio/graphics crates
- **Contains**: Audio processing, rendering, event system, and business logic
- **Type**: `cdylib` + `rlib` for WebAssembly compilation and testing

#### 2. `dev-console` (Development Tools Library)
- **Purpose**: Reusable development console component
- **Dependencies**: Minimal - only `yew`, `web-sys`, `serde` for core functionality
- **Contains**: Generic console UI, command system, history management
- **Type**: `rlib` library crate for reuse across projects
- **Design**: Framework-agnostic with pluggable command registry system

### Crate Benefits
- **Modularity**: Clear separation between application logic and development tools
- **Reusability**: `dev-console` can be used in other Rust/WebAssembly projects
- **Dependency Isolation**: Development tools don't pull in application-specific dependencies
- **Testing Independence**: Each crate can be tested in isolation
- **Build Optimization**: Conditional compilation of development features

## Component Architecture

### Domain-Specific Event Dispatcher Architecture

The application uses a **domain-specific event dispatcher pattern** rather than a single global event bus. This architecture provides significant performance and maintainability benefits for real-time audio and graphics processing.

#### Architecture Pattern

```rust
// Generic foundation for type-safe dispatchers
SharedEventDispatcher<T> = Rc<RefCell<EventDispatcher<T>>>

// Domain-specific dispatchers
AudioEventDispatcher = SharedEventDispatcher<AudioEvent>
// Future: UIEventDispatcher = SharedEventDispatcher<UIEvent>
// Future: GraphicsEventDispatcher = SharedEventDispatcher<GraphicsEvent>
```

#### Key Benefits for Real-Time Performance

**1. Audio Pipeline Isolation**
- Audio events (pitch detection, volume analysis, buffer management) are processed independently
- UI events cannot block critical audio processing
- Maintains ≤30ms audio latency requirement regardless of UI complexity

**2. Graphics Pipeline Isolation**
- Rendering events don't interfere with audio thread processing
- Theme changes and visual updates isolated from audio subsystem
- Maintains 60fps rendering without audio dropouts

**3. Type Safety & Domain Boundaries**
- Generic `EventDispatcher<T>` prevents mixing event types between domains
- Compile-time prevention of cross-domain event contamination
- Clear architectural boundaries between audio, graphics, UI, and debug systems

**4. Memory and Performance Locality**
- Domain-specific event processing reduces cache misses
- Events processed in domain-appropriate contexts
- Better memory access patterns for real-time processing

**5. Testability and Modularity**
- Each domain can be tested with isolated event dispatchers
- Components only receive dispatchers for their specific domain
- Simplified dependency injection and mocking

#### Current Implementation

- **AudioEventDispatcher**: Handles all audio-related events (device changes, pitch detection, volume analysis, buffer events)
- **Distribution Pattern**: Created once per application context, dependency-injected to audio components
- **Event Types**: 12 distinct AudioEvent variants covering complete audio pipeline lifecycle

#### Future Expansion

The architecture supports adding new domain-specific dispatchers as needed:
- **UIEventDispatcher**: User interactions, theme changes, navigation
- **GraphicsEventDispatcher**: Rendering commands, visual effects, animation
- **DebugEventDispatcher**: Development tools, logging, performance metrics

### Core Components

#### 1. Event Dispatcher (Domain-Specific Communication System)
- **Purpose**: Type-safe event routing within domain-specific subsystems
- **Architecture Pattern**: Domain-specific dispatchers (AudioEventDispatcher, future UIEventDispatcher, etc.)
- **Key Features**:
  - Type-safe event isolation between domains (audio, graphics, UI, debug)
  - Performance isolation preventing cross-domain interference
  - Subscription management with automatic cleanup
  - Event logging and debugging support
  - Real-time audio pipeline protection from UI event blocking

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
- **Performance**: ≤50ms analysis latency, zero-allocation processing

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

##### Graphics Renderer (three-d) - PRIMARY USER INTERFACE
- **Responsibilities**:
  - **ALL end-user interface rendering** - No HTML/CSS for production UI
  - GPU-accelerated rendering pipeline for complete user experience
  - Immersive full-screen visualizations
  - Interactive GPU-rendered controls (buttons, sliders, theme selection)
  - 60fps performance with adaptive resolution
- **Capabilities**: WebGL-based graphics via three-d engine
- **Critical Constraint**: HTML/CSS forbidden for end-user interface elements

#### 4. Development Infrastructure

##### Debug Console (Yew) - HTML/CSS ALLOWED
- **Location**: `dev-console` crate (extracted from main application)
- **Purpose**: Reusable synchronous command input/output interface
- **Architecture**: Standalone library crate designed for reusability across projects
- **Features**:
  - Command execution system with extensible ConsoleCommand trait
  - Pure synchronous command I/O operations
  - History management with local storage persistence
  - Framework-agnostic design for maximum reusability
  - Command registry system for pluggable command implementations
- **Service Integration**: Uses ConsoleAudioService interface for audio operations
- **Implementation**: HTML/CSS/Yew components (development tools exception)
- **Availability**: Development builds only
- **Crate Structure**: Independent library crate with minimal dependencies

##### Debug Live Panel (Yew) - HTML/CSS ALLOWED
- **Location**: `src/debug/live_panel`
- **Purpose**: Real-time monitoring and asynchronous data visualization
- **Architecture**: Event-driven component for live data updates
- **Features**:
  - Device list monitoring and updates
  - Microphone permission status display
  - Performance metrics (FPS, audio latency, memory usage)
  - Audio processing metrics (volume, pitch detection updates)
- **Implementation**: HTML/CSS/Yew components (development tools exception)
- **Integration**: Real-time metrics from all system components via event system

##### Debug Permission Button (Yew) - HTML/CSS ALLOWED
- **Location**: `src/debug/permission_button`
- **Purpose**: Dedicated microphone permission request interface
- **Architecture**: Isolated component for permission management
- **Features**:
  - Microphone permission request triggering
  - Permission status indication
  - Conflict-free operation with production permission systems
  - Separate UI element placement
- **Implementation**: HTML/CSS/Yew components (development tools exception)
- **Integration**: Event-driven communication with audio system

#### 5. Audio-Console Decoupling Architecture

##### ConsoleAudioService Interface
- **Purpose**: Clean abstraction layer between console and audio modules
- **Pattern**: Service interface pattern for dependency injection
- **Key Features**:
  - Permission management delegation (respects browser gesture requirements)
  - Device management interface for audio device enumeration
  - Event publishing for real-time state updates
  - Maintains user interaction context for browser API compliance
- **Implementation**: Trait-based interface with concrete implementation in audio module

##### Event System Architecture
- **Purpose**: Provide domain-specific, event-driven communication layers that enable real-time updates **and** loose coupling between modules within each domain (audio, graphics, UI, theme, configuration, etc.).
- **Architecture Pattern**: Multiple domain-specific dispatchers instead of single global event bus
- **Components**:
  - **Domain-Specific Dispatchers**: Separate event dispatchers for each domain (AudioEventDispatcher, future UIEventDispatcher, GraphicsEventDispatcher, etc.)
  - **Typed Events**: Strongly-typed events within each domain (AudioEvents, GraphicsEvents, UIEvents, ThemeEvents, SystemEvents, etc.)
  - **Type Safety**: Generic `EventDispatcher<T>` prevents mixing event types between domains
  - **Subscription Model**: Callback-based event handling with automatic cleanup within each domain
- **Performance Benefits for Real-Time Audio & Graphics**:
  - **Audio Pipeline Isolation**: Audio events processed independently without UI event blocking
  - **Graphics Pipeline Isolation**: Rendering events don't interfere with audio processing
  - **Latency Protection**: Critical audio events (pitch detection, volume) never blocked by debug/UI events
  - **Memory Locality**: Domain-specific event processing reduces cache misses
  - **Thread Safety**: Each domain's events can be processed on appropriate threads
- **Integration**: Each component receives only the domain-specific dispatcher(s) it needs
- **Guideline**: New modules SHOULD use domain-appropriate event dispatchers and avoid cross-domain direct references

##### Benefits of Decoupling
- **Separation of Concerns**: Console focuses on UI/UX, audio module handles audio processing
- **Testability**: Console component can be tested independently with mock service
- **Maintainability**: Changes to audio internals don't affect console implementation
- **Browser Compliance**: Permission requests maintain proper user gesture context
- **Extensibility**: Service interface allows for future audio system enhancements

## Platform Requirements & Browser Compatibility

### Critical API Dependencies

Pitch Toy implements a **zero-tolerance, fail-fast policy** for missing browser APIs. The application will **prevent startup** rather than provide degraded functionality when critical APIs are unavailable.

#### Required Browser APIs (Application Cannot Start Without)

1. **WebAssembly Support**
   - Required for: Core application logic and audio processing
   - Minimum browser versions enforced during startup validation

2. **Web Audio API & getUserMedia**
   - Required for: Microphone access and real-time audio processing
   - Includes: MediaDevices API for device enumeration and permission management
   - Validation: API availability checked before audio system initialization

3. **AudioWorklet Support**
   - Required for: Low-latency real-time audio processing in dedicated thread
   - Performance target: ≤30ms audio processing latency
   - Alternative: None - application will not start without AudioWorklet support

4. **WebGL/Canvas Support**
   - Required for: GPU-accelerated graphics rendering (primary user interface)
   - All end-user interactions must be GPU-rendered - no HTML/CSS fallbacks
   - Performance target: Consistent 60fps rendering

### Platform Validation Architecture

#### Startup Validation Flow
```
Application Load
    │
    ▼
Platform Feature Detection
    │
    ├─── WebAssembly Available? ──── NO ──┐
    ├─── Web Audio API Available? ─── NO ──┤
    ├─── AudioWorklet Available? ──── NO ──┤
    ├─── WebGL/Canvas Available? ───── NO ──┤
    │                                      │
    ▼ ALL YES                              ▼
Initialize Application                Error Screen
    │                                   │
    ▼                                   ├─── Browser Requirements
Normal Startup Flow                     ├─── Upgrade Instructions
                                        └─── Technical Details
```

#### Error Handling Strategy

- **Critical API Missing**: Application displays informative error screen and prevents initialization
- **User Permission Denied**: Graceful handling with retry options and user guidance  
- **Device Unavailable**: Graceful handling with device reconnection logic
- **Runtime API Failures**: Error recovery where possible, graceful degradation for non-critical features only

### Browser Compatibility Matrix

#### Minimum Requirements (Enforced at Startup)
| Browser | Version | WebAssembly | Web Audio | AudioWorklet | WebGL | Status |
|---------|---------|-------------|-----------|--------------|-------|--------|
| Chrome  | 66+     | ✅          | ✅        | ✅           | ✅    | **Required** |
| Firefox | 76+     | ✅          | ✅        | ✅           | ✅    | **Required** |
| Safari  | 14.1+   | ✅          | ✅        | ✅           | ✅    | **Required** |
| Edge    | 79+     | ✅          | ✅        | ✅           | ✅    | **Required** |

#### Mobile Support (Enforced at Startup)
| Platform | Version | Status | Notes |
|----------|---------|--------|-------|
| iOS Safari | 14.5+ | **Required** | AudioWorklet and WebGL support enforced |
| Chrome Android | 66+ | **Required** | Full feature support validation |
| Samsung Internet | 10+ | ⚠️ **Conditional** | Feature detection determines compatibility |
| Firefox Mobile | 79+ | ⚠️ **Conditional** | AudioWorklet support validation required |

#### Feature Detection Implementation

The platform validation system checks each required API during application startup:

- **Immediate Validation**: All APIs checked before any component initialization
- **Clear Error Messaging**: Specific missing APIs identified in error screens
- **No Graceful Degradation**: Missing critical APIs prevent application startup
- **Development Console Integration**: Platform validation results accessible via console commands

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
- **three-d 0.17**: High-level 3D graphics engine
- **WebGL**: Cross-platform browser graphics API
- **GLSL**: OpenGL Shading Language for GPU shaders

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
- **Minimum Requirements**: Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+ (enforced at startup)
- **Required APIs**: WebAssembly, Web Audio API, AudioWorklet, WebGL (fail-fast validation)
- **Mobile Support**: iOS Safari 14.5+, Chrome Android 66+ (startup validation required)
- **Feature Detection**: **Fail-fast policy** - application prevents startup when critical APIs are missing

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

#### Phased Testing Approach
- **Phase 1 (Current) - Native Tests (cargo test)**: Fast feedback loop for Rust logic validation
  - Immediate feedback during development
  - No browser dependency for core logic testing
- **Phase 2 (Future) - WASM Tests (wasm-pack test)**: WebAssembly-specific functionality validation
  - Planned for when we have audio processing and module interactions
  - Focus on WASM compilation, memory management, and module boundaries
  - **NOT** for browser API integration (use E2E tools like Cypress/Playwright instead)
- **Phase 3 (Later) - E2E Tests**: Browser integration and user workflows
  - Cypress/Playwright for Canvas/WebGPU, Web Audio API, user interactions

#### Unit Testing
- **Phase 1 (Current)**: Build configuration detection (native tests)
- **Phase 2 (Future)**: Audio algorithms, mathematical functions, event system (WASM tests when implemented)
- **Module Structure**: Validates module imports following YAGNI principle
- **Theme System**: Color calculations and transitions (when implemented)

#### Integration Testing
- **End-to-End Pipeline**: Microphone to visualization (E2E tools like Cypress/Playwright)
- **Performance Testing**: Real-time performance under load
- **Cross-Browser Testing**: Automated compatibility validation via E2E testing with fail-fast API validation
- **Mobile Testing**: Different screen sizes and capabilities
- **Canvas Integration**: three-d canvas initialization and GPU rendering setup (E2E tools)
- **WASM Integration**: Module-to-module communication and data boundaries (wasm-pack test)

#### Performance Testing
- **Latency Measurement**: Audio processing and rendering latency
- **Frame Rate Consistency**: 60fps maintenance under load
- **Memory Profiling**: Leak detection and usage optimization
- **Bundle Analysis**: Load time measurement

#### Testing Commands
- **Per-Crate Testing**: 
  - `cd dev-console && cargo test` for console library tests
  - `cd pitch-toy && cargo test` for main application tests
- **Current Phase**: `cargo test` for fast native feedback across all crates
- **Future Phase**: `wasm-pack test --headless --firefox` for WebAssembly validation (when implemented)
- **Later Phase**: Cypress/Playwright for end-to-end browser testing
- **Full Suite**: Phased approach based on development maturity

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
- **Permission-Based Access**: Explicit microphone consent required (after API validation)
- **Local Processing**: No audio data transmission or storage
- **Session Isolation**: No persistent audio data retention
- **Secure Context**: HTTPS requirement for getUserMedia (validated at startup)

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