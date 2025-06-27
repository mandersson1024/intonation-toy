# Epic 5: Presentation Layer Restructure - Story Breakdown

**Epic ID:** `EPIC-005`  
**Priority:** High  
**Dependencies:** Audio Foundations Module (EPIC-003), Graphics Foundations preparation  
**Total Stories:** 6  
**Completed:** 5/6 (Stories 026, 027, 028, 029, 030 complete)

---

## Story 026: Developer UI Module Creation

**Story ID:** `STORY-026`  
**Epic:** Presentation Layer Restructure  
**Priority:** Critical  
**Story Points:** 13  
**Dependencies:** EPIC-003 complete  

### User Story
> As a **developer**, I want **existing Yew debug components organized in a dedicated Developer UI module** so that I can **maintain comprehensive debugging tools while keeping them completely separate from immersive user experience code**.

### Acceptance Criteria
- [x] New `developer_ui` module created with clear separation from presentation layer
- [x] All existing Yew debug components moved to `src/modules/developer_ui/`
- [x] Developer UI module conditionally compiled for debug builds only
- [x] Component registration system for debug overlay management
- [x] All existing debug functionality preserved
- [x] Zero impact on production builds (module completely excluded)
- [x] Clear architectural boundaries between debug tools and user experience

### Technical Requirements
- **Module Isolation:** Complete separation between developer tools and user experience
- **Conditional Compilation:** Developer UI entirely excluded from production builds
- **Performance:** Zero impact on production build size or runtime performance
- **Debug Experience:** All current debugging capabilities preserved and enhanced

### Definition of Done
- [x] Developer UI module structure created and properly configured
- [x] All debug components migrated to developer_ui module
- [x] Conditional compilation working (excluded from production)
- [x] Debug component registration system implemented
- [x] All existing debug functionality verified working
- [x] Production build size unaffected by debug components
- [x] Developer documentation for debug module architecture

### Implementation Notes
```rust
// Clear module separation:
src/modules/
├── presentation_layer/          # Immersive UI coordination (wgpu-based)
│   ├── mod.rs
│   ├── immersive_coordinator.rs
│   └── theme_manager.rs
└── developer_ui/               # Debug tools (Yew-based, conditionally compiled)
    ├── mod.rs                  # #[cfg(debug_assertions)]
    ├── components/
    │   ├── audio_controls/
    │   │   ├── mod.rs
    │   │   ├── audio_control_panel.rs
    │   │   ├── microphone_panel.rs
    │   │   └── test_signal_generator.rs
    │   ├── debug_interface/
    │   │   ├── mod.rs
    │   │   ├── debug_panel.rs
    │   │   └── debug_interface.rs
    │   ├── error_display/
    │   │   ├── mod.rs
    │   │   ├── error_display.rs
    │   │   └── error_toast.rs
    │   ├── metrics_display/
    │   │   ├── mod.rs
    │   │   ├── metrics_display.rs
    │   │   └── performance_monitor.rs
    │   └── microphone_permission/
    │       ├── mod.rs
    │       ├── microphone_permission.rs
    │       └── fallback_ui.rs
    ├── overlay_manager.rs      # Debug overlay coordination
    └── debug_app.rs           # Main debug application

// Conditional compilation for entire module:
#[cfg(debug_assertions)]
pub mod developer_ui;

// Debug component registration (debug builds only):
#[cfg(debug_assertions)]
pub trait DebugComponentRegistry {
    fn register_debug_component<T: Component + 'static>(&mut self, name: &str);
    fn create_debug_overlay(&self) -> DebugOverlay;
}
```

---

## Story 027: UI Coordinator Architecture (Foundation for Future Immersive UI)

**Story ID:** `STORY-027`  
**Epic:** Presentation Layer Restructure  
**Priority:** Critical  
**Story Points:** 21  
**Dependencies:** STORY-026  

### User Story
> As a **developer**, I want **UI Coordinator architecture with debug overlay integration** so that I can **establish the foundation for future immersive UI while maintaining comprehensive debug capabilities**.

### **CRITICAL ARCHITECTURAL NOTE:**
This story establishes the **coordination layer and debug overlay system** ONLY. The actual immersive audio visualization rendering will be implemented in the subsequent Graphics Epic after Graphics Foundations (Story 028) is complete. This story provides the architectural foundation but not the visual immersive experience itself.

### Acceptance Criteria
- [x] UI Coordinator architecture implemented (without actual immersive rendering)
- [x] Debug overlay system for Yew-based development tools
- [x] Debug overlay conditionally compiled for development builds only
- [x] Event routing infrastructure between future immersive UI and debug overlay
- [x] State synchronization framework for UI and debug information
- [x] Debug overlay can be toggled on/off during development
- [x] Zero performance impact when debug overlay disabled
- [x] Placeholder/stub immersive renderer for coordination testing

### Technical Requirements
- **Architecture:** UI Coordinator foundation for future immersive UI with debug overlay
- **Performance:** Coordination layer overhead <1ms (actual rendering performance in Graphics Epic)
- **Debug Performance:** Debug overlay rendering <5ms to avoid interfering with future main UI
- **Conditional Compilation:** Debug overlay completely removed from production builds
- **Future Compatibility:** Architecture must support wgpu-based rendering integration

### Definition of Done
- [x] UI Coordinator architecture implemented for future immersive + debug coordination
- [x] Stub/placeholder immersive renderer for testing coordination architecture
- [x] Debug overlay system with Yew components working
- [x] Conditional compilation for debug features working
- [x] Event system integration infrastructure between future immersive UI and debug overlay
- [x] Performance monitoring shows no coordination overhead when debug disabled
- [x] Debug overlay toggle functionality working
- [x] Architecture validated as ready for Graphics Foundations integration (Story 029)

### Implementation Notes
```rust
// COORDINATION ARCHITECTURE ONLY - Actual rendering in Graphics Epic
pub trait UICoordinator: Send + Sync {
    fn render_immersive_ui(&mut self, state: &UIState) -> Result<(), RenderError>;
    #[cfg(debug_assertions)]
    fn render_debug_overlay(&mut self, debug_state: &DebugState) -> Result<(), RenderError>;
    #[cfg(debug_assertions)]
    fn toggle_debug_overlay(&mut self, visible: bool);
    fn handle_ui_event(&mut self, event: UIEvent) -> Result<(), UIError>;
    fn update_state(&mut self, state: UIState);
}

pub struct PresentationCoordinator {
    // STUB RENDERER - Will be replaced with wgpu renderer in Graphics Epic
    immersive_renderer: Box<dyn ImmersiveRenderer>,
    #[cfg(debug_assertions)]
    developer_ui: Option<crate::modules::developer_ui::DeveloperUI>,
    event_bus: Arc<dyn EventBus>,
}

#[cfg(debug_assertions)]
pub struct DebugOverlay {
    yew_app: App<DebugApp>,
    overlay_visible: bool,
    debug_components: Vec<Box<dyn DebugComponent>>,
}

// PLACEHOLDER INTERFACE - Actual immersive rendering in Graphics Epic
pub trait ImmersiveRenderer: Send + Sync {
    fn render(&mut self, state: &UIState) -> Result<(), RenderError>;
    fn handle_interaction(&mut self, interaction: UserInteraction) -> Result<UIEvent, UIError>;
    fn initialize(&mut self, canvas: &web_sys::HtmlCanvasElement) -> Result<(), RenderError>;
}

// STUB IMPLEMENTATION for Story 027 testing
pub struct StubImmersiveRenderer {
    canvas: Option<web_sys::HtmlCanvasElement>,
}

impl ImmersiveRenderer for StubImmersiveRenderer {
    fn render(&mut self, _state: &UIState) -> Result<(), RenderError> {
        // Placeholder rendering - just clear canvas or show simple placeholder
        Ok(())
    }
    
    fn handle_interaction(&mut self, _interaction: UserInteraction) -> Result<UIEvent, UIError> {
        // Stub interaction handling
        Ok(UIEvent::NoOp)
    }
    
    fn initialize(&mut self, canvas: &web_sys::HtmlCanvasElement) -> Result<(), RenderError> {
        self.canvas = Some(canvas.clone());
        // Basic canvas setup without actual rendering pipeline
        Ok(())
    }
}
```

---

## Story 028: Graphics Foundations Module Structure

**Story ID:** `STORY-028`  
**Epic:** Presentation Layer Restructure  
**Priority:** High  
**Story Points:** 8  
**Dependencies:** STORY-027 (UI Coordinator architecture must be complete)

### User Story
> As a **developer**, I want **Graphics Foundations module structure prepared** so that I can **begin implementing wgpu-based immersive visualizations in the next phase**.

### **ARCHITECTURAL DEPENDENCY NOTE:**
This story requires Story 027's UI Coordinator architecture to be complete because Graphics Foundations must integrate with the established coordination layer. This story prepares the structure for actual immersive rendering which will happen in the subsequent Graphics Epic.

### Acceptance Criteria
- [x] Graphics Foundations module directory structure created
- [x] Module trait definitions for wgpu integration
- [x] Integration points with Presentation Layer defined
- [x] Canvas element management for wgpu rendering
- [x] Basic wgpu context initialization (without complex rendering)
- [x] Error handling for graphics capability detection
- [x] Module registration with Application Core

### Technical Requirements
- **Architecture:** Clear separation between graphics and presentation concerns
- **Compatibility:** Graceful degradation when WebGL/WebGPU unavailable
- **Performance:** Graphics context initialization in <200ms
- **Integration:** Seamless integration with UI Coordinator

### Definition of Done
- [x] Module structure and interfaces defined
- [x] Basic wgpu context initialization working
- [x] Canvas integration with immersive UI rendering
- [x] Graphics capability detection implemented
- [x] Error handling and fallback systems working
- [x] Module registration complete
- [x] Documentation for future graphics development

### Implementation Notes
```rust
// Module structure preparation:
src/modules/graphics_foundations/
├── mod.rs                       # Module exports and registration
├── wgpu_context.rs             # wgpu initialization and context management
├── render_pipeline.rs          # Future rendering pipeline coordination
├── visualization_renderer.rs   # Audio-specific visualization interfaces
└── capabilities.rs             # Graphics capability detection

pub trait GraphicsFoundations: Module {
    fn create_rendering_context(&self, canvas: &web_sys::HtmlCanvasElement) -> Result<RenderingContext, GraphicsError>;
    fn get_graphics_capabilities(&self) -> GraphicsCapabilities;
    fn is_graphics_available(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct GraphicsCapabilities {
    pub webgl_supported: bool,
    pub webgpu_supported: bool,
    pub max_texture_size: u32,
    pub supported_formats: Vec<TextureFormat>,
    pub performance_tier: GraphicsPerformanceTier,
}
```

---

## Story 029: Developer-Defined Theme System

**Story ID:** `STORY-029`  
**Epic:** Presentation Layer Restructure  
**Priority:** High  
**Story Points:** 13  
**Dependencies:** STORY-028 (Graphics Foundations) - **MUST BE COMPLETE FIRST**


### User Stories

#### Primary User Story
> As a **user**, I want to **select from beautiful pre-made visual themes** so that I can **personalize my audio visualization experience without complexity**.

#### Developer Story  
> As a **developer**, I want **comprehensive theme configuration with shaders and animations** so that I can **create stunning visual experiences and maintain creative control**.

### Acceptance Criteria

#### User Experience
- [x] Simple theme selection UI with 2 curated themes
- [x] Instant theme switching without interruption (<100ms)
- [x] Theme persistence in browser local storage
- [x] Each theme provides distinct visual personality
- [x] Themes are placeholders and will be fleshed out later

#### Developer Configuration
- [x] Compile-time theme definition system
- [x] Per-theme shader variants and material properties
- [x] Animation configuration (timing, curves, effects)
- [x] Lighting setup per theme
- [x] Particle system configuration per theme
- [x] Color palette and gradient definitions

#### Technical Implementation
- [x] wgpu-compatible theme abstractions
- [x] Hot-swappable shader pipeline per theme
- [x] Theme registry for compile-time definitions
- [x] Debug overlay maintains simple functional styling (separate from themes)

### Technical Requirements
- **Performance:** Theme switching in <100ms without visual artifacts
- **Graphics Compatibility:** Full integration with wgpu rendering pipeline  
- **Developer Experience:** Rich configuration API for shaders and animations
- **User Experience:** Zero configuration complexity for end users
- **Build Integration:** Themes compiled into application, no runtime loading

### Definition of Done
- [x] Both placeholder themes, simple but distinct, fully configured and working (theme names: Playful, Scientific)
- [x] Theme selection UI with preview functionality implemented
- [x] Theme switching working seamlessly (<100ms)
- [x] Theme persistence implemented and tested
- [x] Developer theme definition system supports all configuration options
- [x] All shader variants implemented and tested
- [x] Performance requirements met for theme switching
- [x] Debug overlay styling remains simple and separate

### Implementation Notes
```rust
// Developer-defined theme architecture
pub struct ThemeDefinition {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    
    // Visual Configuration
    pub color_palette: ColorPalette,
    pub material_properties: MaterialProperties,
    pub lighting_config: LightingRig,
    
    // Rendering Configuration  
    pub shader_variants: ShaderSet,
    pub particle_systems: ParticleConfig,
    pub animation_timings: AnimationConfig,
    pub post_effects: EffectChain,
}

// Compile-time theme registry
const AVAILABLE_THEMES: &[ThemeDefinition] = &[
    AURORA_THEME,      // Northern lights inspired
    OCEANIC_THEME,     // Deep sea visualization  
    NEON_THEME,        // Cyberpunk aesthetic
    FOREST_THEME,      // Natural earth tones
    MINIMAL_THEME,     // Clean, focused
    COSMIC_THEME,      // Space/galaxy theme
];

// User selection interface (no customization complexity)
pub enum UserThemeChoice {
    Aurora, Oceanic, Neon, Forest, Minimal, Cosmic,
}

pub trait ThemeManager: Send + Sync {
    fn get_available_themes(&self) -> Vec<ThemeMetadata>;
    fn get_current_theme(&self) -> &ThemeDefinition;  
    fn set_theme(&mut self, choice: UserThemeChoice) -> Result<(), ThemeError>;
    fn get_theme_preview(&self, choice: UserThemeChoice) -> ThemePreview;
}

// Theme file organization
// src/themes/
// ├── mod.rs                    # Theme registry and manager
// ├── definitions/              # Individual theme definitions
// │   ├── aurora.rs, oceanic.rs, neon.rs, forest.rs, minimal.rs, cosmic.rs
// ├── shaders/                  # Per-theme shader variants
// │   ├── aurora/, oceanic/, neon/, forest/, minimal/, cosmic/
// ├── animations/               # Animation configuration per theme
// └── materials/                # Material property definitions per theme

// Debug overlay uses simple, hardcoded functional styling (unchanged)
#[cfg(debug_assertions)]
pub struct DebugOverlayStyle {
    pub background: &'static str,     // e.g., "rgba(0,0,0,0.8)"
    pub text_color: &'static str,     // e.g., "#ffffff"
    pub border_color: &'static str,   // e.g., "#666666"
    // No theming complexity - just functional readability
}
```


---

## Story 030: Developer UI Event Integration

**Story ID:** `STORY-030`  
**Epic:** Presentation Layer Restructure  
**Priority:** High  
**Story Points:** 13  
**Dependencies:** STORY-026, STORY-027  

### User Story
> As a **developer**, I want **developer UI components integrated with the module event system** so that I can **monitor application state changes and trigger debug actions through the debug overlay**.

### Acceptance Criteria
- [x] Developer UI components can subscribe to module events for monitoring
- [x] Debug components can publish debug/control events to the application event bus
- [x] Event subscription cleanup on debug component unmount
- [x] Type-safe event handling in developer UI components
- [x] Event-driven state updates for debug monitoring displays
- [x] Performance optimized event subscription management for debug overlay
- [x] Developer UI component lifecycle integration with event system

### Technical Requirements
- **Performance:** Event subscription/unsubscription in <1ms
- **Type Safety:** Compile-time verification of event types
- **Memory Management:** Automatic cleanup prevents memory leaks
- **Integration:** Seamless integration with existing Yew patterns

### Definition of Done
- [x] Event subscription hooks for developer UI components
- [x] Debug event publishing utilities implemented
- [x] Type-safe event handling working in debug overlay
- [x] Developer UI component lifecycle integration complete
- [x] Performance benchmarks meet requirements (zero production impact)
- [x] Memory leak prevention verified for debug components
- [x] Developer documentation and debug integration examples complete

### Implementation Notes
```rust
// Yew component integration patterns:
use yew::prelude::*;
use crate::modules::application_core::{EventBus, TypedEventBus};
use crate::modules::audio_foundations::AudioEvent;

#[derive(Properties, PartialEq)]
pub struct AudioControlPanelProps {
    pub event_bus: Rc<dyn EventBus>,
}

#[function_component(AudioControlPanel)]
pub fn audio_control_panel(props: &AudioControlPanelProps) -> Html {
    // Subscribe to audio events
    let audio_state = use_event_subscription::<AudioEvent>(props.event_bus.clone());
    
    // Publish events on user interaction
    let event_bus = props.event_bus.clone();
    let on_start_recording = Callback::from(move |_| {
        event_bus.publish(AudioEvent::StartRecording);
    });

    html! {
        <div class="audio-control-panel">
            <button onclick={on_start_recording}>
                { if audio_state.is_recording { "Stop" } else { "Start" } }
            </button>
        </div>
    }
}

// Custom hook for event subscription
#[hook]
pub fn use_event_subscription<T: Event + Clone + 'static>(
    event_bus: Rc<dyn EventBus>
) -> UseStateHandle<Option<T>> {
    let state = use_state(|| None);
    
    use_effect_with_deps(move |event_bus| {
        let event_bus = event_bus.clone();
        let state = state.clone();
        
        let subscription = event_bus.subscribe(move |event: T| {
            state.set(Some(event));
        });
        
        // Cleanup subscription on component unmount
        move || drop(subscription)
    }, event_bus);
    
    state
}
```

---

## Story 031: Developer UI and Immersive UI Integration Testing

**Story ID:** `STORY-031`  
**Epic:** Presentation Layer Restructure  
**Priority:** High  
**Story Points:** 8  
**Dependencies:** STORY-026, STORY-027, STORY-028, STORY-029, STORY-030  

### User Story
> As a **QA engineer**, I want **comprehensive integration tests for both Developer UI and Immersive UI coordination** so that I can **verify debug tools work correctly and have zero impact on production builds**.

### Acceptance Criteria
- [ ] Integration tests for developer UI component functionality
- [ ] UI Coordinator functionality tests for immersive UI + debug overlay coordination
- [ ] Immersive UI theme switching and persistence tests
- [ ] Event system integration tests for developer UI components
- [ ] Performance regression tests ensuring debug overlay has zero production impact
- [ ] Conditional compilation verification (debug features excluded from production)
- [ ] Developer UI accessibility and usability verification

### Technical Requirements
- **Coverage:** 80% minimum test coverage for developer UI module and immersive UI coordination
- **Performance:** Debug UI tests complete in <30 seconds, zero impact on production builds
- **Conditional Compilation:** Verify debug features completely excluded from production
- **Automation:** Integrated with existing CI/CD pipeline with production build verification

### Definition of Done
- [ ] Comprehensive test suite for developer UI and immersive UI coordination implemented
- [ ] All developer UI component functionality verified working
- [ ] Immersive UI theme system tests passing
- [ ] Developer UI event integration tests passing
- [ ] Production build verification shows zero debug code inclusion
- [ ] Performance benchmarks confirm zero production impact
- [ ] CI/CD integration with conditional compilation verification complete

### Implementation Notes
```rust
// Integration test structure:
tests/integration/presentation_layer/
├── developer_ui_tests.rs           # Verify developer UI component functionality
├── ui_coordinator_tests.rs         # Test immersive UI + debug overlay coordination
├── immersive_theme_tests.rs        # Immersive UI theme switching and persistence
├── developer_ui_event_tests.rs     # Developer UI event system integration
├── performance_tests.rs            # Zero production impact verification
├── conditional_compilation_tests.rs # Verify debug features excluded from production
└── developer_ui_usability_tests.rs # Developer UI accessibility and usability

#[cfg(test)]
mod component_migration_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_debug_audio_control_panel() {
        // Verify debug audio control panel works in developer UI
        #[cfg(debug_assertions)]
        {
            let app = create_test_app_with_debug().await;
            let debug_overlay = app.get_developer_ui().await;
            let component = debug_overlay.find_component::<AudioControlPanel>().await;
            
            // Test debug functionality preserved
            assert!(component.is_rendered());
            assert_eq!(component.get_button_count(), 3);
            
            // Test debug event integration
            component.click_start_button().await;
            assert!(app.received_debug_event::<AudioEvent>().await);
        }
    }
}
```

---

## Epic Summary

### Epic Completion Criteria
- [x] Developer UI module created with clear separation from immersive UI
- [x] UI Coordinator manages immersive UI with optional developer UI module
- [x] Immersive UI theme system enables rich user customization; developer UI uses simple functional styling
- [x] Graphics Foundations module structure prepared for wgpu integration
- [x] Developer UI module event integration enables reactive debugging
- [ ] Comprehensive testing verifies debug functionality preservation with zero production impact

### Integration with Other Epics
- **EPIC-003 (Audio Foundations):** Developer UI module components consume audio events for monitoring and control
- **EPIC-002 (Application Core):** Presentation Layer registers as module and uses event bus for communication
- **Future Graphics Epic:** Graphics Foundations structure enables immersive UI implementation with wgpu

### **CRITICAL ARCHITECTURAL CLARIFICATION**
**Epic 005 Scope:** UI coordination architecture and debug overlay system ONLY
**Future Graphics Epic Scope:** Actual wgpu-based immersive audio visualization rendering
**Story 027:** Provides coordination layer with stub/placeholder renderer for testing
**Story 029:** Provides Graphics Foundations structure for future immersive implementation
**No User Mode Switching:** Single immersive experience for users, debug overlay for developers

### **Implementation Sequence Dependency Chain:**
1. **Story 026** (Complete): Developer UI module creation
2. **Story 027** (Complete): UI Coordinator with stub renderer + debug overlay
3. **Story 029** (Next): Graphics Foundations structure
4. **Story 028** (Themes): Requires Story 029 complete first
5. **Future Graphics Epic** (Dependent on 029): Actual immersive UI rendering

### Risk Mitigation
- **Component Migration Risk:** Migration to debug overlay with conditional compilation safety
- **Performance Risk:** Debug overlay zero impact when disabled, <5ms when enabled
- **User Experience Risk:** Debug features completely removed from production builds
- **Developer Experience Risk:** Rich debug overlay maintains all current development capabilities

This epic establishes the foundation for Pitch-Toy's immersive audio visualization while preserving comprehensive development tools through an optional debug overlay system. 