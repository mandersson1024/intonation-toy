## intonation-toy

Real-time intonation analysis and visualization tool with musical interval analysis for web browsers.

### Prerequisites

- **Rust 1.70+** with Cargo
- **Trunk** - WebAssembly application bundler (`cargo install trunk`)
- **Modern web browser** with WebAssembly and Web Audio API support:
  - Chrome 66+ / Firefox 76+ / Safari 14.1+ / Edge 79+

### Quick Start

```bash
# Testing
./scripts/test-all.sh           # Run all tests (see TESTING.md)

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

### Building and Testing

This project supports both web (WASM) and native builds with conditional dependencies and feature flags.

#### Feature Flags

- `web` (default): Enables web-specific dependencies and browser APIs
- `test-native`: Enables native testing without web dependencies  
- `separation-logging`: Enables additional debug logging

#### Building

```bash
# Web build (default)
trunk build                          # Development build
trunk build --release                # Production build

# Native build for testing
cargo build --features test-native   # Native development build
cargo build --features test-native --release # Native release build

# Check compilation for both targets
cargo check --features test-native   # Check native compilation
wasm-pack build intonation-toy       # Check web compilation
```

#### Testing

```bash
# Run all tests (both web and native)
./scripts/test-all.sh

# Run only native tests
./scripts/test-native.sh

# Run only web tests  
./scripts/test-all.sh web

# Run specific test pattern on native
./scripts/test-native.sh platform
```

#### Development Workflow

For web development:
```bash
trunk serve                          # Start dev server
./scripts/test-all.sh web           # Test web build
```

For native development/testing:
```bash
cargo check --features test-native  # Quick compilation check
./scripts/test-native.sh            # Full native test suite
```

#### Troubleshooting

**Build Issues:**
- Ensure web features are only used with `wasm32` targets
- Use `--features test-native` for native builds
- Check that all required tools are installed (trunk, wasm-pack)

**Test Issues:**
- Native tests require `--features test-native` flag
- Web tests require Node.js and wasm-pack
- Some tests are platform-specific and won't run on both targets

### Browser Compatibility

| Browser | Version | WebAssembly | Web Audio | AudioWorklet |
|---------|---------|-------------|-----------|--------------|
| Chrome  | 66+     | ✅          | ✅        | ✅           |
| Firefox | 76+     | ✅          | ✅        | ✅           |
| Safari  | 14.1+   | ✅          | ✅        | ✅           |
| Edge    | 79+     | ✅          | ✅        | ✅           |

### License

MIT License - see LICENSE file for details. 
