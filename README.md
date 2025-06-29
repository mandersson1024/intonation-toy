## pitch-toy

Real-time pitch detection and visualization tool with musical interval analysis for web browsers.

### Getting Started

- Rust 1.70+ with Cargo
- Modern web browser with WebAssembly and Web Audio API support

```bash
# Run tests (current: native tests for fast feedback)
cargo test

# Start development server (builds + serves with hot reload)
# Visit http://localhost:8080/
trunk serve
```

### Testing Strategy

This project uses a phased testing approach:

**Phase 1 (Current):**
- **Native Tests**: `cargo test` runs 1 meaningful test for fast feedback on Rust logic

**Phase 2 (Future):**
- **WASM Tests**: `wasm-pack test --headless --firefox` for WebAssembly-specific functionality (when we have audio processing modules)

**Phase 3 (Later):**
- **Browser Integration**: Cypress/Playwright for end-to-end browser API testing

This ensures appropriate testing tools for each development phase.

### License

MIT License - see LICENSE file for details. 
