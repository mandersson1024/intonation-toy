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

# Building
trunk build                     # Development build
trunk build --release           # Release build

# Development
trunk serve                     # Start dev server (localhost:8080, development build)
trunk serve --release           # Start dev server (localhost:8080, release build)


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
