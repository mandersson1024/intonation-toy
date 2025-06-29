# Tech Stack Documentation

## Core Technology Stack

### Frontend Framework
- **Yew 0.21**: React-like framework for Rust/WebAssembly applications
  - Features: CSR (Client-Side Rendering)
  - Purpose: Component-based UI architecture and state management
  - Why chosen: Native Rust integration, type safety, minimal bundle overhead

### WebAssembly Runtime
- **Rust → WebAssembly**: High-performance compute engine
  - Target: `wasm32-unknown-unknown`
  - Binary format: `.wasm` modules
  - Purpose: Real-time audio processing with native performance

### Build System
- **Trunk**: Rust WebAssembly application bundler
  - Hot reload development server
  - Asset optimization and bundling
  - WebAssembly module integration
  - Static file serving

### Audio Processing Stack

#### Web Audio API
- **AudioContext**: Main audio processing context
  - Sample rates: 44.1kHz and 48kHz (standard), 22.05kHz-96kHz (development)
  - Buffer sizes: 1024 samples (production), 128-2048 (development)

- **AudioWorklet**: Real-time audio processing thread
  - Fixed 128-sample processing chunks
  - Isolated audio thread execution
  - Low-latency audio pipeline

- **getUserMedia API**: Microphone input capture
  - Permission management
  - Stream lifecycle handling
  - Device enumeration and selection

#### Pitch Detection Libraries
- **pitch-detection 0.3**: Multi-algorithm pitch detection
  - YIN algorithm implementation
  - Confidence scoring
  - Optimized for real-time processing

- **rustfft 6.0**: Fast Fourier Transform implementation
  - Spectral analysis capabilities
  - Frequency domain processing
  - Optimized for WASM target

### Graphics Rendering Stack

#### GPU Graphics
- **wgpu 0.17**: Cross-platform GPU abstraction
  - Vulkan/Metal/DirectX 12 backends
  - WebGPU browser API integration
  - High-performance graphics pipeline

#### Shader Languages
- **WGSL**: WebGPU Shading Language
  - Cross-platform shader compilation
  - GPU-accelerated visual effects
  - Real-time rendering optimizations

### Development Tools

#### Browser APIs
- **web-sys 0.3**: Web API bindings for Rust
  - DOM manipulation
  - Canvas integration
  - Browser feature detection

- **wasm-bindgen 0.2**: Rust-JavaScript interop
  - Type-safe bindings
  - Custom JS integration
  - Performance optimization

#### Development Environment
- **Browser DevTools**: Debugging and profiling
- **Performance Monitor**: Real-time metrics
- **WASM Inspector**: Module analysis
- **Audio Analyzer**: Web Audio debugging

#### Testing Framework
- **wasm-bindgen-test 0.3**: WebAssembly test framework
  - Browser environment testing
  - Headless browser automation
  - Web API integration testing
  - Cross-browser compatibility validation

## Performance Stack

### Latency Optimization
- **Audio Processing**: ≤30ms (production), ≤50ms (development)
- **Frame Rendering**: ≤16.67ms (60fps target)
- **Event Dispatch**: <1ms for critical audio events

### Memory Management
- **WebAssembly Linear Memory**: Efficient memory layout
- **GPU Memory**: ≤50MB for textures and buffers
- **Circular Buffers**: Zero-allocation audio processing
- **Arc<T> Sharing**: Minimize data copying

### Bundle Optimization
- **Production Bundle**: Compressed WebAssembly
- **Code Splitting**: Lazy loading for development features
- **Asset Optimization**: Texture compression and shader minification
- **Tree Shaking**: Dead code elimination

## Browser Compatibility Matrix

### Minimum Requirements
| Browser | Version | WebAssembly | Web Audio | AudioWorklet | WebGPU |
|---------|---------|-------------|-----------|--------------|--------|
| Chrome  | 66+     | ✅          | ✅        | ✅           | 113+   |
| Firefox | 76+     | ✅          | ✅        | ✅           | 113+   |
| Safari  | 14.1+   | ✅          | ✅        | ✅           | 18+    |
| Edge    | 79+     | ✅          | ✅        | ✅           | 113+   |

### Mobile Support
| Platform | Version | Status | Notes |
|----------|---------|--------|-------|
| iOS Safari | 14.5+ | ✅ | AudioWorklet supported |
| Chrome Android | 66+ | ✅ | Full feature support |
| Samsung Internet | 10+ | ⚠️ | Limited testing |
| Firefox Mobile | 79+ | ⚠️ | AudioWorklet limited |

## Development vs Production Configurations

### Development Stack
```toml
# Additional dependencies for development
serde = { version = "1.0", features = ["derive"] }
js-sys = "0.3"
console_log = "0.2"

# TODO: Add WASM testing in future story when we have WASM-specific functionality to test
# [dev-dependencies]
# wasm-bindgen-test = "0.3"  # For testing WASM compilation and module boundaries
```

**Features:**
- Full debugging symbols
- Development console
- Performance metrics overlay
- Test signal generation
- Hot reload capability
- Verbose logging
- Phased testing strategy (Native → WASM → E2E)

**Testing Commands:**
- **Phase 1**: `cargo test` - native tests for fast feedback (current)
- **Phase 2**: `wasm-pack test --headless --firefox` - WASM functionality validation (future)
- **Phase 3**: Cypress/Playwright - browser integration testing (later)

### Production Stack
```toml
# Minimal dependencies for production
yew = { version = "0.21", features = ["csr"] }
web-sys = "0.3"
pitch-detection = "0.3"
rustfft = "6.0"
wgpu = "0.17"
wasm-bindgen = "0.2"
```

**Optimizations:**
- Maximum compile optimization (`opt-level = 3`)
- Debug symbols stripped
- Bundle compression
- Dead code elimination
- Aggressive inlining

## Architecture Patterns

### Event-Driven Architecture
- **Event Dispatcher**: Central message routing
- **Typed Events**: Compile-time event validation
- **Subscription Management**: Automatic cleanup
- **Priority Queuing**: Performance-critical event handling

### Modular Design
- **Audio Modules**: Independent processing units
- **UI Components**: Reusable Yew components
- **Theme System**: Pluggable visual themes
- **Command System**: Extensible console commands

### Performance Patterns
- **Zero-Copy Architecture**: Minimize memory allocations
- **GPU Offloading**: Compute-intensive tasks on GPU
- **Circular Buffers**: Efficient audio streaming
- **Frame-Rate Independent**: Time-based animations

## Security Considerations

### WebAssembly Security
- **Sandboxed Execution**: Isolated WASM runtime
- **Memory Safety**: Rust memory guarantees
- **No File System Access**: Browser security model
- **CORS Compliance**: Cross-origin resource sharing

### Audio Privacy
- **Permission-Based Access**: Explicit microphone consent
- **Local Processing**: No audio data transmission
- **Session Isolation**: No persistent audio storage
- **Secure Context**: HTTPS requirement for getUserMedia

## Deployment Architecture

### Static Hosting
- **GitHub Pages**: Primary deployment target
- **Netlify/Vercel**: Alternative hosting platforms
- **CDN Integration**: Global content distribution
- **Asset Caching**: Optimized cache strategies

### Progressive Web App
- **Service Worker**: Offline capability (future)
- **Web App Manifest**: Installable web app
- **Cache First**: Performance optimization
- **Background Processing**: Audio worker persistence

## Monitoring and Analytics

### Performance Monitoring
- **Real-time Metrics**: FPS, latency, memory usage
- **Performance API**: Browser timing measurements
- **Error Reporting**: Crash and exception tracking
- **Usage Analytics**: Feature utilization metrics

### Development Telemetry
- **Debug Logging**: Structured development logs
- **Performance Profiling**: CPU and memory analysis
- **Network Monitoring**: Asset loading performance
- **Browser Compatibility**: Feature detection reporting