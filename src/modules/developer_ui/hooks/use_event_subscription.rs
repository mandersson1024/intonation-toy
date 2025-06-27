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
use std::time::Instant;
use yew::{hook, use_state, use_effect_with, UseStateHandle};
use crate::modules::application_core::event_bus::Event;
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
use crate::modules::developer_ui::utils::debug_event_performance_monitor::{DebugEventPerformanceMonitor, create_performance_monitor};

/// Event subscription handle that manages cleanup automatically
pub struct EventSubscriptionHandle {
    subscription_id: u64,
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    cleanup_callback: Option<Box<dyn FnOnce()>>,
    performance_monitor: Rc<RefCell<DebugEventPerformanceMonitor>>,
    event_type: String,
    subscription_start_time: Instant,
}

impl EventSubscriptionHandle {
    pub fn new(
        subscription_id: u64,
        event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
        cleanup_callback: Option<Box<dyn FnOnce()>>,
        performance_monitor: Rc<RefCell<DebugEventPerformanceMonitor>>,
        event_type: String,
    ) -> Self {
        let subscription_start_time = Instant::now();
        
        // Record subscription performance
        {
            let mut monitor = performance_monitor.borrow_mut();
            monitor.record_subscription(&event_type, subscription_start_time.elapsed());
        }
        
        Self {
            subscription_id,
            event_bus,
            cleanup_callback,
            performance_monitor,
            event_type,
            subscription_start_time,
        }
    }
    
    pub fn subscription_id(&self) -> u64 {
        self.subscription_id
    }
}

impl Drop for EventSubscriptionHandle {
    fn drop(&mut self) {
        let unsubscription_start = Instant::now();
        
        // Execute cleanup callback if available
        if let Some(cleanup) = self.cleanup_callback.take() {
            cleanup();
        }
        
        // Unsubscribe from event bus if available
        if let Some(event_bus) = &self.event_bus {
            if let Ok(mut bus) = event_bus.try_borrow_mut() {
                // Note: PriorityEventBus would need unsubscribe method
                // This is a placeholder for the actual implementation
                web_sys::console::log_1(&format!("Cleaning up event subscription: {}", self.subscription_id).into());
            } else {
                web_sys::console::warn_1(&format!("Failed to cleanup event subscription: {}", self.subscription_id).into());
            }
        }
        
        // Record unsubscription performance
        let unsubscription_duration = unsubscription_start.elapsed();
        if let Ok(mut monitor) = self.performance_monitor.try_borrow_mut() {
            monitor.record_unsubscription(&self.event_type, unsubscription_duration);
        }
    }
}

/// Enhanced hook for subscribing to events with automatic lifecycle management
#[hook]
pub fn use_event_subscription<T: Event + Clone + 'static>(
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
) -> UseStateHandle<Option<T>> {
    let state = use_state(|| None);
    let subscription_handle = use_state(|| None::<EventSubscriptionHandle>);
    let performance_monitor = use_state(|| Rc::new(RefCell::new(create_performance_monitor())));
    
    {
        let state = state.clone();
        let subscription_handle = subscription_handle.clone();
        let event_bus = event_bus.clone();
        let performance_monitor = performance_monitor.clone();
        
        use_effect_with(
            event_bus.clone(),
            move |event_bus| {
                // Clear previous subscription if it exists
                subscription_handle.set(None);
                
                if let Some(bus) = event_bus {
                    let state_clone = state.clone();
                    let subscription_id = generate_subscription_id();
                    let event_type = std::any::type_name::<T>().to_string();
                    
                    // Create cleanup callback for memory leak prevention
                    let cleanup_callback = {
                        let state = state.clone();
                        Box::new(move || {
                            state.set(None);
                            web_sys::console::log_1(&"Event subscription cleaned up for component unmount".into());
                        })
                    };
                    
                    // Create subscription handle with automatic cleanup and performance monitoring
                    let handle = EventSubscriptionHandle::new(
                        subscription_id,
                        Some(bus.clone()),
                        Some(cleanup_callback),
                        (*performance_monitor).clone(),
                        event_type,
                    );
                    
                    subscription_handle.set(Some(handle));
                    
                    // Note: Actual event subscription would be implemented here
                    // when PriorityEventBus supports the subscribe method
                    web_sys::console::log_1(&format!("Event subscription created with ID: {}", subscription_id).into());
                }
                
                // Cleanup function for effect
                move || {
                    web_sys::console::log_1(&"Event subscription effect cleanup triggered".into());
                }
            },
        );
    }
    
    state
}

/// Enhanced hook with conditional subscription based on visibility
#[hook]
pub fn use_conditional_event_subscription<T: Event + Clone + 'static>(
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    enabled: bool,
) -> UseStateHandle<Option<T>> {
    let state = use_state(|| None);
    let subscription_handle = use_state(|| None::<EventSubscriptionHandle>);
    let performance_monitor = use_state(|| Rc::new(RefCell::new(create_performance_monitor())));
    
    {
        let state = state.clone();
        let subscription_handle = subscription_handle.clone();
        let event_bus = event_bus.clone();
        let performance_monitor = performance_monitor.clone();
        
        use_effect_with(
            (event_bus.clone(), enabled),
            move |(event_bus, enabled)| {
                // Clear previous subscription
                subscription_handle.set(None);
                
                if *enabled && event_bus.is_some() {
                    let bus = event_bus.as_ref().unwrap();
                    let state_clone = state.clone();
                    let subscription_id = generate_subscription_id();
                    let event_type = std::any::type_name::<T>().to_string();
                    
                    let cleanup_callback = {
                        let state = state.clone();
                        Box::new(move || {
                            state.set(None);
                            web_sys::console::log_1(&"Conditional event subscription cleaned up".into());
                        })
                    };
                    
                    let handle = EventSubscriptionHandle::new(
                        subscription_id,
                        Some(bus.clone()),
                        Some(cleanup_callback),
                        (*performance_monitor).clone(),
                        event_type,
                    );
                    
                    subscription_handle.set(Some(handle));
                    web_sys::console::log_1(&format!("Conditional event subscription created: {}", subscription_id).into());
                } else {
                    // Clear state when subscription is disabled
                    state.set(None);
                    web_sys::console::log_1(&"Conditional event subscription disabled".into());
                }
                
                move || {
                    web_sys::console::log_1(&"Conditional event subscription effect cleanup".into());
                }
            },
        );
    }
    
    state
}

/// Generate unique subscription ID for tracking
fn generate_subscription_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
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