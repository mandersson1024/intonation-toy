## pitch-toy

Real-time pitch detection and visualization tool with musical interval analysis for web browsers.

### Prerequisites

- **Rust 1.70+** with Cargo
- **Trunk** - WebAssembly application bundler (`cargo install trunk`)
- **Modern web browser** with WebAssembly and Web Audio API support:
  - Chrome 66+ / Firefox 76+ / Safari 14.1+ / Edge 79+

### Quick Start

```bash
# Testing
cargo test                      # Run native tests (fast feedback)
wasm-pack test --node           # Run WASM tests in Node.js environment
cargo test && wasm-pack test --node # Run full test suite

# Building
trunk build                     # Development build
trunk build --release           # Release build

# Development
trunk serve                     # Start dev server (localhost:8080, development build)
trunk serve --release           # Start dev server (localhost:8080, release build)

# Documentation
cargo doc --no-deps --document-private-items # Module documentation

# Cleanup  
cargo clean                     # Clean Rust build artifacts
rm -rf dist/                    # Clean Trunk build output
```

### Browser Compatibility

| Browser | Version | WebAssembly | Web Audio | AudioWorklet |
|---------|---------|-------------|-----------|--------------|
| Chrome  | 66+     | ✅          | ✅        | ✅           |
| Firefox | 76+     | ✅          | ✅        | ✅           |
| Safari  | 14.1+   | ✅          | ✅        | ✅           |
| Edge    | 79+     | ✅          | ✅        | ✅           |

### License

MIT License - see LICENSE file for details. 
