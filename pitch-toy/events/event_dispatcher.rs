//! Event Dispatcher Integration
//!
//! This module provides integration with the generic event-dispatcher crate,
//! adding application-specific logging and re-exporting the core functionality.

use crate::common::dev_log;
pub use event_dispatcher::{Event, EventCallback, EventDispatcher, SharedEventDispatcher};
use std::rc::Rc;
use std::cell::RefCell;

/// Creates a shared event dispatcher for dependency injection across components.
/// Each application context should create one instance and distribute it to all
/// components that need event communication, rather than using a global singleton.
/// 
/// This version includes application-specific logging integration.
pub fn create_shared_dispatcher<T: Event>() -> SharedEventDispatcher<T> {
    let dispatcher = Rc::new(RefCell::new(EventDispatcher::new()));
    
    // TODO: Add application-specific logging integration here in the future
    dev_log!("EventDispatcher: Created new shared dispatcher");
    
    dispatcher
}

/// Application-specific event dispatcher wrapper that adds logging
pub struct LoggingEventDispatcher<T: Event> {
    inner: EventDispatcher<T>,
}

impl<T: Event> LoggingEventDispatcher<T> {
    pub fn new() -> Self {
        Self {
            inner: EventDispatcher::new(),
        }
    }
    
    pub fn subscribe<F>(&mut self, event_type: &str, callback: F)
    where
        F: Fn(T) + 'static,
    {
        dev_log!("EventDispatcher: Subscribing to event type: {}", event_type);
        self.inner.subscribe(event_type, callback);
    }
    
    pub fn publish(&self, event: T) {
        let event_type = event.event_type();
        dev_log!("EventDispatcher: Publishing event: {}", event.description());
        
        let subscriber_count = self.inner.subscriber_count(event_type);
        if subscriber_count > 0 {
            dev_log!("EventDispatcher: Notifying {} subscribers", subscriber_count);
        } else {
            dev_log!("EventDispatcher: No subscribers for event type: {}", event_type);
        }
        
        self.inner.publish(event);
    }
    
    pub fn subscriber_count(&self, event_type: &str) -> usize {
        self.inner.subscriber_count(event_type)
    }
    
    pub fn subscribed_event_types(&self) -> Vec<String> {
        self.inner.subscribed_event_types()
    }
    
    pub fn clear_subscribers(&mut self, event_type: &str) {
        dev_log!("EventDispatcher: Clearing subscribers for event type: {}", event_type);
        self.inner.clear_subscribers(event_type);
    }
    
    pub fn clear_all_subscribers(&mut self) {
        dev_log!("EventDispatcher: Clearing all subscribers");
        self.inner.clear_all_subscribers();
    }
}

impl<T: Event> Default for LoggingEventDispatcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

