# Conditional Compilation Architecture Strategy

## Executive Summary

This document analyzes the current conditional compilation approach in the pitch-toy codebase and proposes architectural improvements to minimize `#[cfg(target_arch = "wasm32")]` repetition while maintaining testability.

## Current State Analysis

### Pain Points
1. **Excessive repetition**: Files like `web/main_scene_ui.rs` have 29+ individual `#[cfg]` attributes
2. **Duplicate stub implementations**: Every web-specific function needs a no-op stub for non-WASM
3. **Mixed concerns**: Platform-specific and cross-platform code intermingled
4. **Maintenance burden**: Changes require updating both WASM and stub implementations

### Current Architecture
```
intonation-toy/
├── engine/        # Mixed: Core logic + Web Audio API
├── model/         # Pure Rust (mostly platform-agnostic)
├── presentation/  # Mixed: Core logic + three-d rendering
├── web/           # Pure web-specific (heavily cfg'd)
└── theory/        # Pure Rust (platform-agnostic)
```

### Why We Need Conditional Compilation
- **Production**: WASM-only browser app
- **Development**: `cargo test` runs faster in native mode
- **IDE support**: rust-analyzer works better without forcing WASM target
- **CI/CD**: Both `cargo test` and `wasm-pack test` need to work

## Proposed Architecture: Hybrid Platform Abstraction

### Core Principle
Separate platform-specific code at the module boundary, not at the function level.

### Three-Tier Structure

```
┌─────────────────────────────────────────┐
│          Application Layer              │
│    (Pure Rust - No Platform Code)       │
├─────────────────────────────────────────┤
│         Platform Abstraction             │
│    (Traits & Platform Selection)        │
├─────────────────────────────────────────┤
│      Platform Implementations           │
│  ┌──────────────┐  ┌──────────────┐    │
│  │   Web/WASM   │  │   Testing    │    │
│  │   (Default)  │  │   (Stubs)    │    │
│  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────┘
```

## Implementation Strategy

### Phase 1: Module-Level Isolation

#### 1.1 Create Platform Module Structure
```rust
// lib.rs
#[cfg(target_arch = "wasm32")]
pub mod platform {
    pub mod web;
    pub use web::*;
}

#[cfg(not(target_arch = "wasm32"))]
pub mod platform {
    pub mod stubs;
    pub use stubs::*;
}
```

#### 1.2 Extract Platform Interfaces
```rust
// platform/traits.rs (always compiled)
pub trait AudioContext {
    fn create() -> Result<Self, Error>;
    fn get_sample_rate(&self) -> f32;
    fn connect_stream(&mut self, stream: MediaStream) -> Result<(), Error>;
}

pub trait UIController {
    fn setup_ui(&self);
    fn update_display(&self, data: &DisplayData);
    fn cleanup(&self);
}

pub trait PerformanceTimer {
    fn now(&self) -> f64;
}
```

#### 1.3 Consolidate Web Implementation
```rust
// platform/web/mod.rs (entire module is #[cfg(target_arch = "wasm32")])
mod audio;
mod ui;
mod timing;

pub use audio::WebAudioContext;
pub use ui::WebUIController;
pub use timing::WebPerformanceTimer;
```

#### 1.4 Provide Test Stubs
```rust
// platform/stubs/mod.rs (entire module is #[cfg(not(target_arch = "wasm32"))])
pub struct StubAudioContext;
impl AudioContext for StubAudioContext { /* minimal implementations */ }

pub struct StubUIController;
impl UIController for StubUIController { /* no-ops */ }
```

### Phase 2: Refactor Existing Modules

#### 2.1 Engine Module Refactoring
**Before:**
```rust
// engine/audio/context.rs
#[cfg(target_arch = "wasm32")]
use web_sys::AudioContext;

#[cfg(target_arch = "wasm32")]
fn create_context() -> AudioContext { /* ... */ }

#[cfg(not(target_arch = "wasm32"))]
fn create_context() -> MockContext { /* ... */ }
```

**After:**
```rust
// engine/audio/context.rs
use crate::platform::AudioContext;

fn create_context() -> impl AudioContext {
    platform::create_audio_context()
}
```

#### 2.2 Web Module Consolidation
**Current:** Individual files with repeated `#[cfg]` attributes
**Proposed:** Single-point module gating

```rust
// web/mod.rs
#![cfg(target_arch = "wasm32")]  // Gate entire module

pub mod error_message_box;
pub mod main_scene_ui;
// All code below is automatically WASM-only
```

### Phase 3: Build Configuration

#### 3.1 Cargo.toml Features
```toml
[features]
default = ["web"]
web = []  # Production web build
test-native = []  # Native test stubs

[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys = { version = "0.3", features = [...] }
wasm-bindgen = "0.2"

[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]
# Test-only dependencies
```

#### 3.2 Build Scripts
```bash
# build-web.sh
wasm-pack build --target web --features web

# test-native.sh  
cargo test --features test-native

# test-wasm.sh
wasm-pack test --features web
```

## Benefits of This Approach

### 1. Reduced Repetition
- Single `#[cfg]` per module instead of per-function
- No duplicate stub implementations scattered across files
- Clear separation of platform code

### 2. Better Maintainability
- Platform-specific code isolated in dedicated modules
- Changes don't require updating multiple locations
- Easier to add new platform targets if needed

### 3. Improved Testing
- Test stubs centralized in one location
- Can mock platform behavior for better test coverage
- Native tests run faster without WASM overhead

### 4. Type Safety
- Trait boundaries enforce consistent interfaces
- Compiler verifies all platforms implement required functionality
- No runtime platform checks needed

## Migration Plan

### Step 1: Low-Risk Modules (Week 1)
- [ ] Extract `web/performance.rs` timer functions to platform module
- [ ] Move `web/utils.rs` utilities to platform module
- [ ] Create initial trait definitions

### Step 2: UI Modules (Week 2)
- [ ] Consolidate `web/main_scene_ui.rs` behind module boundary
- [ ] Extract UI trait interface
- [ ] Create stub implementations

### Step 3: Audio Engine (Week 3)
- [ ] Refactor Web Audio API usage to platform module
- [ ] Extract audio context traits
- [ ] Update engine to use platform abstraction

### Step 4: Testing & Documentation (Week 4)
- [ ] Update all tests to use new structure
- [ ] Document platform abstraction patterns
- [ ] Update CI/CD scripts

## Alternative Approaches Considered

### 1. Separate Crates
**Pros:** Complete isolation, clear boundaries
**Cons:** Overhead of managing multiple crates, versioning complexity
**Verdict:** Overkill for a single-target app

### 2. Feature Flags Only
**Pros:** Flexible configuration
**Cons:** Feature flags don't map well to target architecture
**Verdict:** Better for optional functionality than platform support

### 3. Runtime Platform Detection
**Pros:** Single binary for all platforms
**Cons:** Runtime overhead, impossible for WASM vs native
**Verdict:** Not applicable for compile-time platform differences

### 4. Macro-Based Generation
**Pros:** Can generate stubs automatically
**Cons:** Complex macros, harder to debug
**Verdict:** Consider for Phase 2 if manual stubs become burdensome

## Code Examples

### Example 1: Timer Abstraction
```rust
// platform/traits.rs
pub trait Timer {
    fn now(&self) -> f64;
}

// platform/web/timer.rs
#[cfg(target_arch = "wasm32")]
pub struct WebTimer;

#[cfg(target_arch = "wasm32")]
impl Timer for WebTimer {
    fn now(&self) -> f64 {
        web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now()
    }
}

// platform/stubs/timer.rs  
#[cfg(not(target_arch = "wasm32"))]
pub struct StubTimer {
    start: std::time::Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl Timer for StubTimer {
    fn now(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

// Usage in application code (no cfg needed!)
use crate::platform::Timer;

fn measure_performance(timer: &impl Timer) {
    let start = timer.now();
    // ... do work ...
    let elapsed = timer.now() - start;
}
```

### Example 2: UI Controller
```rust
// platform/traits.rs
pub trait UIController: Send + Sync {
    fn update_tuning_fork(&self, note: MidiNote);
    fn update_tuning_system(&self, system: TuningSystem);
    fn show_error(&self, message: &str);
}

// platform/web/ui_controller.rs (entire file is WASM-only)
pub struct WebUIController {
    // web-specific fields
}

impl UIController for WebUIController {
    fn update_tuning_fork(&self, note: MidiNote) {
        // DOM manipulation code
    }
    // ... other methods
}

// platform/stubs/ui_controller.rs (entire file is test-only)
pub struct StubUIController {
    pub last_error: RefCell<Option<String>>,
}

impl UIController for StubUIController {
    fn show_error(&self, message: &str) {
        *self.last_error.borrow_mut() = Some(message.to_string());
    }
    // ... other no-op methods
}
```

## Metrics for Success

1. **Code Reduction**: >50% fewer `#[cfg]` attributes
2. **Test Speed**: Native tests run 10x faster than WASM tests
3. **Maintenance**: Platform changes isolated to platform/ directory
4. **Developer Experience**: No IDE warnings about unused code
5. **Build Time**: Parallel native and WASM builds possible

## Conclusion

The proposed hybrid platform abstraction architecture provides the best balance between:
- Clean separation of concerns
- Minimal conditional compilation noise
- Fast native testing
- Production WASM deployment

By isolating platform-specific code at module boundaries and using trait abstractions, we can eliminate repetitive `#[cfg]` attributes while maintaining full testability and development velocity.

## Next Steps

1. Review and approve this architecture proposal
2. Create platform module structure with initial traits
3. Migrate one small module as proof of concept
4. Iterate based on learnings
5. Complete migration following the phased plan

## Appendix: Current cfg Attribute Count

| Module | #[cfg] Count | Lines of Code | Ratio |
|--------|--------------|---------------|-------|
| web/main_scene_ui.rs | 29 | 566 | 5.1% |
| web/error_message_box.rs | 11 | 200 | 5.5% |
| web/first_click_handler.rs | 2 | 150 | 1.3% |
| engine/audio/* | 15 | 2000 | 0.75% |
| presentation/mod.rs | 8 | 500 | 1.6% |
| **Total** | **65+** | **3416** | **1.9%** |

Target: Reduce to <10 total `#[cfg]` attributes (module-level only).