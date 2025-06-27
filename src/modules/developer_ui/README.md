# Developer UI Module - Event Integration Guide

## Overview

The Developer UI module provides a comprehensive event-driven debug interface for the pitch-toy application. This module integrates with the application's event bus system to enable real-time debugging, performance monitoring, and developer tools.

**Key Features:**
- Type-safe event handling across all debug components
- Automatic event subscription cleanup and memory leak prevention
- Performance monitoring for debug event system overhead
- Conditional compilation for zero production impact
- Cross-component event communication

## Event System Architecture

### Core Components

#### 1. Event Bus Integration
The Developer UI module integrates with the application's `PriorityEventBus` for all event communication:

```rust
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
use std::rc::Rc;
use std::cell::RefCell;

// Event bus shared across debug components
let event_bus = Rc<RefCell<PriorityEventBus>>::new(RefCell::new(PriorityEventBus::new()));
```

#### 2. Debug Event Publisher
Central publisher for all debug-related events with built-in performance monitoring:

```rust
use crate::modules::developer_ui::utils::debug_event_publisher::DebugEventPublisher;

let mut publisher = DebugEventPublisher::new(Some(event_bus.clone()));

// Publish debug control events
publisher.publish_control_event(DebugControlEvent::StartRecording)?;

// Publish custom events
publisher.publish(custom_event)?;
```

#### 3. Event Subscription Management
Type-safe event subscriptions with automatic cleanup:

```rust
use crate::modules::developer_ui::hooks::use_event_subscription::{
    EventSubscriptionHandle, EventSubscriptionConfig
};

// Configure subscription
let config = EventSubscriptionConfig {
    event_type_name: "AudioEvent",
    priority_filter: Some(EventPriority::High),
};

// Create subscription handle with automatic cleanup
let handle = EventSubscriptionHandle::new(
    subscription_id,
    Some(event_bus.clone()),
    Some(cleanup_callback),
    performance_monitor.clone(),
    config.event_type_name.to_string(),
);
```

## Event Types and Patterns

### 1. Debug Control Events
Predefined events for common debug operations:

```rust
pub enum DebugControlEvent {
    StartRecording,
    StopRecording,
    RequestPerformanceReport,
    SelectDevice { device_id: String },
    ToggleDebugMode,
    ClearLogs,
}
```

**Usage Pattern:**
```rust
// Audio control panel
publisher.publish_control_event(DebugControlEvent::StartRecording)?;

// Device selection
publisher.publish_control_event(DebugControlEvent::SelectDevice {
    device_id: "microphone_1".to_string()
})?;
```

### 2. Custom Event Implementation
For custom debug events, implement the `Event` trait:

```rust
use crate::modules::application_core::event_bus::{Event, EventPriority};

#[derive(Debug, Clone)]
pub struct CustomDebugEvent {
    pub component_id: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

impl Event for CustomDebugEvent {
    fn event_type(&self) -> &'static str {
        "CustomDebugEvent"
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
```

### 3. Type-Safe Event Handling
Use Rust's type system for compile-time safety:

```rust
// Type-safe event handler
fn handle_audio_event(event: &TestAudioEvent) {
    println!("Received audio event: ID={}, Data={}", 
             event.id, event.audio_data);
}

// Generic event dispatcher
fn dispatch_event<T: Event + 'static>(event: T, handlers: &[fn(&T)]) {
    for handler in handlers {
        handler(&event);
    }
}
```

## Performance Monitoring Integration

### 1. Performance Monitor Setup
The performance monitor tracks event system overhead:

```rust
use crate::modules::developer_ui::utils::debug_event_performance_monitor::*;

let mut performance_monitor = create_performance_monitor();

// Record subscription performance (target: <1ms)
performance_monitor.record_subscription("EventType", duration);

// Record event processing (target: <0.1ms)
performance_monitor.record_event_processed("EventType");

// Update memory usage (target: <5MB)
performance_monitor.update_memory_usage(subscription_count, memory_kb);
```

### 2. Performance Benchmarks
Built-in benchmarks validate performance requirements:

```rust
let benchmark_results = performance_monitor.run_benchmarks();

assert!(benchmark_results.subscription_benchmark.meets_requirement);
assert!(benchmark_results.publishing_benchmark.meets_requirement);
assert!(benchmark_results.memory_benchmark.meets_requirement);
```

### 3. Performance Alerts
Automatic alerts for performance bottlenecks:

```rust
// Configure alert thresholds
let config = PerformanceConfig {
    subscription_threshold_ms: 1.0,
    memory_threshold_kb: 5000.0,
    throughput_threshold: 1000,
};

// Check for alerts
let report = performance_monitor.get_performance_report();
for alert in &report.recent_alerts {
    match alert.alert_type {
        AlertType::SlowSubscription => handle_slow_subscription(alert),
        AlertType::HighMemoryUsage => handle_memory_alert(alert),
        AlertType::HighThroughput => handle_throughput_alert(alert),
    }
}
```

## Memory Leak Prevention

### 1. Automatic Cleanup
Event subscriptions are automatically cleaned up when components unmount:

```rust
use crate::modules::developer_ui::utils::memory_leak_prevention::*;

let mut prevention_manager = MemoryLeakPreventionManager::new();

// Register subscription
prevention_manager.register_subscription(
    subscription_id, 
    "component_id", 
    "EventType"
);

// Automatic cleanup on component unmount
impl Drop for DebugComponent {
    fn drop(&mut self) {
        prevention_manager.cleanup_subscription(self.subscription_id);
    }
}
```

### 2. Memory Leak Detection
Proactive detection of potential memory leaks:

```rust
// Check for memory leaks
prevention_manager.check_for_memory_leaks();

// Get leak statistics
let stats = prevention_manager.get_statistics();
if stats.potential_leaks > 0 {
    log::warn!("Detected {} potential memory leaks", stats.potential_leaks);
}
```

## Component Integration Patterns

### 1. Audio Control Panel Integration
```rust
// Audio control component with event integration
pub struct AudioControlPanel {
    publisher: DebugEventPublisher,
    subscription_handle: Option<EventSubscriptionHandle>,
}

impl AudioControlPanel {
    pub fn new(event_bus: Rc<RefCell<PriorityEventBus>>) -> Self {
        Self {
            publisher: DebugEventPublisher::new(Some(event_bus)),
            subscription_handle: None,
        }
    }

    pub fn start_recording(&mut self) -> Result<(), String> {
        self.publisher.publish_control_event(DebugControlEvent::StartRecording)
    }

    pub fn subscribe_to_audio_events(&mut self) -> Result<(), String> {
        // Subscribe with automatic cleanup
        let handle = create_subscription_handle(
            "AudioEvents",
            Box::new(|event| handle_audio_event(event))
        );
        self.subscription_handle = Some(handle);
        Ok(())
    }
}
```

### 2. Debug Panel Integration
```rust
// Debug panel with performance monitoring
pub struct DebugPanel {
    publisher: DebugEventPublisher,
    performance_monitor: Rc<RefCell<DebugEventPerformanceMonitor>>,
}

impl DebugPanel {
    pub fn request_performance_report(&mut self) -> Result<(), String> {
        self.publisher.publish_control_event(
            DebugControlEvent::RequestPerformanceReport
        )
    }

    pub fn get_current_metrics(&self) -> PerformanceReport {
        self.performance_monitor.borrow().get_performance_report()
    }
}
```

### 3. Cross-Component Communication
```rust
// Pattern for communication between debug components
pub struct ComponentCommunicator {
    publisher: DebugEventPublisher,
    subscriptions: Vec<EventSubscriptionHandle>,
}

impl ComponentCommunicator {
    pub fn setup_cross_component_communication(&mut self) {
        // Audio control -> Debug panel communication
        self.subscribe_to_events("AudioControlEvents", |event| {
            // Forward to debug panel
            debug_panel.handle_audio_control_event(event);
        });

        // Performance monitor -> Error display communication
        self.subscribe_to_events("PerformanceAlerts", |alert| {
            // Display performance warnings
            error_display.show_performance_warning(alert);
        });
    }
}
```

## Best Practices

### 1. Event Naming Conventions
- Use descriptive, hierarchical event names: `"Audio.Device.Connected"`
- Include component prefix: `"DebugPanel.PerformanceReport"`
- Use PascalCase for event type names

### 2. Error Handling
```rust
// Always handle event publishing errors
match publisher.publish_control_event(event) {
    Ok(_) => log::debug!("Event published successfully"),
    Err(e) => {
        log::error!("Failed to publish event: {}", e);
        // Implement fallback behavior
        handle_publish_failure(e);
    }
}
```

### 3. Performance Considerations
- Use `EventPriority::Low` for debug events to avoid impacting audio processing
- Implement event batching for high-frequency events
- Monitor subscription count and clean up unused subscriptions

### 4. Conditional Compilation
```rust
#[cfg(debug_assertions)]
pub fn debug_event_handler() {
    // Debug-only event handling
}

#[cfg(not(debug_assertions))]
pub fn debug_event_handler() {
    // No-op in production builds
}
```

### 5. Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::developer_ui::test_utilities::*;

    #[test]
    fn test_event_integration() {
        let event_bus = create_test_event_bus();
        let mut publisher = create_test_debug_publisher(Some(event_bus));
        
        // Test event publishing
        let result = publisher.publish_control_event(
            DebugControlEvent::StartRecording
        );
        assert!(result.is_ok());
        
        // Verify metrics
        let metrics = publisher.get_metrics().unwrap();
        assert_eq!(metrics.total_published, 1);
    }
}
```

## Integration Checklist

When integrating a new debug component with the event system:

- [ ] Implement event types with proper `Event` trait
- [ ] Set up `DebugEventPublisher` for publishing events
- [ ] Create `EventSubscriptionHandle` for subscriptions
- [ ] Implement automatic cleanup on component unmount
- [ ] Add performance monitoring integration
- [ ] Configure memory leak prevention
- [ ] Add comprehensive integration tests
- [ ] Verify conditional compilation works correctly
- [ ] Document event types and usage patterns
- [ ] Test cross-component communication

## Troubleshooting

### Common Issues and Solutions

1. **Events not being received**
   - Verify event bus is properly shared between components
   - Check event type names match exactly
   - Ensure subscription is active before events are published

2. **Memory leaks**
   - Ensure `EventSubscriptionHandle` is properly dropped
   - Use `MemoryLeakPreventionManager` for tracking
   - Implement cleanup in component `Drop` trait

3. **Performance issues**
   - Use `DebugEventPerformanceMonitor` to identify bottlenecks
   - Check for excessive event publishing frequency
   - Verify event priorities are set appropriately

4. **Type safety errors**
   - Ensure event types implement `Event` trait correctly
   - Use proper downcasting with `as_any()` method
   - Implement type-safe event handlers

## Advanced Patterns

### 1. Event Filtering
```rust
// Filter events by priority or type
let high_priority_filter = |event: &dyn Event| {
    event.priority() == EventPriority::High
};

// Apply filter to subscription
let filtered_handle = create_filtered_subscription(
    "AllEvents",
    high_priority_filter,
    event_handler
);
```

### 2. Event Transformation
```rust
// Transform events before processing
let transformer = |event: AudioEvent| -> DebugEvent {
    DebugEvent {
        source: "AudioSystem".to_string(),
        data: serde_json::to_value(event).unwrap(),
        timestamp: get_timestamp_ns(),
    }
};
```

### 3. Event Aggregation
```rust
// Aggregate multiple events into summaries
pub struct EventAggregator {
    buffer: Vec<AudioEvent>,
    timer: Timer,
}

impl EventAggregator {
    pub fn aggregate_events(&mut self) -> Option<AudioSummaryEvent> {
        if self.buffer.len() >= 10 || self.timer.elapsed() > Duration::from_secs(1) {
            let summary = AudioSummaryEvent::from_events(&self.buffer);
            self.buffer.clear();
            self.timer.reset();
            Some(summary)
        } else {
            None
        }
    }
}
```

This documentation provides comprehensive guidance for integrating with the Developer UI event system while maintaining performance, type safety, and proper resource management. 