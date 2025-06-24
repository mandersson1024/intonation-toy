# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Pitch-Toy** is a real-time audio pitch detection and visualization application built with Rust/WebAssembly and Yew for web browsers. It analyzes audio input from microphones, detects pitch using advanced algorithms, and provides both debug interfaces and immersive visualization capabilities.

## Development Commands

### Building and Running
```bash
# Development server with hot reload (recommended)
./serve.sh dev

# Production build and serve
./serve.sh release

# Build only (without serving)
./build.sh dev
./build.sh release

# Unit tests
cargo test
```

### Development Workflow
- The `trunk serve` command provides hot reload - changes are automatically rebuilt and reloaded
- Visit http://localhost:8080/ for the development server
- No need to restart the server after code changes

## Architecture Overview

### Current Architecture (Yew-based)
The project uses Yew framework for a React-like component architecture compiled to WebAssembly:

- **src/main.rs**: Application entry point using Yew renderer
- **src/lib.rs**: WASM exports and module coordination
- **src/components/**: Yew UI components (debug interfaces, audio controls, error displays)
- **src/services/**: Business logic services (audio engine, error management)
- **src/audio/**: Core audio processing (pitch detection, signal analysis, performance monitoring)
- **src/hooks/**: Custom Yew hooks for shared component logic
- **src/types/**: Shared type definitions

### Planned Modular Architecture
The project is transitioning to a modular architecture with 8 distinct modules:
1. **Application Core**: Module orchestration and lifecycle management
2. **Audio Foundations**: Real-time audio processing and pitch detection
3. **Graphics Foundations**: WebGL rendering and visualization (planned)
4. **Data Management**: Audio buffers and configuration persistence
5. **Platform Abstraction**: Browser compatibility and device capabilities
6. **Presentation Layer**: UI coordination between HTML and immersive rendering
7. **Development Tools**: Debug interfaces and performance monitoring
8. **Performance & Observability**: System monitoring and error tracking

### Key Technologies
- **Language**: Rust 1.70+ with WebAssembly compilation
- **Frontend**: Yew 0.21 framework with component-based architecture
- **Audio**: Web Audio API with AudioWorklet for real-time processing
- **Build**: Trunk for hot reload and WASM compilation
- **Pitch Detection**: YIN and McLeod algorithms via pitch_detection crate

## Audio Processing Architecture

### Real-time Pipeline
1. **Microphone Input**: getUserMedia API captures audio stream
2. **Audio Processing**: AudioWorklet processes audio buffers in real-time
3. **Pitch Detection**: YIN/McLeod algorithms analyze frequency content
4. **Visualization**: Debug interfaces display real-time audio data

### Performance Requirements
- Audio latency: <50ms target
- Buffer sizes: 1024-2048 samples
- Processing budget: <70% of AudioWorklet time
- Memory efficiency: Careful WASM allocation management

## Browser Compatibility

### Supported Browsers
- Chrome 69+, Firefox 76+, Safari 14.1+, Edge 79+
- Requires: WebAssembly, Web Audio API, AudioWorklet, getUserMedia, Canvas/WebGL

### Compatibility Strategy
- No fallbacks for unsupported browsers - users directed to upgrade
- Cross-browser compatibility layer in `src/browser_compat.rs`
- Feature detection for audio capabilities

## Development Features

### Debug Interface
- Real-time audio visualization and metrics
- Performance monitoring and profiling tools
- Test signal generation for debugging
- Error tracking and reporting

### Feature Flags
The project uses Cargo feature flags for conditional compilation:
- `debug-features`: Development-time debugging tools
- `performance-profiling`: Performance monitoring
- `stress-testing`: Audio processing stress tests
- `enhanced-validation`: Additional validation layers

### Build Profiles
- **dev**: Full debugging, unoptimized for fast compilation
- **release**: Maximum optimization, minimal debug info

## File Organization

### Source Structure
```
src/
├── main.rs              # Yew application entry point
├── lib.rs               # WASM exports and module coordination
├── audio/               # Core audio processing modules
├── components/          # Yew UI components
├── services/            # Business logic services  
├── hooks/               # Custom Yew hooks
└── types/               # Shared type definitions
```

### Documentation
- **docs/system/architecture/**: Technical architecture documentation
- **docs/production/stories/**: Development user stories and requirements
- **.cursor/rules/**: Development guidelines and project onboarding

## Testing Strategy

### Unit Testing
```bash
cargo test              # Run all unit tests
cargo test audio        # Test specific module
```

### Manual Testing
- Comprehensive manual testing guides in `tests/manual-testing/`
- Browser compatibility validation procedures
- Performance testing methodologies

## Common Development Patterns

### Error Handling
- Centralized error management via `ErrorManager` service
- Error recovery strategies for audio processing failures
- User-friendly error displays with recovery suggestions

### Performance Monitoring
- Real-time performance tracking in debug builds
- Memory usage monitoring for WASM optimization
- Audio processing latency measurement

### Component Development
- Follow existing Yew component patterns in `src/components/`
- Use custom hooks for shared state logic
- Implement proper error boundaries for audio components

## Browser Development Notes

### Web Audio API Considerations
- AudioContext requires user interaction to start
- AudioWorklet requires separate processor files
- Microphone permissions must be requested explicitly

### WebAssembly Integration
- Use `wasm-bindgen` for JavaScript interop
- Careful memory management between WASM and JS
- Console logging via `console_log!` macro for debugging