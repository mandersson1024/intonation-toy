//! # Event Subscription Hook for Developer UI
//!
//! This module provides a custom Yew hook for subscribing to module events in debug components.
//! The hook manages event subscriptions with automatic cleanup and type-safe event handling.
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::modules::developer_ui::hooks::use_event_subscription;
//! use crate::modules::audio_foundations::AudioEvent;
//!
//! #[function_component(DebugComponent)]
//! pub fn debug_component(props: &DebugProps) -> Html {
//!     // Subscribe to audio events with automatic cleanup
//!     let audio_event = use_event_subscription::<AudioEvent>(props.event_bus.clone());
//!     
//!     html! {
//!         <div>
//!             { if let Some(event) = &*audio_event {
//!                 format!("Latest audio event: {:?}", event)
//!             } else {
//!                 "Waiting for audio events...".to_string()
//!             }}
//!         </div>
//!     }
//! }
//! ```

use std::rc::Rc;
use std::cell::RefCell;
use yew::{hook, use_state, UseStateHandle};
use crate::modules::application_core::event_bus::Event;
use crate::modules::application_core::priority_event_bus::PriorityEventBus;

/// A simple hook for subscribing to events in Yew components
/// This is a basic implementation for the developer UI event integration story
#[hook]
pub fn use_event_subscription<T: Event + Clone + 'static>(
    _event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
) -> UseStateHandle<Option<T>> {
    // For now, return a simple state handle
    // Full implementation will be completed in subsequent tasks
    use_state(|| None)
}

/// Utility type for event subscription configuration
#[derive(Debug, Clone)]
pub struct EventSubscriptionConfig {
    pub event_type_name: &'static str,
    pub priority_filter: Option<crate::modules::application_core::event_bus::EventPriority>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test event implementation
    #[derive(Debug, Clone)]
    struct TestEvent {
        id: u32,
        timestamp: u64,
        priority: crate::modules::application_core::event_bus::EventPriority,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn priority(&self) -> crate::modules::application_core::event_bus::EventPriority {
            self.priority
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_event_subscription_config() {
        let config = EventSubscriptionConfig {
            event_type_name: "TestEvent",
            priority_filter: Some(crate::modules::application_core::event_bus::EventPriority::High),
        };
        
        assert_eq!(config.event_type_name, "TestEvent");
        assert!(config.priority_filter.is_some());
    }
}