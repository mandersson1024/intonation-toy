# Frontend Architecture
## Real-time Pitch Visualizer

**Version**: 1.0  
**Last Updated**: June 2025  
**Purpose**: Define frontend architecture, UI patterns, and browser integration strategies

---

## Overview

The frontend architecture uses a **modern web standards approach** with vanilla JavaScript ES modules, CSS Grid/Flexbox layouts, and progressive enhancement patterns. The design prioritizes performance, accessibility, and cross-browser compatibility.

**Key Design Principles:**
- **Performance First**: Minimal JavaScript overhead, efficient DOM manipulation
- **Progressive Enhancement**: Core functionality works without advanced features
- **Mobile-First Design**: Responsive layouts that scale from mobile to desktop
- **Accessibility**: WCAG 2.1 AA compliance for inclusive user experience
- **Browser Compatibility**: Support for modern browsers (Chrome, Firefox, Safari, Edge)

---

## Architecture Patterns

### Module Architecture
```javascript
// ES6 Module Pattern
import init, { AudioEngine } from '../pkg/pitch_toy.js';

// Class-based Architecture
class TestFramework {
    constructor() { /* Centralized state management */ }
    async measurePerformance(fn, name) { /* Performance utilities */ }
    log(message, type) { /* Logging system */ }
}
```

### State Management
- **Centralized State**: Single TestFramework class manages application state
- **Event-Driven**: DOM events trigger state changes
- **Immutable Updates**: State changes create new objects rather than mutations
- **Error Boundaries**: Graceful error handling and recovery

### UI Component Pattern
```javascript
// Component-like functions for UI updates
function updateStatus(element, status, message) {
    element.className = `status ${status}`;
    element.innerHTML = message;
}

function updateMetric(id, value) {
    document.getElementById(id).textContent = value;
}
```

---

## User Interface Design

### Layout Architecture
**CSS Grid System**: Modern responsive layout with semantic HTML structure

```css
.container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
}

@media (max-width: 768px) {
    .container {
        grid-template-columns: 1fr;
    }
}
```

### Design System
**CSS Variables**: Consistent theming and maintainability
```css
:root {
    --primary: #007bff;
    --success: #28a745;
    --error: #dc3545;
    --warning: #ffc107;
    --info: #17a2b8;
}
```

### Component Library
| Component | Purpose | Location |
|-----------|---------|----------|
| **Status Indicators** | Test result visualization | `.status` classes |
| **Progress Bars** | Test execution tracking | `.progress-bar` |
| **Metrics Dashboard** | Performance monitoring | `.metrics` grid |
| **Interactive Controls** | Test execution buttons | Button system |
| **Log Console** | Development output | `#output` terminal |

---

## Browser Testing Interface

### Story 1.1 Implementation
**Location**: `/web/index.html`
**Architecture**: Professional test suite with enterprise-grade UI

#### Key Features

##### Test Framework Architecture
```javascript
class TestFramework {
    constructor() {
        this.audioEngine = null;
        this.metrics = { /* Performance tracking */ };
        this.testResults = [];
        this.startTime = performance.now();
    }
    
    async measurePerformance(fn, name) {
        // High-precision timing with performance.now()
        const start = performance.now();
        const result = await fn();
        const duration = performance.now() - start;
        return { result, duration };
    }
}
```

##### Performance Monitoring
- **Real-time Metrics**: Live performance dashboard with μs precision
- **Performance Thresholds**: Quality gates for optimization guidance
- **Benchmark Testing**: Statistical analysis with 100+ iteration averages
- **Memory Tracking**: Resource usage monitoring

##### User Experience
- **Progressive Loading**: Tests unlock as dependencies initialize
- **Visual Feedback**: Progress bars, status indicators, color-coded logs
- **Responsive Design**: Works on desktop, tablet, and mobile devices
- **Accessibility**: Keyboard navigation, screen reader support

##### Test Automation
- **One-Click Testing**: Comprehensive test suite execution
- **Stress Testing**: High-volume validation (1000+ iterations)
- **Auto-Run Options**: Configurable test automation
- **Error Recovery**: Graceful failure handling

---

## Performance Optimization

### Loading Strategy
- **ES6 Modules**: Tree-shaking and efficient bundling
- **WASM Lazy Loading**: Initialize only when needed
- **Progressive Enhancement**: Core functionality first, enhancements layer on
- **Resource Hints**: Preload critical resources

### Rendering Optimization
- **CSS Grid**: Hardware-accelerated layout
- **Minimal DOM Manipulation**: Batch updates, avoid layout thrashing
- **Event Delegation**: Efficient event handling
- **Memory Management**: Cleanup listeners and references

### Performance Targets
| Metric | Target | Implementation |
|--------|--------|----------------|
| **Initial Load** | <2s | Optimized assets, minimal JavaScript |
| **Test Execution** | <100ms | Efficient DOM updates, batched operations |
| **Memory Usage** | <50MB | Cleanup patterns, avoid memory leaks |
| **Mobile Performance** | 60 FPS | Hardware acceleration, optimized animations |

---

## Cross-Browser Compatibility

### Browser Support Matrix
| Browser | Version | Status | Notes |
|---------|---------|--------|-------|
| **Chrome** | 69+ | ✅ Full Support | AudioWorklet + WASM support |
| **Firefox** | 76+ | ✅ Full Support | AudioWorklet + WASM support |
| **Safari** | 14.1+ | ✅ Full Support | AudioWorklet + WASM support |
| **Edge** | 79+ | ✅ Full Support | AudioWorklet + WASM support |
| **Internet Explorer** | Any | ❌ Not Supported | No WASM support |
| **Chrome** | <69 | ❌ Not Supported | No AudioWorklet support |
| **Firefox** | <76 | ❌ Not Supported | No AudioWorklet support |
| **Safari** | <14.1 | ❌ Not Supported | No AudioWorklet support |

**Design Decision**: WebAssembly support is mandatory - no JavaScript fallbacks provided.

### Feature Detection
```javascript
const checks = {
    webassembly: typeof WebAssembly !== 'undefined',
    webaudio: typeof AudioContext !== 'undefined',
    promises: typeof Promise !== 'undefined',
    modules: typeof Worker !== 'undefined',
    performance: typeof performance !== 'undefined',
    arraybuffer: typeof ArrayBuffer !== 'undefined',
    float32array: typeof Float32Array !== 'undefined'
};
```

### Polyfills and Fallbacks
- **Web Audio API**: Graceful degradation for older browsers
- **ES6 Features**: Babel transpilation for legacy support
- **CSS Grid**: Flexbox fallbacks for older browsers
- **Performance API**: Date.now() fallback for timing

---

## Development Workflow

### Development Server
**Ruby-based Server**: Reliable cross-platform development environment
```bash
# Start development server
./dev.sh

# Server details
ruby serve.rb 8080  # Always port 8080
```

**Server Features:**
- **WASM MIME Types**: Proper `application/wasm` content type
- **CORS Headers**: Cross-origin resource sharing support
- **Auto-restart**: Port cleanup and conflict resolution
- **Development Logging**: Request monitoring and error reporting

### Build Process
```bash
# Build WASM package
wasm-pack build --target web --out-dir pkg

# Start development server
ruby serve.rb 8080

# Access testing interface
open http://localhost:8080/web/
```

### Testing Workflow
1. **Unit Testing**: `cargo test` for Rust components
2. **Integration Testing**: Browser test suite at `/web/index.html`
3. **Manual Testing**: Interactive test controls and automation
4. **Performance Testing**: Built-in benchmarking and metrics

---

## Future Enhancements

### Planned Improvements
- **Web Components**: Modular, reusable UI components
- **Service Workers**: Offline functionality and caching
- **WebGL Integration**: Hardware-accelerated graphics
- **PWA Features**: Install prompts, push notifications

### Scalability Considerations
- **Component Architecture**: Modular, testable components
- **State Management**: Centralized application state
- **Performance Monitoring**: Real-time metrics and alerting
- **A/B Testing**: Feature flags and user experience optimization

---

## Migration Notes

**Created**: June 2025 as part of Story 1.1 implementation
**Purpose**: Document enhanced browser testing interface and frontend architecture patterns
**Next Steps**: Update for Stories 1.2-1.3 as UI evolves for pitch detection and visualization 