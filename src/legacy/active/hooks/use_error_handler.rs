use yew::prelude::*;
use crate::legacy::active::services::error_manager::{ErrorManager, ApplicationError, ErrorCategory, ErrorSeverity};
use crate::legacy::active::services::browser_compat::BrowserInfo;

pub struct ErrorHandler {
    manager: UseStateHandle<ErrorManager>,
}

impl ErrorHandler {
    pub fn add_error(&self, error: ApplicationError) {
        let mut manager = (*self.manager).clone();
        manager.add_error(error);
        self.manager.set(manager);
    }
    
    pub fn remove_error(&self, error_id: &str) {
        let mut manager = (*self.manager).clone();
        manager.remove_error(error_id);
        self.manager.set(manager);
    }
    
    pub fn get_all_errors(&self) -> Vec<ApplicationError> {
        self.manager.get_all_errors().into_iter().cloned().collect()
    }
    
    pub fn get_critical_errors(&self) -> Vec<ApplicationError> {
        self.manager.get_critical_errors().into_iter().cloned().collect()
    }
    
    pub fn get_warnings(&self) -> Vec<ApplicationError> {
        self.manager.get_warnings().into_iter().cloned().collect()
    }
    
    pub fn can_app_continue(&self) -> bool {
        self.manager.can_app_continue()
    }
    
    pub fn get_fallback_message(&self) -> Option<String> {
        self.manager.get_fallback_message()
    }
    
    pub fn handle_permission_error(&self, dom_exception: &str, details: &str) {
        let mut manager = (*self.manager).clone();
        manager.handle_permission_error(dom_exception, details);
        self.manager.set(manager);
    }
    
    pub fn handle_audio_context_error(&self, error_details: &str) {
        let mut manager = (*self.manager).clone();
        manager.handle_audio_context_error(error_details);
        self.manager.set(manager);
    }
    
    pub fn handle_wasm_loading_error(&self, error_details: &str) {
        let mut manager = (*self.manager).clone();
        manager.handle_wasm_loading_error(error_details);
        self.manager.set(manager);
    }
    
    pub fn retry_error(&self, error_id: &str) {
        if let Some(error) = self.manager.get_error(error_id) {
            if error.can_retry() {
                // For now, just log the retry attempt
                // TODO: Implement proper retry logic with timeout handling
                gloo::console::log!(&format!("Retrying error {}", error_id));
                
                // Remove the error to indicate it's being retried
                self.remove_error(error_id);
            }
        }
    }
    
    pub fn initialize_browser_compatibility(&self, browser_info: BrowserInfo) {
        let mut manager = (*self.manager).clone();
        manager.initialize(browser_info);
        self.manager.set(manager);
    }
    
    pub fn clear_expired_errors(&self) {
        let mut manager = (*self.manager).clone();
        // This would typically call a cleanup method
        // For now, we'll filter out old errors manually
        let current_errors = manager.get_all_errors();
        let expired_ids: Vec<String> = current_errors
            .iter()
            .filter(|e| e.is_expired(300)) // 5 minutes
            .map(|e| e.id.clone())
            .collect();
        
        for id in expired_ids {
            manager.remove_error(&id);
        }
        
        self.manager.set(manager);
    }
}

impl Clone for ErrorHandler {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
        }
    }
}

#[hook]
pub fn use_error_handler() -> ErrorHandler {
    let manager = use_state(|| ErrorManager::new());
    
    ErrorHandler {
        manager,
    }
}

// Additional hook for simplified error handling with automatic UI integration
#[derive(Clone)]
pub struct SimpleErrorHandler {
    error_handler: ErrorHandler,
    show_toasts: UseStateHandle<bool>,
    toast_position: UseStateHandle<String>,
}

impl SimpleErrorHandler {
    pub fn report_error(&self, category: ErrorCategory, message: &str, details: Option<&str>) {
        let error = ApplicationError::new(
            category,
            ErrorSeverity::Warning,
            message.to_string(),
            details.map(|d| d.to_string()),
            crate::legacy::active::services::error_manager::RecoveryStrategy::UserGuidedRetry {
                instructions: "Please try again or refresh the page if the problem persists.".to_string(),
            },
        );
        self.error_handler.add_error(error);
    }
    
    pub fn report_critical_error(&self, category: ErrorCategory, message: &str, details: Option<&str>) {
        let error = ApplicationError::new(
            category,
            ErrorSeverity::Critical,
            message.to_string(),
            details.map(|d| d.to_string()),
            crate::legacy::active::services::error_manager::RecoveryStrategy::ApplicationReset {
                reset_message: "Please refresh the page to continue.".to_string(),
            },
        );
        self.error_handler.add_error(error);
    }
    
    pub fn dismiss_error(&self, error_id: &str) {
        self.error_handler.remove_error(error_id);
    }
    
    pub fn retry_error(&self, error_id: &str) {
        self.error_handler.retry_error(error_id);
    }
    
    pub fn get_errors_for_toast(&self) -> Vec<ApplicationError> {
        if *self.show_toasts {
            self.error_handler.get_warnings()
        } else {
            Vec::new()
        }
    }
    
    pub fn get_critical_errors(&self) -> Vec<ApplicationError> {
        self.error_handler.get_critical_errors()
    }
    
    pub fn can_app_continue(&self) -> bool {
        self.error_handler.can_app_continue()
    }
    
    pub fn get_fallback_message(&self) -> Option<String> {
        self.error_handler.get_fallback_message()
    }
    
    pub fn set_toast_enabled(&self, enabled: bool) {
        self.show_toasts.set(enabled);
    }
    
    pub fn set_toast_position(&self, position: &str) {
        self.toast_position.set(position.to_string());
    }
    
    pub fn get_toast_position(&self) -> String {
        (*self.toast_position).clone()
    }
}

#[hook]
pub fn use_simple_error_handler() -> SimpleErrorHandler {
    let error_handler = use_error_handler();
    let show_toasts = use_state(|| true);
    let toast_position = use_state(|| "top-right".to_string());
    
    SimpleErrorHandler {
        error_handler,
        show_toasts,
        toast_position,
    }
} 