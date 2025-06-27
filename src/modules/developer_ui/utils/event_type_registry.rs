//! # Event Type Registry for Developer UI
//!
//! This module provides type-safe event handling utilities for the Developer UI,
//! ensuring compile-time verification of event types and efficient event routing.

use std::collections::HashMap;
use std::any::TypeId;
use crate::modules::application_core::event_bus::{Event, EventPriority};

/// Registry for managing event type metadata and validation
#[derive(Debug)]
pub struct EventTypeRegistry {
    /// Map of event type names to their metadata
    type_metadata: HashMap<String, EventTypeMetadata>,
    
    /// Map of TypeId to event type names for fast lookup
    type_id_to_name: HashMap<TypeId, String>,
}

/// Metadata for a registered event type
#[derive(Debug, Clone)]
pub struct EventTypeMetadata {
    /// Human-readable name of the event type
    pub name: String,
    
    /// TypeId for compile-time type checking
    pub type_id: TypeId,
    
    /// Default priority for this event type
    pub default_priority: EventPriority,
    
    /// Whether this event type should be included in debug monitoring
    pub debug_monitored: bool,
    
    /// Event filtering configuration
    pub filter_config: EventFilterConfig,
}

/// Configuration for event filtering in debug components
#[derive(Debug, Clone)]
pub struct EventFilterConfig {
    /// Whether to apply priority filtering
    pub priority_filtering: bool,
    
    /// Minimum priority level to include (if priority filtering enabled)
    pub min_priority: EventPriority,
    
    /// Maximum events per second for rate limiting (0 = no limit)
    pub max_events_per_second: u32,
    
    /// Whether to batch events for performance
    pub batching_enabled: bool,
    
    /// Batch size for event batching
    pub batch_size: usize,
}

impl Default for EventFilterConfig {
    fn default() -> Self {
        Self {
            priority_filtering: false,
            min_priority: EventPriority::Low,
            max_events_per_second: 0,
            batching_enabled: false,
            batch_size: 10,
        }
    }
}

impl EventTypeRegistry {
    /// Create a new event type registry
    pub fn new() -> Self {
        Self {
            type_metadata: HashMap::new(),
            type_id_to_name: HashMap::new(),
        }
    }

    /// Register an event type with the registry
    pub fn register_event_type<T: Event + 'static>(
        &mut self,
        name: String,
        default_priority: EventPriority,
        debug_monitored: bool,
        filter_config: EventFilterConfig,
    ) -> Result<(), EventRegistryError> {
        let type_id = TypeId::of::<T>();
        
        // Check for duplicate registrations
        if self.type_id_to_name.contains_key(&type_id) {
            return Err(EventRegistryError::DuplicateType(name));
        }
        
        if self.type_metadata.contains_key(&name) {
            return Err(EventRegistryError::DuplicateName(name));
        }

        let metadata = EventTypeMetadata {
            name: name.clone(),
            type_id,
            default_priority,
            debug_monitored,
            filter_config,
        };

        self.type_metadata.insert(name.clone(), metadata);
        self.type_id_to_name.insert(type_id, name);

        Ok(())
    }

    /// Get metadata for an event type by name
    pub fn get_metadata(&self, name: &str) -> Option<&EventTypeMetadata> {
        self.type_metadata.get(name)
    }

    /// Get metadata for an event type by TypeId
    pub fn get_metadata_by_type<T: Event + 'static>(&self) -> Option<&EventTypeMetadata> {
        let type_id = TypeId::of::<T>();
        self.type_id_to_name.get(&type_id)
            .and_then(|name| self.type_metadata.get(name))
    }

    /// Get event type name from TypeId
    pub fn get_type_name<T: Event + 'static>(&self) -> Option<&str> {
        let type_id = TypeId::of::<T>();
        self.type_id_to_name.get(&type_id).map(|s| s.as_str())
    }

    /// Check if an event type is registered for debug monitoring
    pub fn is_debug_monitored<T: Event + 'static>(&self) -> bool {
        self.get_metadata_by_type::<T>()
            .map(|meta| meta.debug_monitored)
            .unwrap_or(false)
    }

    /// Get all registered event types for debug monitoring
    pub fn get_debug_monitored_types(&self) -> Vec<&EventTypeMetadata> {
        self.type_metadata.values()
            .filter(|meta| meta.debug_monitored)
            .collect()
    }

    /// Validate that an event type is suitable for debug subscription
    pub fn validate_debug_subscription<T: Event + 'static>(&self) -> Result<(), EventRegistryError> {
        let metadata = self.get_metadata_by_type::<T>()
            .ok_or_else(|| EventRegistryError::UnregisteredType(std::any::type_name::<T>().to_string()))?;

        if !metadata.debug_monitored {
            return Err(EventRegistryError::NotDebugMonitored(metadata.name.clone()));
        }

        Ok(())
    }

    /// Get the total number of registered event types
    pub fn len(&self) -> usize {
        self.type_metadata.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.type_metadata.is_empty()
    }

    /// Clear all registered event types
    pub fn clear(&mut self) {
        self.type_metadata.clear();
        self.type_id_to_name.clear();
    }
}

impl Default for EventTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during event type registration
#[derive(Debug, Clone)]
pub enum EventRegistryError {
    /// Event type with this TypeId is already registered
    DuplicateType(String),
    
    /// Event name is already registered
    DuplicateName(String),
    
    /// Event type is not registered
    UnregisteredType(String),
    
    /// Event type is not configured for debug monitoring
    NotDebugMonitored(String),
}

impl std::fmt::Display for EventRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventRegistryError::DuplicateType(name) => {
                write!(f, "Event type '{}' is already registered", name)
            }
            EventRegistryError::DuplicateName(name) => {
                write!(f, "Event name '{}' is already registered", name)
            }
            EventRegistryError::UnregisteredType(name) => {
                write!(f, "Event type '{}' is not registered", name)
            }
            EventRegistryError::NotDebugMonitored(name) => {
                write!(f, "Event type '{}' is not configured for debug monitoring", name)
            }
        }
    }
}

impl std::error::Error for EventRegistryError {}

/// Type-safe event subscription helper
pub struct TypeSafeEventSubscriber {
    registry: EventTypeRegistry,
}

impl TypeSafeEventSubscriber {
    /// Create a new type-safe event subscriber
    pub fn new(registry: EventTypeRegistry) -> Self {
        Self { registry }
    }

    /// Validate that a subscription is valid for debug monitoring
    pub fn validate_subscription<T: Event + 'static>(&self) -> Result<&EventTypeMetadata, EventRegistryError> {
        self.registry.validate_debug_subscription::<T>()?;
        self.registry.get_metadata_by_type::<T>()
            .ok_or_else(|| EventRegistryError::UnregisteredType(std::any::type_name::<T>().to_string()))
    }

    /// Get filter configuration for an event type
    pub fn get_filter_config<T: Event + 'static>(&self) -> Option<&EventFilterConfig> {
        self.registry.get_metadata_by_type::<T>()
            .map(|meta| &meta.filter_config)
    }

    /// Check if an event should be filtered based on priority
    pub fn should_filter_by_priority<T: Event + 'static>(&self, event: &T) -> bool {
        if let Some(config) = self.get_filter_config::<T>() {
            config.priority_filtering && event.priority() < config.min_priority
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::application_core::event_bus::get_timestamp_ns;

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

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_event_type_registration() {
        let mut registry = EventTypeRegistry::new();
        
        let result = registry.register_event_type::<TestEvent>(
            "TestEvent".to_string(),
            EventPriority::Normal,
            true,
            EventFilterConfig::default(),
        );
        
        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = EventTypeRegistry::new();
        
        // First registration should succeed
        let result1 = registry.register_event_type::<TestEvent>(
            "TestEvent".to_string(),
            EventPriority::Normal,
            true,
            EventFilterConfig::default(),
        );
        assert!(result1.is_ok());
        
        // Second registration should fail
        let result2 = registry.register_event_type::<TestEvent>(
            "TestEvent2".to_string(),
            EventPriority::High,
            true,
            EventFilterConfig::default(),
        );
        assert!(result2.is_err());
    }

    #[test]
    fn test_metadata_retrieval() {
        let mut registry = EventTypeRegistry::new();
        
        registry.register_event_type::<TestEvent>(
            "TestEvent".to_string(),
            EventPriority::High,
            true,
            EventFilterConfig::default(),
        ).unwrap();
        
        let metadata = registry.get_metadata_by_type::<TestEvent>();
        assert!(metadata.is_some());
        
        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, "TestEvent");
        assert_eq!(metadata.default_priority, EventPriority::High);
        assert!(metadata.debug_monitored);
    }

    #[test]
    fn test_debug_monitoring_check() {
        let mut registry = EventTypeRegistry::new();
        
        // Register event with debug monitoring enabled
        registry.register_event_type::<TestEvent>(
            "TestEvent".to_string(),
            EventPriority::Normal,
            true,
            EventFilterConfig::default(),
        ).unwrap();
        
        assert!(registry.is_debug_monitored::<TestEvent>());
        
        let debug_types = registry.get_debug_monitored_types();
        assert_eq!(debug_types.len(), 1);
        assert_eq!(debug_types[0].name, "TestEvent");
    }

    #[test]
    fn test_type_safe_subscriber() {
        let mut registry = EventTypeRegistry::new();
        
        registry.register_event_type::<TestEvent>(
            "TestEvent".to_string(),
            EventPriority::Normal,
            true,
            EventFilterConfig {
                priority_filtering: true,
                min_priority: EventPriority::High,
                ..Default::default()
            },
        ).unwrap();
        
        let subscriber = TypeSafeEventSubscriber::new(registry);
        
        // Validation should succeed for registered debug event
        let validation = subscriber.validate_subscription::<TestEvent>();
        assert!(validation.is_ok());
        
        // Test priority filtering
        let high_event = TestEvent {
            id: 1,
            timestamp: get_timestamp_ns(),
            priority: EventPriority::High,
        };
        
        let low_event = TestEvent {
            id: 2,
            timestamp: get_timestamp_ns(),
            priority: EventPriority::Low,
        };
        
        assert!(!subscriber.should_filter_by_priority(&high_event));
        assert!(subscriber.should_filter_by_priority(&low_event));
    }
}