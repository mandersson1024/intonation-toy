# pitch-toy

Real-time pitch detection and visualization tool with musical interval analysis for web browsers.

## Getting Started

### Prerequisites

- Rust 1.70+ with Cargo
- Modern web browser with WebAssembly and Web Audio API support

### Building

```bash
# Clone the repository
git clone https://github.com/your-username/pitch-toy.git
cd pitch-toy

# Start server (builds automatically with hot reload)
./serve.sh
```

### Development

```bash
# Run Rust unit tests
cargo test

# Start server (builds + serves with hot reload)
# Visit http://localhost:8080/
./serve.sh dev
./serve.sh release

# Format code
cargo fmt
```

## License

MIT License - see LICENSE file for details. 
