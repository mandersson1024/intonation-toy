# pitch-toy

Real-time pitch detection and visualization tool with musical interval analysis for web browsers.

## Features

- **Real-time pitch detection** with sub-50ms latency in web browsers
- **Musical interval analysis** relative to configurable reference pitches  
- **WebAssembly-powered audio processing** with Rust for performance-critical DSP
- **60 FPS visualization** with seamless audio-visual synchronization
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

### P0: Core Foundation
- [x] ✅ **Story 1.1**: WASM compilation pipeline and basic audio processing structure
- [ ] 🔄 **Story 1.2**: Pitch detection algorithms implementation
- [ ] 🔄 **Story 1.3**: AudioWorklet integration for real-time processing

### P1: Educational Features  
- [ ] Reference pitch selection and management
- [ ] Interval calculation algorithms (12-TET and Just Intonation)
- [ ] Real-time interval feedback and display

### P2: User Experience
- [ ] Child-friendly web interface design
- [ ] Canvas-based real-time pitch visualization
- [ ] Responsive design for tablets and desktop
- [ ] Cross-browser compatibility optimization

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
- WASM module loading and initialization
- AudioEngine performance benchmarking
- Cross-browser compatibility validation
- Professional test suite with real-time metrics
- Story 1.1 acceptance criteria verification

## Architecture

See `docs/architecture/` for detailed technical documentation:
- `tech-stack.md` - Technology choices and dependencies
- `unified-project-structure.md` - File organization and naming conventions
- `testing-strategy.md` - Testing approaches and requirements
- `frontend-architecture.md` - Browser interface and UI patterns

## Stories

Current implementation:
- ✅ **Story 1.1**: WASM Audio Processing Foundation (**COMPLETE**)

Next up:
- 🔄 **Story 1.2**: Pitch Detection Implementation
- 🔄 **Story 1.3**: AudioWorklet Integration

## Performance Standards

**Established in Story 1.1:**
- **Excellent**: < 100μs per buffer processing
- **Good**: < 500μs per buffer processing  
- **Audio Latency Target**: < 50ms total (web platform constraint)
- **Visual Updates**: 60 FPS for smooth user experience
