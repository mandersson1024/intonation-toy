# Pitch Visualizer

Real-time pitch detection and visualization tool with musical interval analysis.

## Features

- **Real-time pitch detection** with sub-20ms latency
- **Musical interval analysis** relative to configurable reference pitches  
- **Custom GPU graphics** with egui controls overlay
- **60 FPS visualization** with seamless audio-visual synchronization
- **Educational focus** designed for musicians and music students

## Architecture

- **Audio Processing**: Rust with `cpal` for cross-platform audio I/O
- **Graphics**: `wgpu` for custom rendering + `egui` for UI controls
- **Communication**: Lock-free message passing between audio and GUI threads
- **Platform**: macOS native with plans for cross-platform support

## Getting Started

### Prerequisites

- Rust 1.70+ with Cargo
- macOS 10.15+ (current target platform)
- Audio input device (microphone or audio interface)

### Building

```bash
# Clone the repository
git clone https://github.com/your-username/pitch-visualizer.git
cd pitch-visualizer

# Build and run
cargo run

# For optimized release build
cargo run --release
```

### Development

```bash
# Run tests
cargo test

# Check code
cargo check

# Format code
cargo fmt

# Run with logging
RUST_LOG=debug cargo run
```

## Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library exports
├── audio/               # Audio processing and analysis
│   ├── mod.rs           # Module exports and common types
│   ├── engine.rs        # Core audio engine
│   ├── pitch_detector.rs # Pitch detection algorithms
│   └── interval_calc.rs  # Musical interval calculations
├── gui/                 # Graphics and user interface
│   ├── mod.rs           # GUI module exports
│   ├── app.rs           # Main application controller
│   ├── renderer.rs      # wgpu + egui rendering
│   └── widgets.rs       # Custom UI components
└── bridge/              # Audio-GUI communication
    ├── mod.rs           # Communication types
    └── message_bus.rs   # Thread-safe message passing
```

## MVP Roadmap

### P0: Core Foundation
- [x] Basic audio input and pitch detection
- [x] Minimal visual feedback (frequency + note + cents)
- [x] Audio pipeline stability

### P1: Educational Features  
- [ ] Reference pitch selection
- [ ] Interval calculation and display
- [ ] Performance optimization (<20ms latency)

### P2: User Experience
- [ ] Headphone audio output
- [ ] Tuning system selection (12-TET vs Just Intonation)
- [ ] Polished native GUI

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
- WASM module loading
- AudioEngine initialization
- Real-time audio processing validation
- Story 1.1 acceptance criteria testing

## Architecture

See `docs/architecture/` for detailed technical documentation.

## Stories

Current implementation:
- ✅ Story 1.1: WASM Audio Processing Foundation

Next up:
- 🔄 Story 1.2: Pitch Detection Implementation
- 🔄 Story 1.3: AudioWorklet Integration 
