//! Event Dispatcher
//!
//! This module provides a generic event dispatcher that allows components to subscribe
//! to events and publish events to all subscribers. This enables loose coupling between
//! components while maintaining real-time communication.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::audio_events::AudioEvent;
use crate::common::dev_log;

/// Callback type for event subscribers
pub type EventCallback = Box<dyn Fn(AudioEvent)>;

/// Event dispatcher that manages event subscriptions and publishing
pub struct EventDispatcher {
    /// Map of event type to list of subscriber callbacks
    subscribers: HashMap<String, Vec<EventCallback>>,
}

impl EventDispatcher {
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
        F: Fn(AudioEvent) + 'static,
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
    pub fn publish(&self, event: AudioEvent) {
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

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared event dispatcher that can be used across the application
pub type SharedEventDispatcher = Rc<RefCell<EventDispatcher>>;

/// Create a new shared event dispatcher
pub fn create_shared_dispatcher() -> SharedEventDispatcher {
    Rc::new(RefCell::new(EventDispatcher::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioPermission;
    use std::cell::RefCell;
    use std::rc::Rc;
    
    #[test]
    fn test_event_dispatcher_creation() {
        let dispatcher = EventDispatcher::new();
        assert_eq!(dispatcher.subscriber_count("test_event"), 0);
        assert!(dispatcher.subscribed_event_types().is_empty());
    }
    
    #[test]
    fn test_event_subscription() {
        let mut dispatcher = EventDispatcher::new();
        
        // Subscribe to permission changes
        dispatcher.subscribe("permission_changed", |event| {
            match event {
                AudioEvent::PermissionChanged(_) => {
                    // Test callback received the right event
                }
                _ => panic!("Wrong event type received"),
            }
        });
        
        assert_eq!(dispatcher.subscriber_count("permission_changed"), 1);
        assert!(dispatcher.subscribed_event_types().contains(&"permission_changed".to_string()));
    }
    
    #[test]
    fn test_event_publishing() {
        let mut dispatcher = EventDispatcher::new();
        let received_events = Rc::new(RefCell::new(Vec::new()));
        
        // Subscribe to permission changes
        let received_events_clone = received_events.clone();
        dispatcher.subscribe("permission_changed", move |event| {
            received_events_clone.borrow_mut().push(event);
        });
        
        // Publish an event
        let event = AudioEvent::PermissionChanged(AudioPermission::Granted);
        dispatcher.publish(event);
        
        // Verify the event was received
        assert_eq!(received_events.borrow().len(), 1);
        let events = received_events.borrow();
        match &events[0] {
            AudioEvent::PermissionChanged(permission) => {
                assert_eq!(*permission, AudioPermission::Granted);
            }
            _ => panic!("Wrong event type received"),
        }
    }
    
    #[test]
    fn test_multiple_subscribers() {
        let mut dispatcher = EventDispatcher::new();
        let call_count = Rc::new(RefCell::new(0));
        
        // Subscribe multiple callbacks to the same event
        let call_count_clone1 = call_count.clone();
        dispatcher.subscribe("permission_changed", move |_| {
            *call_count_clone1.borrow_mut() += 1;
        });
        
        let call_count_clone2 = call_count.clone();
        dispatcher.subscribe("permission_changed", move |_| {
            *call_count_clone2.borrow_mut() += 1;
        });
        
        assert_eq!(dispatcher.subscriber_count("permission_changed"), 2);
        
        // Publish an event
        let event = AudioEvent::PermissionChanged(AudioPermission::Granted);
        dispatcher.publish(event);
        
        // Both callbacks should have been called
        assert_eq!(*call_count.borrow(), 2);
    }
    
    #[test]
    fn test_clear_subscribers() {
        let mut dispatcher = EventDispatcher::new();
        
        dispatcher.subscribe("permission_changed", |_| {});
        dispatcher.subscribe("device_list_changed", |_| {});
        
        assert_eq!(dispatcher.subscriber_count("permission_changed"), 1);
        assert_eq!(dispatcher.subscriber_count("device_list_changed"), 1);
        
        dispatcher.clear_subscribers("permission_changed");
        assert_eq!(dispatcher.subscriber_count("permission_changed"), 0);
        assert_eq!(dispatcher.subscriber_count("device_list_changed"), 1);
        
        dispatcher.clear_all_subscribers();
        assert_eq!(dispatcher.subscriber_count("device_list_changed"), 0);
    }
    
    #[test]
    fn test_shared_dispatcher() {
        let shared_dispatcher = create_shared_dispatcher();
        
        // Subscribe through shared dispatcher
        shared_dispatcher.borrow_mut().subscribe("permission_changed", |_| {});
        
        assert_eq!(shared_dispatcher.borrow().subscriber_count("permission_changed"), 1);
        
        // Publish through shared dispatcher
        let event = AudioEvent::PermissionChanged(AudioPermission::Granted);
        shared_dispatcher.borrow().publish(event);
    }
}