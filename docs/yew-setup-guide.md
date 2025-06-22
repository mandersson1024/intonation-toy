# Yew Development Environment Setup Guide

## Overview

This guide covers the complete setup and usage of the Yew development environment for the pitch-toy project. Yew is a modern Rust framework for creating multi-threaded frontend web apps with WebAssembly.

## Prerequisites

- Rust 1.70+ installed
- Node.js 16+ (for browser testing)
- Modern web browser with WebAssembly support

## Installation

### 1. Install Required Tools

```bash
# Install Yew build tools
cargo install trunk
cargo install wasm-pack

# Verify installations
trunk --version
wasm-pack --version
```

### 2. Project Dependencies

The following dependencies are configured in `Cargo.toml`:

```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-hooks = "0.3"
yew-router = "0.18"
web-sys = { version = "0.3", features = ["AudioContext", "MediaDevices", "Navigator", "Window", "Document", "Element", "HtmlElement"] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
gloo = "0.10"
serde = { version = "1.0", features = ["derive"] }
```

## Development Workflow

### Starting Development Server

```bash
# Start Yew development server with hot reload
./dev.sh yew

# Alternative: Direct trunk command
trunk serve --port 8080
```

The development server will:
- ✅ Serve the app at http://localhost:8080
- 🔄 Enable hot reload for instant code changes
- 🐛 Include debug symbols and source maps
- 📡 WebSocket on port 8081 for hot reload

### Building for Production

```bash
# Build optimized production bundle
./scripts/build-yew-prod.sh

# Alternative: Direct trunk command
trunk build --release
```

Production builds include:
- ⚡ WASM optimization with `wasm-opt`
- 📦 Dead code elimination
- 🗜️ Minimal bundle size
- 🚀 Maximum performance optimizations

### Stopping Development Server

```bash
# Stop all development servers
./stop.sh
```

This will stop:
- Main development server (port 8080)
- Hot reload WebSocket server (port 8081)
- Any remaining trunk processes

## Project Structure

```
pitch-toy/
├── src/
│   ├── main.rs              # Yew app entry point
│   ├── browser_compat.rs    # Browser compatibility detection
│   └── audio/               # Existing audio processing modules
├── index.html               # HTML template for Yew app
├── Trunk.toml              # Trunk build configuration
├── dist/                   # Build output directory (auto-generated)
├── scripts/
│   ├── build-yew-dev.sh    # Development build script
│   └── build-yew-prod.sh   # Production build script
└── tests/
    └── browser-automation/
        └── yew-compatibility.spec.js  # Browser compatibility tests
```

## Browser Compatibility

### Supported Browsers

| Browser | Minimum Version | WebAssembly | Web Audio API | Status |
|---------|----------------|-------------|---------------|---------|
| Chrome  | 69+            | ✅          | ✅            | ✅ Full Support |
| Firefox | 76+            | ✅          | ✅            | ✅ Full Support |
| Safari  | 14.1+          | ✅          | ✅            | ✅ Full Support |
| Edge    | 79+            | ✅          | ✅            | ✅ Full Support |

### Unsupported Browsers

- Internet Explorer (all versions)
- Chrome < 69
- Firefox < 76
- Safari < 14.1
- Edge < 79

### Browser Detection

The app automatically detects browser capabilities and displays appropriate messages:

```rust
// Browser compatibility is automatically checked
let browser_info = BrowserInfo::detect()?;
if browser_info.is_supported {
    // Full app functionality
} else {
    // Display upgrade message
}
```

## Testing

### Manual Testing

1. **Development Mode:**
   ```bash
   ./dev.sh yew
   # Visit http://localhost:8080
   # Verify "Hello World from Yew!" appears
   # Check browser compatibility status
   ```

2. **Production Mode:**
   ```bash
   trunk build --release
   # Check dist/ directory for optimized files
   # Verify WASM bundle size < 500KB
   ```

3. **Hot Reload:**
   ```bash
   # With dev server running, edit src/main.rs
   # Browser should auto-refresh with changes
   ```

### Automated Testing

```bash
# Run browser compatibility tests
npx playwright test tests/browser-automation/yew-compatibility.spec.js

# Run cross-browser tests
npm run test:browsers
```

## Development Commands

### Available Scripts

| Command | Description |
|---------|-------------|
| `./dev.sh yew` | Start Yew development server |
| `./dev.sh legacy` | Start legacy WASM + Ruby server |
| `./scripts/build-yew-dev.sh` | Build for development |
| `./scripts/build-yew-prod.sh` | Build for production |
| `./stop.sh` | Stop all servers |
| `trunk clean` | Clean build artifacts |

### Trunk Commands

```bash
# Development server with hot reload
trunk serve --port 8080

# Build for development (debug)
trunk build

# Build for production (optimized)
trunk build --release

# Clean build artifacts
trunk clean

# Watch and build on changes
trunk watch
```

## Troubleshooting

### Common Issues

#### 1. "trunk: command not found"
```bash
# Solution: Install trunk
cargo install trunk
```

#### 2. "WebAssembly is not supported"
- Ensure you're using a supported browser (Chrome 69+, Firefox 76+, Safari 14.1+, Edge 79+)
- Check browser console for specific errors

#### 3. "Hot reload not working"
- Verify WebSocket connection on port 8081
- Check firewall settings
- Restart development server

#### 4. "WASM files not loading"
- Check MIME types in server configuration
- Verify WASM files exist in dist/ directory
- Check browser network tab for 404 errors

#### 5. "Build fails with 'Is a directory' error"
```bash
# Solution: Clean conflicting directories
rm -rf dist
trunk build
```

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=debug trunk serve --port 8080

# Check build output
trunk build --verbose
```

## Performance Tips

### Development

- Use `trunk serve` for fast development builds
- Hot reload is optimized for quick iteration
- Source maps enabled for debugging

### Production

- Always use `trunk build --release` for deployment
- WASM optimization reduces bundle size by ~60%
- Dead code elimination removes unused Rust code
- Enable gzip compression on your web server

### Bundle Analysis

```bash
# Check WASM bundle size
ls -lh dist/*.wasm

# Analyze bundle contents
wasm-pack build --target web --dev
wee_alloc::size_of_wasm_memory
```

## Integration with Existing Infrastructure

### Ruby Server Compatibility

The Yew setup is designed to work alongside the existing Ruby development server:

- **Port 8080**: Used by both Yew (`trunk serve`) and Ruby (`serve.rb`)
- **Mode Selection**: Use `./dev.sh yew` or `./dev.sh legacy` to choose
- **File Serving**: Both can serve static files from appropriate directories

### Audio Processing Integration

Yew components can interact with existing Rust audio processing modules:

```rust
// Import existing audio modules
use crate::audio::pitch_detector::PitchDetector;

// Use in Yew components
#[function_component(AudioApp)]
pub fn audio_app() -> Html {
    // Yew component can call Rust audio processing functions
    html! { /* ... */ }
}
```

## Next Steps

1. **Component Development**: Create Yew components for audio visualization
2. **Audio Integration**: Connect Yew UI to existing audio processing
3. **Performance Optimization**: Profile and optimize WASM bundles
4. **Cross-browser Testing**: Expand automated test coverage
5. **Deployment**: Configure production deployment pipeline

## Resources

- [Yew Documentation](https://yew.rs/)
- [Trunk Documentation](https://trunkrs.dev/)
- [WebAssembly](https://webassembly.org/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Web-sys API Reference](https://docs.rs/web-sys/) 