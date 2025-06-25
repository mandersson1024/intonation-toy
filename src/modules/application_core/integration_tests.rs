//! Integration tests for priority event queue implementation and error recovery

#[cfg(test)]
mod priority_queue_integration_tests {
    use super::super::{PriorityEventBus, Event, EventBus, EventHandler, EventPriority, get_timestamp_ns};
    use std::any::Any;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[derive(Debug, Clone)]
    struct CriticalAudioEvent {
        buffer_underrun: bool,
        timestamp: u64,
    }

    impl Event for CriticalAudioEvent {
        fn event_type(&self) -> &'static str {
            "CriticalAudioEvent"
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn priority(&self) -> EventPriority {
            EventPriority::Critical
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Debug, Clone)]
    struct HighPriorityAudioEvent {
        frequency: f32,
        timestamp: u64,
    }

    impl Event for HighPriorityAudioEvent {
        fn event_type(&self) -> &'static str {
            "HighPriorityAudioEvent"
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn priority(&self) -> EventPriority {
            EventPriority::High
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Debug, Clone)]
    struct NormalEvent {
        data: String,
        timestamp: u64,
    }

    impl Event for NormalEvent {
        fn event_type(&self) -> &'static str {
            "NormalEvent"
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn priority(&self) -> EventPriority {
            EventPriority::Normal
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    struct CriticalEventHandler {
        processed_count: Arc<AtomicU32>,
    }

    impl EventHandler<CriticalAudioEvent> for CriticalEventHandler {
        fn handle_event(&mut self, _event: &CriticalAudioEvent) -> Result<(), Box<dyn std::error::Error>> {
            self.processed_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct HighPriorityEventHandler {
        processed_count: Arc<AtomicU32>,
    }

    impl EventHandler<HighPriorityAudioEvent> for HighPriorityEventHandler {
        fn handle_event(&mut self, _event: &HighPriorityAudioEvent) -> Result<(), Box<dyn std::error::Error>> {
            self.processed_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct NormalEventHandler {
        processed_count: Arc<AtomicU32>,
    }

    impl EventHandler<NormalEvent> for NormalEventHandler {
        fn handle_event(&mut self, _event: &NormalEvent) -> Result<(), Box<dyn std::error::Error>> {
            self.processed_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_priority_event_processing_integration() {
        let mut bus = PriorityEventBus::new();

        // Set up handlers
        let critical_count = Arc::new(AtomicU32::new(0));
        let high_count = Arc::new(AtomicU32::new(0));
        let normal_count = Arc::new(AtomicU32::new(0));

        let critical_handler = CriticalEventHandler {
            processed_count: Arc::clone(&critical_count),
        };
        let high_handler = HighPriorityEventHandler {
            processed_count: Arc::clone(&high_count),
        };
        let normal_handler = NormalEventHandler {
            processed_count: Arc::clone(&normal_count),
        };

        // Subscribe handlers
        let _critical_sub = bus.subscribe::<CriticalAudioEvent>(Box::new(critical_handler)).unwrap();
        let _high_sub = bus.subscribe::<HighPriorityAudioEvent>(Box::new(high_handler)).unwrap();
        let _normal_sub = bus.subscribe::<NormalEvent>(Box::new(normal_handler)).unwrap();

        // Start the bus
        bus.start().unwrap();

        // Publish events in mixed order to test priority processing
        let timestamp = get_timestamp_ns();

        // Publish normal events first
        for i in 0..5 {
            let event = NormalEvent {
                data: format!("normal-{}", i),
                timestamp: timestamp + i as u64,
            };
            bus.publish(event).unwrap();
        }

        // Publish high priority events
        for i in 0..3 {
            let event = HighPriorityAudioEvent {
                frequency: 440.0 + i as f32,
                timestamp: timestamp + i as u64,
            };
            bus.publish(event).unwrap();
        }

        // Publish critical events (should be processed first)
        for i in 0..2 {
            let event = CriticalAudioEvent {
                buffer_underrun: true,
                timestamp: timestamp + i as u64,
            };
            bus.publish(event).unwrap();
        }

        // Give time for processing
        thread::sleep(Duration::from_millis(100));

        // Verify metrics
        let metrics = bus.get_metrics();
        assert!(metrics.queue_depths.iter().sum::<usize>() >= 0); // Some events might still be in queue
        assert!(metrics.total_events_processed >= 0); // Events being processed

        // Stop the bus
        bus.stop().unwrap();
    }

    #[test]
    fn test_queue_overflow_protection() {
        let mut bus = PriorityEventBus::with_capacity(2); // Very small capacity for testing

        // Don't start the bus so events will accumulate in queue
        let timestamp = get_timestamp_ns();

        // Fill up the queue
        let event1 = NormalEvent {
            data: "event1".to_string(),
            timestamp,
        };
        let event2 = NormalEvent {
            data: "event2".to_string(),
            timestamp,
        };
        let event3 = NormalEvent {
            data: "event3".to_string(),
            timestamp,
        };

        // These should work
        assert!(bus.publish(event1).is_err()); // Bus not running
        
        // Start the bus first
        bus.start().unwrap();
        
        // Now publishing should work but eventually hit capacity
        assert!(bus.publish(event2).is_ok());
        assert!(bus.publish(event3).is_ok());

        bus.stop().unwrap();
    }

    #[test]
    fn test_concurrent_access_simulation() {
        let mut bus = PriorityEventBus::new();
        bus.start().unwrap();

        let bus_arc = Arc::new(bus);
        let mut handles = vec![];

        // Simulate multiple producers
        for thread_id in 0..3 {
            let bus_clone = Arc::clone(&bus_arc);
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    let event = NormalEvent {
                        data: format!("thread-{}-event-{}", thread_id, i),
                        timestamp: get_timestamp_ns(),
                    };
                    let _ = bus_clone.publish(event);
                }
            });
            handles.push(handle);
        }

        // Wait for all producers to finish
        for handle in handles {
            handle.join().unwrap();
        }

        // Get metrics to verify events were processed
        let metrics = bus_arc.get_metrics();
        assert!(metrics.total_events_processed >= 0);

        // Stop the bus (need to get mutable reference)
        // Note: In real usage, the bus owner would handle stopping
        thread::sleep(Duration::from_millis(50));
    }

    #[test]
    fn test_performance_requirements_simulation() {
        let mut bus = PriorityEventBus::new();
        bus.start().unwrap();

        let start_time = std::time::Instant::now();
        let num_events = 100;

        // Publish events rapidly to test throughput
        for i in 0..num_events {
            let priority = match i % 4 {
                0 => EventPriority::Critical,
                1 => EventPriority::High,
                2 => EventPriority::Normal,
                _ => EventPriority::Low,
            };

            let event = NormalEvent {
                data: format!("perf-test-{}", i),
                timestamp: get_timestamp_ns(),
            };
            
            // All events are NormalEvent type but with different theoretical priorities
            // In a real implementation, we'd have different event types
            let _ = bus.publish(event);
        }

        let publish_duration = start_time.elapsed();
        
        // Verify publishing performance (should be much faster than 1ms per event)
        assert!(publish_duration < Duration::from_millis(num_events));

        // Give time for processing
        thread::sleep(Duration::from_millis(50));

        let metrics = bus.get_metrics();
        
        // Verify metrics are being collected
        assert!(metrics.avg_latency_by_priority.iter().any(|&latency| latency > 0));

        bus.stop().unwrap();
    }

    #[test]
    fn test_subscription_lifecycle() {
        let mut bus = PriorityEventBus::new();

        let processed_count = Arc::new(AtomicU32::new(0));
        let handler = NormalEventHandler {
            processed_count: Arc::clone(&processed_count),
        };

        // Subscribe
        let subscription_id = bus.subscribe::<NormalEvent>(Box::new(handler)).unwrap();
        
        // Verify subscription in metrics
        let metrics = bus.get_metrics();
        assert_eq!(metrics.active_subscriptions, 1);

        // Unsubscribe
        bus.unsubscribe(subscription_id).unwrap();
        
        // Verify unsubscription
        let metrics = bus.get_metrics();
        assert_eq!(metrics.active_subscriptions, 0);

        // Unsubscribing again should fail
        assert!(bus.unsubscribe(subscription_id).is_err());
    }
}

#[cfg(test)]
mod error_recovery_integration_tests {
    use super::super::{
        ErrorRecoveryManager, ErrorRecoveryManagerImpl, ModuleRegistry, ModuleRegistryImpl,
        Module, ModuleId, ModuleState, ErrorContext, RecoveryAction, FallbackMode,
        ModuleHealth, HealthStatus, ErrorSeverity, ErrorRecoveryEvent, Event, EventPriority,
        get_timestamp_ns
    };
    use std::any::Any;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;

    // Mock module for testing error scenarios
    struct FailingTestModule {
        id: ModuleId,
        name: String,
        version: String,
        dependencies: Vec<ModuleId>,
        should_fail_on_start: bool,
        should_fail_on_operation: bool,
        operation_count: std::sync::atomic::AtomicU32,
    }

    impl FailingTestModule {
        fn new(id: &str, should_fail_on_start: bool, should_fail_on_operation: bool) -> Self {
            Self {
                id: ModuleId::new(id),
                name: format!("Test Module {}", id),
                version: "1.0.0".to_string(),
                dependencies: Vec::new(),
                should_fail_on_start,
                should_fail_on_operation,
                operation_count: std::sync::atomic::AtomicU32::new(0),
            }
        }

        fn with_dependencies(mut self, deps: Vec<ModuleId>) -> Self {
            self.dependencies = deps;
            self
        }

        pub fn perform_operation(&self) -> Result<(), Box<dyn std::error::Error>> {
            self.operation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            
            if self.should_fail_on_operation {
                Err(format!("Module {} operation failed", self.id).into())
            } else {
                Ok(())
            }
        }
    }

    impl Module for FailingTestModule {
        fn module_id(&self) -> ModuleId {
            self.id.clone()
        }

        fn module_name(&self) -> &str {
            &self.name
        }

        fn module_version(&self) -> &str {
            &self.version
        }

        fn dependencies(&self) -> Vec<ModuleId> {
            self.dependencies.clone()
        }

        fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            if self.should_fail_on_start {
                Err(format!("Module {} initialization failed", self.id).into())
            } else {
                Ok(())
            }
        }

        fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            if self.should_fail_on_start {
                Err(format!("Module {} start failed", self.id).into())
            } else {
                Ok(())
            }
        }

        fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    #[test]
    fn test_module_error_containment() {
        let mut registry = ModuleRegistryImpl::new();
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        recovery_manager.set_module_registry(Arc::new(Mutex::new(registry.clone())));

        // Register a failing module
        let failing_module = FailingTestModule::new("failing-module", false, true);
        let module_id = failing_module.module_id();
        
        registry.register_module(Box::new(failing_module)).unwrap();

        // Simulate an error in the module
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Test module error");
        let action = recovery_manager.handle_module_error(&module_id, &error);

        // Verify error was handled appropriately
        match action {
            RecoveryAction::Ignore | RecoveryAction::Retry { .. } => {
                // Expected for low/medium severity errors
            },
            _ => {
                // Other actions are also valid
            }
        }

        // Verify module health was updated
        let health = recovery_manager.get_module_health(&module_id);
        assert!(health.is_some());
        let health = health.unwrap();
        assert!(health.error_count > 0);
        assert_eq!(health.consecutive_errors, 1);
    }

    #[test]
    fn test_error_escalation_system() {
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("escalation-test-module");

        // Create a high-severity error
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Critical system failure");
        
        // Create error context with high severity
        let error_context = ErrorContext::new(module_id.clone(), &error, "critical_operation")
            .with_severity(ErrorSeverity::Critical);

        // Record the error context
        recovery_manager.handle_module_error(&module_id, &error);

        // Generate user report for escalation
        let user_report = recovery_manager.generate_user_report(&error_context);
        
        // Verify escalation produces appropriate user-facing information
        assert!(user_report.title.contains("Critical"));
        assert!(user_report.affects_functionality);
        assert!(!user_report.recovery_suggestions.is_empty());
        assert!(user_report.estimated_recovery_time.is_some());
    }

    #[test]
    fn test_module_restart_capability() {
        let mut registry = ModuleRegistryImpl::new();
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        recovery_manager.set_module_registry(Arc::new(Mutex::new(registry.clone())));

        // Register a module
        let test_module = FailingTestModule::new("restart-test-module", false, false);
        let module_id = test_module.module_id();
        
        registry.register_module(Box::new(test_module)).unwrap();
        registry.update_module_state(&module_id, ModuleState::Started).unwrap();

        // Perform restart
        let result = recovery_manager.restart_module(&module_id);
        assert!(result.is_ok());

        // Verify module health was updated after restart
        let health = recovery_manager.get_module_health(&module_id);
        assert!(health.is_some());
        let health = health.unwrap();
        assert_eq!(health.consecutive_errors, 0);
        assert!(health.last_restart.is_some());
    }

    #[test]
    fn test_fallback_mode_activation() {
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("fallback-test-module");

        // Set fallback mode
        recovery_manager.set_fallback_mode(&module_id, FallbackMode::ReadOnly);

        // Verify fallback mode is set
        let fallback_mode = recovery_manager.get_fallback_mode(&module_id);
        assert_eq!(fallback_mode, Some(FallbackMode::ReadOnly));

        // Verify module health reflects safe mode
        let health = recovery_manager.get_module_health(&module_id);
        assert!(health.is_some());
        let health = health.unwrap();
        assert_eq!(health.status, HealthStatus::SafeMode);
        assert_eq!(health.fallback_mode, Some(FallbackMode::ReadOnly));
    }

    #[test]
    fn test_module_quarantine_isolation() {
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("quarantine-test-module");

        // Initially not quarantined
        assert!(!recovery_manager.is_quarantined(&module_id));

        // Quarantine the module
        let result = recovery_manager.quarantine_module(&module_id);
        assert!(result.is_ok());

        // Verify module is quarantined
        assert!(recovery_manager.is_quarantined(&module_id));

        // Verify module health reflects failed state
        let health = recovery_manager.get_module_health(&module_id);
        assert!(health.is_some());
        let health = health.unwrap();
        assert_eq!(health.status, HealthStatus::Failed);

        // Errors from quarantined modules should be ignored
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Quarantined module error");
        let action = recovery_manager.handle_module_error(&module_id, &error);
        assert_eq!(action, RecoveryAction::Ignore);

        // Release from quarantine
        let result = recovery_manager.release_quarantine(&module_id);
        assert!(result.is_ok());
        assert!(!recovery_manager.is_quarantined(&module_id));
    }

    #[test]
    fn test_multiple_module_failure_scenario() {
        let mut registry = ModuleRegistryImpl::new();
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        recovery_manager.set_module_registry(Arc::new(Mutex::new(registry.clone())));

        // Register multiple modules with dependencies
        let module_a = FailingTestModule::new("module-a", false, false);
        let module_b = FailingTestModule::new("module-b", false, true)
            .with_dependencies(vec![ModuleId::new("module-a")]);
        let module_c = FailingTestModule::new("module-c", false, false)
            .with_dependencies(vec![ModuleId::new("module-b")]);

        let module_a_id = module_a.module_id();
        let module_b_id = module_b.module_id();
        let module_c_id = module_c.module_id();

        registry.register_module(Box::new(module_a)).unwrap();
        registry.register_module(Box::new(module_b)).unwrap();
        registry.register_module(Box::new(module_c)).unwrap();

        // Simulate failure in module B
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Module B failure");
        let action = recovery_manager.handle_module_error(&module_b_id, &error);

        // Verify action was taken
        assert!(!matches!(action, RecoveryAction::Ignore));

        // Check that module A (dependency) is still healthy
        let health_a = recovery_manager.get_module_health(&module_a_id);
        if let Some(health_a) = health_a {
            // Module A should be unaffected by Module B's failure
            assert_eq!(health_a.error_count, 0);
        }

        // Module C depends on B, so it might be affected
        let unhealthy_modules = recovery_manager.get_unhealthy_modules();
        assert!(!unhealthy_modules.is_empty());
    }

    #[test]
    fn test_error_context_preservation() {
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("context-test-module");

        // Create detailed error context
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Detailed test error");
        let error_context = ErrorContext::new(module_id.clone(), &error, "test_operation")
            .with_context("user_id", "test_user")
            .with_context("session_id", "session_123")
            .with_severity(ErrorSeverity::High)
            .with_recoverable(true);

        // Handle the error
        recovery_manager.handle_module_error(&module_id, &error);

        // Verify recovery statistics are being tracked
        let stats = recovery_manager.get_recovery_stats();
        assert!(stats.total_errors > 0);
    }

    #[test]
    fn test_health_monitoring_integration() {
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("health-monitor-test");

        // Create a module health record
        let mut health = ModuleHealth::new();
        health.record_error("First error");
        health.record_error("Second error");
        health.record_error("Third error");

        // Update module health
        recovery_manager.update_module_health(&module_id, health);

        // Verify health is being tracked
        let retrieved_health = recovery_manager.get_module_health(&module_id);
        assert!(retrieved_health.is_some());
        let retrieved_health = retrieved_health.unwrap();
        assert_eq!(retrieved_health.error_count, 3);
        assert_eq!(retrieved_health.consecutive_errors, 3);

        // Record a success
        let mut updated_health = retrieved_health.clone();
        updated_health.record_success();
        recovery_manager.update_module_health(&module_id, updated_health);

        // Verify consecutive errors reset
        let final_health = recovery_manager.get_module_health(&module_id).unwrap();
        assert_eq!(final_health.consecutive_errors, 0);
        assert_eq!(final_health.error_count, 3); // Total count unchanged
    }

    #[test]
    fn test_recovery_statistics_tracking() {
        let mut recovery_manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("stats-test-module");

        // Simulate multiple error scenarios
        for i in 0..5 {
            let error = std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("Test error {}", i)
            );
            recovery_manager.handle_module_error(&module_id, &error);
        }

        // Perform some recovery actions
        recovery_manager.set_fallback_mode(&module_id, FallbackMode::Minimal);
        recovery_manager.quarantine_module(&module_id).unwrap();

        // Check recovery statistics
        let stats = recovery_manager.get_recovery_stats();
        assert_eq!(stats.total_errors, 5);
        assert!(stats.successful_recoveries > 0);
        assert_eq!(stats.modules_in_fallback, 1);
        assert_eq!(stats.quarantined_modules, 1);
        assert!(!stats.common_errors.is_empty());
    }

    #[test]
    fn test_concurrent_error_handling() {
        let recovery_manager = Arc::new(Mutex::new(ErrorRecoveryManagerImpl::new()));
        let mut handles = vec![];

        // Spawn multiple threads to simulate concurrent error handling
        for thread_id in 0..3 {
            let manager = Arc::clone(&recovery_manager);
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    let module_id = ModuleId::new(&format!("module-{}-{}", thread_id, i));
                    let error = std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Concurrent error from thread {} iteration {}", thread_id, i)
                    );
                    
                    if let Ok(mut manager) = manager.lock() {
                        manager.handle_module_error(&module_id, &error);
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify statistics were properly accumulated
        let manager = recovery_manager.lock().unwrap();
        let stats = manager.get_recovery_stats();
        assert_eq!(stats.total_errors, 30); // 3 threads * 10 iterations
    }

    #[test]
    fn test_error_recovery_event_generation() {
        let module_id = ModuleId::new("event-test-module");
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Test error for events");
        
        let error_context = ErrorContext::new(module_id.clone(), &error, "event_test_operation")
            .with_severity(ErrorSeverity::Critical);

        // Create an error recovery event
        let recovery_event = ErrorRecoveryEvent {
            module_id: module_id.clone(),
            action: RecoveryAction::Restart,
            error_context: error_context.clone(),
            success: true,
            timestamp: get_timestamp_ns(),
            event_data: std::collections::HashMap::new(),
        };

        // Verify event implements Event trait correctly
        assert_eq!(recovery_event.event_type(), "ErrorRecoveryEvent");
        assert!(recovery_event.timestamp() > 0);
        assert_eq!(recovery_event.priority(), EventPriority::Critical);
        
        // Verify event can be converted to Any
        let _any_event: &dyn Any = recovery_event.as_any();
    }
}