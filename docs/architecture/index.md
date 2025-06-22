# Architecture Documentation Index
## Real-time Pitch Visualizer

**Purpose**: Navigate architecture documentation

---

## Architecture Overview

Real-time audio processing using **Rust + Yew**, achieving <50ms audio latency and 60 FPS graphics in modern browsers.

**Key Principles:**
- **Unified Rust**: Frontend and audio processing in Rust
- **Type Safety**: Compile-time guarantees throughout
- **Component Architecture**: React-like experience with Rust performance
- **Minimal JavaScript**: Only ~15KB bridge for browser APIs

---

## Core Documents

| Document | Purpose |
|----------|---------|
| **[tech-stack.md](tech-stack.md)** | Technologies, dependencies, browser support |
| **[unified-project-structure.md](unified-project-structure.md)** | File organization and naming |
| **[testing-strategy.md](testing-strategy.md)** | Testing approach and tools |

---

## System Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        Browser Environment                       │
│                                                                  │
│  ┌─────────────────┐         ┌─────────────────────────────────┐ │
│  │  Web Audio API  │         │         Main Thread             │ │
│  │                 │         │        (Yew + WASM)             │ │
│  │ ┌─────────────┐ │         │                                 │ │
│  │ │ScriptProc/  │ │         │ ┌─────────────┐ ┌─────────────┐ │ │
│  │ │MediaStream  │◄┼────────►│ │   Frontend  │ │AudioEngine  │ │ │
│  │ │             │ │         │ │    (Yew)    │ │   (WASM)    │ │ │
│  │ └─────────────┘ │         │ │• Components │ │• Pitch Det. │ │ │
│  │                 │         │ │• UI State   │ │• Processing │ │ │
│  │ ┌─────────────┐ │         │ │• Rendering  │ │• Analysis   │ │ │
│  │ │AudioContext │ │         │ └─────────────┘ └─────────────┘ │ │
│  │ │   Setup     │ │         │                                 │ │
│  │ └─────────────┘ │         │     wasm-bindgen generated      │ │
│  └─────────────────┘         │         bindings (~55KB)        │ │
│                              └─────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

## Performance Targets

| Metric | Target |
|--------|--------|
| **Audio Latency** | <50ms |
| **Graphics FPS** | 60 FPS |
| **Pitch Accuracy** | ±5 cents |
| **Memory Usage** | <100MB |
| **Browser Support** | Chrome 69+, Firefox 76+, Safari 14.1+, Edge 79+ | 