# Epic 5: Presentation Layer Restructure - Story Breakdown

**Epic ID:** `EPIC-005`  
**Priority:** High  
**Dependencies:** Audio Foundations Module (EPIC-003), Graphics Foundations preparation  
**Total Stories:** 6

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
- [ ] New `developer_ui` module created with clear separation from presentation layer
- [ ] All existing Yew debug components moved to `src/modules/developer_ui/`
- [ ] Developer UI module conditionally compiled for debug builds only
- [ ] Component registration system for debug overlay management
- [ ] All existing debug functionality preserved
- [ ] Zero impact on production builds (module completely excluded)
- [ ] Clear architectural boundaries between debug tools and user experience

### Technical Requirements
- **Module Isolation:** Complete separation between developer tools and user experience
- **Conditional Compilation:** Developer UI entirely excluded from production builds
- **Performance:** Zero impact on production build size or runtime performance
- **Debug Experience:** All current debugging capabilities preserved and enhanced

### Definition of Done
- [ ] Developer UI module structure created and properly configured
- [ ] All debug components migrated to developer_ui module
- [ ] Conditional compilation working (excluded from production)
- [ ] Debug component registration system implemented
- [ ] All existing debug functionality verified working
- [ ] Production build size unaffected by debug components
- [ ] Developer documentation for debug module architecture

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

## Story 027: Immersive UI with Debug Overlay Coordinator

**Story ID:** `STORY-027`  
**Epic:** Presentation Layer Restructure  
**Priority:** Critical  
**Story Points:** 21  
**Dependencies:** STORY-026  

### User Story
> As a **user**, I want **an immersive audio visualization interface** with **optional debug overlay for developers** so that I can **enjoy rich visual feedback while developers can troubleshoot and monitor system performance**.

### Acceptance Criteria
- [ ] UI Coordinator manages immersive UI as primary interface
- [ ] Debug overlay system for Yew-based development tools
- [ ] Debug overlay conditionally compiled for development builds only
- [ ] Event routing between immersive UI and debug overlay
- [ ] State synchronization between main UI and debug information
- [ ] Debug overlay can be toggled on/off during development
- [ ] Zero performance impact when debug overlay disabled

### Technical Requirements
- **Architecture:** Single immersive UI with optional debug overlay
- **Performance:** <16ms render loop for 60fps immersive experience
- **Debug Performance:** Debug overlay rendering <5ms to avoid interfering with main UI
- **Conditional Compilation:** Debug overlay completely removed from production builds

### Definition of Done
- [ ] UI Coordinator architecture implemented for immersive + debug coordination
- [ ] Immersive UI foundation established (preparation for wgpu)
- [ ] Debug overlay system with Yew components working
- [ ] Conditional compilation for debug features working
- [ ] Event system integration between immersive UI and debug overlay
- [ ] Performance monitoring shows no impact when debug disabled
- [ ] Debug overlay toggle functionality working

### Implementation Notes
```rust
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

pub trait ImmersiveRenderer: Send + Sync {
    fn render(&mut self, state: &UIState) -> Result<(), RenderError>;
    fn handle_interaction(&mut self, interaction: UserInteraction) -> Result<UIEvent, UIError>;
    fn initialize(&mut self, canvas: &web_sys::HtmlCanvasElement) -> Result<(), RenderError>;
}
```

---

## Story 028: Immersive UI Theme System

**Story ID:** `STORY-028`  
**Epic:** Presentation Layer Restructure  
**Priority:** High  
**Story Points:** 13  
**Dependencies:** STORY-027  

### User Story
> As a **user**, I want **rich visual theming for the immersive audio visualization** so that I can **customize the visual experience to match my preferences and environment**.

### Acceptance Criteria
- [ ] Theme definition system for immersive UI (color palettes, materials, effects)
- [ ] Runtime theme switching for immersive experience without interruption
- [ ] Theme persistence in browser local storage
- [ ] wgpu-compatible theme abstractions (shaders, materials, lighting)
- [ ] Dark/light mode themes optimized for audio visualization
- [ ] Custom theme creation and import capabilities
- [ ] Simple, functional debug overlay styling (separate from immersive themes)

### Technical Requirements
- **Performance:** Theme switching in <100ms without visual artifacts
- **Graphics Compatibility:** Themes compatible with wgpu rendering pipeline
- **Debug Simplicity:** Debug overlay uses minimal, functional styling focused on readability
- **Accessibility:** Immersive themes consider visual accessibility for audio feedback

### Definition of Done
- [ ] Immersive UI theme system implemented
- [ ] Runtime theme switching for immersive experience working
- [ ] Theme persistence implemented
- [ ] wgpu-compatible theme abstractions working
- [ ] Default immersive themes (dark/light) implemented
- [ ] Debug overlay uses simple, functional styling
- [ ] Custom theme import/export working

### Implementation Notes
```rust
pub trait ImmersiveThemeManager: Send + Sync {
    fn get_current_theme(&self) -> &ImmersiveTheme;
    fn set_theme(&mut self, theme: ImmersiveTheme) -> Result<(), ThemeError>;
    fn list_available_themes(&self) -> Vec<ThemeMetadata>;
    fn export_theme(&self, theme_id: &str) -> Result<String, ThemeError>;
    fn import_theme(&mut self, theme_data: &str) -> Result<String, ThemeError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmersiveTheme {
    pub id: String,
    pub name: String,
    pub visual_palette: VisualPalette,
    pub material_properties: MaterialProperties,
    pub lighting_config: LightingConfig,
    pub effects_config: EffectsConfig,
}

#[derive(Debug, Clone)]
pub struct VisualPalette {
    pub primary_color: [f32; 4],      // RGBA for wgpu
    pub secondary_color: [f32; 4],
    pub background_color: [f32; 4],
    pub accent_colors: Vec<[f32; 4]>,
    pub gradient_stops: Vec<[f32; 4]>,
}

// Debug overlay uses simple, hardcoded functional styling
#[cfg(debug_assertions)]
pub struct DebugOverlayStyle {
    // Simple CSS variables for functional debug interface
    pub background: &'static str,     // e.g., "rgba(0,0,0,0.8)"
    pub text_color: &'static str,     // e.g., "#ffffff"
    pub border_color: &'static str,   // e.g., "#666666"
    // No theming complexity - just functional readability
}
```

---

## Story 029: Graphics Foundations Module Structure

**Story ID:** `STORY-029`  
**Epic:** Presentation Layer Restructure  
**Priority:** High  
**Story Points:** 8  
**Dependencies:** STORY-027  

### User Story
> As a **developer**, I want **Graphics Foundations module structure prepared** so that I can **begin implementing wgpu-based immersive visualizations in the next phase**.

### Acceptance Criteria
- [ ] Graphics Foundations module directory structure created
- [ ] Module trait definitions for wgpu integration
- [ ] Integration points with Presentation Layer defined
- [ ] Canvas element management for wgpu rendering
- [ ] Basic wgpu context initialization (without complex rendering)
- [ ] Error handling for graphics capability detection
- [ ] Module registration with Application Core

### Technical Requirements
- **Architecture:** Clear separation between graphics and presentation concerns
- **Compatibility:** Graceful degradation when WebGL/WebGPU unavailable
- **Performance:** Graphics context initialization in <200ms
- **Integration:** Seamless integration with UI Coordinator

### Definition of Done
- [ ] Module structure and interfaces defined
- [ ] Basic wgpu context initialization working
- [ ] Canvas integration with immersive UI rendering
- [ ] Graphics capability detection implemented
- [ ] Error handling and fallback systems working
- [ ] Module registration complete
- [ ] Documentation for future graphics development

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

## Story 030: Developer UI Event Integration

**Story ID:** `STORY-030`  
**Epic:** Presentation Layer Restructure  
**Priority:** High  
**Story Points:** 13  
**Dependencies:** STORY-026, STORY-027  

### User Story
> As a **developer**, I want **developer UI components integrated with the module event system** so that I can **monitor application state changes and trigger debug actions through the debug overlay**.

### Acceptance Criteria
- [ ] Developer UI components can subscribe to module events for monitoring
- [ ] Debug components can publish debug/control events to the application event bus
- [ ] Event subscription cleanup on debug component unmount
- [ ] Type-safe event handling in developer UI components
- [ ] Event-driven state updates for debug monitoring displays
- [ ] Performance optimized event subscription management for debug overlay
- [ ] Developer UI component lifecycle integration with event system

### Technical Requirements
- **Performance:** Event subscription/unsubscription in <1ms
- **Type Safety:** Compile-time verification of event types
- **Memory Management:** Automatic cleanup prevents memory leaks
- **Integration:** Seamless integration with existing Yew patterns

### Definition of Done
- [ ] Event subscription hooks for developer UI components
- [ ] Debug event publishing utilities implemented
- [ ] Type-safe event handling working in debug overlay
- [ ] Developer UI component lifecycle integration complete
- [ ] Performance benchmarks meet requirements (zero production impact)
- [ ] Memory leak prevention verified for debug components
- [ ] Developer documentation and debug integration examples complete

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
- [ ] Developer UI module created with clear separation from immersive UI
- [ ] UI Coordinator manages immersive UI with optional developer UI module
- [ ] Immersive UI theme system enables rich user customization; developer UI uses simple functional styling
- [ ] Graphics Foundations module structure prepared for wgpu integration
- [ ] Developer UI module event integration enables reactive debugging
- [ ] Comprehensive testing verifies debug functionality preservation with zero production impact

### Integration with Other Epics
- **EPIC-003 (Audio Foundations):** Developer UI module components consume audio events for monitoring and control
- **EPIC-002 (Application Core):** Presentation Layer registers as module and uses event bus for communication
- **Future Graphics Epic:** Graphics Foundations structure enables immersive UI implementation with wgpu

### Architectural Clarification
**Primary UI:** wgpu-based immersive audio visualization (main user experience)
**Debug Overlay:** Yew-based development tools (conditionally compiled, developer-only)
**No User Mode Switching:** Single immersive experience for users, debug overlay for developers

### Risk Mitigation
- **Component Migration Risk:** Migration to debug overlay with conditional compilation safety
- **Performance Risk:** Debug overlay zero impact when disabled, <5ms when enabled
- **User Experience Risk:** Debug features completely removed from production builds
- **Developer Experience Risk:** Rich debug overlay maintains all current development capabilities

This epic establishes the foundation for Pitch-Toy's immersive audio visualization while preserving comprehensive development tools through an optional debug overlay system. 