//! Modular Error Service Implementation
//!
//! Wraps the legacy ErrorManager to provide the new modular interface
//! with event bus integration for cross-module error handling.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::rc::Rc;

use super::error_service::{
    ErrorService, ServiceError, ErrorCallback, SubscriptionId, 
    ErrorEvent, RecoveryEvent
};
use super::{Event, EventBus, EventPriority};
use crate::legacy::active::services::error_manager::{
    ErrorManager, ApplicationError, ErrorCategory, ErrorSeverity
};

/// Modular error service implementation that wraps legacy ErrorManager
/// 
/// This implementation provides the new modular interface with event bus
/// integration while using the existing, proven ErrorManager underneath.
pub struct ModularErrorService {
    legacy_manager: Rc<RefCell<ErrorManager>>,
    event_bus: Option<Arc<dyn EventBus>>,
    subscribers: HashMap<SubscriptionId, ErrorCallback>,
    next_subscription_id: u64,
}

impl ModularErrorService {
    /// Create a new modular error service instance
    pub fn new() -> Self {
        Self {
            legacy_manager: Rc::new(RefCell::new(ErrorManager::new())),
            event_bus: None,
            subscribers: HashMap::new(),
            next_subscription_id: 0,
        }
    }
    
    /// Create with existing legacy error manager (for compatibility)
    pub fn with_legacy_manager(legacy_manager: Rc<RefCell<ErrorManager>>) -> Self {
        Self {
            legacy_manager,
            event_bus: None,
            subscribers: HashMap::new(),
            next_subscription_id: 0,
        }
    }
    
    /// Get reference to legacy manager for backward compatibility
    pub fn get_legacy_manager(&self) -> Rc<RefCell<ErrorManager>> {
        self.legacy_manager.clone()
    }
    
    /// Generate next subscription ID
    fn generate_subscription_id(&mut self) -> SubscriptionId {
        self.next_subscription_id += 1;
        SubscriptionId::new(&format!("modular_error_service_{}", self.next_subscription_id))
    }
    
    /// Notify all subscribers about an error
    fn notify_subscribers(&self, error: &ApplicationError) {
        for callback in self.subscribers.values() {
            callback(error);
        }
    }
    
    /// Publish error event to event bus if available
    fn publish_error_event(&self, error: &ApplicationError, source_module: &str) {
        if let Some(ref event_bus) = self.event_bus {
            let error_event = ErrorEvent::new(error.clone(), source_module);
            if let Err(e) = event_bus.publish(Box::new(error_event)) {
                web_sys::console::warn_1(&format!("Failed to publish error event: {:?}", e).into());
            }
        }
    }
    
    /// Publish recovery event to event bus if available
    fn publish_recovery_event(&self, error_id: &str, recovery_action: &str, success: bool) {
        if let Some(ref event_bus) = self.event_bus {
            let recovery_event = RecoveryEvent::new(error_id, recovery_action, success);
            if let Err(e) = event_bus.publish(Box::new(recovery_event)) {
                web_sys::console::warn_1(&format!("Failed to publish recovery event: {:?}", e).into());
            }
        }
    }
}

impl Default for ModularErrorService {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorService for ModularErrorService {
    fn report_error(&mut self, error: ApplicationError, source_module: Option<&str>) -> Result<(), ServiceError> {
        // Add error to legacy manager
        self.legacy_manager.borrow_mut().add_error(error.clone());
        
        // Notify subscribers
        self.notify_subscribers(&error);
        
        // Publish to event bus if available
        let module_name = source_module.unwrap_or("unknown");
        self.publish_error_event(&error, module_name);
        
        Ok(())
    }
    
    fn get_recent_errors(&self, max_count: usize) -> Vec<ApplicationError> {
        let all_errors = self.legacy_manager.borrow().get_all_errors();
        
        // Sort by timestamp (most recent first) and take max_count
        let mut errors: Vec<ApplicationError> = all_errors.into_iter().cloned().collect();
        errors.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        errors.truncate(max_count);
        errors
    }
    
    fn get_errors_by_category(&self, category: ErrorCategory) -> Vec<ApplicationError> {
        self.legacy_manager.borrow()
            .get_errors_by_category(category)
            .into_iter()
            .cloned()
            .collect()
    }
    
    fn get_errors_by_severity(&self, severity: ErrorSeverity) -> Vec<ApplicationError> {
        self.legacy_manager.borrow()
            .get_errors_by_severity(severity)
            .into_iter()
            .cloned()
            .collect()
    }
    
    fn subscribe_to_errors(&mut self, callback: ErrorCallback) -> Result<SubscriptionId, ServiceError> {
        let subscription_id = self.generate_subscription_id();
        self.subscribers.insert(subscription_id.clone(), callback);
        Ok(subscription_id)
    }
    
    fn unsubscribe_from_errors(&mut self, subscription_id: &SubscriptionId) -> Result<(), ServiceError> {
        match self.subscribers.remove(subscription_id) {
            Some(_) => Ok(()),
            None => Err(ServiceError::SubscriptionFailed(
                format!("Subscription {} not found", subscription_id.0)
            )),
        }
    }
    
    fn clear_errors(&mut self) -> Result<(), ServiceError> {
        // Clear all errors from legacy manager
        let all_errors = self.legacy_manager.borrow().get_all_errors();
        let error_ids: Vec<String> = all_errors.iter().map(|e| e.id.clone()).collect();
        
        for error_id in error_ids {
            self.legacy_manager.borrow_mut().remove_error(&error_id);
        }
        
        Ok(())
    }
    
    fn clear_error(&mut self, error_id: &str) -> Result<(), ServiceError> {
        self.legacy_manager.borrow_mut().remove_error(error_id);
        Ok(())
    }
    
    fn can_app_continue(&self) -> bool {
        self.legacy_manager.borrow().can_app_continue()
    }
    
    fn get_fallback_message(&self) -> Option<String> {
        self.legacy_manager.borrow().get_fallback_message()
    }
    
    fn report_recovery(&mut self, error_id: &str, recovery_action: &str, success: bool) -> Result<(), ServiceError> {
        // Publish recovery event to event bus
        self.publish_recovery_event(error_id, recovery_action, success);
        
        // If recovery was successful, remove the error
        if success {
            self.clear_error(error_id)?;
        }
        
        Ok(())
    }
    
    fn set_event_bus(&mut self, event_bus: Arc<dyn EventBus>) -> Result<(), ServiceError> {
        self.event_bus = Some(event_bus);
        Ok(())
    }
    
    fn has_event_bus(&self) -> bool {
        self.event_bus.is_some()
    }
}

/// Factory for creating modular error service instances
pub struct ModularErrorServiceFactory;

impl ModularErrorServiceFactory {
    pub fn new() -> Self {
        Self
    }
}

impl super::error_service::ErrorServiceFactory for ModularErrorServiceFactory {
    fn create_error_service(&self) -> Box<dyn ErrorService> {
        Box::new(ModularErrorService::new())
    }
    
    fn create_error_service_with_event_bus(&self, event_bus: Arc<dyn EventBus>) -> Result<Box<dyn ErrorService>, ServiceError> {
        let mut service = ModularErrorService::new();
        service.set_event_bus(event_bus)?;
        Ok(Box::new(service))
    }
}

impl Default for ModularErrorServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}