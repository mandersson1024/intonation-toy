//! Event System
//!
//! This module provides an event-driven architecture for loose coupling between
//! application components. It allows components to publish and subscribe to events
//! without direct dependencies on each other.
//!
//! ## Architecture
//!
//! - **Audio Events**: Audio-specific events like device changes, permission changes
//! - **Event Dispatcher**: Central dispatcher that manages subscriptions and publishing
//! - **Shared Dispatcher**: Application-wide shared event dispatcher instance
//!
//! ## Usage
//!
//! ```rust,no_run
//! use pitch_toy::events::{AudioEvent, EventDispatcher, create_shared_audio_dispatcher};
//! use pitch_toy::audio::AudioContextState;
//!
//! // Create a dispatcher for audio events
//! let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
//!
//! // Subscribe to context state changes
//! dispatcher.subscribe("context_state_changed", |event| {
//!     match event {
//!         AudioEvent::ContextStateChanged(state) => {
//!             println!("Context state changed to: {}", state);
//!         }
//!         _ => {}
//!     }
//! });
//!
//! // Publish a context state change event
//! let event = AudioEvent::ContextStateChanged(AudioContextState::Running);
//! dispatcher.publish(&event);
//!
//! // Or use the shared dispatcher for cross-component communication
//! let shared_dispatcher = create_shared_audio_dispatcher();
//! ```

pub mod audio_events;

pub use audio_events::{AudioEvent, AudioEventDispatcher, create_shared_audio_dispatcher};
pub use event_dispatcher::{Event, EventDispatcher, SharedEventDispatcher, create_shared_dispatcher};

use std::cell::RefCell;

// Global shared event dispatcher for cross-component communication
thread_local! {
    static GLOBAL_EVENT_DISPATCHER: RefCell<Option<AudioEventDispatcher>> = RefCell::new(None);
}

/// Get or create the global shared event dispatcher
pub fn get_global_event_dispatcher() -> AudioEventDispatcher {
    GLOBAL_EVENT_DISPATCHER.with(|dispatcher| {
        // First check if we already have a dispatcher
        {
            let borrow = dispatcher.borrow();
            if let Some(ref existing) = *borrow {
                return existing.clone();
            }
        } // Release the borrow here
        
        // Create new dispatcher if none exists
        let new_dispatcher = create_shared_dispatcher::<AudioEvent>();
        *dispatcher.borrow_mut() = Some(new_dispatcher.clone());
        new_dispatcher
    })
}