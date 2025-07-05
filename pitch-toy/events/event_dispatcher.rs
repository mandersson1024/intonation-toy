//! Event Dispatcher
//!
//! This module provides a generic event dispatcher that allows components to subscribe
//! to events and publish events to all subscribers. This enables loose coupling between
//! components while maintaining real-time communication.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::common::dev_log;

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
        dev_log!("EventDispatcher: Subscribing to event type: {}", event_type);
        
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
        dev_log!("EventDispatcher: Publishing event: {}", event.description());
        
        if let Some(callbacks) = self.subscribers.get(event_type) {
            dev_log!("EventDispatcher: Notifying {} subscribers", callbacks.len());
            
            for callback in callbacks {
                callback(event.clone());
            }
        } else {
            dev_log!("EventDispatcher: No subscribers for event type: {}", event_type);
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
        dev_log!("EventDispatcher: Clearing subscribers for event type: {}", event_type);
        self.subscribers.remove(event_type);
    }
    
    /// Clear all subscribers
    pub fn clear_all_subscribers(&mut self) {
        dev_log!("EventDispatcher: Clearing all subscribers");
        self.subscribers.clear();
    }
}

impl<T: Event> Default for EventDispatcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared event dispatcher that can be used across the application
pub type SharedEventDispatcher<T> = Rc<RefCell<EventDispatcher<T>>>;

/// Create a new shared event dispatcher
pub fn create_shared_dispatcher<T: Event>() -> SharedEventDispatcher<T> {
    Rc::new(RefCell::new(EventDispatcher::new()))
}

