## pitch-toy

Real-time pitch detection and visualization tool with musical interval analysis for web browsers.

### Getting Started

- Rust 1.70+ with Cargo
- Modern web browser with WebAssembly and Web Audio API support

```bash
# Development Testing (fast feedback)
cargo test

# Browser Testing (WebAssembly validation)
wasm-pack test --headless --firefox

# Start development server (builds + serves with hot reload)
# Visit http://localhost:8080/
trunk serve
```

### Testing Strategy

This project uses a dual testing approach:

- **Native Tests**: `cargo test` runs 3 tests for fast feedback on Rust logic
- **WASM Tests**: `wasm-pack test --headless --firefox` runs 6 tests in real browser environment

This ensures both rapid development iteration and production environment validation.

### License

MIT License - see LICENSE file for details. 
