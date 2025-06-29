# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## House Rules
- When inserting today's date, use the `date` command in the terminal to check. Never guess or use a placeholder.
- Never implement changes unless explicitly instructed to do so. If you are uncertain, ask something like "Do you want me to implement these changes?"
- Never refer to Epics, Stories or Acceptance Criteria etc in comments or names.
- When refactoring, never refer to before/after, old/new, legacy/enhanced etc in comments or names.
- Never mention that "this was added", "this replaced that" or "that was deleted/removed" etc in comments.
- Never pretend that you can test something that requires manual testing. Always tell the user to test manually, be specific about what to test, and wait for confirmation.
- Never create unreferenced infrastructure for future tasks. It will only create compiler warnings, complicate code review. The roadmap might change before we get to use it anyway. Assume for now that you aren't gonna need it (YAGNI). Write TODO comments for expected future code. Use stubs as placeholders for incomplete but referenced implementations.
- All UI placeholder/fake values should be drawn in magenta color to make it absolutely clear that they are not implemented.

## Critical UI Architecture Rules
**IMMERSIVE GPU-ONLY USER INTERFACE**: All end-user interactions, controls, and visualizations MUST be rendered via wgpu GPU graphics. HTML/CSS is FORBIDDEN for production user interface.

### HTML/CSS Usage Restrictions:
- ✅ **ALLOWED**: Development Console (Yew component, dev builds only)
- ✅ **ALLOWED**: Debug Overlay (Yew component, dev builds only)  
- ✅ **ALLOWED**: Performance metrics and debugging tools
- ❌ **FORBIDDEN**: Any end-user interface elements (pitch visualization, controls, themes)
- ❌ **FORBIDDEN**: Production UI components
- ❌ **FORBIDDEN**: Theme switching controls via HTML
- ❌ **FORBIDDEN**: Volume or pitch display via HTML

### GPU Rendering Requirements:
- All user-facing controls (buttons, sliders, theme selection) must be GPU-rendered
- All visualizations (pitch detection, spectrum, volume) must be GPU-rendered  
- All themes and visual customization must be implemented in GPU shaders
- Canvas element serves only as wgpu render target, not for HTML overlay

## Project Overview

**Pitch Toy** is a real-time pitch detection and visualization web application built with Rust/WebAssembly and the Yew framework. The project aims to create a browser-based tool that captures microphone input, performs pitch detection with confidence scoring, and visualizes pitch and volume using GPU-powered graphics.

### Key Details
- **Language**: Rust (targeting WebAssembly)
- **Frontend Framework**: Yew (React-like framework for Rust)
- **Rendering Backend**: wgpu (planned for GPU-powered graphics)
- **Build Tool**: Trunk (Rust WASM web application bundler)
- **License**: MIT

## Project Architecture

### Current State
The project is in early development with a minimal "Hello World" Yew application. The codebase structure is set up but most modules are not yet implemented.

### Planned Architecture (from project brief)
```
[Mic Input] ──▶ [Audio Processor] ──▶ [Event Dispatcher]
                                        │
                                        ▼
                           ┌────────────────────────┐
                           ▼                        ▼
               [Presentation Layer]          [Dev Console]
                     │                            │
                     ▼                            ▼
            [Theme Manager]              [Realtime Meters (Yew)]
                     │
                     ▼
            [Graphics Renderer (wgpu)]
```

### Core Modules (Planned)

1. **Microphone Audio Input**
   - Uses `getUserMedia` via `web-sys` or `wasm-bindgen`
   - Streams audio buffers into processing pipeline

2. **Audio Processor**
   - Real-time pitch detection with confidence scoring
   - Emits structured pitch events:
     ```rust
     struct PitchEvent {
         frequency: f32,
         confidence: f32,
     }
     ```

3. **Presentation Layer**
   - Reacts to pitch and volume data
   - Visual pitch/note display
   - Delegates theming to Theme Manager

4. **Theme Manager**
   - Supports user-switchable themes (light/dark/etc.)
   - Supplies color palettes, fonts, and layout hints

5. **Graphics Renderer (wgpu)**
   - Low-level GPU drawing for 60fps performance
   - Receives visualization instructions from Presentation Layer

6. **Dev Console (Yew Component)**
   - Toggleable development/debugging panel
   - Live volume/pitch meters
   - Command registration system via trait:
     ```rust
     trait DevCommand {
         fn name(&self) -> &str;
         fn execute(&self, args: Vec<String>) -> DevCommandOutput;
     }
     ```

7. **Debug Overlay (Yew Component)**
   - FPS monitoring
   - Processing latency metrics
   - Input audio volume/pitch display

## Dependencies

### Core Dependencies
- **yew**: `0.21` with `csr` feature - React-like framework for Rust/WASM
- **wasm-bindgen**: `0.2.100` - Binding layer between Rust and JavaScript

### Build Configuration
- **crate-type**: `["cdylib"]` - Produces a dynamic library for WASM

## Development Workflow

### Environment Requirements
- Rust 1.70+ with Cargo
- Trunk (WASM bundler) - installed at `/Users/mikael/.cargo/bin/trunk`
- Modern web browser with WebAssembly and Web Audio API support

### Commands
```bash
# Run unit tests
cargo test

# Start development server with hot reload
# Serves at http://localhost:8080/
trunk serve

# Build for development (with debug symbols)
# Uses build-configs/dev.toml settings
trunk build --config build-configs/dev.toml

# Build for production (optimized)
# Uses build-configs/release.toml settings
trunk build --config build-configs/release.toml --release
```

### Build Configurations

#### Development (`build-configs/dev.toml`)
- Full debugging capabilities
- Source maps enabled
- Verbose logging
- Hot reload support
- Multiple buffer sizes and sample rates for testing
- Output: `dist/development/`

#### Production (`build-configs/release.toml`)
- Maximum optimization (level 3)
- No debug symbols or source maps
- Minimal bundle size
- Aggressive WASM optimization
- Single optimized buffer size (1024)
- Output: `dist/production/`

## Testing Strategy

- Unit tests via `cargo test`
- Integration tests planned for audio processing
- Performance profiling enabled in development builds
- Test framework included in development builds only

## Key Technical Considerations

### Audio Processing
- Target latency: 50ms (dev) / 30ms (prod)
- Sample rates: 44.1kHz standard, with dev support for 22.05kHz to 96kHz
- Buffer sizes: 1024 samples (prod), 128-2048 samples (dev)

### Performance Targets
- 60fps rendering performance
- Real-time audio processing
- Minimal WASM bundle size in production

### Browser Compatibility
- Requires WebAssembly support
- Requires Web Audio API
- Requires `getUserMedia` for microphone access

## Development Tooling

### Cursor AI Integration
The project includes Cursor AI agent rules in `.cursor/rules/`:
- **Architect** (`@architect`): System design and architecture
- **Developer** (`@dev`): Code implementation and debugging
- **Product Manager** (`@pm`): Product planning
- **QA** (`@qa`): Quality assurance
- **UX Expert** (`@ux-expert`): User experience design

### Project Management
- Uses structured documentation approach
- Supports story-driven development
- Includes templates for architecture and documentation

## Current Status

The project is in the **initial setup phase** with:
- ✅ Basic Yew application structure
- ✅ Build configuration system
- ✅ Development tooling setup
- ⏳ Core modules not yet implemented
- ⏳ Audio processing not implemented
- ⏳ Graphics rendering not implemented

## Next Steps for Development

1. Implement microphone input using Web Audio API
2. Add basic audio processing and pitch detection
3. Create Yew components for visualization
4. Implement theme management system
5. Add development console and debugging tools
6. Integrate wgpu for GPU-powered graphics
7. Add comprehensive testing suite

## Important Notes

- This is a WebAssembly project requiring `trunk` for building
- Audio processing happens in real-time, requiring careful performance optimization
- The project uses a modular architecture designed for extensibility
- Development and production builds have significantly different optimization profiles
- Microphone permissions are required for core functionality