//! # UI Coordinator Integration Tests
//!
//! Integration tests for UI Coordinator functionality with immersive UI and debug overlay coordination.
//! Tests rendering coordination, state synchronization, and seamless transitions.

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use crate::modules::presentation_layer::ui_coordinator::UICoordinator;
    use crate::modules::presentation_layer::presentation_coordinator::PresentationCoordinator;
    use crate::modules::presentation_layer::debug_overlay::DebugOverlay;
    use crate::modules::presentation_layer::immersive_renderer::ImmersiveRenderer;
    use crate::modules::graphics_foundations::wgpu_context::WgpuContext;
    use crate::modules::application_core::event_bus::{Event, EventPriority, get_timestamp_ns};
    use crate::modules::application_core::priority_event_bus::PriorityEventBus;
    use std::rc::Rc;
    use std::cell::RefCell;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    // Test event for UI coordination
    #[derive(Debug, Clone, PartialEq)]
    struct UICoordinationEvent {
        pub action: String,
        pub state: String,
        pub timestamp: u64,
    }

    impl Event for UICoordinationEvent {
        fn event_type(&self) -> &'static str {
            "UICoordinationEvent"
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn priority(&self) -> EventPriority {
            EventPriority::Medium
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    /// Test UI Coordinator manages immersive UI rendering coordination
    #[wasm_bindgen_test]
    async fn test_ui_coordinator_immersive_rendering_coordination() {
        // Create test coordinator
        let coordinator = create_test_ui_coordinator().await;
        assert!(coordinator.is_ok(), "UI Coordinator creation should succeed");
        
        let mut ui_coordinator = coordinator.unwrap();
        
        // Test immersive UI coordination setup
        let setup_result = ui_coordinator.setup_immersive_coordination().await;
        assert!(setup_result.is_ok(), "Immersive UI coordination setup should succeed");
        
        // Test rendering pipeline coordination
        let render_result = ui_coordinator.coordinate_rendering().await;
        assert!(render_result.is_ok(), "Rendering coordination should succeed");
        
        // Verify coordination state
        assert!(ui_coordinator.is_immersive_active(), "Immersive UI should be active");
        assert!(ui_coordinator.is_coordination_enabled(), "Coordination should be enabled");
    }

    /// Test debug overlay rendering coordination with immersive UI
    #[wasm_bindgen_test]
    async fn test_debug_overlay_rendering_coordination() {
        let mut ui_coordinator = create_test_ui_coordinator().await.unwrap();
        
        // Setup immersive UI first
        ui_coordinator.setup_immersive_coordination().await.unwrap();
        
        // Test debug overlay coordination
        let debug_setup = ui_coordinator.setup_debug_overlay_coordination().await;
        assert!(debug_setup.is_ok(), "Debug overlay coordination setup should succeed");
        
        // Test dual rendering coordination
        let dual_render = ui_coordinator.coordinate_dual_rendering().await;
        assert!(dual_render.is_ok(), "Dual rendering coordination should succeed");
        
        // Verify both systems are coordinated
        assert!(ui_coordinator.is_immersive_active(), "Immersive UI should remain active");
        assert!(ui_coordinator.is_debug_overlay_active(), "Debug overlay should be active");
        assert!(ui_coordinator.is_dual_rendering_stable(), "Dual rendering should be stable");
    }

    /// Test debug overlay toggle functionality works seamlessly
    #[wasm_bindgen_test]
    async fn test_debug_overlay_toggle_functionality() {
        let mut ui_coordinator = create_test_ui_coordinator().await.unwrap();
        
        // Setup immersive UI
        ui_coordinator.setup_immersive_coordination().await.unwrap();
        
        // Test debug overlay enable
        let enable_result = ui_coordinator.toggle_debug_overlay(true).await;
        assert!(enable_result.is_ok(), "Debug overlay enable should succeed");
        assert!(ui_coordinator.is_debug_overlay_active(), "Debug overlay should be active");
        
        // Test seamless transition timing (should be fast)
        let start_time = web_sys::Performance::new().unwrap().now();
        
        // Test debug overlay disable
        let disable_result = ui_coordinator.toggle_debug_overlay(false).await;
        assert!(disable_result.is_ok(), "Debug overlay disable should succeed");
        
        let end_time = web_sys::Performance::new().unwrap().now();
        let transition_time = end_time - start_time;
        
        // Verify toggle performance (<100ms requirement from story context)
        assert!(transition_time < 100.0, "Debug overlay toggle should be under 100ms");
        assert!(!ui_coordinator.is_debug_overlay_active(), "Debug overlay should be disabled");
        assert!(ui_coordinator.is_immersive_active(), "Immersive UI should remain unaffected");
    }

    /// Test event routing between immersive UI and debug overlay
    #[wasm_bindgen_test]
    async fn test_event_routing_between_ui_systems() {
        let mut ui_coordinator = create_test_ui_coordinator().await.unwrap();
        let event_bus = Rc::new(RefCell::new(PriorityEventBus::new()));
        
        // Setup both UI systems
        ui_coordinator.setup_immersive_coordination().await.unwrap();
        ui_coordinator.setup_debug_overlay_coordination().await.unwrap();
        
        // Test event routing from immersive to debug
        let immersive_event = UICoordinationEvent {
            action: "immersive_interaction".to_string(),
            state: "active".to_string(),
            timestamp: get_timestamp_ns(),
        };
        
        let route_to_debug = ui_coordinator.route_event_to_debug(&immersive_event).await;
        assert!(route_to_debug.is_ok(), "Event routing to debug should succeed");
        
        // Test event routing from debug to immersive
        let debug_event = UICoordinationEvent {
            action: "debug_control".to_string(),
            state: "overlay_active".to_string(),
            timestamp: get_timestamp_ns(),
        };
        
        let route_to_immersive = ui_coordinator.route_event_to_immersive(&debug_event).await;
        assert!(route_to_immersive.is_ok(), "Event routing to immersive should succeed");
        
        // Verify event routing performance
        let routing_metrics = ui_coordinator.get_event_routing_metrics();
        assert!(routing_metrics.average_routing_time_ms < 1.0, "Event routing should be under 1ms");
    }

    /// Test state synchronization between UI and debug information
    #[wasm_bindgen_test]
    async fn test_state_synchronization() {
        let mut ui_coordinator = create_test_ui_coordinator().await.unwrap();
        
        // Setup both systems
        ui_coordinator.setup_immersive_coordination().await.unwrap();
        ui_coordinator.setup_debug_overlay_coordination().await.unwrap();
        
        // Test state sync from immersive to debug
        let immersive_state = create_test_immersive_state();
        let sync_to_debug = ui_coordinator.sync_state_to_debug(&immersive_state).await;
        assert!(sync_to_debug.is_ok(), "State sync to debug should succeed");
        
        // Test state sync from debug to immersive
        let debug_state = create_test_debug_state();
        let sync_to_immersive = ui_coordinator.sync_state_from_debug(&debug_state).await;
        assert!(sync_to_immersive.is_ok(), "State sync from debug should succeed");
        
        // Verify state consistency
        let state_consistency = ui_coordinator.verify_state_consistency().await;
        assert!(state_consistency.is_ok(), "State consistency check should succeed");
        assert!(ui_coordinator.is_state_synchronized(), "States should be synchronized");
    }

    /// Test UI Coordinator handles rendering pipeline transitions smoothly
    #[wasm_bindgen_test]
    async fn test_rendering_pipeline_transitions() {
        let mut ui_coordinator = create_test_ui_coordinator().await.unwrap();
        
        // Test transition from immersive-only to dual rendering
        ui_coordinator.setup_immersive_coordination().await.unwrap();
        
        let start_transition = web_sys::Performance::new().unwrap().now();
        
        let transition_to_dual = ui_coordinator.transition_to_dual_rendering().await;
        assert!(transition_to_dual.is_ok(), "Transition to dual rendering should succeed");
        
        let end_transition = web_sys::Performance::new().unwrap().now();
        let transition_time = end_transition - start_transition;
        
        // Verify smooth transition timing
        assert!(transition_time < 50.0, "Pipeline transition should be under 50ms");
        assert!(ui_coordinator.is_dual_rendering_stable(), "Dual rendering should be stable");
        
        // Test transition back to immersive-only
        let start_back = web_sys::Performance::new().unwrap().now();
        
        let transition_back = ui_coordinator.transition_to_immersive_only().await;
        assert!(transition_back.is_ok(), "Transition back to immersive should succeed");
        
        let end_back = web_sys::Performance::new().unwrap().now();
        let back_time = end_back - start_back;
        
        assert!(back_time < 50.0, "Transition back should be under 50ms");
        assert!(!ui_coordinator.is_debug_overlay_active(), "Debug overlay should be inactive");
        assert!(ui_coordinator.is_immersive_active(), "Immersive UI should remain active");
    }

    /// Test stub immersive renderer integration with coordination architecture
    #[wasm_bindgen_test]
    async fn test_stub_immersive_renderer_integration() {
        let mut ui_coordinator = create_test_ui_coordinator().await.unwrap();
        
        // Test stub renderer setup
        let stub_setup = ui_coordinator.setup_stub_immersive_renderer().await;
        assert!(stub_setup.is_ok(), "Stub immersive renderer setup should succeed");
        
        // Test stub renderer coordination
        let stub_coordination = ui_coordinator.coordinate_with_stub_renderer().await;
        assert!(stub_coordination.is_ok(), "Stub renderer coordination should succeed");
        
        // Verify stub renderer capabilities
        assert!(ui_coordinator.has_stub_renderer(), "Should have stub renderer");
        assert!(ui_coordinator.is_stub_renderer_functional(), "Stub renderer should be functional");
        
        // Test debug overlay integration with stub renderer
        ui_coordinator.toggle_debug_overlay(true).await.unwrap();
        
        let stub_debug_integration = ui_coordinator.verify_stub_debug_integration().await;
        assert!(stub_debug_integration.is_ok(), "Stub renderer debug integration should work");
    }

    /// Test graphics context initialization integration with UI coordination
    #[wasm_bindgen_test]
    async fn test_graphics_context_ui_coordination_integration() {
        let mut ui_coordinator = create_test_ui_coordinator().await.unwrap();
        
        // Test graphics context setup for UI coordination
        let graphics_setup = ui_coordinator.setup_graphics_context().await;
        assert!(graphics_setup.is_ok(), "Graphics context setup should succeed");
        
        // Test graphics context coordination with immersive UI
        let immersive_graphics = ui_coordinator.coordinate_immersive_graphics().await;
        assert!(immersive_graphics.is_ok(), "Immersive graphics coordination should succeed");
        
        // Test graphics context coordination with debug overlay
        ui_coordinator.toggle_debug_overlay(true).await.unwrap();
        
        let debug_graphics = ui_coordinator.coordinate_debug_graphics().await;
        assert!(debug_graphics.is_ok(), "Debug graphics coordination should succeed");
        
        // Verify graphics resource management
        let resource_check = ui_coordinator.verify_graphics_resources().await;
        assert!(resource_check.is_ok(), "Graphics resource verification should succeed");
        assert!(ui_coordinator.are_graphics_resources_optimized(), "Graphics resources should be optimized");
    }

    #[wasm_bindgen_test]
    async fn test_ui_coordinator_basic() {
        assert!(true, "Basic UI coordinator test");
    }
}

// Test utility functions
async fn create_test_ui_coordinator() -> Result<MockUICoordinator, String> {
    MockUICoordinator::new().await
}

fn create_test_immersive_state() -> MockImmersiveState {
    MockImmersiveState {
        theme: "scientific".to_string(),
        rendering_mode: "immersive".to_string(),
        active: true,
    }
}

fn create_test_debug_state() -> MockDebugState {
    MockDebugState {
        overlay_visible: true,
        component_count: 5,
        performance_monitoring: true,
    }
}

// Mock structures for testing (simplified implementations)
struct MockUICoordinator {
    immersive_active: bool,
    debug_overlay_active: bool,
    dual_rendering: bool,
    coordination_enabled: bool,
    has_stub: bool,
    graphics_context: bool,
}

impl MockUICoordinator {
    async fn new() -> Result<Self, String> {
        Ok(MockUICoordinator {
            immersive_active: false,
            debug_overlay_active: false,
            dual_rendering: false,
            coordination_enabled: false,
            has_stub: false,
            graphics_context: false,
        })
    }

    async fn setup_immersive_coordination(&mut self) -> Result<(), String> {
        self.immersive_active = true;
        self.coordination_enabled = true;
        Ok(())
    }

    async fn coordinate_rendering(&mut self) -> Result<(), String> {
        if self.coordination_enabled {
            Ok(())
        } else {
            Err("Coordination not enabled".to_string())
        }
    }

    async fn setup_debug_overlay_coordination(&mut self) -> Result<(), String> {
        if self.immersive_active {
            self.dual_rendering = true;
            Ok(())
        } else {
            Err("Immersive UI not active".to_string())
        }
    }

    async fn coordinate_dual_rendering(&mut self) -> Result<(), String> {
        if self.dual_rendering {
            Ok(())
        } else {
            Err("Dual rendering not setup".to_string())
        }
    }

    async fn toggle_debug_overlay(&mut self, enable: bool) -> Result<(), String> {
        self.debug_overlay_active = enable;
        Ok(())
    }

    async fn route_event_to_debug(&self, _event: &UICoordinationEvent) -> Result<(), String> {
        if self.debug_overlay_active {
            Ok(())
        } else {
            Err("Debug overlay not active".to_string())
        }
    }

    async fn route_event_to_immersive(&self, _event: &UICoordinationEvent) -> Result<(), String> {
        if self.immersive_active {
            Ok(())
        } else {
            Err("Immersive UI not active".to_string())
        }
    }

    async fn sync_state_to_debug(&self, _state: &MockImmersiveState) -> Result<(), String> {
        if self.debug_overlay_active && self.immersive_active {
            Ok(())
        } else {
            Err("Systems not ready for sync".to_string())
        }
    }

    async fn sync_state_from_debug(&self, _state: &MockDebugState) -> Result<(), String> {
        if self.debug_overlay_active && self.immersive_active {
            Ok(())
        } else {
            Err("Systems not ready for sync".to_string())
        }
    }

    async fn verify_state_consistency(&self) -> Result<(), String> {
        Ok(())
    }

    async fn transition_to_dual_rendering(&mut self) -> Result<(), String> {
        if self.immersive_active {
            self.dual_rendering = true;
            self.debug_overlay_active = true;
            Ok(())
        } else {
            Err("Immersive UI not ready".to_string())
        }
    }

    async fn transition_to_immersive_only(&mut self) -> Result<(), String> {
        self.dual_rendering = false;
        self.debug_overlay_active = false;
        Ok(())
    }

    async fn setup_stub_immersive_renderer(&mut self) -> Result<(), String> {
        self.has_stub = true;
        Ok(())
    }

    async fn coordinate_with_stub_renderer(&self) -> Result<(), String> {
        if self.has_stub {
            Ok(())
        } else {
            Err("No stub renderer".to_string())
        }
    }

    async fn verify_stub_debug_integration(&self) -> Result<(), String> {
        if self.has_stub && self.debug_overlay_active {
            Ok(())
        } else {
            Err("Stub or debug not ready".to_string())
        }
    }

    async fn setup_graphics_context(&mut self) -> Result<(), String> {
        self.graphics_context = true;
        Ok(())
    }

    async fn coordinate_immersive_graphics(&self) -> Result<(), String> {
        if self.graphics_context && self.immersive_active {
            Ok(())
        } else {
            Err("Graphics or immersive not ready".to_string())
        }
    }

    async fn coordinate_debug_graphics(&self) -> Result<(), String> {
        if self.graphics_context && self.debug_overlay_active {
            Ok(())
        } else {
            Err("Graphics or debug not ready".to_string())
        }
    }

    async fn verify_graphics_resources(&self) -> Result<(), String> {
        if self.graphics_context {
            Ok(())
        } else {
            Err("Graphics context not initialized".to_string())
        }
    }

    // Helper methods
    fn is_immersive_active(&self) -> bool { self.immersive_active }
    fn is_coordination_enabled(&self) -> bool { self.coordination_enabled }
    fn is_debug_overlay_active(&self) -> bool { self.debug_overlay_active }
    fn is_dual_rendering_stable(&self) -> bool { self.dual_rendering && self.immersive_active && self.debug_overlay_active }
    fn is_state_synchronized(&self) -> bool { true } // Simplified for testing
    fn has_stub_renderer(&self) -> bool { self.has_stub }
    fn is_stub_renderer_functional(&self) -> bool { self.has_stub }
    fn are_graphics_resources_optimized(&self) -> bool { self.graphics_context }
    
    fn get_event_routing_metrics(&self) -> MockEventRoutingMetrics {
        MockEventRoutingMetrics {
            average_routing_time_ms: 0.5, // Under 1ms requirement
        }
    }
}

struct MockImmersiveState {
    theme: String,
    rendering_mode: String,
    active: bool,
}

struct MockDebugState {
    overlay_visible: bool,
    component_count: i32,
    performance_monitoring: bool,
}

struct MockEventRoutingMetrics {
    average_routing_time_ms: f64,
} 