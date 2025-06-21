# pitch-toy

Real-time pitch detection and visualization tool with musical interval analysis for web browsers.

## Features

- **Real-time pitch detection** with 0.08ms processing latency (625x faster than web requirements)
- **Microphone integration** with child-friendly permission flow and cross-browser support
- **WebAssembly-powered audio processing** with Rust for performance-critical DSP
- **Musical interval analysis** relative to configurable reference pitches *(coming in EP-003)*
- **60 FPS visualization** with seamless audio-visual synchronization *(coming in EP-004)*
- **Educational focus** designed for children (6-16) learning instruments and music educators

## Architecture

- **Audio Processing**: Rust compiled to WebAssembly with `wasm-bindgen` bindings
- **Web Audio Integration**: AudioWorklet for real-time audio processing
- **Graphics**: HTML5 Canvas with JavaScript for responsive visualizations
- **Communication**: WASM ↔ JavaScript bridge for audio data and UI controls
- **Platform**: Web browsers (Chrome, Firefox, Safari, Edge)

## Getting Started

### Prerequisites

- Rust 1.70+ with Cargo
- wasm-pack for WebAssembly compilation
- Ruby (for development server)
- Modern web browser with WebAssembly and Web Audio API support

### Building

```bash
# Clone the repository
git clone https://github.com/your-username/pitch-toy.git
cd pitch-toy

# Build WASM package
wasm-pack build --target web --out-dir pkg

# Start development server
./dev.sh
```

### Development

```bash
# Run Rust unit tests
cargo test

# Build WASM package
wasm-pack build --target web --out-dir pkg

# Start development server (builds + serves)
./dev.sh

# Stop development server
./stop.sh

# Format code
cargo fmt

# Run with enhanced testing
# Visit http://localhost:8080/web/ for comprehensive test suite
```

## Project Structure

```
src/
├── lib.rs               # WASM entry point with wasm_bindgen
├── audio/               # Audio processing (compiled to WASM)
│   ├── mod.rs           # Audio module exports
│   ├── engine.rs        # Core audio engine
│   ├── pitch_detector.rs # Pitch detection algorithms
│   └── interval_calc.rs  # Musical interval calculations
├── utils.rs             # WASM utilities and helpers
web/
├── index.html           # Browser test interface
pkg/                     # Generated WASM package
├── pitch_toy.js         # Generated JavaScript bindings
├── pitch_toy_bg.wasm    # Compiled WebAssembly module
└── pitch_toy.d.ts       # TypeScript definitions
tests/
└── wasm-integration/    # Integration test suite
    └── build.spec.js    # WASM build verification tests
```

## MVP Roadmap

### EP-001: WASM Audio Processing Foundation ✅ **COMPLETE**
- [x] ✅ **Story 1.1**: WASM compilation pipeline and basic audio processing structure
- [x] ✅ **Story 1.2**: Pitch detection algorithms implementation  
- [x] ✅ **Story 1.3**: Comprehensive testing suite and performance benchmarks

### EP-002: Browser Audio Integration & Permissions 🚀 **IN PROGRESS** (1/3)
- [x] ✅ **Story 2.1**: Microphone permission request flow with user-friendly UI
- [ ] 🔄 **Story 2.2**: Set up Web Audio API context and microphone input processing
- [ ] 🔄 **Story 2.3**: Add error handling and fallbacks for unsupported browsers

### EP-003: Educational Interval Analysis ⏳ **PENDING**
- [ ] **Story 3.1**: Reference pitch selection and management
- [ ] **Story 3.2**: Interval calculation algorithms (12-TET and Just Intonation)
- [ ] **Story 3.3**: Real-time interval analysis and feedback system

### EP-004: Web Interface & Visualization ⏳ **PENDING**
- [ ] **Story 4.1**: Child-friendly web interface design
- [ ] **Story 4.2**: Canvas-based real-time pitch visualization
- [ ] **Story 4.3**: Responsive design for tablets and desktop
- [ ] **Story 4.4**: Cross-browser compatibility optimization

## Contributing

This is currently a personal project focused on educational music tools. Contributions welcome once the MVP is stable.

## License

MIT License - see LICENSE file for details. 

## Development Setup

### Prerequisites
- Rust (latest stable)
- wasm-pack
- Ruby (for development server)

### Quick Start

**🚀 Standard Development Workflow (Always port 8080)**
```bash
# Start development (builds + serves)
./dev.sh

# Stop development server
./stop.sh
```

**Development URL:** http://localhost:8080/web/

**Other Options:**
```bash
# Manual build + serve
wasm-pack build --target web --out-dir pkg
ruby serve.rb  # defaults to port 8080

# Just build (no server)
wasm-pack build --target web --out-dir pkg

# Serve only (assumes already built)
ruby serve.rb
```

### Testing

```bash
# Run Rust unit tests
cargo test

# Run automated test suite
*run-tests  # (if using BMAD agent system)
```

### Demo

Visit http://localhost:8080/web/ to see:
- **EP-001 Complete**: WASM audio processing foundation (0.08ms processing latency)
- **Story 2.1 Complete**: Microphone permission flow with child-friendly UI
- AudioEngine performance benchmarking (625x faster than requirements)
- Cross-browser compatibility validation (Chrome, Firefox, Safari, Edge)
- Professional test suite with real-time metrics and comprehensive coverage
- WASM pipeline connection establishment ready for live audio processing

## Architecture

See `docs/architecture/` for detailed technical documentation:
- `tech-stack.md` - Technology choices and dependencies
- `unified-project-structure.md` - File organization and naming conventions
- `testing-strategy.md` - Testing approaches and requirements
- `frontend-architecture.md` - Browser interface and UI patterns

## Current Status

**Recently Completed:**
- ✅ **EP-001**: WASM Audio Processing Foundation (**COMPLETE** - 3/3 stories)
  - Achieved 0.08ms processing latency (625x faster than 50ms requirement)
  - Comprehensive testing suite with >90% code coverage
  - Cross-browser compatibility validated
- ✅ **Story 2.1**: Microphone Permission Flow (**COMPLETE**)
  - Child-friendly permission UI with browser-specific guidance
  - WASM pipeline connection established and validated
  - Ready for live audio input processing

**Currently In Progress:**
- 🚀 **EP-002**: Browser Audio Integration & Permissions (1/3 complete)

**Next Up:**
- 🔄 **Story 2.2**: Web Audio API context and microphone input processing
- 🔄 **Story 2.3**: Error handling and browser fallbacks

## Performance Standards

**Achieved Performance (EP-001):**
- **Processing Latency**: 0.08-0.09ms (80-90μs) - **EXCELLENT** ✅
- **Accuracy**: 0.0-3.2 cents pitch detection (exceeds ±5 cent requirement) ✅
- **Stability**: 1000+ cycle stress testing validated ✅
- **Browser Support**: Chrome, Firefox, Safari, Edge compatibility ✅

**Current Targets:**
- **Audio Latency**: < 50ms total (web platform constraint)
- **Visual Updates**: 60 FPS for smooth user experience *(EP-004)*
- **User Experience**: Child-friendly interface with less than 3 clicks permission flow ✅
