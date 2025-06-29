## pitch-toy

Real-time pitch detection and visualization tool with musical interval analysis for web browsers.

### Prerequisites

- **Rust 1.70+** with Cargo
- **Trunk** - WebAssembly application bundler (`cargo install trunk`)
- **Modern web browser** with WebAssembly and Web Audio API support:
  - Chrome 66+ / Firefox 76+ / Safari 14.1+ / Edge 79+

### Quick Start

```bash
# Install Trunk (if not already installed)
cargo install trunk

# Start development server with hot reload
trunk serve

# Visit http://localhost:8080/ in your browser
```

### Development Commands

```bash
# Testing
cargo test                      # Run native tests (fast feedback)

# Development
trunk serve                     # Start dev server with hot reload at localhost:8080

# Building
trunk build --release           # Create production build in dist/

# Cleanup  
cargo clean                     # Clean Rust build artifacts
rm -rf dist/                    # Clean Trunk build output
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

### Development Workflow

1. **Make code changes** in `src/`
2. **Hot reload** automatically rebuilds and refreshes browser
3. **Run tests** with `cargo test` for immediate feedback
4. **Check build** with `trunk build --release` before commits

### Project Structure

```
src/
├── lib.rs           # Main application entry point  
└── modules/         # Modular architecture (following YAGNI principle)
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
