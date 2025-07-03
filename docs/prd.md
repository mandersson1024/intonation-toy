# Pitch-Toy Product Requirements Document (PRD)

## Goals and Background Context

### Goals

- Deliver a high-performance, real-time pitch detection web application built with Rust/WebAssembly
- Provide accurate pitch feedback with confidence scoring for audio enthusiasts
- Create GPU-powered visualization experience with 60fps performance using three-d graphics engine
- Establish comprehensive development tools and debugging capabilities
- Enable cross-platform browser compatibility with no installation requirements
- Support extensible architecture for future audio processing enhancements
- **Adopt a system-wide event-driven design using typed events and a central Event Dispatcher to maximize decoupling and maintainability across all modules**

### Background Context

Pitch-Toy is a browser-based real-time pitch detection and visualization application built with Rust and WebAssembly for maximum performance. The application provides musicians, vocalists, and audio enthusiasts with precise pitch feedback and beautiful visualizations, emphasizing low-latency audio processing, smooth 60fps graphics, and an extensible architecture for future enhancements.

### Change Log

| Date | Version | Description | Author |
| :--- | :------ | :---------- | :----- |
| 2025-06-29 | 1.0 | Initial PRD creation | John (PM) |
| 2025-07-01 | 1.1 | Added Platform Requirements & fail-fast policy for critical browser APIs | Sarah (PO) |
| 2025-07-02 | 1.2 | Updated graphics stack from wgpu to three-d, changed WebGPU requirement to WebGL | Sarah (PO) |

## Requirements

### Functional

- FR1: The application shall capture real-time microphone input through Web Audio API with user permission management
- FR2: The system shall perform pitch detection with confidence scoring using YIN algorithm and FFT analysis
- FR3: The application shall display real-time pitch visualization with note identification and frequency display
- FR4: The system shall provide volume level monitoring and visualization
- FR5: The application shall include a development console with interactive debugging commands
- FR6: The system shall support theme management for different visual preferences
- FR7: The application shall provide test signal generation for development and validation
- FR8: The system shall display performance metrics and debugging information in development builds
- FR9: The application shall support multiple tuning systems including equal temperament and just intonation
- FR10: The system shall provide extensible command system for runtime configuration
- FR11: The application shall validate all required browser APIs during initialization and prevent startup when critical APIs are unavailable

### Non Functional

- NFR1: Audio processing latency shall not exceed 30ms in production builds (50ms in development)
- NFR2: Graphics rendering shall maintain consistent 60fps performance
- NFR4: Application load time shall not exceed 3 seconds on 3G connections
- NFR5: CPU usage for audio processing shall not exceed 5% on modern devices
- NFR6: GPU memory usage shall not exceed 50MB for texture and buffer allocation
- NFR7: Application shall support Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+ with **mandatory** WebAssembly, Web Audio API, AudioWorklet, and WebGL support. Application startup SHALL be prevented if any required API is unavailable
- NFR8: Mobile compatibility shall include iOS Safari 14.5+ and Chrome Android 66+
- NFR9: Sample rate support of 44.1kHz and 48kHz standard with 22.05kHz-96kHz for development testing
- NFR10: Buffer sizes of 1024 samples (production) and 128-2048 samples (development)

## User Interface Design Goals

### Overall UX Vision

Create a **fully immersive, GPU-rendered** real-time pitch detection interface that provides immediate visual feedback for audio input. **All end-user interactions must be rendered via GPU graphics using three-d** - HTML/CSS is strictly limited to development tools (console and debug overlay). The interface should prioritize performance and responsiveness while offering comprehensive development tools for debugging and configuration.

### Critical UI Architecture Constraint

**GPU-Only End-User Interface**: All user-facing controls, visualizations, and interactions must be rendered through the three-d graphics pipeline. HTML/CSS usage is restricted to:
- Development Console (Yew component for debugging)
- Debug Overlay (Yew component for performance metrics)
- Development-only tools and metering

**No HTML/CSS for Production UI**: Theme switching, pitch visualization, volume controls, and all user interactions must be implemented as GPU-rendered elements, not DOM elements.

### Key Interaction Paradigms

- **Audio-Driven Interface**: Primary interaction through audio input with visual responses
- **Development-First Tooling**: Comprehensive debugging capabilities accessible through interactive console
- **Theme-Based Customization**: Visual theming system for different user preferences
- **Real-Time Responsiveness**: All visual elements respond to audio within specified latency targets

### Core Screens and Views

- **Main Pitch Visualization** (GPU-rendered): Immersive real-time pitch detection display with note identification and frequency information
- **Development Console** (HTML/Yew): Interactive command-line interface for debugging and configuration (development builds only)
- **Performance Metrics Display** (HTML/Yew): Real-time monitoring of frame rate, latency, and resource usage (development builds only)
- **Theme Selection Interface** (GPU-rendered): GPU-rendered controls for switching between available visual themes

### Accessibility: WCAG 2.1 AA

- Browser compatibility across specified minimum versions
- Keyboard navigation for all interactive development console commands
- Clear visual indicators for audio processing states
- Responsive design for desktop and mobile browsers

### Branding

- **Theme System**: Support for multiple visual themes as specified in technical architecture
- **Performance-First Design**: Visual design decisions guided by 60fps rendering requirements
- **Developer-Oriented Interface**: Clear, technical aesthetic appropriate for debugging and development

### Target Device and Platforms

Web Responsive (Desktop and Mobile) - Modern browsers with WebAssembly and Web Audio API support including Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+, iOS Safari 14.5+, and Chrome Android 66+

## Platform Requirements & Compatibility

### Critical Browser APIs (Application Cannot Start Without)

The application has **zero tolerance for missing critical APIs** and implements a **fail-fast policy** for platform validation:

- **WebAssembly Support**: Required for core application logic and audio processing
- **Web Audio API**: Required for microphone access and real-time audio processing  
- **getUserMedia API**: Required for microphone input capture
- **AudioWorklet**: Required for low-latency real-time audio processing
- **WebGL/Canvas Support**: Required for GPU-accelerated graphics rendering
- **MediaDevices API**: Required for device enumeration and permission management

### Application Startup Behavior

- **Fail Fast Policy**: If any critical browser API is missing, the application SHALL display a clear error message and prevent initialization
- **No Graceful Degradation**: The application SHALL NOT attempt to provide alternative implementations when critical APIs are unavailable
- **User Notification**: Missing feature errors SHALL include specific browser requirements and upgrade recommendations
- **Development Console Access**: Platform validation errors SHALL be accessible through development console commands for debugging

### Error Handling Strategy

- **Critical API Missing**: Application prevents startup with informative error screen
- **User Permission Denied**: Graceful handling with retry options and user guidance  
- **Device Unavailable**: Graceful handling with device reconnection logic
- **Runtime API Failures**: Error recovery where possible, graceful degradation for non-critical features only

## Technical Assumptions

### Repository Structure: Monorepo

Single repository structure using Rust/WebAssembly with Trunk build system, organized into modular components as specified in the technical architecture.

### Service Architecture

Client-side only application with no backend dependencies. All processing occurs in the browser using WebAssembly modules for audio processing and three-d for GPU-accelerated graphics rendering.

### Testing Requirements

- Unit tests for audio processing algorithms with deterministic signal inputs
- Performance benchmarks for latency and frame rate consistency
- Cross-browser compatibility testing with automated test runners
- Real-time testing using test signal generation capabilities

### Additional Technical Assumptions and Requests

- Build system uses Trunk for WebAssembly bundling with separate development and production configurations as specified
- Audio processing leverages AudioWorklet for low-latency real-time processing
- Graphics rendering uses three-d engine for WebGL-based GPU acceleration
- Development environment includes comprehensive debugging tools and interactive console
- No external dependencies or network requests required for core functionality
- **Event-driven architecture with typed events through Event Dispatcher** ➜ **MANDATORY system-wide event-driven architecture using strongly-typed events and the central Event Dispatcher for ALL inter-module communication (audio, graphics, UI, configuration, etc.)**
- Modular component design enabling independent development and testing

## Epics

1. **Epic 1 - Foundation & Development Infrastructure**: Establish project setup, build system, and core development tools
2. **Epic 2 - Audio Processing Core**: Implement real-time audio input, pitch detection, and volume analysis
3. **Epic 3 - Visual Presentation System**: Create GPU-powered graphics rendering and visualization
4. **Epic 4 - Theme & Configuration System**: Develop theme management and interactive configuration
5. **Epic 5 - Performance & Production Readiness**: Optimize performance and prepare for production deployment

## Epic 1 - Foundation & Development Infrastructure

Establish the foundational project structure, build system, and development tools necessary for efficient development and debugging. This epic ensures proper tooling and infrastructure while delivering a basic functional application.

### Story 1.1 - Project Setup and Basic Yew Application

As a developer,
I want a properly configured Rust/WebAssembly project with Yew framework,
so that I can begin building the pitch detection application with hot reload and debugging capabilities.

#### Acceptance Criteria

- 1: Cargo.toml configured with Yew 0.21, web-sys, pitch-detection, and rustfft dependencies
- 2: Trunk configuration files created for development and production builds
- 3: Basic Yew application renders and serves at localhost:8080
- 4: Hot reload functionality works correctly during development
- 5: Build system produces optimized WebAssembly for production
- 6: Project structure follows modular architecture as specified

### Story 1.2 - Development Console Component

As a developer,
I want an interactive development console within the application,
so that I can debug audio processing, test features, and monitor application state.

#### Acceptance Criteria

- 1: DevConsole Yew component renders as toggleable overlay
- 2: Command input field accepts and executes commands through DevCommand trait
- 3: Command history maintained with navigation support
- 4: Built-in help command displays available commands and usage
- 5: Console output displays command results and error messages
- 6: Console only available in development builds, hidden in production

### Story 1.3 - Performance Metrics and Debug Overlay

As a developer,
I want real-time performance metrics displayed,
so that I can monitor frame rate, latency, and resource usage during development.

#### Acceptance Criteria

- 1: Performance metrics display shows FPS counter and frame timing
- 2: Audio processing latency measurements displayed
- 3: Memory usage statistics shown for audio buffers and GPU resources
- 4: CPU usage monitoring for audio processing thread
- 5: Metrics update in real-time without affecting performance targets
- 6: Debug overlay can be toggled on/off during development

### Story 1.4 - Test Signal Generator

As a developer,
I want to generate deterministic audio signals for testing,
so that I can validate pitch detection algorithms without requiring microphone input.

#### Acceptance Criteria

- 1: TestSignalGenerator module generates sine waves at specified frequencies
- 2: Console commands available for signal generation and control
- 3: Generated signals replace microphone input when active
- 4: Support for musical intervals and harmonic content as specified
- 5: Phase-coherent signal generation without audible artifacts
- 6: Signal types include sine, harmonic, frequency sweep, and noise as detailed in technical specification

## Epic 2 - Audio Processing Core

Implement the core audio processing capabilities including microphone input, real-time pitch detection using YIN algorithm, and volume analysis. This epic delivers the fundamental audio functionality.

### Story 2.1 - Microphone Input and Audio Context

As a user,
I want the application to access my microphone for audio input,
so that I can use my voice or instrument for pitch detection.

#### Acceptance Criteria

- 1: Application requests microphone permission using getUserMedia API
- 2: AudioContext initialized at appropriate sample rate (44.1kHz and 48kHz standard)
- 3: AudioWorklet processes incoming audio in real-time
- 4: Stream reconnection logic handles device disconnection/reconnection
- 5: Console commands for microphone status and manual permission requests
- 6: Critical API validation with application startup prevention when Web Audio API, getUserMedia, or AudioWorklet are unavailable; graceful error handling only for user permission states

### Story 2.2 - Audio Buffer Management

As a developer,
I want efficient audio buffer management for real-time processing,
so that pitch detection can operate with minimal latency and memory overhead.

#### Acceptance Criteria

- 1: Audio buffer system handles real-time input streaming
- 2: Configurable buffer sizes as specified (1024 production, 128-2048 development)
- 3: Sequential buffer analysis without overlap
- 4: Memory usage stays within specified limits (≤50MB GPU memory)
- 5: Buffer overflow protection with proper error handling
- 6: Zero-allocation operations during steady-state processing

### Story 2.3 - YIN Pitch Detection Implementation

As a user,
I want accurate pitch detection from my audio input,
so that I can see my voice or instrument's pitch in real-time with confidence scoring.

#### Acceptance Criteria

- 1: YIN algorithm implementation using pitch-detection crate
- 2: Pitch detection operates on configurable sample windows
- 3: Frequency output with confidence score (0.0-1.0 range)
- 4: Musical note mapping from frequency to note names and octaves
- 5: Processing latency meets performance requirements (≤30ms production)
- 6: Support for multiple tuning systems as specified

### Story 2.4 - Volume Level Detection

As a user,
I want to see my audio input volume levels,
so that I can monitor my input and ensure proper signal levels for pitch detection.

#### Acceptance Criteria

- 1: Volume level calculation from audio input stream
- 2: Peak level detection with appropriate time constants
- 3: Volume events published with timestamp information
- 4: Configurable sensitivity settings for different input sources
- 5: Visual indication when input levels are too low or too high
- 6: Integration with pitch detection for confidence weighting

### Story 2.5 - FFT Spectral Analysis

As a developer,
I want FFT-based spectral analysis capabilities,
so that I can provide enhanced pitch detection confidence and visualization data.

#### Acceptance Criteria

- 1: FFT implementation using rustfft crate for frequency domain analysis
- 2: Windowing functions applied for spectral accuracy
- 3: Magnitude spectrum calculation with appropriate scaling
- 4: Frequency bins mapped to musical notes and octaves
- 5: Spectrum data available for visualization components
- 6: Configurable FFT size based on analysis requirements

## Epic 3 - Visual Presentation System

Create the GPU-powered graphics rendering system using three-d and implement real-time pitch visualization. This epic establishes the visual presentation layer.

### Story 3.1 - three-d Graphics Pipeline Setup

As a developer,
I want a GPU-accelerated rendering pipeline using three-d,
so that I can create smooth 60fps visualizations that respond to real-time audio data.

#### Acceptance Criteria

- 1: three-d context and canvas initialized for WebGL rendering
- 2: Basic render pipeline created using three-d objects and materials
- 3: Uniform data management for passing audio data to GPU
- 4: Frame timing ensures consistent 60fps rendering performance
- 5: Canvas resizing handled properly for responsive design
- 6: Error handling for WebGL context initialization and context loss

### Story 3.2 - Event Dispatcher Integration

As a developer,
I want the graphics renderer to respond to audio events,
so that visualizations update efficiently based on audio data changes.

#### Acceptance Criteria

- 1: Graphics renderer subscribes to PitchEvent and VolumeEvent types
- 2: Event-driven rendering updates rather than continuous polling
- 3: Render commands generated from audio events for GPU processing
- 4: Frame-rate independent animation with proper time-based interpolation
- 5: Background rendering continues even when no audio events occur
- 6: Memory management prevents texture and buffer leaks during operation

### Story 3.3 - Basic Pitch Visualization

As a user,
I want to see a visual representation of my pitch,
so that I can understand my voice or instrument's fundamental frequency in real-time.

#### Acceptance Criteria

- 1: Pitch indicator renders as visual element responding to frequency changes
- 2: Musical note display shows note name, octave, and frequency information
- 3: Confidence level affects visual representation appropriately
- 4: Smooth interpolation between pitch changes to avoid flickering
- 5: Visual feedback updates within latency requirements (≤30ms)
- 6: Clear indication when no pitch is detected (below confidence threshold)

### Story 3.4 - Volume Visualization

As a user,
I want to see my audio input volume level visually,
so that I can monitor my input and ensure optimal signal levels.

#### Acceptance Criteria

- 1: Volume visualization renders as appropriate visual element
- 2: Real-time updates synchronized with audio processing
- 3: Peak level display with appropriate hold and decay characteristics
- 4: Visual indicators for optimal, low, and excessive input levels
- 5: Integration with pitch visualization without performance impact
- 6: Maintains 60fps rendering performance during volume changes

### Story 3.5 - Presentation Layer Architecture

As a developer,
I want a structured presentation layer that coordinates visual responses,
so that I can manage complex visualizations and maintain performance targets.

#### Acceptance Criteria

- 1: Presentation layer component coordinates between audio events and graphics rendering
- 2: State management for current pitch, volume, and visualization parameters
- 3: Render command generation for GPU graphics pipeline
- 4: Animation coordination and timing management
- 5: Interface with Theme Manager for visual customization
- 6: Performance monitoring to maintain 60fps target

## Epic 4 - Theme & Configuration System

Develop the theme management system and interactive configuration capabilities. This epic provides visual customization and runtime configuration features.

### Story 4.1 - Theme Manager Implementation

As a user,
I want to switch between different visual themes,
so that I can customize the application appearance to match my preferences.

#### Acceptance Criteria

- 1: ThemeManager component manages theme state and transitions
- 2: Support for multiple visual themes as specified in technical architecture
- 3: Theme switching updates all visual elements appropriately
- 4: Theme preference persistence across browser sessions
- 5: Console command for theme switching and management
- 6: Theme transitions do not interrupt audio processing or rendering performance

### Story 4.2 - Runtime Configuration System

As a developer,
I want runtime configuration capabilities for audio and visual parameters,
so that I can adjust application behavior without rebuilding.

#### Acceptance Criteria

- 1: Configuration system supports audio processing parameters
- 2: Visual settings configurable through console commands
- 3: Settings validation to ensure performance targets are maintained
- 4: Configuration persistence where appropriate
- 5: Real-time application of configuration changes
- 6: Help system documenting available configuration options

### Story 4.3 - Interactive Console Commands

As a developer,
I want comprehensive console commands for all system functions,
so that I can efficiently control and debug the application during development.

#### Acceptance Criteria

- 1: Command system implements DevCommand trait as specified
- 2: Commands available for microphone control, signal generation, and system status
- 3: Theme management commands integrated with console
- 4: Performance monitoring and debugging commands
- 5: Command autocomplete and help functionality
- 6: Error handling and feedback for invalid commands

### Story 4.4 - Tuning System Support

As a user,
I want support for different tuning systems,
so that I can use the application for various musical contexts.

#### Acceptance Criteria

- 1: Equal temperament tuning system implementation
- 2: Just intonation support with configurable ratios
- 3: Tuning system selection through configuration interface
- 4: Accurate note mapping for selected tuning system
- 5: Visual indication of active tuning system
- 6: Console commands for tuning system management

## Epic 5 - Performance & Production Readiness

Optimize application performance, validate all requirements are met, and prepare for production deployment. This epic ensures all performance targets are achieved.

### Story 5.1 - Performance Optimization and Validation

As a developer,
I want comprehensive performance optimization,
so that the application meets all specified latency and frame rate requirements.

#### Acceptance Criteria

- 1: Audio processing latency verified under 30ms through automated testing
- 2: Graphics rendering maintains consistent 60fps under typical usage
- 3: Memory usage profiling confirms compliance with specified limits
- 4: CPU usage optimization for both audio processing and graphics rendering
- 6: Performance regression testing integrated into development workflow

### Story 5.2 - Production Build Configuration

As a developer,
I want optimized production builds,
so that users experience fast loading and minimal resource usage.

#### Acceptance Criteria

- 2: Application loads in under 3 seconds on 3G connections
- 3: Debug features and development console removed in production builds
- 4: Asset optimization including shaders and textures
- 5: Build configuration validation ensures all requirements are met
- 6: Production build testing across all supported browsers

### Story 5.3 - Cross-Browser Compatibility Validation

As a user,
I want the application to work consistently across different browsers,
so that I can rely on it regardless of my platform choice.

#### Acceptance Criteria

- 1: Comprehensive testing on Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+
- 2: Mobile compatibility verified on iOS Safari 14.5+ and Chrome Android 66+
- 3: WebAssembly and Web Audio API feature detection with **fail-fast behavior** for missing critical APIs
- 4: Browser-specific optimization for audio latency and graphics performance
- 5: Automated browser testing integrated into development workflow
- 6: Performance benchmarking across different device capabilities

### Story 5.4 - Documentation and Deployment Preparation

As a developer,
I want comprehensive documentation and deployment configuration,
so that the application can be successfully deployed and maintained.

#### Acceptance Criteria

- 1: Technical documentation covers all implemented modules and APIs
- 2: User documentation explains application features and browser requirements
- 3: Deployment configuration for static hosting environments
- 4: Performance monitoring and error reporting integration
- 5: Browser compatibility documentation with **fail-fast policy** for critical API requirements
- 6: Development setup instructions for future contributors

## Next Steps

### UX Expert Prompt

Please review this corrected PRD and update the front-end specification to align with the actual technical requirements. Focus on the specific modules and capabilities defined in the technical specification rather than invented features.

### Architect Prompt

Please create a full-stack architecture document based on this PRD and the existing technical specification. Focus on the modular system design as specified, performance optimization strategies, and integration patterns between the defined components.