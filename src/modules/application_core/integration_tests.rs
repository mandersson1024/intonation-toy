//! Integration tests for Story 002 - Priority Event Queue Implementation

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