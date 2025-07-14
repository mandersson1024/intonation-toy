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
//! use event_dispatcher::{Event, EventDispatcher, SharedEventDispatcher, create_shared_dispatcher, SubscriptionId};
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
//! // Subscribe to events (returns a SubscriptionId)
//! let sub_id = dispatcher.borrow_mut().subscribe("something_happened", |event| {
//!     if let MyEvent::SomethingHappened { data } = event {
//!         println!("Received: {}", data);
//!     }
//! });
//!
//! // Unsubscribe from events using the SubscriptionId
//! dispatcher.borrow_mut().unsubscribe(sub_id);
//!
//! // Publish events
//! let event = MyEvent::SomethingHappened { 
//!     data: "Hello World".to_string() 
//! };
//! dispatcher.borrow().publish(&event);
//! ```

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

/// Trait that all events must implement to be used with EventDispatcher
pub trait Event: Clone {
    /// Get the event type as a string for subscription matching
    fn event_type(&self) -> &'static str;
    
    /// Get a human-readable description of the event
    fn description(&self) -> String;
}

/// Unique identifier for a subscription
///
/// Returned by [`EventDispatcher::subscribe`]. Use it with [`EventDispatcher::unsubscribe`] to remove a subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

/// Internal subscription information
struct Subscription<T> {
    id: SubscriptionId,
    callback: Box<dyn Fn(T)>,
}

/// Event dispatcher that manages event subscriptions and publishing
pub struct EventDispatcher<T: Event> {
    /// Map of event type to list of subscriptions
    subscribers: HashMap<&'static str, Vec<Subscription<T>>>,
    /// Counter for generating unique subscription IDs
    next_subscription_id: AtomicU64,
}

impl<T: Event> EventDispatcher<T> {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            next_subscription_id: AtomicU64::new(1),
        }
    }
    
    /// Subscribe to events of a specific type
    /// 
    /// # Arguments
    /// * `event_type` - The type of event to subscribe to (e.g., "device_list_changed")
    /// * `callback` - The callback function to call when the event is published
    /// 
    /// # Returns
    /// A `SubscriptionId` that can be used to unsubscribe from this subscription
    pub fn subscribe<F>(&mut self, event_type: &'static str, callback: F) -> SubscriptionId
    where
        F: Fn(T) + 'static,
    {
        let id = SubscriptionId(self.next_subscription_id.fetch_add(1, Ordering::Relaxed));
        let subscription = Subscription {
            id,
            callback: Box::new(callback),
        };
        
        self.subscribers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(subscription);
            
        id
    }
    
    /// Publish an event to all subscribers of its type
    /// 
    /// # Arguments
    /// * `event` - The event to publish
    pub fn publish(&self, event: &T) {
        let event_type = event.event_type();
        
        if let Some(subscriptions) = self.subscribers.get(event_type) {
            for subscription in subscriptions {
                (subscription.callback)(event.clone());
            }
        }
    }
    
    /// Directly publish an event to all subscribers of its type, bypassing RefCell overhead.
    ///
    /// This method is intended for exclusive (non-shared) use of the dispatcher.
    pub fn publish_direct(&mut self, event: &T) {
        let event_type = event.event_type();
        if let Some(subscriptions) = self.subscribers.get(event_type) {
            for subscription in subscriptions {
                (subscription.callback)(event.clone());
            }
        }
    }
    
    /// Unsubscribe from a specific subscription using its ID
    ///
    /// # Arguments
    /// * `subscription_id` - The [`SubscriptionId`] returned from the [`subscribe`](Self::subscribe) method
    ///
    /// # Returns
    /// `true` if the subscription was found and removed, `false` otherwise
    ///
    /// # Example
    /// ```
    /// # use event_dispatcher::{EventDispatcher, Event};
    /// # #[derive(Clone)] struct E; impl Event for E { fn event_type(&self) -> &'static str { "e" } fn description(&self) -> String { String::new() } }
    /// let mut dispatcher = EventDispatcher::new();
    /// let sub_id = dispatcher.subscribe("e", |_| {});
    /// assert!(dispatcher.unsubscribe(sub_id));
    /// ```
    pub fn unsubscribe(&mut self, subscription_id: SubscriptionId) -> bool {
        for subscriptions in self.subscribers.values_mut() {
            if let Some(index) = subscriptions.iter().position(|sub| sub.id == subscription_id) {
                subscriptions.remove(index);
                return true;
            }
        }
        false
    }
    
    /// Get the number of subscribers for a specific event type
    pub fn subscriber_count(&self, event_type: &'static str) -> usize {
        self.subscribers.get(event_type).map(|v| v.len()).unwrap_or(0)
    }
    
    /// Get all event types that have subscribers
    pub fn subscribed_event_types(&self) -> Vec<&'static str> {
        self.subscribers.keys().copied().collect()
    }
    
    /// Clear all subscribers for a specific event type
    pub fn clear_subscribers(&mut self, event_type: &'static str) {
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
    use wasm_bindgen_test::*;

    // No wasm_bindgen_test_configure! needed for Node.js

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

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_event_dispatcher_basic_functionality() {
        use std::sync::{Arc, Mutex};
        
        let mut dispatcher = EventDispatcher::new();
        let received_events: Arc<Mutex<Vec<TestEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let received_events_clone = Arc::clone(&received_events);
        
        // Subscribe to test_a events
        let _subscription_id = dispatcher.subscribe("test_a", move |event| {
            received_events_clone.lock().unwrap().push(event);
        });
        
        // Test subscription count
        assert_eq!(dispatcher.subscriber_count("test_a"), 1);
        assert_eq!(dispatcher.subscriber_count("test_b"), 0);
        
        // Test subscribed event types
        let event_types = dispatcher.subscribed_event_types();
        assert!(event_types.contains(&"test_a"));
        assert!(!event_types.contains(&"test_b"));
        
        // Test publishing
        let test_event = TestEvent::TestA { value: 42 };
        dispatcher.publish(&test_event);
        
        // Verify callback was called with correct event
        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_event);
        
        // Test clearing subscribers
        dispatcher.clear_subscribers("test_a");
        assert_eq!(dispatcher.subscriber_count("test_a"), 0);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_publish_direct() {
        use std::sync::{Arc, Mutex};
        
        let mut dispatcher = EventDispatcher::new();
        let received_events: Arc<Mutex<Vec<TestEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let received_events_clone = Arc::clone(&received_events);
        
        // Subscribe to test_a events
        let _subscription_id = dispatcher.subscribe("test_a", move |event| {
            received_events_clone.lock().unwrap().push(event);
        });
        
        // Test publishing with publish_direct
        let test_event = TestEvent::TestA { value: 42 };
        dispatcher.publish_direct(&test_event);
        
        // Verify callback was called with correct event
        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_event);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_multiple_subscribers() {
        use std::sync::{Arc, Mutex};
        
        let mut dispatcher = EventDispatcher::new();
        let received_events: Arc<Mutex<Vec<TestEvent>>> = Arc::new(Mutex::new(Vec::new()));
        
        // Add multiple subscribers
        for _i in 0..3 {
            let received_events_clone = Arc::clone(&received_events);
            let _sub_id = dispatcher.subscribe("test_a", move |event| {
                received_events_clone.lock().unwrap().push(event);
            });
        }
        
        // Test subscription count
        assert_eq!(dispatcher.subscriber_count("test_a"), 3);
        
        // Publish event
        let test_event = TestEvent::TestA { value: 42 };
        dispatcher.publish(&test_event);
        
        // Verify all callbacks were called
        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 3);
        for event in events.iter() {
            assert_eq!(*event, test_event);
        }
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_different_event_types() {
        use std::sync::{Arc, Mutex};
        
        let mut dispatcher = EventDispatcher::new();
        let received_a_events: Arc<Mutex<Vec<TestEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let received_b_events: Arc<Mutex<Vec<TestEvent>>> = Arc::new(Mutex::new(Vec::new()));
        
        let received_a_clone = Arc::clone(&received_a_events);
        let received_b_clone = Arc::clone(&received_b_events);
        
        // Subscribe to both event types
        let _sub_id_a = dispatcher.subscribe("test_a", move |event| {
            received_a_clone.lock().unwrap().push(event);
        });
        
        let _sub_id_b = dispatcher.subscribe("test_b", move |event| {
            received_b_clone.lock().unwrap().push(event);
        });
        
        // Publish different event types
        let event_a = TestEvent::TestA { value: 42 };
        let event_b = TestEvent::TestB { message: "hello".to_string() };
        
        dispatcher.publish(&event_a);
        dispatcher.publish(&event_b);
        
        // Verify each subscriber only gets their respective events
        let a_events = received_a_events.lock().unwrap();
        let b_events = received_b_events.lock().unwrap();
        
        assert_eq!(a_events.len(), 1);
        assert_eq!(b_events.len(), 1);
        assert_eq!(a_events[0], event_a);
        assert_eq!(b_events[0], event_b);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_shared_event_dispatcher() {
        use std::sync::{Arc, Mutex};
        
        let shared_dispatcher = create_shared_dispatcher::<TestEvent>();
        let received_events: Arc<Mutex<Vec<TestEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let received_events_clone = Arc::clone(&received_events);
        
        // Test that we can borrow and use the shared dispatcher
        assert_eq!(shared_dispatcher.borrow().subscriber_count("test_a"), 0);
        
        // Test that we can add subscribers through the shared interface
        let _subscription_id = shared_dispatcher.borrow_mut().subscribe("test_a", move |event| {
            received_events_clone.lock().unwrap().push(event);
        });
        
        assert_eq!(shared_dispatcher.borrow().subscriber_count("test_a"), 1);
        
        // Test that we can publish through the shared interface
        let test_event = TestEvent::TestA { value: 123 };
        shared_dispatcher.borrow().publish(&test_event);
        
        // Verify callback was called
        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_event);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_unsubscribe() {
        use std::sync::{Arc, Mutex};
        let mut dispatcher = EventDispatcher::new();
        let received_events: Arc<Mutex<Vec<TestEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let received_events_clone = Arc::clone(&received_events);
        // Subscribe and store the id
        let sub_id = dispatcher.subscribe("test_a", move |event| {
            received_events_clone.lock().unwrap().push(event);
        });
        // Unsubscribe
        let unsubscribed = dispatcher.unsubscribe(sub_id);
        assert!(unsubscribed);
        // Publish event
        let test_event = TestEvent::TestA { value: 99 };
        dispatcher.publish(&test_event);
        // Verify callback was NOT called
        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 0);
    }
}