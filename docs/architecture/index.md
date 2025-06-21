# Architecture Documentation Index
## Real-time Pitch Visualizer

**Version**: 1.0  
**Last Updated**: June 2025  
**Purpose**: Navigate the sharded architecture documentation

---

## Architecture Overview

The Real-time Pitch Visualizer uses a **Rust + WebAssembly architecture** with Web Audio API integration to achieve sub-50ms audio latency while maintaining 60 FPS graphics performance in modern browsers.

**Key Design Principles:**
- **Real-time Performance**: Audio processing optimized for WASM with minimal JS boundary crossings
- **Modular Architecture**: Clean separation between WASM audio core, web graphics, and browser APIs
- **Future-Proof Design**: Supports evolution from simple MVP to immersive WebGL graphics
- **Hybrid Language Strategy**: Rust/WASM for performance-critical audio, JavaScript for browser integration

---

## Document Structure

### Core Architecture Documents

| Document | Purpose | When to Read |
|----------|---------|--------------|
| **[tech-stack.md](tech-stack.md)** | Technology choices, dependencies, versions | All stories - understand constraints |
| **[unified-project-structure.md](unified-project-structure.md)** | File organization, naming conventions | All stories - know where code goes |
| **[testing-strategy.md](testing-strategy.md)** | Testing requirements, tools, coverage | All stories - include testing tasks |

### Component-Specific Documentation

| Document | Purpose | Relevant For |
|----------|---------|--------------|
| **[frontend-architecture.md](frontend-architecture.md)** | UI architecture, browser testing interface | Frontend/UI stories, Story 1.1+ |
| **[backend-architecture.md](backend-architecture.md)** | (Future) Server-side components | Backend/API stories |
| **[data-models.md](data-models.md)** | (Future) Data structures, validation | Data-related stories |

---

## Reading Guide by Story Type

### Foundation Stories (EP-001, EP-002)
**Required Reading:**
1. `tech-stack.md` - Understand WASM/Rust requirements
2. `unified-project-structure.md` - Know project organization
3. `testing-strategy.md` - Include proper testing
4. `frontend-architecture.md` - Browser testing interface and UI patterns

### Audio Processing Stories  
**Required Reading:**
1. `tech-stack.md` - Audio dependencies and versions
2. `unified-project-structure.md` - Audio module structure
3. `testing-strategy.md` - Unit testing for audio algorithms

### UI/Frontend Stories
**Required Reading:**
1. `tech-stack.md` - Graphics and UI technologies
2. `unified-project-structure.md` - Web frontend structure
3. `testing-strategy.md` - E2E and integration testing
4. `frontend-architecture.md` - UI patterns, testing interface, browser compatibility

### Full-Stack Stories
**Required Reading:**
- All core architecture documents
- Component-specific documents as needed

---

## System Architecture Summary

### High-Level Components

```
┌─────────────────────────────────────────────────────────────────┐
│                        Browser Environment                       │
│                                                                 │
│  ┌─────────────────┐         ┌─────────────────────────────────┐ │
│  │  AudioWorklet   │         │         Main Thread             │ │
│  │   (WASM Core)   │         │      (JavaScript/WASM)         │ │
│  │                 │         │                                 │ │
│  │ ┌─────────────┐ │         │ ┌─────────────┐ ┌─────────────┐ │ │
│  │ │Audio Engine │ │         │ │   Renderer  │ │Web Controls │ │ │
│  │ │   (Rust)    │ │         │ │(Canvas/WebGL│ │ (HTML/CSS)  │ │ │
│  │ │• Pitch Det. │◄┼────────►│ │             │ │             │ │ │
│  │ │• Intervals  │ │         │ │• Background │ │• UI State   │ │ │
│  │ │• DSP Core   │ │         │ │  Graphics   │ │• User Input │ │ │
│  │ └─────────────┘ │         │ │• 60 FPS     │ │             │ │ │
│  └─────────────────┘         │ └─────────────┘ └─────────────┘ │ │
│           │                   │                                 │ │
│           ▼                   │                                 │ │
│  ┌─────────────────┐         │                                 │ │
│  │  Web Audio API  │         │                                 │ │
│  │  (Browser)      │         │                                 │ │
│  └─────────────────┘         │                                 │ │
│                               └─────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Key Integration Points

1. **WASM/JS Bridge**: Message passing between audio processing and UI
2. **Web Audio API**: Browser audio input/output handling
3. **Canvas/WebGL**: Real-time graphics rendering
4. **DOM Controls**: User interface and settings

---

## Performance Targets

| Metric | Target | Architecture Impact |
|--------|--------|-------------------|
| **Audio Latency** | <50ms | WASM processing, minimal boundary crossings |
| **Graphics FPS** | 60 FPS | Canvas/WebGL optimization |
| **Pitch Accuracy** | ±5 cents | Quality audio algorithms |
| **Memory Usage** | <100MB | WASM linear memory management |
| **Cross-browser** | Chrome, Firefox, Safari | Web standards compliance |

---

## Migration Notes

**Source**: This sharded architecture was created from `docs/technical-architecture.md` (542 lines) for better navigation and story creation workflow compatibility.

**Benefits of Sharded Structure:**
- Easier to reference specific sections in stories
- Reduced context switching during development
- Better maintainability of architecture documentation
- Improved story creation workflow efficiency 