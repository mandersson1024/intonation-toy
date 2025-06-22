# Technology Stack & Dependencies
## Real-time Pitch Visualizer

**Purpose**: Core technologies and browser requirements

---

## Core Stack

| Component | Technology | Why |
|-----------|------------|-----|
| **Frontend** | Yew 0.21 | Rust components, type safety |
| **Audio** | Web Audio API | Real-time browser audio |
| **Build** | trunk | Yew build system with hot reload |
| **Pitch Detection** | pitch_detection 0.4 | YIN/McLeod algorithms |
| **Graphics** | Canvas/WebGL | Hardware acceleration |
| **WASM Bridge** | wasm-bindgen 0.2 | Rust ↔ JS communication |

## Key Dependencies

**Rust:**
- `yew` - Web framework 
- `web-sys` - Browser API bindings
- `wasm-bindgen` - WASM bridge
- `pitch_detection` - Audio algorithms
- `gloo` - WASM utilities

**Tools:**
- `trunk` - Build and dev server
- `cargo` - Rust compiler

## Browser Support

**Required Features:**
- WebAssembly
- Web Audio API + AudioWorklet  
- getUserMedia (microphone)
- Canvas/WebGL

**Supported Browsers:**
- Chrome 69+, Firefox 76+, Safari 14.1+, Edge 79+

**Unsupported:**
- Internet Explorer (any version)
- Older browsers without AudioWorklet

> No fallbacks provided - users are directed to upgrade browsers.

## Performance Targets

- **Audio Latency**: <50ms
- **Buffer Size**: 1024-2048 samples  
- **Processing**: <70% AudioWorklet budget
- **Memory**: Efficient WASM allocation 