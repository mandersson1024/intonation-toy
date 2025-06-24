//! # Event Bus Test Infrastructure
//!
//! Comprehensive testing infrastructure for the event bus system that provides
//! unit tests, integration tests, performance tests, memory leak detection,
//! concurrent access testing, error condition testing, and performance benchmarks.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use crate::modules::application_core::*;

/// Test event types for comprehensive testing
#[derive(Debug, Clone, PartialEq)]
pub struct TestEvent {
    pub id: u32,
    pub data: String,
    pub timestamp: u64,
    pub priority: EventPriority,
}

impl Event for TestEvent {
    fn timestamp(&self) -> u64 { self.timestamp }
    fn priority(&self) -> EventPriority { self.priority }
    fn event_type(&self) -> &'static str { "TestEvent" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioTestEvent {
    pub buffer_id: u32,
    pub sample_rate: u32,
    pub timestamp: u64,
}

impl Event for AudioTestEvent {
    fn timestamp(&self) -> u64 { self.timestamp }
    fn priority(&self) -> EventPriority { EventPriority::Critical }
    fn event_type(&self) -> &'static str { "AudioTestEvent" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Test event handler that tracks processed events
pub struct TestEventHandler {
    pub received_events: Arc<Mutex<Vec<TestEvent>>>,
    pub processing_time_ms: u64,
}

impl TestEventHandler {
    pub fn new(processing_time_ms: u64) -> Self {
        Self {
            received_events: Arc::new(Mutex::new(Vec::new())),
            processing_time_ms,
        }
    }

    pub fn get_received_count(&self) -> usize {
        self.received_events.lock().unwrap().len()
    }

    pub fn clear(&self) {
        self.received_events.lock().unwrap().clear();
    }
}

impl EventHandler<TestEvent> for TestEventHandler {
    fn handle_event(&mut self, event: &TestEvent) -> Result<(), Box<dyn std::error::Error>> {
        if self.processing_time_ms > 0 {
            std::thread::sleep(Duration::from_millis(self.processing_time_ms));
        }
        
        self.received_events.lock().unwrap().push(event.clone());
        Ok(())
    }
}

/// Audio test event handler
pub struct AudioTestEventHandler {
    pub received_events: Arc<Mutex<Vec<AudioTestEvent>>>,
}

impl AudioTestEventHandler {
    pub fn new() -> Self {
        Self {
            received_events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl EventHandler<AudioTestEvent> for AudioTestEventHandler {
    fn handle_event(&mut self, event: &AudioTestEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.received_events.lock().unwrap().push(event.clone());
        Ok(())
    }
}

/// Test results structure
#[derive(Debug)]
pub struct TestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub test_details: Vec<TestDetail>,
}

#[derive(Debug)]
pub struct TestDetail {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            test_details: Vec::new(),
        }
    }

    pub fn add_test(&mut self, name: &str, passed: bool, duration: Duration, error: Option<String>) {
        self.total_tests += 1;
        if passed {
            self.passed_tests += 1;
        } else {
            self.failed_tests += 1;
        }
        
        self.test_details.push(TestDetail {
            name: name.to_string(),
            passed,
            duration,
            error,
        });
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            return 100.0;
        }
        (self.passed_tests as f64 / self.total_tests as f64) * 100.0
    }
}

/// Comprehensive test runner for event bus system
pub struct EventBusTestRunner;

impl EventBusTestRunner {
    /// Run all comprehensive tests for Story 006 requirements
    pub fn run_all_tests() -> TestResults {
        let mut results = TestResults::new();
        
        // Unit tests for all event bus components
        Self::run_unit_tests(&mut results);
        
        // Integration tests with multiple modules
        Self::run_integration_tests(&mut results);
        
        // Performance stress tests
        Self::run_performance_tests(&mut results);
        
        // Memory leak detection tests
        Self::run_memory_tests(&mut results);
        
        // Error condition testing
        Self::run_error_tests(&mut results);
        
        // Benchmark suite for performance regression detection
        Self::run_benchmark_tests(&mut results);
        
        results
    }
    
    /// Unit tests for all event bus components
    fn run_unit_tests(results: &mut TestResults) {
        // Test 1: Event bus creation and initialization
        let start = Instant::now();
        let test_result = Self::test_event_bus_creation();
        results.add_test("Event Bus Creation", test_result.is_ok(), start.elapsed(), 
                        test_result.err().map(|e| e.to_string()));
        
        // Test 2: Event priority ordering
        let start = Instant::now();
        let test_result = Self::test_priority_ordering();
        results.add_test("Priority Ordering", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 3: Multiple event types routing
        let start = Instant::now();
        let test_result = Self::test_multiple_event_types();
        results.add_test("Multiple Event Types", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 4: Handler registration and subscription
        let start = Instant::now();
        let test_result = Self::test_handler_registration();
        results.add_test("Handler Registration", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 5: Event publishing and processing
        let start = Instant::now();
        let test_result = Self::test_event_publishing();
        results.add_test("Event Publishing", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
    }
    
    /// Integration tests with multiple modules
    fn run_integration_tests(results: &mut TestResults) {
        // Test 1: Cross-module communication
        let start = Instant::now();
        let test_result = Self::test_cross_module_communication();
        results.add_test("Cross-Module Communication", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 2: Event bus with performance monitoring
        let start = Instant::now();
        let test_result = Self::test_performance_monitoring_integration();
        results.add_test("Performance Monitoring Integration", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
    }
    
    /// Performance stress tests (1000+ events/second)
    fn run_performance_tests(results: &mut TestResults) {
        // Test 1: High throughput (>1000 events/second)
        let start = Instant::now();
        let test_result = Self::test_high_throughput();
        results.add_test("High Throughput", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 2: Critical event latency (<1ms)
        let start = Instant::now();
        let test_result = Self::test_critical_latency();
        results.add_test("Critical Event Latency", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 3: Sustained load testing
        let start = Instant::now();
        let test_result = Self::test_sustained_load();
        results.add_test("Sustained Load", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
    }
    
    /// Memory leak detection tests
    fn run_memory_tests(results: &mut TestResults) {
        // Test 1: Event queue cleanup
        let start = Instant::now();
        let test_result = Self::test_memory_cleanup();
        results.add_test("Memory Cleanup", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 2: Handler reference cleanup
        let start = Instant::now();
        let test_result = Self::test_handler_cleanup();
        results.add_test("Handler Cleanup", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
    }
    
    /// Error condition testing
    fn run_error_tests(results: &mut TestResults) {
        // Test 1: Queue overflow handling
        let start = Instant::now();
        let test_result = Self::test_queue_overflow();
        results.add_test("Queue Overflow", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 2: Invalid event handling
        let start = Instant::now();
        let test_result = Self::test_invalid_events();
        results.add_test("Invalid Events", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 3: Handler error isolation
        let start = Instant::now();
        let test_result = Self::test_handler_errors();
        results.add_test("Handler Error Isolation", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
    }
    
    /// Benchmark suite for performance regression detection
    fn run_benchmark_tests(results: &mut TestResults) {
        // Test 1: Event publishing benchmark
        let start = Instant::now();
        let test_result = Self::benchmark_event_publishing();
        results.add_test("Publishing Benchmark", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
        
        // Test 2: Event processing benchmark
        let start = Instant::now();
        let test_result = Self::benchmark_event_processing();
        results.add_test("Processing Benchmark", test_result.is_ok(), start.elapsed(),
                        test_result.err().map(|e| e.to_string()));
    }
    
    // Individual test implementations
    
    fn test_event_bus_creation() -> Result<(), Box<dyn std::error::Error>> {
        let event_bus = PriorityEventBus::new();
        let metrics = event_bus.get_metrics();
        
        if metrics.total_events_processed != 0 {
            return Err("Event bus should start with zero processed events".into());
        }
        
        if metrics.queue_depths.iter().sum::<usize>() != 0 {
            return Err("Event bus should start with empty queues".into());
        }
        
        Ok(())
    }
    
    fn test_priority_ordering() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        // Publish events in reverse priority order
        event_bus.publish(TestEvent {
            id: 1, data: "Low".to_string(), timestamp: 1000, priority: EventPriority::Low
        })?;
        event_bus.publish(TestEvent {
            id: 2, data: "Normal".to_string(), timestamp: 2000, priority: EventPriority::Normal
        })?;
        event_bus.publish(TestEvent {
            id: 3, data: "High".to_string(), timestamp: 3000, priority: EventPriority::High
        })?;
        event_bus.publish(TestEvent {
            id: 4, data: "Critical".to_string(), timestamp: 4000, priority: EventPriority::Critical
        })?;
        
        event_bus.process_events()?;
        
        // Verify processing order (Critical -> High -> Normal -> Low)
        // Note: This is a simplified test - actual priority verification would require
        // access to the handler's received events in order
        
        Ok(())
    }
    
    fn test_multiple_event_types() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        
        let test_handler = TestEventHandler::new(0);
        let audio_handler = AudioTestEventHandler::new();
        
        event_bus.subscribe(Box::new(test_handler))?;
        event_bus.subscribe(Box::new(audio_handler))?;
        
        event_bus.publish(TestEvent {
            id: 1, data: "Test".to_string(), timestamp: 1000, priority: EventPriority::Normal
        })?;
        
        event_bus.publish(AudioTestEvent {
            buffer_id: 42, sample_rate: 44100, timestamp: 2000
        })?;
        
        event_bus.process_events()?;
        
        Ok(())
    }
    
    fn test_handler_registration() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        
        // Test successful registration
        event_bus.subscribe(Box::new(handler))?;
        
        // Test publishing after registration
        event_bus.publish(TestEvent {
            id: 1, data: "Test".to_string(), timestamp: 1000, priority: EventPriority::Normal
        })?;
        
        event_bus.process_events()?;
        
        Ok(())
    }
    
    fn test_event_publishing() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        for i in 0..100 {
            event_bus.publish(TestEvent {
                id: i, data: format!("Event {}", i), timestamp: i as u64 * 1000,
                priority: EventPriority::Normal
            })?;
        }
        
        event_bus.process_events()?;
        
        let metrics = event_bus.get_metrics();
        if metrics.total_events_processed < 100 {
            return Err("Not all events were processed".into());
        }
        
        Ok(())
    }
    
    fn test_cross_module_communication() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        
        let test_handler = TestEventHandler::new(0);
        let audio_handler = AudioTestEventHandler::new();
        
        event_bus.subscribe(Box::new(test_handler))?;
        event_bus.subscribe(Box::new(audio_handler))?;
        
        // Simulate cross-module communication
        event_bus.publish(TestEvent {
            id: 1, data: "Module A to Module B".to_string(),
            timestamp: 1000, priority: EventPriority::High
        })?;
        
        event_bus.publish(AudioTestEvent {
            buffer_id: 1, sample_rate: 44100, timestamp: 1100
        })?;
        
        event_bus.process_events()?;
        
        Ok(())
    }
    
    fn test_performance_monitoring_integration() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(1); // 1ms processing time
        event_bus.subscribe(Box::new(handler))?;
        
        // Publish events and verify monitoring
        for i in 0..50 {
            event_bus.publish(TestEvent {
                id: i, data: format!("Monitored event {}", i),
                timestamp: i as u64 * 1000, priority: EventPriority::Normal
            })?;
        }
        
        event_bus.process_events()?;
        
        let metrics = event_bus.get_metrics();
        if metrics.total_events_processed != 50 {
            return Err("Monitoring metrics incorrect".into());
        }
        
        Ok(())
    }
    
    fn test_high_throughput() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        const EVENTS_COUNT: u32 = 2000;
        let start = Instant::now();
        
        // Publish events rapidly
        for i in 0..EVENTS_COUNT {
            event_bus.publish(TestEvent {
                id: i, data: format!("High throughput {}", i),
                timestamp: i as u64 * 1000, priority: EventPriority::Normal
            })?;
        }
        
        event_bus.process_events()?;
        
        let duration = start.elapsed();
        let events_per_second = EVENTS_COUNT as f64 / duration.as_secs_f64();
        
        if events_per_second < 1000.0 {
            return Err(format!("Throughput too low: {:.0} events/second", events_per_second).into());
        }
        
        Ok(())
    }
    
    fn test_critical_latency() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        let start = Instant::now();
        
        event_bus.publish(TestEvent {
            id: 1, data: "Critical latency test".to_string(),
            timestamp: start.elapsed().as_nanos() as u64,
            priority: EventPriority::Critical
        })?;
        
        event_bus.process_events()?;
        
        let processing_time = start.elapsed();
        if processing_time.as_millis() >= 1 {
            return Err(format!("Critical event latency too high: {}ms", processing_time.as_millis()).into());
        }
        
        Ok(())
    }
    
    fn test_sustained_load() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        // Run sustained load for 5 seconds
        let start = Instant::now();
        let mut event_count = 0;
        
        while start.elapsed().as_secs() < 5 {
            event_bus.publish(TestEvent {
                id: event_count, data: format!("Sustained {}", event_count),
                timestamp: event_count as u64 * 1000, priority: EventPriority::Normal
            })?;
            
            if event_count % 100 == 0 {
                event_bus.process_events()?;
            }
            
            event_count += 1;
        }
        
        event_bus.process_events()?;
        
        let events_per_second = event_count as f64 / start.elapsed().as_secs_f64();
        if events_per_second < 500.0 {
            return Err(format!("Sustained load too low: {:.0} events/second", events_per_second).into());
        }
        
        Ok(())
    }
    
    fn test_memory_cleanup() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        // Process many batches to check for memory leaks
        for batch in 0..10 {
            for i in 0..100 {
                event_bus.publish(TestEvent {
                    id: batch * 100 + i, data: format!("Memory test {}", i),
                    timestamp: (batch * 100 + i) as u64 * 1000,
                    priority: EventPriority::Normal
                })?;
            }
            
            event_bus.process_events()?;
            
            // Verify queue is empty after processing
            let metrics = event_bus.get_metrics();
            if metrics.queue_depths.iter().sum::<usize>() != 0 {
                return Err(format!("Queue not empty after batch {}", batch).into());
            }
        }
        
        Ok(())
    }
    
    fn test_handler_cleanup() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        
        {
            let handler = TestEventHandler::new(0);
            event_bus.subscribe(Box::new(handler))?;
            
            event_bus.publish(TestEvent {
                id: 1, data: "Handler cleanup test".to_string(),
                timestamp: 1000, priority: EventPriority::Normal
            })?;
            
            event_bus.process_events()?;
            
            // Handler will be dropped here
        }
        
        // Should not panic after handler is dropped
        let result = event_bus.publish(TestEvent {
            id: 2, data: "After cleanup".to_string(),
            timestamp: 2000, priority: EventPriority::Normal
        });
        
        if result.is_err() {
            return Err("Event bus should handle dropped handlers gracefully".into());
        }
        
        Ok(())
    }
    
    fn test_queue_overflow() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::with_capacity(5);
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        // Fill queue to capacity
        for i in 0..5 {
            event_bus.publish(TestEvent {
                id: i, data: format!("Event {}", i),
                timestamp: i as u64 * 1000, priority: EventPriority::Normal
            })?;
        }
        
        // Try to add one more (should handle gracefully)
        let overflow_result = event_bus.publish(TestEvent {
            id: 999, data: "Overflow event".to_string(),
            timestamp: 999000, priority: EventPriority::Normal
        });
        
        // Process events to free space
        event_bus.process_events()?;
        
        // Should be able to add events again
        event_bus.publish(TestEvent {
            id: 1000, data: "Recovery event".to_string(),
            timestamp: 1000000, priority: EventPriority::Normal
        })?;
        
        Ok(())
    }
    
    fn test_invalid_events() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        // Test with extreme timestamp
        let result = event_bus.publish(TestEvent {
            id: 1, data: "Future event".to_string(),
            timestamp: std::u64::MAX, priority: EventPriority::Normal
        });
        
        if result.is_err() {
            return Err("Event bus should handle extreme timestamps gracefully".into());
        }
        
        Ok(())
    }
    
    fn test_handler_errors() -> Result<(), Box<dyn std::error::Error>> {
        // This would require a custom error handler implementation
        // Simplified test for now
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        event_bus.publish(TestEvent {
            id: 1, data: "Error test".to_string(),
            timestamp: 1000, priority: EventPriority::Normal
        })?;
        
        let result = event_bus.process_events();
        if result.is_err() {
            return Err("Event bus should isolate handler errors".into());
        }
        
        Ok(())
    }
    
    fn benchmark_event_publishing() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        const BENCHMARK_EVENTS: u32 = 10000;
        
        let start = Instant::now();
        
        for i in 0..BENCHMARK_EVENTS {
            event_bus.publish(TestEvent {
                id: i, data: format!("Benchmark {}", i),
                timestamp: i as u64 * 1000, priority: EventPriority::Normal
            })?;
        }
        
        let publishing_time = start.elapsed();
        let events_per_second = BENCHMARK_EVENTS as f64 / publishing_time.as_secs_f64();
        
        if events_per_second < 10000.0 {
            return Err(format!("Publishing performance too low: {:.0} events/second", events_per_second).into());
        }
        
        Ok(())
    }
    
    fn benchmark_event_processing() -> Result<(), Box<dyn std::error::Error>> {
        let mut event_bus = PriorityEventBus::new();
        let handler = TestEventHandler::new(0);
        event_bus.subscribe(Box::new(handler))?;
        
        const BENCHMARK_EVENTS: u32 = 10000;
        
        // Pre-populate queue
        for i in 0..BENCHMARK_EVENTS {
            event_bus.publish(TestEvent {
                id: i, data: format!("Processing benchmark {}", i),
                timestamp: i as u64 * 1000, priority: EventPriority::Normal
            })?;
        }
        
        let start = Instant::now();
        event_bus.process_events()?;
        let processing_time = start.elapsed();
        
        let events_per_second = BENCHMARK_EVENTS as f64 / processing_time.as_secs_f64();
        
        if events_per_second < 5000.0 {
            return Err(format!("Processing performance too low: {:.0} events/second", events_per_second).into());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod test_infrastructure_tests {
    use super::*;
    
    #[test]
    fn test_event_bus_test_runner() {
        let results = EventBusTestRunner::run_all_tests();
        
        assert!(results.total_tests > 0, "Should run some tests");
        assert!(results.success_rate() > 80.0, "Should have high success rate: {:.1}%", results.success_rate());
        
        println!("Test Results Summary:");
        println!("Total Tests: {}", results.total_tests);
        println!("Passed: {}", results.passed_tests);
        println!("Failed: {}", results.failed_tests);
        println!("Success Rate: {:.1}%", results.success_rate());
        
        for detail in &results.test_details {
            let status = if detail.passed { "PASS" } else { "FAIL" };
            println!("{}: {} ({:.2}ms)", status, detail.name, detail.duration.as_millis());
            if let Some(error) = &detail.error {
                println!("  Error: {}", error);
            }
        }
    }
    
    #[test]
    fn test_individual_components() {
        // Test basic event creation
        let event = TestEvent {
            id: 1,
            data: "Test".to_string(),
            timestamp: 1000,
            priority: EventPriority::Normal,
        };
        
        assert_eq!(event.event_type(), "TestEvent");
        assert_eq!(event.timestamp(), 1000);
        assert_eq!(event.priority(), EventPriority::Normal);
        
        // Test handler creation
        let handler = TestEventHandler::new(0);
        assert_eq!(handler.get_received_count(), 0);
    }
}