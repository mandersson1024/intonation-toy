//! Error Service Abstraction
//!
//! Provides centralized error management with event bus integration
//! for the modular architecture system.

use std::error::Error;
use std::fmt::{self, Display};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use crate::modules::application_core::{Event, EventBus, EventPriority};
use super::error_types::{ApplicationError, ErrorCategory, ErrorSeverity};

/// Error service error types
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceError {
    EventBusNotAvailable,
    SubscriptionFailed(String),
    ErrorProcessingFailed(String),
    InvalidError(String),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::EventBusNotAvailable => write!(f, "Event bus not available"),
            ServiceError::SubscriptionFailed(msg) => write!(f, "Subscription failed: {}", msg),
            ServiceError::ErrorProcessingFailed(msg) => write!(f, "Error processing failed: {}", msg),
            ServiceError::InvalidError(msg) => write!(f, "Invalid error: {}", msg),
        }
    }
}

impl Error for ServiceError {}

/// Error callback function type
pub type ErrorCallback = Box<dyn Fn(&ApplicationError) + Send + Sync>;

/// Event identifier for tracking specific events
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct EventId(pub String);

impl EventId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
    
    pub fn generate() -> Self {
        let timestamp = js_sys::Date::now() as u64;
        let random = (js_sys::Math::random() * 1000000.0) as u32;
        Self(format!("event_{}_{}", timestamp, random))
    }
}

/// Subscription identifier for error callbacks
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct SubscriptionId(pub String);

impl SubscriptionId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
    
    pub fn generate() -> Self {
        let timestamp = js_sys::Date::now() as u64;
        let random = (js_sys::Math::random() * 1000000.0) as u32;
        Self(format!("error_sub_{}_{}", timestamp, random))
    }
}

/// Error event for event bus integration
#[derive(Debug, Clone)]
pub struct ErrorEvent {
    pub error: ApplicationError,
    pub source_module: String,
    pub event_id: EventId,
    pub priority: EventPriority,
}

impl Event for ErrorEvent {
    fn event_type(&self) -> &'static str {
        "ErrorEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.error.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        self.priority
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ErrorEvent {
    pub fn new(error: ApplicationError, source_module: &str) -> Self {
        let priority = match error.severity {
            ErrorSeverity::Critical => EventPriority::High,
            ErrorSeverity::Warning => EventPriority::Normal,
            ErrorSeverity::Info => EventPriority::Low,
        };
        
        Self {
            error,
            source_module: source_module.to_string(),
            event_id: EventId::generate(),
            priority,
        }
    }
}

/// Recovery event for error resolution notifications
#[derive(Debug, Clone)]
pub struct RecoveryEvent {
    pub error_id: String,
    pub recovery_action: String,
    pub success: bool,
    pub event_id: EventId,
}

impl Event for RecoveryEvent {
    fn event_type(&self) -> &'static str {
        "RecoveryEvent"
    }
    
    fn timestamp(&self) -> u64 {
        js_sys::Date::now() as u64
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl RecoveryEvent {
    pub fn new(error_id: &str, recovery_action: &str, success: bool) -> Self {
        Self {
            error_id: error_id.to_string(),
            recovery_action: recovery_action.to_string(),
            success,
            event_id: EventId::generate(),
        }
    }
}

/// Core error service interface for modular architecture
/// 
/// This trait abstracts error management functionality to enable consistent
/// error handling across all modules with event bus integration.
pub trait ErrorService: Send + Sync {
    /// Report an error to the service
    /// 
    /// # Arguments
    /// * `error` - Application error to report
    /// * `source_module` - Optional source module identifier
    /// 
    /// # Returns
    /// * `Ok(())` - Error reported successfully
    /// * `Err(ServiceError)` - Failed to report error
    fn report_error(&mut self, error: ApplicationError, source_module: Option<&str>) -> Result<(), ServiceError>;
    
    /// Get recent errors from the service
    /// 
    /// # Arguments
    /// * `max_count` - Maximum number of errors to return
    /// 
    /// # Returns
    /// Vector of recent application errors
    fn get_recent_errors(&self, max_count: usize) -> Vec<ApplicationError>;
    
    /// Get errors by category
    /// 
    /// # Arguments
    /// * `category` - Error category to filter by
    /// 
    /// # Returns
    /// Vector of errors matching the category
    fn get_errors_by_category(&self, category: ErrorCategory) -> Vec<ApplicationError>;
    
    /// Get errors by severity
    /// 
    /// # Arguments
    /// * `severity` - Error severity to filter by
    /// 
    /// # Returns
    /// Vector of errors matching the severity
    fn get_errors_by_severity(&self, severity: ErrorSeverity) -> Vec<ApplicationError>;
    
    /// Subscribe to error notifications
    /// 
    /// # Arguments
    /// * `callback` - Callback function to receive error notifications
    /// 
    /// # Returns
    /// * `Ok(SubscriptionId)` - Subscription created successfully
    /// * `Err(ServiceError)` - Failed to create subscription
    fn subscribe_to_errors(&mut self, callback: ErrorCallback) -> Result<SubscriptionId, ServiceError>;
    
    /// Unsubscribe from error notifications
    /// 
    /// # Arguments
    /// * `subscription_id` - Subscription to remove
    /// 
    /// # Returns
    /// * `Ok(())` - Subscription removed successfully
    /// * `Err(ServiceError)` - Failed to remove subscription
    fn unsubscribe_from_errors(&mut self, subscription_id: &SubscriptionId) -> Result<(), ServiceError>;
    
    /// Clear all errors from the service
    /// 
    /// # Returns
    /// * `Ok(())` - Errors cleared successfully
    /// * `Err(ServiceError)` - Failed to clear errors
    fn clear_errors(&mut self) -> Result<(), ServiceError>;
    
    /// Clear specific error by ID
    /// 
    /// # Arguments
    /// * `error_id` - Error ID to clear
    /// 
    /// # Returns
    /// * `Ok(())` - Error cleared successfully
    /// * `Err(ServiceError)` - Failed to clear error
    fn clear_error(&mut self, error_id: &str) -> Result<(), ServiceError>;
    
    /// Check if the application can continue running
    /// 
    /// Analyzes current errors to determine if the application
    /// can continue operating safely.
    /// 
    /// # Returns
    /// * `true` - Application can continue
    /// * `false` - Application should stop due to critical errors
    fn can_app_continue(&self) -> bool;
    
    /// Get fallback message for critical errors
    /// 
    /// Returns a user-friendly message explaining critical errors
    /// and what actions the user should take.
    /// 
    /// # Returns
    /// * `Some(String)` - Fallback message for user
    /// * `None` - No critical errors requiring fallback
    fn get_fallback_message(&self) -> Option<String>;
    
    /// Report error recovery attempt
    /// 
    /// # Arguments
    /// * `error_id` - Error that recovery was attempted for
    /// * `recovery_action` - Description of recovery action taken
    /// * `success` - Whether recovery was successful
    /// 
    /// # Returns
    /// * `Ok(())` - Recovery reported successfully
    /// * `Err(ServiceError)` - Failed to report recovery
    fn report_recovery(&mut self, error_id: &str, recovery_action: &str, success: bool) -> Result<(), ServiceError>;
    
    /// Set event bus for error event publishing
    /// 
    /// # Arguments
    /// * `event_bus` - Event bus instance for publishing error events
    /// 
    /// # Returns
    /// * `Ok(())` - Event bus set successfully
    /// * `Err(ServiceError)` - Failed to set event bus
    fn set_event_bus(&mut self, event_bus: Arc<dyn EventBus>) -> Result<(), ServiceError>;
    
    /// Check if event bus is available
    /// 
    /// # Returns
    /// * `true` - Event bus is available for publishing
    /// * `false` - Event bus is not available
    fn has_event_bus(&self) -> bool;
}

/// Error service factory trait for creating service instances
pub trait ErrorServiceFactory {
    /// Create a new error service instance
    /// 
    /// # Returns
    /// New error service instance
    fn create_error_service(&self) -> Box<dyn ErrorService>;
    
    /// Create error service with event bus integration
    /// 
    /// # Arguments
    /// * `event_bus` - Event bus for error event publishing
    /// 
    /// # Returns
    /// * `Ok(Box<dyn ErrorService>)` - Service created successfully
    /// * `Err(ServiceError)` - Failed to create service
    fn create_error_service_with_event_bus(&self, event_bus: Arc<dyn EventBus>) -> Result<Box<dyn ErrorService>, ServiceError>;
}