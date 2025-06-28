//! Legacy Error Manager Bridge
//!
//! Provides backward compatibility by implementing the legacy ErrorManager
//! interface while using the new modular ErrorService underneath.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::rc::Rc;

use super::error_service::{ErrorService, ErrorCallback, SubscriptionId};
use crate::legacy::active::services::error_manager::{
    ErrorManager, ApplicationError, ErrorCategory, ErrorSeverity
};
use crate::legacy::active::services::browser_compat::{BrowserInfo, CompatibilityLevel};

/// Legacy compatibility bridge that wraps modular ErrorService
/// 
/// This bridge allows legacy components to continue using the familiar
/// ErrorManager interface while actually using the new modular
/// implementation underneath.
pub struct LegacyErrorBridge {
    modular_service: Arc<Mutex<dyn ErrorService>>,
    browser_info: Option<BrowserInfo>,
    error_tracking_enabled: bool,
    max_errors: usize,
    error_expiry_seconds: u64,
    subscriptions: HashMap<String, SubscriptionId>,
}

impl LegacyErrorBridge {
    /// Create new legacy bridge with modular service
    pub fn new(modular_service: Arc<Mutex<dyn ErrorService>>) -> Self {
        Self {
            modular_service,
            browser_info: None,
            error_tracking_enabled: true,
            max_errors: 50,
            error_expiry_seconds: 300,
            subscriptions: HashMap::new(),
        }
    }
    
    /// Initialize with browser information (legacy interface)
    pub fn initialize(&mut self, browser_info: BrowserInfo) {
        self.browser_info = Some(browser_info.clone());
        self.analyze_browser_compatibility(&browser_info);
    }
    
    /// Add error to the service (legacy interface)
    pub fn add_error(&mut self, error: ApplicationError) {
        if let Ok(mut service) = self.modular_service.lock() {
            let _ = service.report_error(error, Some("legacy_bridge"));
        }
    }
    
    /// Remove error by ID (legacy interface)
    pub fn remove_error(&mut self, error_id: &str) {
        if let Ok(mut service) = self.modular_service.lock() {
            let _ = service.clear_error(error_id);
        }
    }
    
    /// Get error by ID (legacy interface)
    pub fn get_error(&self, error_id: &str) -> Option<ApplicationError> {
        if let Ok(service) = self.modular_service.lock() {
            // Search through recent errors for matching ID
            service.get_recent_errors(self.max_errors)
                .into_iter()
                .find(|e| e.id == error_id)
        } else {
            None
        }
    }
    
    /// Get all errors (legacy interface)
    pub fn get_all_errors(&self) -> Vec<ApplicationError> {
        if let Ok(service) = self.modular_service.lock() {
            service.get_recent_errors(self.max_errors)
        } else {
            Vec::new()
        }
    }
    
    /// Get errors by category (legacy interface)
    pub fn get_errors_by_category(&self, category: ErrorCategory) -> Vec<ApplicationError> {
        if let Ok(service) = self.modular_service.lock() {
            service.get_errors_by_category(category)
        } else {
            Vec::new()
        }
    }
    
    /// Get errors by severity (legacy interface)
    pub fn get_errors_by_severity(&self, severity: ErrorSeverity) -> Vec<ApplicationError> {
        if let Ok(service) = self.modular_service.lock() {
            service.get_errors_by_severity(severity)
        } else {
            Vec::new()
        }
    }
    
    /// Get critical errors (legacy interface)
    pub fn get_critical_errors(&self) -> Vec<ApplicationError> {
        self.get_errors_by_severity(ErrorSeverity::Critical)
    }
    
    /// Get warnings (legacy interface)
    pub fn get_warnings(&self) -> Vec<ApplicationError> {
        self.get_errors_by_severity(ErrorSeverity::Warning)
    }
    
    /// Check if app can continue (legacy interface)
    pub fn can_app_continue(&self) -> bool {
        if let Ok(service) = self.modular_service.lock() {
            service.can_app_continue()
        } else {
            false
        }
    }
    
    /// Get fallback message (legacy interface)
    pub fn get_fallback_message(&self) -> Option<String> {
        if let Ok(service) = self.modular_service.lock() {
            service.get_fallback_message()
        } else {
            Some("Error service unavailable".to_string())
        }
    }
    
    /// Handle permission error (legacy interface)
    pub fn handle_permission_error(&mut self, dom_exception: &str, details: &str) {
        let error = match dom_exception {
            "NotAllowedError" => ApplicationError::microphone_permission_denied(details),
            "NotFoundError" => ApplicationError::new(
                ErrorCategory::DeviceAccess,
                ErrorSeverity::Warning,
                "No microphone found".to_string(),
                Some(details.to_string()),
                crate::legacy::active::services::error_manager::RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please connect a microphone and try again".to_string(),
                },
            ),
            "NotReadableError" => ApplicationError::new(
                ErrorCategory::DeviceAccess,
                ErrorSeverity::Warning,
                "Microphone is being used by another application".to_string(),
                Some(details.to_string()),
                crate::legacy::active::services::error_manager::RecoveryStrategy::UserGuidedRetry {
                    instructions: "Close other applications using your microphone and try again".to_string(),
                },
            ),
            _ => ApplicationError::new(
                ErrorCategory::MicrophonePermission,
                ErrorSeverity::Warning,
                format!("Microphone access error: {}", dom_exception),
                Some(details.to_string()),
                crate::legacy::active::services::error_manager::RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please check your microphone permissions and try again".to_string(),
                },
            ),
        };
        
        self.add_error(error);
    }
    
    /// Handle audio context error (legacy interface)
    pub fn handle_audio_context_error(&mut self, error_details: &str) {
        let error = ApplicationError::audio_context_creation_failed(error_details);
        self.add_error(error);
    }
    
    /// Handle WASM loading error (legacy interface)
    pub fn handle_wasm_loading_error(&mut self, error_details: &str) {
        let error = ApplicationError::wasm_loading_failed(error_details);
        self.add_error(error);
    }
    
    /// Enable/disable error tracking (legacy interface)
    pub fn enable_error_tracking(&mut self, enabled: bool) {
        self.error_tracking_enabled = enabled;
    }
    
    /// Set maximum number of errors (legacy interface)
    pub fn set_max_errors(&mut self, max_errors: usize) {
        self.max_errors = max_errors;
    }
    
    /// Set error expiry time (legacy interface)
    pub fn set_error_expiry(&mut self, expiry_seconds: u64) {
        self.error_expiry_seconds = expiry_seconds;
    }
    
    /// Analyze browser compatibility and add appropriate errors
    fn analyze_browser_compatibility(&mut self, browser_info: &BrowserInfo) {
        match browser_info.compatibility_level {
            CompatibilityLevel::Unsupported => {
                self.add_critical_browser_errors(browser_info);
            }
            CompatibilityLevel::PartiallySupported => {
                self.add_warning_browser_errors(browser_info);
            }
            CompatibilityLevel::MostlySupported => {
                self.add_info_browser_messages(browser_info);
            }
            CompatibilityLevel::FullySupported => {
                // No errors needed
            }
        }
    }
    
    /// Add critical browser errors
    fn add_critical_browser_errors(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_wasm {
            let error = ApplicationError::new(
                ErrorCategory::WebAssemblySupport,
                ErrorSeverity::Critical,
                "WebAssembly is not supported".to_string(),
                Some("This application requires WebAssembly to function properly.".to_string()),
                crate::legacy::active::services::error_manager::RecoveryStrategy::ApplicationReset {
                    reset_message: "Please upgrade your browser to continue".to_string(),
                },
            ).with_recommendations(self.get_browser_upgrade_recommendations());
            self.add_error(error);
        }
        
        if !browser_info.capabilities.supports_audio_context {
            let error = ApplicationError::new(
                ErrorCategory::WebAudioSupport,
                ErrorSeverity::Critical,
                "Web Audio API is not supported".to_string(),
                Some("This application requires Web Audio API for pitch detection.".to_string()),
                crate::legacy::active::services::error_manager::RecoveryStrategy::ApplicationReset {
                    reset_message: "Please upgrade your browser to continue".to_string(),
                },
            ).with_recommendations(self.get_browser_upgrade_recommendations());
            self.add_error(error);
        }
    }
    
    /// Add warning browser errors
    fn add_warning_browser_errors(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_audio_worklet {
            let error = ApplicationError::new(
                ErrorCategory::AudioWorkletLoading,
                ErrorSeverity::Warning,
                "AudioWorklet not supported - using fallback audio processing".to_string(),
                Some("Audio latency may be higher than optimal.".to_string()),
                crate::legacy::active::services::error_manager::RecoveryStrategy::GracefulDegradation {
                    fallback_description: "Using ScriptProcessorNode for audio processing".to_string(),
                },
            ).with_recommendations(vec![
                "Consider upgrading your browser for better performance".to_string(),
                "Use headphones for better audio quality".to_string(),
            ]);
            self.add_error(error);
        }
    }
    
    /// Add info browser messages
    fn add_info_browser_messages(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_shared_array_buffer {
            let error = ApplicationError::new(
                ErrorCategory::BrowserCompatibility,
                ErrorSeverity::Info,
                "SharedArrayBuffer not available".to_string(),
                Some("Advanced multithreading features are disabled.".to_string()),
                crate::legacy::active::services::error_manager::RecoveryStrategy::None,
            ).with_recommendations(vec![
                "This is normal for most browsers due to security requirements".to_string(),
            ]);
            self.add_error(error);
        }
    }
    
    /// Get browser upgrade recommendations
    fn get_browser_upgrade_recommendations(&self) -> Vec<String> {
        vec![
            "Upgrade to Chrome 69+: https://www.google.com/chrome/".to_string(),
            "Upgrade to Firefox 76+: https://www.mozilla.org/firefox/".to_string(),
            "Upgrade to Safari 14.1+ (macOS only)".to_string(),
            "Upgrade to Edge 79+: https://www.microsoft.com/edge/".to_string(),
        ]
    }
}

impl Clone for LegacyErrorBridge {
    fn clone(&self) -> Self {
        Self {
            modular_service: self.modular_service.clone(),
            browser_info: self.browser_info.clone(),
            error_tracking_enabled: self.error_tracking_enabled,
            max_errors: self.max_errors,
            error_expiry_seconds: self.error_expiry_seconds,
            subscriptions: HashMap::new(), // Don't clone subscriptions
        }
    }
}

impl PartialEq for LegacyErrorBridge {
    fn eq(&self, other: &Self) -> bool {
        // Simple equality check for Yew properties
        self.error_tracking_enabled == other.error_tracking_enabled &&
        self.max_errors == other.max_errors &&
        self.error_expiry_seconds == other.error_expiry_seconds
    }
}

impl std::fmt::Debug for LegacyErrorBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LegacyErrorBridge")
            .field("error_tracking_enabled", &self.error_tracking_enabled)
            .field("max_errors", &self.max_errors)
            .field("error_expiry_seconds", &self.error_expiry_seconds)
            .finish()
    }
}