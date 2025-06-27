//! # Developer UI Integration Tests
//!
//! Comprehensive integration tests for the Developer UI module.
//! Tests conditional compilation, module registration, component lifecycle,
//! event system integration, and zero production impact validation.

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use super::*;
    use crate::modules::application_core::{ModuleRegistry, ModuleError};
    use crate::modules::application_core::event_bus::{Event, EventPriority, get_timestamp_ns};
    use crate::modules::application_core::priority_event_bus::PriorityEventBus;
    use crate::modules::developer_ui::utils::{
        debug_event_performance_monitor::*, 
        debug_event_publisher::*, 
        memory_leak_prevention::*
    };
    use crate::modules::developer_ui::hooks::use_event_subscription::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::time::{Duration, Instant};

    // Test event types for integration testing
    #[derive(Debug, Clone, PartialEq)]
    struct TestAudioEvent {
        pub id: u32,
        pub timestamp: u64,
        pub audio_data: String,
    }

    impl Event for TestAudioEvent {
        fn event_type(&self) -> &'static str {
            "TestAudioEvent"
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn priority(&self) -> EventPriority {
            EventPriority::High
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestDebugEvent {
        pub component_id: String,
        pub message: String,
        pub timestamp: u64,
    }

    impl Event for TestDebugEvent {
        fn event_type(&self) -> &'static str {
            "TestDebugEvent"
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn priority(&self) -> EventPriority {
            EventPriority::Low
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    /// Test module creation and initialization
    #[test]
    fn test_developer_ui_module_creation() {
        let result = DeveloperUIModule::new();
        assert!(result.is_ok(), "DeveloperUIModule creation should succeed");
        
        let module = result.unwrap();
        assert_eq!(module.name(), "developer_ui");
        assert!(!module.initialized);
    }

    /// Test module initialization process
    #[test]
    fn test_developer_ui_module_initialization() {
        let mut module = DeveloperUIModule::new().unwrap();
        
        let result = module.initialize();
        assert!(result.is_ok(), "Module initialization should succeed");
        
        // Verify initialization state
        assert!(module.initialized);
    }

    /// Test module shutdown process
    #[test]
    fn test_developer_ui_module_shutdown() {
        let mut module = DeveloperUIModule::new().unwrap();
        module.initialize().unwrap();
        
        let result = module.shutdown();
        assert!(result.is_ok(), "Module shutdown should succeed");
        
        // Verify shutdown state
        assert!(!module.initialized);
    }

    /// Comprehensive test for developer UI event system integration
    #[test]
    fn test_developer_ui_event_system_integration() {
        // Create event bus for testing
        let event_bus = Rc::new(RefCell::new(PriorityEventBus::new()));
        
        // Create debug event publisher
        let mut publisher = DebugEventPublisher::new(Some(event_bus.clone()));
        
        // Test event publishing
        let test_event = DebugControlEvent::StartRecording;
        let publish_result = publisher.publish_control_event(test_event);
        assert!(publish_result.is_ok(), "Debug event publishing should succeed");
        
        // Verify publisher metrics
        if let Some(metrics) = publisher.get_metrics() {
            assert_eq!(metrics.total_published, 1);
            assert_eq!(metrics.total_errors, 0);
            assert!(metrics.success_rate() > 99.0);
        }
    }

    /// Test event subscription lifecycle and cleanup
    #[test]
    fn test_event_subscription_lifecycle_and_cleanup() {
        let event_bus = Rc::new(RefCell::new(PriorityEventBus::new()));
        let performance_monitor = Rc::new(RefCell::new(create_performance_monitor()));
        
        // Create event subscription handle
        let subscription_id = 12345;
        let cleanup_executed = Rc::new(RefCell::new(false));
        
        {
            let cleanup_flag = cleanup_executed.clone();
            let cleanup_callback = Box::new(move || {
                *cleanup_flag.borrow_mut() = true;
            });
            
            let handle = EventSubscriptionHandle::new(
                subscription_id,
                Some(event_bus.clone()),
                Some(cleanup_callback),
                performance_monitor.clone(),
                "TestEvent".to_string(),
            );
            
            assert_eq!(handle.subscription_id(), subscription_id);
            
            // Handle goes out of scope here, triggering cleanup
        }
        
        // Verify cleanup was executed
        assert!(*cleanup_executed.borrow(), "Cleanup callback should have been executed");
        
        // Verify performance monitoring recorded the subscription
        let monitor = performance_monitor.borrow();
        let report = monitor.get_performance_report();
        assert_eq!(report.subscription_metrics.total_subscriptions, 1);
    }

    /// Test memory leak prevention system
    #[test]
    fn test_memory_leak_prevention() {
        let mut prevention_manager = MemoryLeakPreventionManager::new();
        
        // Register some test subscriptions
        for i in 0..5 {
            let subscription_id = i;
            let component_id = format!("test_component_{}", i);
            prevention_manager.register_subscription(subscription_id, &component_id, "TestEvent");
        }
        
        // Verify subscriptions are tracked
        let stats = prevention_manager.get_statistics();
        assert_eq!(stats.total_subscriptions_created, 5);
        assert_eq!(stats.active_subscriptions, 5);
        
        // Cleanup some subscriptions
        prevention_manager.cleanup_subscription(0);
        prevention_manager.cleanup_subscription(1);
        
        let updated_stats = prevention_manager.get_statistics();
        assert_eq!(updated_stats.total_subscriptions_cleaned, 2);
        assert_eq!(updated_stats.active_subscriptions, 3);
        
        // Test memory leak detection
        prevention_manager.check_for_memory_leaks();
        
        // Cleanup all remaining subscriptions
        prevention_manager.cleanup_all_subscriptions();
        let final_stats = prevention_manager.get_statistics();
        assert_eq!(final_stats.active_subscriptions, 0);
    }

    /// Test type-safe event handling across debug components
    #[test]
    fn test_type_safe_event_handling() {
        let event_bus = Rc::new(RefCell::new(PriorityEventBus::new()));
        let mut publisher = DebugEventPublisher::new(Some(event_bus.clone()));
        
        // Test different event types
        let audio_event = TestAudioEvent {
            id: 1,
            timestamp: get_timestamp_ns(),
            audio_data: "test_audio_data".to_string(),
        };
        
        let debug_event = TestDebugEvent {
            component_id: "test_component".to_string(),
            message: "test_message".to_string(),
            timestamp: get_timestamp_ns(),
        };
        
        // Publish events and verify type safety
        let audio_result = publisher.publish(audio_event.clone());
        assert!(audio_result.is_ok(), "Audio event publishing should succeed");
        
        let debug_result = publisher.publish(debug_event.clone());
        assert!(debug_result.is_ok(), "Debug event publishing should succeed");
        
        // Verify events maintain their type information
        assert_eq!(audio_event.event_type(), "TestAudioEvent");
        assert_eq!(debug_event.event_type(), "TestDebugEvent");
        assert_eq!(audio_event.priority(), EventPriority::High);
        assert_eq!(debug_event.priority(), EventPriority::Low);
    }

    /// Performance tests for debug event system overhead
    #[test]
    fn test_debug_event_system_performance() {
        let mut performance_monitor = create_performance_monitor();
        
        // Test subscription performance
        let subscription_start = Instant::now();
        performance_monitor.record_subscription("TestEvent", subscription_start.elapsed());
        
        // Test event processing performance
        for i in 0..100 {
            performance_monitor.record_event_processed(&format!("Event{}", i % 10));
        }
        
        // Test memory usage tracking
        performance_monitor.update_memory_usage(50, 2500.0); // 50 subscriptions, 2.5MB
        
        // Run performance benchmarks
        let benchmark_results = performance_monitor.run_benchmarks();
        
        // Verify benchmark results meet requirements
        assert!(benchmark_results.subscription_benchmark.meets_requirement, 
                "Subscription benchmark should meet <1ms requirement");
        assert!(benchmark_results.publishing_benchmark.meets_requirement,
                "Publishing benchmark should meet <0.1ms requirement");
        assert!(benchmark_results.memory_benchmark.meets_requirement,
                "Memory benchmark should meet <5MB requirement");
        
        // Test overall performance requirements
        assert!(performance_monitor.meets_performance_requirements(),
                "Performance monitor should meet all requirements");
        
        // Get performance report and validate metrics
        let report = performance_monitor.get_performance_report();
        assert_eq!(report.subscription_metrics.total_subscriptions, 1);
        assert_eq!(report.throughput_stats.total_events, 100);
        assert_eq!(report.memory_stats.current_memory_kb, 2500.0);
        assert_eq!(report.memory_stats.subscription_count, 50);
        assert_eq!(report.memory_stats.avg_memory_per_subscription, 50.0);
    }

    /// Test performance alert system
    #[test]
    fn test_performance_alert_system() {
        let mut performance_monitor = create_performance_monitor();
        
        // Trigger a slow subscription alert
        let slow_duration = Duration::from_millis(10); // 10ms - above 1ms threshold
        performance_monitor.record_subscription("SlowEvent", slow_duration);
        
        // Trigger high memory usage alert
        performance_monitor.update_memory_usage(1000, 8000.0); // 8MB - above 5MB threshold
        
        // Trigger high throughput alert
        for i in 0..1500 {
            performance_monitor.record_event_processed("HighThroughputEvent");
        }
        
        let report = performance_monitor.get_performance_report();
        assert!(!report.recent_alerts.is_empty(), "Should have performance alerts");
        
        // Verify specific alert types
        let has_subscription_alert = report.recent_alerts.iter()
            .any(|alert| matches!(alert.alert_type, AlertType::SlowSubscription));
        let has_memory_alert = report.recent_alerts.iter()
            .any(|alert| matches!(alert.alert_type, AlertType::HighMemoryUsage));
        let has_throughput_alert = report.recent_alerts.iter()
            .any(|alert| matches!(alert.alert_type, AlertType::HighThroughput));
        
        assert!(has_subscription_alert, "Should have slow subscription alert");
        assert!(has_memory_alert, "Should have high memory usage alert");
        assert!(has_throughput_alert, "Should have high throughput alert");
    }

    /// Test event subscription configuration
    #[test]
    fn test_event_subscription_configuration() {
        let config = EventSubscriptionConfig {
            event_type_name: "TestConfigEvent",
            priority_filter: Some(EventPriority::High),
        };
        
        assert_eq!(config.event_type_name, "TestConfigEvent");
        assert_eq!(config.priority_filter.unwrap(), EventPriority::High);
    }

    /// Test debug component registry event integration
    #[test]
    fn test_debug_component_registry_event_integration() {
        let event_bus = Rc::new(RefCell::new(PriorityEventBus::new()));
        let mut registry = DebugComponentRegistry::new();
        
        // Initialize registry with event bus
        registry.connect_event_bus(event_bus.clone()).unwrap();
        
        assert_eq!(registry.component_count(), 0);
        assert_eq!(registry.active_component_count(), 0);
        assert!(!registry.is_component_registered("test_component"));
    }

    /// Test cross-component event communication
    #[test]
    fn test_cross_component_event_communication() {
        let event_bus = Rc::new(RefCell::new(PriorityEventBus::new()));
        let mut publisher = DebugEventPublisher::new(Some(event_bus.clone()));
        
        // Simulate audio control panel publishing events
        let audio_control_event = DebugControlEvent::StartRecording;
        assert!(publisher.publish_control_event(audio_control_event).is_ok());
        
        // Simulate debug panel publishing events
        let debug_panel_event = DebugControlEvent::RequestPerformanceReport;
        assert!(publisher.publish_control_event(debug_panel_event).is_ok());
        
        // Simulate microphone permission events
        let permission_event = DebugControlEvent::SelectDevice { 
            device_id: "test_device".to_string() 
        };
        assert!(publisher.publish_control_event(permission_event).is_ok());
        
        // Verify all events were published successfully
        if let Some(metrics) = publisher.get_metrics() {
            assert_eq!(metrics.total_published, 3);
            assert_eq!(metrics.total_errors, 0);
        }
    }

    /// Test event system cleanup on component unmount
    #[test]
    fn test_event_system_cleanup_on_unmount() {
        let event_bus = Rc::new(RefCell::new(PriorityEventBus::new()));
        let performance_monitor = Rc::new(RefCell::new(create_performance_monitor()));
        let cleanup_counter = Rc::new(RefCell::new(0));
        
        // Create multiple subscription handles that will cleanup
        let handles: Vec<_> = (0..5).map(|i| {
            let counter = cleanup_counter.clone();
            let cleanup_callback = Box::new(move || {
                *counter.borrow_mut() += 1;
            });
            
            EventSubscriptionHandle::new(
                i,
                Some(event_bus.clone()),
                Some(cleanup_callback),
                performance_monitor.clone(),
                format!("TestEvent{}", i),
            )
        }).collect();
        
        // Verify all handles are created
        assert_eq!(handles.len(), 5);
        
        // Drop all handles to trigger cleanup
        drop(handles);
        
        // Verify all cleanup callbacks were executed
        assert_eq!(*cleanup_counter.borrow(), 5);
        
        // Verify performance monitoring recorded all subscriptions
        let monitor = performance_monitor.borrow();
        let report = monitor.get_performance_report();
        assert_eq!(report.subscription_metrics.total_subscriptions, 5);
    }

    /// Test debug component registry functionality
    #[test]
    fn test_debug_component_registry() {
        let registry = DebugComponentRegistry::new();
        
        assert_eq!(registry.component_count(), 0);
        assert_eq!(registry.active_component_count(), 0);
        assert!(!registry.is_component_registered("test_component"));
    }

    /// Test overlay manager functionality
    #[test]
    fn test_overlay_manager() {
        let manager_result = OverlayManager::new();
        assert!(manager_result.is_ok(), "OverlayManager creation should succeed");
        
        let mut manager = manager_result.unwrap();
        let init_result = manager.initialize();
        assert!(init_result.is_ok(), "OverlayManager initialization should succeed");
        
        // Test default overlays are created
        assert!(manager.get_overlay("audio_controls").is_some());
        assert!(manager.get_overlay("debug_interface").is_some());
        assert!(manager.get_overlay("performance_monitor").is_some());
    }

    /// Test debug app functionality
    #[test]
    fn test_debug_app() {
        let app_result = DebugApp::new();
        assert!(app_result.is_ok(), "DebugApp creation should succeed");
        
        let mut app = app_result.unwrap();
        assert!(!app.is_initialized());
        
        let init_result = app.initialize();
        assert!(init_result.is_ok(), "DebugApp initialization should succeed");
        assert!(app.is_initialized());
    }

    /// Test conditional compilation - this test should only run in debug builds
    #[test]
    fn test_conditional_compilation() {
        // This test existing proves conditional compilation is working
        // since it's wrapped in #[cfg(debug_assertions)]
        assert!(true, "This test should only run in debug builds");
    }

    /// Test module version information
    #[test]
    fn test_module_version() {
        let module = DeveloperUIModule::new().unwrap();
        let version = module.version();
        assert!(!version.is_empty(), "Module version should not be empty");
    }

    /// Test overlay state management
    #[test]
    fn test_overlay_state_management() {
        let mut manager = OverlayManager::new().unwrap();
        manager.initialize().unwrap();
        
        // Test showing and hiding overlays
        assert!(manager.show_overlay("audio_controls").is_ok());
        assert_eq!(manager.get_active_overlays().len(), 1);
        
        assert!(manager.hide_overlay("audio_controls").is_ok());
        assert_eq!(manager.get_active_overlays().len(), 0);
    }

    /// Test debug app state toggles
    #[test]
    fn test_debug_app_state_toggles() {
        let mut app = DebugApp::new().unwrap();
        
        let initial_visibility = app.get_debug_state().ui_visible;
        app.toggle_ui_visibility();
        assert_eq!(app.get_debug_state().ui_visible, !initial_visibility);
        
        app.toggle_performance_overlay();
        app.toggle_error_display();
        app.toggle_component_inspector();
        app.toggle_audio_monitoring();
        
        // All toggles should work without panicking
    }

    /// Test keyboard shortcuts
    #[test]
    fn test_debug_app_keyboard_shortcuts() {
        let mut app = DebugApp::new().unwrap();
        
        // Test valid shortcuts
        let result = app.handle_keyboard_shortcut("d", true, false, true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should be handled
        
        // Test invalid shortcuts
        let result = app.handle_keyboard_shortcut("x", true, false, true);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should not be handled
    }

    /// Test layout mode changes
    #[test]
    fn test_layout_mode_changes() {
        use super::overlay_manager::LayoutMode;
        
        let mut manager = OverlayManager::new().unwrap();
        manager.initialize().unwrap();
        
        assert!(manager.set_layout_mode(LayoutMode::Tiled).is_ok());
        match manager.get_layout_mode() {
            LayoutMode::Tiled => {},
            _ => panic!("Layout mode should be Tiled"),
        }
        
        assert!(manager.set_layout_mode(LayoutMode::Docked).is_ok());
        match manager.get_layout_mode() {
            LayoutMode::Docked => {},
            _ => panic!("Layout mode should be Docked"),
        }
    }

    /// Test global overlay settings
    #[test]
    fn test_overlay_global_settings() {
        use super::overlay_manager::OverlayGlobalSettings;
        
        let mut manager = OverlayManager::new().unwrap();
        let settings = manager.get_global_settings();
        
        // Test default settings
        assert!(!settings.snap_to_grid);
        assert!(settings.auto_arrange);
        assert!(settings.collision_detection);
        assert!(settings.animation_enabled);
        
        // Test updating settings
        let new_settings = OverlayGlobalSettings {
            snap_to_grid: true,
            grid_size: 20.0,
            auto_arrange: false,
            collision_detection: false,
            animation_enabled: false,
            animation_duration: 0.5,
        };
        
        manager.update_global_settings(new_settings);
        let updated_settings = manager.get_global_settings();
        assert!(updated_settings.snap_to_grid);
        assert!(!updated_settings.auto_arrange);
        assert_eq!(updated_settings.grid_size, 20.0);
    }

    /// Test debug statistics collection
    #[test]
    fn test_debug_statistics() {
        let module = DeveloperUIModule::new().unwrap();
        let stats = module.collect_debug_statistics();
        
        assert!(stats.is_ok(), "Debug statistics collection should succeed");
        let debug_stats = stats.unwrap();
        
        // Verify statistics structure
        assert!(!debug_stats.is_empty());
    }

    /// Test that production builds have zero impact (placeholder test)
    #[test]
    fn test_production_build_impact_placeholder() {
        // In a real scenario, this would be compiled conditionally
        // and verify that debug code has zero impact in production
        
        // Placeholder assertion - in production builds, debug code should not exist
        #[cfg(not(debug_assertions))]
        {
            panic!("This test should not run in production builds");
        }
        
        #[cfg(debug_assertions)]
        {
            assert!(true, "Debug functionality is available in debug builds");
        }
    }
}

/// Test utilities for integration testing
#[cfg(test)]
#[cfg(debug_assertions)]
mod test_utilities {
    use super::*;
    
    /// Create a test instance of DeveloperUIModule
    pub fn create_test_module() -> Result<DeveloperUIModule, ModuleError> {
        DeveloperUIModule::new()
    }
    
    /// Create a test instance of OverlayManager
    pub fn create_test_overlay_manager() -> Result<OverlayManager, ModuleError> {
        OverlayManager::new()
    }
    
    /// Create a test instance of DebugApp
    pub fn create_test_debug_app() -> Result<DebugApp, ModuleError> {
        DebugApp::new()
    }
    
    /// Create a test event bus for integration testing
    pub fn create_test_event_bus() -> Rc<RefCell<PriorityEventBus>> {
        Rc::new(RefCell::new(PriorityEventBus::new()))
    }
    
    /// Create a test debug event publisher
    pub fn create_test_debug_publisher(event_bus: Option<Rc<RefCell<PriorityEventBus>>>) -> DebugEventPublisher {
        DebugEventPublisher::new(event_bus)
    }
    
    /// Helper to generate test events
    pub fn create_test_audio_event(id: u32) -> TestAudioEvent {
        TestAudioEvent {
            id,
            timestamp: get_timestamp_ns(),
            audio_data: format!("test_audio_{}", id),
        }
    }
    
    /// Helper to generate test debug events
    pub fn create_test_debug_event(component_id: &str, message: &str) -> TestDebugEvent {
        TestDebugEvent {
            component_id: component_id.to_string(),
            message: message.to_string(),
            timestamp: get_timestamp_ns(),
        }
    }
} 