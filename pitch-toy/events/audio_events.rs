//! Audio Event Types
//!
//! This module defines audio-specific events that can be published and subscribed to
//! by various components of the application. These events enable loose coupling between
//! the audio subsystem and other components like the console.

use event_dispatcher::{Event, SharedEventDispatcher, create_shared_dispatcher};

/// Audio-related events that can be published throughout the application
#[derive(Debug, Clone)]
pub enum AudioEvent {
    /// Circular buffer has been filled (ready for processing)
    BufferFilled { buffer_index: usize, length: usize },
    /// Circular buffer experienced overflow
    BufferOverflow { buffer_index: usize, overflow_count: usize },
    /// Buffer pool metrics update (periodic)
    BufferMetrics { total_buffers: usize, total_overflows: usize, memory_bytes: usize },
}

impl AudioEvent {
    /// Get the event type as a string for subscription matching
    pub fn event_type(&self) -> &'static str {
        match self {
            AudioEvent::BufferFilled { .. } => "buffer_filled",
            AudioEvent::BufferOverflow { .. } => "buffer_overflow",
            AudioEvent::BufferMetrics { .. } => "buffer_metrics",
        }
    }
    
    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            AudioEvent::BufferFilled { buffer_index, length } => {
                format!("Buffer {} filled ({} samples)", buffer_index, length)
            }
            AudioEvent::BufferOverflow { buffer_index, overflow_count } => {
                format!("Buffer {} overflow (count = {})", buffer_index, overflow_count)
            }
            AudioEvent::BufferMetrics { total_buffers, total_overflows, memory_bytes } => {
                format!("Buffer metrics: {} buffers, {} overflows, {:.2} MB", total_buffers, total_overflows, *memory_bytes as f64 / 1_048_576.0)
            }
        }
    }
}

impl Event for AudioEvent {
    fn event_type(&self) -> &'static str {
        self.event_type()
    }
    
    fn description(&self) -> String {
        self.description()
    }
}

/// Convenience type alias for audio event dispatcher
pub type AudioEventDispatcher = SharedEventDispatcher<AudioEvent>;

/// Creates a shared audio event dispatcher for audio subsystem communication.
/// This creates an audio-specific dispatcher that should be distributed to all 
/// components that need to publish or subscribe to audio events.
pub fn create_shared_audio_dispatcher() -> AudioEventDispatcher {
    create_shared_dispatcher::<AudioEvent>()
}

#[cfg(test)]
mod tests {
    use super::*;
     use wasm_bindgen_test::wasm_bindgen_test;
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_types() {
        
        
        let buffer_event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        assert_eq!(buffer_event.event_type(), "buffer_filled");
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_descriptions() {
        
        
        let buffer_event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        assert!(buffer_event.description().contains("Buffer 0 filled"));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_buffer_event_types_and_descriptions() {
        let filled = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        assert_eq!(filled.event_type(), "buffer_filled");
        assert!(filled.description().contains("Buffer 0 filled"));

        let overflow = AudioEvent::BufferOverflow { buffer_index: 1, overflow_count: 3 };
        assert_eq!(overflow.event_type(), "buffer_overflow");
        assert!(overflow.description().contains("overflow"));

        let metrics = AudioEvent::BufferMetrics { total_buffers: 8, total_overflows: 5, memory_bytes: 32768 };
        assert_eq!(metrics.event_type(), "buffer_metrics");
        assert!(metrics.description().contains("8 buffers"));
    }



    // Event Publishing and Subscription Integration Tests (Task 8 Requirements)
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_event_dispatcher_integration() {
        use event_dispatcher::EventDispatcher;
        use std::rc::Rc;
        use std::cell::RefCell;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        let events_received = Rc::new(RefCell::new(Vec::new()));
        
        // Subscribe to buffer filled events
        let events_clone = events_received.clone();
        dispatcher.subscribe("buffer_filled", move |event| {
            events_clone.borrow_mut().push(event.clone());
        });
        
        // Create and publish a buffer event
        let buffer_event = AudioEvent::BufferFilled {
            buffer_index: 0,
            length: 1024,
        };
        
        // Publish event
        dispatcher.publish(&buffer_event);
        
        // Verify event was received
        let received = events_received.borrow();
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0], AudioEvent::BufferFilled { .. }));
        
        // Verify subscriber count
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 1);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]  
    fn test_event_types_integration() {
        // Test that all event types work correctly
        let buffer_filled = AudioEvent::BufferFilled {
            buffer_index: 0,
            length: 1024,
        };
        assert_eq!(buffer_filled.event_type(), "buffer_filled");
        
        let buffer_overflow = AudioEvent::BufferOverflow {
            buffer_index: 0,
            overflow_count: 5,
        };
        assert_eq!(buffer_overflow.event_type(), "buffer_overflow");
        
        let buffer_metrics = AudioEvent::BufferMetrics {
            total_buffers: 8,
            total_overflows: 2,
            memory_bytes: 65536,
        };
        assert_eq!(buffer_metrics.event_type(), "buffer_metrics");
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_shared_audio_dispatcher() {
        let shared_dispatcher = create_shared_audio_dispatcher();
        
        // Subscribe through shared dispatcher
        shared_dispatcher.borrow_mut().subscribe("buffer_filled", |_| {});
        
        assert_eq!(shared_dispatcher.borrow().subscriber_count("buffer_filled"), 1);
        
        // Publish through shared dispatcher
        let event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        shared_dispatcher.borrow().publish(&event);
    }

    // Generic EventDispatcher tests with AudioEvent
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_dispatcher_creation() {
        use event_dispatcher::EventDispatcher;
        
        let dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        assert_eq!(dispatcher.subscriber_count("test_event"), 0);
        assert!(dispatcher.subscribed_event_types().is_empty());
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_subscription() {
        use event_dispatcher::EventDispatcher;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        
        // Subscribe to buffer filled events
        dispatcher.subscribe("buffer_filled", |event| {
            match event {
                AudioEvent::BufferFilled { .. } => {
                    // Test callback received the right event
                }
                _ => panic!("Wrong event type received"),
            }
        });
        
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 1);
        assert!(dispatcher.subscribed_event_types().contains(&"buffer_filled"));
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_publishing() {
        use event_dispatcher::EventDispatcher;
                use std::rc::Rc;
        use std::cell::RefCell;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        let received_events = Rc::new(RefCell::new(Vec::new()));
        
        // Subscribe to buffer filled events
        let received_events_clone = received_events.clone();
        dispatcher.subscribe("buffer_filled", move |event| {
            received_events_clone.borrow_mut().push(event);
        });
        
        // Publish an event
        let event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        dispatcher.publish(&event);
        
        // Verify the event was received
        assert_eq!(received_events.borrow().len(), 1);
        let events = received_events.borrow();
        match &events[0] {
            AudioEvent::BufferFilled { .. } => {
                // Test passed
            }
            _ => panic!("Wrong event type received"),
        }
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_multiple_subscribers() {
        use event_dispatcher::EventDispatcher;
                use std::rc::Rc;
        use std::cell::RefCell;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        let call_count = Rc::new(RefCell::new(0));
        
        // Subscribe multiple callbacks to the same event
        let call_count_clone1 = call_count.clone();
        dispatcher.subscribe("buffer_filled", move |_| {
            *call_count_clone1.borrow_mut() += 1;
        });
        
        let call_count_clone2 = call_count.clone();
        dispatcher.subscribe("buffer_filled", move |_| {
            *call_count_clone2.borrow_mut() += 1;
        });
        
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 2);
        
        // Publish an event
        let event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        dispatcher.publish(&event);
        
        // Both callbacks should have been called
        assert_eq!(*call_count.borrow(), 2);
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_clear_subscribers() {
        use event_dispatcher::EventDispatcher;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        
        dispatcher.subscribe("buffer_filled", |_| {});
        dispatcher.subscribe("audioworklet_status_changed", |_| {});
        
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 1);
        assert_eq!(dispatcher.subscriber_count("audioworklet_status_changed"), 1);
        
        dispatcher.clear_subscribers("buffer_filled");
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 0);
        assert_eq!(dispatcher.subscriber_count("audioworklet_status_changed"), 1);
        
        dispatcher.clear_all_subscribers();
        assert_eq!(dispatcher.subscriber_count("audioworklet_status_changed"), 0);
    }
}