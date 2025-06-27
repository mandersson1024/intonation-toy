//! # Use Error Handler Hook
//!
//! Placeholder for migrated error handler hook.
//! Will be implemented during hook migration task.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;

// TODO: Update these imports once legacy services are migrated to modules
#[cfg(debug_assertions)]
use crate::legacy::active::services::error_manager::{ApplicationError, ErrorManager, ErrorSeverity};

#[cfg(debug_assertions)]
pub fn use_error_handler(error_manager: Option<Rc<RefCell<ErrorManager>>>) -> Callback<ApplicationError> {
    let error_manager = error_manager.clone();
    
    Callback::from(move |error: ApplicationError| {
        // Log error to console for debugging
        match error.severity {
            ErrorSeverity::Critical => {
                web_sys::console::error_1(&format!("CRITICAL ERROR: {}", error.message).into());
            }
            ErrorSeverity::Warning => {
                web_sys::console::warn_1(&format!("WARNING: {}", error.message).into());
            }
            ErrorSeverity::Info => {
                web_sys::console::log_1(&format!("INFO: {}", error.message).into());
            }
        }
        
        // Add error to error manager if available
        if let Some(ref manager) = error_manager {
            if let Ok(mut manager_ref) = manager.try_borrow_mut() {
                manager_ref.add_error(error.clone());
            } else {
                web_sys::console::warn_1(&"Could not add error to manager (busy)".into());
            }
        }
        
        // Log additional error details for debugging
        if let Some(ref details) = error.details {
            web_sys::console::log_1(&format!("Error details: {}", details).into());
        }
        
        if !error.recommendations.is_empty() {
            web_sys::console::log_1(&"Recommendations:".into());
            for (i, rec) in error.recommendations.iter().enumerate() {
                web_sys::console::log_1(&format!("  {}. {}", i + 1, rec).into());
            }
        }
    })
}

#[cfg(debug_assertions)]
pub struct UseErrorHandler;

#[cfg(debug_assertions)]
impl UseErrorHandler {
    pub fn new() -> Self {
        Self
    }
} 