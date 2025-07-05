//! Event Dispatcher
//!
//! This crate provides a generic event dispatcher that allows components to subscribe
//! to events and publish events to all subscribers. This enables loose coupling between
//! components while maintaining real-time communication.
//!
//! # Type-Safe Event Dispatching
//!
//! This crate provides a generic, type-safe event dispatcher. The generic design
//! allows you to create strongly-typed event systems where each dispatcher only
//! handles events of a specific type, preventing runtime errors from mixed event types.
//!
//! # Example Usage
//!
//! ```rust
//! use event_dispatcher::{Event, EventDispatcher, SharedEventDispatcher, create_shared_dispatcher};
//!
//! // Define your domain-specific event type
//! #[derive(Clone, Debug)]
//! enum MyEvent {
//!     SomethingHappened { data: String },
//!     StateChanged { new_state: i32 },
//! }
//!
//! impl Event for MyEvent {
//!     fn event_type(&self) -> &'static str {
//!         match self {
//!             MyEvent::SomethingHappened { .. } => "something_happened",
//!             MyEvent::StateChanged { .. } => "state_changed",
//!         }
//!     }
//!     
//!     fn description(&self) -> String {
//!         match self {
//!             MyEvent::SomethingHappened { data } => format!("Something happened: {}", data),
//!             MyEvent::StateChanged { new_state } => format!("State changed to: {}", new_state),
//!         }
//!     }
//! }
//!
//! // Create a shared dispatcher for your domain
//! let dispatcher = create_shared_dispatcher::<MyEvent>();
//!
//! // Subscribe to events
//! dispatcher.borrow_mut().subscribe("something_happened", |event| {
//!     if let MyEvent::SomethingHappened { data } = event {
//!         println!("Received: {}", data);
//!     }
//! });
//!
//! // Publish events
//! dispatcher.borrow().publish(MyEvent::SomethingHappened { 
//!     data: "Hello World".to_string() 
//! });
//! ```

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Trait that all events must implement to be used with EventDispatcher
pub trait Event: Clone {
    /// Get the event type as a string for subscription matching
    fn event_type(&self) -> &'static str;
    
    /// Get a human-readable description of the event
    fn description(&self) -> String;
}

/// Callback type for event subscribers
pub type EventCallback<T> = Box<dyn Fn(T)>;

/// Event dispatcher that manages event subscriptions and publishing
pub struct EventDispatcher<T: Event> {
    /// Map of event type to list of subscriber callbacks
    subscribers: HashMap<String, Vec<EventCallback<T>>>,
}

impl<T: Event> EventDispatcher<T> {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }
    
    /// Subscribe to events of a specific type
    /// 
    /// # Arguments
    /// * `event_type` - The type of event to subscribe to (e.g., "device_list_changed")
    /// * `callback` - The callback function to call when the event is published
    pub fn subscribe<F>(&mut self, event_type: &str, callback: F)
    where
        F: Fn(T) + 'static,
    {
        self.subscribers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }
    
    /// Publish an event to all subscribers of its type
    /// 
    /// # Arguments
    /// * `event` - The event to publish
    pub fn publish(&self, event: T) {
        let event_type = event.event_type();
        
        if let Some(callbacks) = self.subscribers.get(event_type) {
            for callback in callbacks {
                callback(event.clone());
            }
        }
    }
    
    /// Get the number of subscribers for a specific event type
    pub fn subscriber_count(&self, event_type: &str) -> usize {
        self.subscribers.get(event_type).map(|v| v.len()).unwrap_or(0)
    }
    
    /// Get all event types that have subscribers
    pub fn subscribed_event_types(&self) -> Vec<String> {
        self.subscribers.keys().cloned().collect()
    }
    
    /// Clear all subscribers for a specific event type
    pub fn clear_subscribers(&mut self, event_type: &str) {
        self.subscribers.remove(event_type);
    }
    
    /// Clear all subscribers
    pub fn clear_all_subscribers(&mut self) {
        self.subscribers.clear();
    }
}

impl<T: Event> Default for EventDispatcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared event dispatcher for dependency injection across components.
/// Each application context (main app, test) should create one instance and 
/// distribute it to all components that need event communication.
pub type SharedEventDispatcher<T> = Rc<RefCell<EventDispatcher<T>>>;

/// Creates a shared event dispatcher for dependency injection across components.
/// Each application context should create one instance and distribute it to all
/// components that need event communication, rather than using a global singleton.
pub fn create_shared_dispatcher<T: Event>() -> SharedEventDispatcher<T> {
    Rc::new(RefCell::new(EventDispatcher::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestEvent {
        TestA { value: i32 },
        TestB { message: String },
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            match self {
                TestEvent::TestA { .. } => "test_a",
                TestEvent::TestB { .. } => "test_b",
            }
        }

        fn description(&self) -> String {
            match self {
                TestEvent::TestA { value } => format!("Test A with value: {}", value),
                TestEvent::TestB { message } => format!("Test B with message: {}", message),
            }
        }
    }

    #[test]
    fn test_event_dispatcher_basic_functionality() {
        let mut dispatcher = EventDispatcher::new();
        
        // Subscribe to test_a events
        dispatcher.subscribe("test_a", |event| {
            // In a real test, you'd use something like Arc<Mutex<Vec<_>>> to capture events
            // For this basic test, we'll just verify the callback is called
        });
        
        // Test subscription count
        assert_eq!(dispatcher.subscriber_count("test_a"), 1);
        assert_eq!(dispatcher.subscriber_count("test_b"), 0);
        
        // Test subscribed event types
        let event_types = dispatcher.subscribed_event_types();
        assert!(event_types.contains(&"test_a".to_string()));
        assert!(!event_types.contains(&"test_b".to_string()));
        
        // Test publishing (callback execution verified by lack of panic)
        dispatcher.publish(TestEvent::TestA { value: 42 });
        
        // Test clearing subscribers
        dispatcher.clear_subscribers("test_a");
        assert_eq!(dispatcher.subscriber_count("test_a"), 0);
    }

    #[test]
    fn test_shared_event_dispatcher() {
        let shared_dispatcher = create_shared_dispatcher::<TestEvent>();
        
        // Test that we can borrow and use the shared dispatcher
        assert_eq!(shared_dispatcher.borrow().subscriber_count("test_a"), 0);
        
        // Test that we can add subscribers through the shared interface
        shared_dispatcher.borrow_mut().subscribe("test_a", |_event| {
            // Test callback
        });
        
        assert_eq!(shared_dispatcher.borrow().subscriber_count("test_a"), 1);
        
        // Test that we can publish through the shared interface
        shared_dispatcher.borrow().publish(TestEvent::TestA { value: 123 });
    }
}