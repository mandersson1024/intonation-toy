//! # Event Bus Infrastructure
//!
//! This module provides the core event bus infrastructure for type-safe inter-module communication.
//! The event system enables modules to publish and subscribe to events without tight coupling,
//! with compile-time type safety and performance optimizations for real-time audio processing.
//!
//! ## Key Components
//!
//! - [`Event`]: Base trait for all events with metadata
//! - [`EventBus`]: Core event publishing and subscription interface
//! - [`EventHandler`]: Type-safe event processing handlers
//! - [`EventPriority`]: Priority levels for event processing order
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::modules::application_core::event_bus::*;
//! use std::time::{SystemTime, UNIX_EPOCH};
//!
//! // Define a custom event
//! #[derive(Debug, Clone)]
//! struct AudioProcessingEvent {
//!     frequency: f32,
//!     confidence: f32,
//!     timestamp: u64,
//! }
//!
//! impl Event for AudioProcessingEvent {
//!     fn event_type(&self) -> &'static str {
//!         "AudioProcessingEvent"
//!     }
//!
//!     fn timestamp(&self) -> u64 {
//!         self.timestamp
//!     }
//!
//!     fn priority(&self) -> EventPriority {
//!         EventPriority::High
//!     }
//! }
//!
//! // Implement event handler
//! struct AudioEventHandler;
//!
//! impl EventHandler<AudioProcessingEvent> for AudioEventHandler {
//!     fn handle_event(&mut self, event: &AudioProcessingEvent) -> Result<(), Box<dyn std::error::Error>> {
//!         println!("Processed audio event: frequency={}, confidence={}", 
//!                  event.frequency, event.confidence);
//!         Ok(())
//!     }
//! }
//! ```

use std::any::Any;
use std::error::Error;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Priority levels for event processing.
///
/// Events are processed in priority order, with Critical events processed immediately
/// to minimize latency for time-sensitive operations like audio processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventPriority {
    /// Critical priority - processed immediately, bypassing queue (<1ms target)
    /// Used for: Audio buffer underruns, system shutdown, critical errors
    Critical = 0,
    
    /// High priority - processed before normal events
    /// Used for: Audio processing events, device state changes, user interactions
    High = 1,
    
    /// Normal priority - standard event processing
    /// Used for: Configuration updates, status notifications, non-urgent state changes
    Normal = 2,
    
    /// Low priority - processed when system is idle
    /// Used for: Logging, analytics, background operations, cleanup tasks
    Low = 3,
}

impl EventPriority {
    /// Returns the numeric priority value for queue ordering
    pub fn as_usize(&self) -> usize {
        *self as usize
    }
    
    /// Returns true if this is a critical priority event that should bypass queuing
    pub fn is_critical(&self) -> bool {
        matches!(self, EventPriority::Critical)
    }
}

impl Default for EventPriority {
    fn default() -> Self {
        EventPriority::Normal
    }
}

impl fmt::Display for EventPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            EventPriority::Critical => "Critical",
            EventPriority::High => "High", 
            EventPriority::Normal => "Normal",
            EventPriority::Low => "Low",
        };
        write!(f, "{}", name)
    }
}

/// Base trait for all events in the system.
///
/// Events must be Send + Sync + Clone to support multi-threaded event processing
/// and efficient event distribution. All events include metadata for debugging,
/// monitoring, and processing optimization.
///
/// ## Implementation Requirements
///
/// - `event_type()`: Must return a unique string identifier for the event type
/// - `timestamp()`: Should use high-precision timestamp for latency analysis
/// - `priority()`: Should match the urgency and timing requirements of the event
///
/// ## Performance Considerations
///
/// - Clone should be efficient (consider Arc for large data)
/// - Avoid large data payloads in events - use buffer references instead
/// - timestamp() should use monotonic time for accurate latency measurement
pub trait Event: Send + Sync + Clone + fmt::Debug {
    /// Returns a unique string identifier for this event type.
    /// Used for event routing, debugging, and monitoring.
    fn event_type(&self) -> &'static str;
    
    /// Returns the timestamp when this event was created (nanoseconds since epoch).
    /// Used for latency analysis and event ordering.
    fn timestamp(&self) -> u64;
    
    /// Returns the processing priority for this event.
    /// Determines queue placement and processing order.
    fn priority(&self) -> EventPriority;
    
    /// Returns optional event metadata for debugging and monitoring.
    /// Default implementation returns None for performance.
    fn metadata(&self) -> Option<&dyn Any> {
        None
    }
    
    /// Returns the event as Any for type-safe downcasting.
    /// Used internally by the event bus for type routing.
    fn as_any(&self) -> &dyn Any;
}

/// Type-safe event handler trait.
///
/// Handlers process specific event types with compile-time type safety.
/// The generic type parameter ensures that handlers can only be registered
/// for compatible event types.
///
/// ## Error Handling
///
/// Handlers should return errors for processing failures. The event bus
/// will handle error reporting and potential retry logic based on the
/// error type and event priority.
///
/// ## Performance Requirements
///
/// - Critical event handlers must complete in <1ms
/// - High priority handlers should complete in <10ms
/// - Avoid blocking operations in event handlers
/// - Use async handlers for I/O operations (future enhancement)
pub trait EventHandler<T: Event>: Send + Sync {
    /// Processes an event of type T.
    ///
    /// # Arguments
    /// * `event` - The event to process
    ///
    /// # Returns
    /// * `Ok(())` - Event processed successfully
    /// * `Err(error)` - Processing failed, error will be logged and reported
    ///
    /// # Performance
    /// Critical events: Must complete in <1ms
    /// High priority events: Should complete in <10ms
    fn handle_event(&mut self, event: &T) -> Result<(), Box<dyn Error>>;
    
    /// Returns handler metadata for debugging and monitoring.
    /// Default implementation returns the handler type name.
    fn handler_info(&self) -> HandlerInfo {
        HandlerInfo {
            name: std::any::type_name::<Self>().to_string(),
            event_type: std::any::type_name::<T>().to_string(),
        }
    }
}

/// Handler metadata for debugging and monitoring
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    pub name: String,
    pub event_type: String,
}

/// Core event bus trait for publishing and subscribing to events.
///
/// The event bus provides type-safe, priority-based event distribution
/// with performance optimizations for real-time applications.
///
/// ## Key Features
///
/// - Type-safe event routing with compile-time verification
/// - Priority-based processing for low-latency critical events
/// - Thread-safe concurrent access for multiple producers/consumers
/// - Performance monitoring and metrics collection
/// - Memory-efficient event distribution with zero-copy buffer references
///
/// ## Usage Patterns
///
/// ```rust
/// // Publishing events
/// let event = MyEvent::new();
/// event_bus.publish(event)?;
///
/// // Subscribing to events  
/// let handler = MyEventHandler::new();
/// event_bus.subscribe::<MyEvent>(Box::new(handler))?;
/// ```
pub trait EventBus: Send + Sync {
    /// Publishes an event to all registered subscribers.
    ///
    /// Events are queued based on priority, with Critical events processed immediately.
    /// The event is cloned for each subscriber to enable concurrent processing.
    ///
    /// # Arguments
    /// * `event` - The event to publish
    ///
    /// # Returns
    /// * `Ok(())` - Event published successfully
    /// * `Err(error)` - Publishing failed (queue full, bus stopped, etc.)
    ///
    /// # Performance
    /// - Critical events bypass queue and process in <1ms
    /// - Other events queued and processed by priority
    /// - Publishing should complete in <100μs
    fn publish<T: Event + 'static>(&self, event: T) -> Result<(), EventBusError>;
    
    /// Subscribes a handler to events of type T.
    ///
    /// The handler will receive all future events of type T until unsubscribed.
    /// Multiple handlers can be registered for the same event type.
    ///
    /// # Arguments
    /// * `handler` - Event handler implementing EventHandler<T>
    ///
    /// # Returns
    /// * `Ok(subscription_id)` - Subscription successful, returns ID for unsubscribing
    /// * `Err(error)` - Subscription failed
    ///
    /// # Type Safety
    /// Generic type parameter ensures compile-time type matching between
    /// event type and handler type.
    fn subscribe<T: Event + 'static>(&mut self, handler: Box<dyn EventHandler<T>>) -> Result<SubscriptionId, EventBusError>;
    
    /// Unsubscribes a handler by subscription ID.
    ///
    /// # Arguments
    /// * `subscription_id` - ID returned from subscribe()
    ///
    /// # Returns
    /// * `Ok(())` - Handler unsubscribed successfully
    /// * `Err(error)` - Unsubscribe failed (invalid ID, etc.)
    fn unsubscribe(&mut self, subscription_id: SubscriptionId) -> Result<(), EventBusError>;
    
    /// Returns current event bus statistics and performance metrics.
    fn get_metrics(&self) -> EventBusMetrics;
    
    /// Starts the event processing loop.
    /// Must be called before events can be processed.
    fn start(&mut self) -> Result<(), EventBusError>;
    
    /// Stops the event processing loop gracefully.
    /// Processes remaining queued events before stopping.
    fn stop(&mut self) -> Result<(), EventBusError>;
    
    /// Returns the current state of the event bus.
    fn state(&self) -> EventBusState;
}

/// Unique identifier for event subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Event bus state for monitoring and debugging
#[derive(Debug, Clone, PartialEq)]
pub enum EventBusState {
    Stopped,
    Starting,  
    Running,
    Stopping,
    Error(String),
}

/// Event bus performance metrics
#[derive(Debug, Clone)]
pub struct EventBusMetrics {
    /// Average event processing latency by priority level (nanoseconds)
    pub avg_latency_by_priority: [u64; 4],
    
    /// Current queue depth by priority level
    pub queue_depths: [usize; 4],
    
    /// Events processed per second
    pub events_per_second: f64,
    
    /// Total events processed since start
    pub total_events_processed: u64,
    
    /// Total memory usage for event bus operations (bytes)
    pub memory_usage_bytes: usize,
    
    /// Number of active subscriptions
    pub active_subscriptions: usize,
    
    /// Error count by error type
    pub error_counts: std::collections::HashMap<String, u32>,
}

impl Default for EventBusMetrics {
    fn default() -> Self {
        Self {
            avg_latency_by_priority: [0; 4],
            queue_depths: [0; 4], 
            events_per_second: 0.0,
            total_events_processed: 0,
            memory_usage_bytes: 0,
            active_subscriptions: 0,
            error_counts: std::collections::HashMap::new(),
        }
    }
}

/// Errors that can occur during event bus operations
#[derive(Debug, Clone)]
pub enum EventBusError {
    /// Event bus is not running
    NotRunning,
    
    /// Event bus is already running
    AlreadyRunning,
    
    /// Event queue is full
    QueueFull,
    
    /// Invalid subscription ID
    InvalidSubscription,
    
    /// Handler registration failed
    HandlerRegistrationFailed(String),
    
    /// Event processing failed 
    ProcessingFailed(String),
    
    /// Internal error with context
    Internal(String),
}

impl fmt::Display for EventBusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventBusError::NotRunning => write!(f, "Event bus is not running"),
            EventBusError::AlreadyRunning => write!(f, "Event bus is already running"),
            EventBusError::QueueFull => write!(f, "Event queue is full"),
            EventBusError::InvalidSubscription => write!(f, "Invalid subscription ID"),
            EventBusError::HandlerRegistrationFailed(msg) => write!(f, "Handler registration failed: {}", msg),
            EventBusError::ProcessingFailed(msg) => write!(f, "Event processing failed: {}", msg),
            EventBusError::Internal(msg) => write!(f, "Internal event bus error: {}", msg),
        }
    }
}

impl Error for EventBusError {}

/// Utility function to get current timestamp in nanoseconds
pub fn get_timestamp_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestEvent {
        id: u32,
        timestamp: u64,
        priority: EventPriority,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn priority(&self) -> EventPriority {
            self.priority
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    struct TestHandler {
        processed_count: usize,
    }

    impl EventHandler<TestEvent> for TestHandler {
        fn handle_event(&mut self, _event: &TestEvent) -> Result<(), Box<dyn Error>> {
            self.processed_count += 1;
            Ok(())
        }
    }

    #[test]
    fn test_event_priority_ordering() {
        assert!(EventPriority::Critical < EventPriority::High);
        assert!(EventPriority::High < EventPriority::Normal);  
        assert!(EventPriority::Normal < EventPriority::Low);
    }

    #[test]
    fn test_event_priority_display() {
        assert_eq!(EventPriority::Critical.to_string(), "Critical");
        assert_eq!(EventPriority::High.to_string(), "High");
        assert_eq!(EventPriority::Normal.to_string(), "Normal");
        assert_eq!(EventPriority::Low.to_string(), "Low");
    }

    #[test]
    fn test_event_implementation() {
        let event = TestEvent {
            id: 1,
            timestamp: get_timestamp_ns(),
            priority: EventPriority::High,
        };

        assert_eq!(event.event_type(), "TestEvent");
        assert_eq!(event.priority(), EventPriority::High);
        assert!(event.timestamp() > 0);
    }

    #[test]
    fn test_handler_implementation() {
        let mut handler = TestHandler { processed_count: 0 };
        let event = TestEvent {
            id: 1,
            timestamp: get_timestamp_ns(),
            priority: EventPriority::Normal,
        };

        let result = handler.handle_event(&event);
        assert!(result.is_ok());
        assert_eq!(handler.processed_count, 1);
    }

    #[test]
    fn test_subscription_id() {
        let id1 = SubscriptionId::new(123);
        let id2 = SubscriptionId::new(123);
        let id3 = SubscriptionId::new(456);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1.as_u64(), 123);
    }

    #[test]
    fn test_event_bus_metrics_default() {
        let metrics = EventBusMetrics::default();
        assert_eq!(metrics.total_events_processed, 0);
        assert_eq!(metrics.events_per_second, 0.0);
        assert_eq!(metrics.active_subscriptions, 0);
    }
}