use crate::modules::platform_abstraction::browser_compat::{BrowserInfo, CompatibilityLevel};
use super::{ApplicationError, ErrorCategory, ErrorSeverity, RecoveryStrategy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use web_sys::{console, window};
use yew::prelude::*;

// WASM-compatible timestamp function
fn get_timestamp() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        // Use JavaScript Date.now() for WASM builds
        use js_sys::Date;
        (Date::now() / 1000.0) as u64
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use SystemTime for native builds
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl ApplicationError {
    pub fn new(
        category: ErrorCategory,
        severity: ErrorSeverity,
        message: String,
        details: Option<String>,
        recovery_strategy: RecoveryStrategy,
    ) -> Self {
        let timestamp = get_timestamp();
        
        let user_agent = window()
            .and_then(|w| w.navigator().user_agent().ok())
            .unwrap_or_else(|| "Unknown".to_string());

        Self {
            id: format!("{:?}_{}", category, timestamp),
            category,
            severity: severity.clone(),
            message,
            details,
            recommendations: Vec::new(),
            recovery_strategy,
            timestamp,
            user_agent,
            can_continue: !matches!(severity, ErrorSeverity::Critical),
            retry_count: 0,
            max_retries: 3,
        }
    }
    
    pub fn with_recommendations(mut self, recommendations: Vec<String>) -> Self {
        self.recommendations = recommendations;
        self
    }
    
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
    
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
    
    pub fn is_expired(&self, max_age_seconds: u64) -> bool {
        let now = get_timestamp();
        now - self.timestamp > max_age_seconds
    }
}

// Error factory methods for common error scenarios
impl ApplicationError {
    pub fn browser_unsupported(browser_name: &str, current_version: &str, required_version: &str) -> Self {
        Self::new(
            ErrorCategory::BrowserCompatibility,
            ErrorSeverity::Critical,
            format!("Browser {} version {} is not supported", browser_name, current_version),
            Some(format!("This application requires {} version {} or later", browser_name, required_version)),
            RecoveryStrategy::ApplicationReset {
                reset_message: "Please upgrade your browser and refresh this page".to_string(),
            },
        ).with_recommendations(vec![
            format!("Upgrade to {} {} or later", browser_name, required_version),
            "Refresh the page after upgrading".to_string(),
        ])
    }
    
    pub fn microphone_permission_denied(error_details: &str) -> Self {
        Self::new(
            ErrorCategory::MicrophonePermission,
            ErrorSeverity::Warning,
            "Microphone access denied".to_string(),
            Some(error_details.to_string()),
            RecoveryStrategy::UserGuidedRetry {
                instructions: "Click the microphone icon in your browser's address bar and allow access".to_string(),
            },
        ).with_recommendations(vec![
            "Check browser permissions for microphone access".to_string(),
            "Try refreshing the page and allowing permissions".to_string(),
            "Ensure no other applications are using your microphone".to_string(),
        ])
    }
    
    pub fn audio_context_creation_failed(error_details: &str) -> Self {
        Self::new(
            ErrorCategory::AudioContextCreation,
            ErrorSeverity::Critical,
            "Failed to create audio context".to_string(),
            Some(error_details.to_string()),
            RecoveryStrategy::AutomaticRetry {
                max_attempts: 3,
                delay_ms: 1000,
            },
        ).with_recommendations(vec![
            "Ensure your browser supports Web Audio API".to_string(),
            "Try refreshing the page".to_string(),
            "Check if other audio applications are running".to_string(),
        ])
    }
    
    pub fn wasm_loading_failed(error_details: &str) -> Self {
        Self::new(
            ErrorCategory::WasmLoading,
            ErrorSeverity::Critical,
            "Failed to load WebAssembly module".to_string(),
            Some(error_details.to_string()),
            RecoveryStrategy::ApplicationReset {
                reset_message: "Please refresh the page to reload the application".to_string(),
            },
        ).with_recommendations(vec![
            "Refresh the page".to_string(),
            "Check your internet connection".to_string(),
            "Ensure WebAssembly is supported in your browser".to_string(),
        ])
    }
    
    pub fn component_render_error(component_name: &str, error_details: &str) -> Self {
        Self::new(
            ErrorCategory::ComponentRender,
            ErrorSeverity::Warning,
            format!("Component '{}' failed to render", component_name),
            Some(error_details.to_string()),
            RecoveryStrategy::GracefulDegradation {
                fallback_description: "Using fallback UI components".to_string(),
            },
        ).with_recommendations(vec![
            "Try refreshing the page".to_string(),
            "Check browser console for additional errors".to_string(),
        ])
    }
    
    pub fn microphone_device_disconnected(error_details: &str) -> Self {
        Self::new(
            ErrorCategory::DeviceAccess,
            ErrorSeverity::Warning,
            "Microphone device disconnected".to_string(),
            Some(error_details.to_string()),
            RecoveryStrategy::UserGuidedRetry {
                instructions: "Reconnect your microphone device and click to resume".to_string(),
            },
        ).with_recommendations(vec![
            "Check microphone connection".to_string(),
            "Try a different USB port".to_string(),
            "Reconnect the device and try again".to_string(),
        ])
    }
}

impl From<JsValue> for ApplicationError {
    fn from(js_value: JsValue) -> Self {
        let error_message = if let Some(error_str) = js_value.as_string() {
            error_str
        } else {
            "Unknown JavaScript error".to_string()
        };

        Self::new(
            ErrorCategory::Unknown,
            ErrorSeverity::Warning,
            error_message,
            None,
            RecoveryStrategy::UserGuidedRetry {
                instructions: "Try refreshing the page".to_string(),
            },
        )
    }
}

pub struct ErrorManager {
    errors: HashMap<String, ApplicationError>,
    browser_info: Option<BrowserInfo>,
    error_tracking_enabled: bool,
    max_errors: usize,
    error_expiry_seconds: u64,
}

impl Default for ErrorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            browser_info: None,
            error_tracking_enabled: true,
            max_errors: 100,
            error_expiry_seconds: 3600, // 1 hour
        }
    }

    pub fn initialize(&mut self, browser_info: BrowserInfo) {
        self.browser_info = Some(browser_info.clone());
        self.analyze_browser_compatibility(&browser_info);
    }

    pub fn add_error(&mut self, error: ApplicationError) {
        if !self.error_tracking_enabled {
            return;
        }

        // Log to console
        self.log_error_to_console(&error);

        // Store error
        self.errors.insert(error.id.clone(), error);

        // Clean up old errors if we've exceeded the limit
        if self.errors.len() > self.max_errors {
            // Remove oldest errors (simple approach - in production might want LRU)
            let mut errors_vec: Vec<_> = self.errors.iter().collect();
            errors_vec.sort_by_key(|(_, error)| error.timestamp);
            
            let to_remove = self.errors.len() - self.max_errors;
            for (id, _) in errors_vec.iter().take(to_remove) {
                self.errors.remove(*id);
            }
        }

        // Clean expired errors
        self.clean_expired_errors();
    }

    pub fn remove_error(&mut self, error_id: &str) {
        self.errors.remove(error_id);
    }

    pub fn get_error(&self, error_id: &str) -> Option<&ApplicationError> {
        self.errors.get(error_id)
    }

    pub fn get_all_errors(&self) -> Vec<&ApplicationError> {
        self.errors.values().collect()
    }

    pub fn get_errors_by_category(&self, category: ErrorCategory) -> Vec<&ApplicationError> {
        self.errors
            .values()
            .filter(|error| error.category == category)
            .collect()
    }

    pub fn get_errors_by_severity(&self, severity: ErrorSeverity) -> Vec<&ApplicationError> {
        self.errors
            .values()
            .filter(|error| error.severity == severity)
            .collect()
    }

    pub fn get_critical_errors(&self) -> Vec<&ApplicationError> {
        self.get_errors_by_severity(ErrorSeverity::Critical)
    }

    pub fn get_warnings(&self) -> Vec<&ApplicationError> {
        self.get_errors_by_severity(ErrorSeverity::Warning)
    }

    pub fn can_app_continue(&self) -> bool {
        !self.get_critical_errors().is_empty()
    }

    pub fn get_fallback_message(&self) -> Option<String> {
        let critical_errors = self.get_critical_errors();
        if critical_errors.is_empty() {
            return None;
        }

        // Return message from the most recent critical error
        critical_errors
            .iter()
            .max_by_key(|error| error.timestamp)
            .map(|error| match &error.recovery_strategy {
                RecoveryStrategy::GracefulDegradation { fallback_description } => {
                    fallback_description.clone()
                }
                RecoveryStrategy::ApplicationReset { reset_message } => {
                    reset_message.clone()
                }
                _ => "Application encountered a critical error".to_string(),
            })
    }

    fn analyze_browser_compatibility(&mut self, browser_info: &BrowserInfo) {
        match browser_info.compatibility_level {
            CompatibilityLevel::FullySupported => {
                self.add_info_browser_messages(browser_info);
            }
            CompatibilityLevel::PartiallySupported => {
                self.add_warning_browser_errors(browser_info);
            }
            CompatibilityLevel::NotSupported => {
                self.add_critical_browser_errors(browser_info);
            }
        }
    }

    fn add_critical_browser_errors(&mut self, browser_info: &BrowserInfo) {
        let error = ApplicationError::browser_unsupported(
            &browser_info.browser_name,
            &self.format_browser_version(browser_info),
            &self.get_required_browser_version(&browser_info.browser_name),
        );
        self.add_error(error);

        // Add specific feature errors
        if !browser_info.features.web_audio_api {
            let error = ApplicationError::new(
                ErrorCategory::WebAudioSupport,
                ErrorSeverity::Critical,
                "Web Audio API not supported".to_string(),
                Some("This application requires Web Audio API for audio processing".to_string()),
                RecoveryStrategy::ApplicationReset {
                    reset_message: "Please upgrade your browser to use this application".to_string(),
                },
            );
            self.add_error(error);
        }

        if !browser_info.features.web_assembly {
            let error = ApplicationError::new(
                ErrorCategory::WebAssemblySupport,
                ErrorSeverity::Critical,
                "WebAssembly not supported".to_string(),
                Some("This application requires WebAssembly for audio processing".to_string()),
                RecoveryStrategy::ApplicationReset {
                    reset_message: "Please upgrade your browser to use this application".to_string(),
                },
            );
            self.add_error(error);
        }
    }

    fn add_warning_browser_errors(&mut self, browser_info: &BrowserInfo) {
        let warnings = vec![
            "Your browser version has limited support for some features",
            "Consider upgrading for the best experience",
        ];

        for warning in warnings {
            let error = ApplicationError::new(
                ErrorCategory::BrowserCompatibility,
                ErrorSeverity::Warning,
                warning.to_string(),
                Some(format!("Browser: {} {}", browser_info.browser_name, self.format_browser_version(browser_info))),
                RecoveryStrategy::GracefulDegradation {
                    fallback_description: "Using compatibility mode".to_string(),
                },
            ).with_recommendations(self.get_browser_upgrade_recommendations());
            
            self.add_error(error);
        }
    }

    fn add_info_browser_messages(&mut self, browser_info: &BrowserInfo) {
        let message = format!(
            "Browser {} {} fully supported",
            browser_info.browser_name,
            self.format_browser_version(browser_info)
        );

        let error = ApplicationError::new(
            ErrorCategory::BrowserCompatibility,
            ErrorSeverity::Info,
            message,
            None,
            RecoveryStrategy::None,
        );
        
        self.add_error(error);
    }

    fn is_browser_version_supported(&self, browser_info: &BrowserInfo) -> bool {
        match browser_info.browser_name.as_str() {
            "Chrome" => browser_info.major_version >= 80,
            "Firefox" => browser_info.major_version >= 76,
            "Safari" => browser_info.major_version >= 14,
            "Edge" => browser_info.major_version >= 80,
            _ => false,
        }
    }

    fn format_browser_version(&self, browser_info: &BrowserInfo) -> String {
        format!("{}.{}", browser_info.major_version, browser_info.minor_version)
    }

    fn get_required_browser_version(&self, browser_name: &str) -> String {
        match browser_name {
            "Chrome" => "80.0".to_string(),
            "Firefox" => "76.0".to_string(),
            "Safari" => "14.0".to_string(),
            "Edge" => "80.0".to_string(),
            _ => "Latest".to_string(),
        }
    }

    fn get_browser_upgrade_recommendations(&self) -> Vec<String> {
        vec![
            "Update your browser to the latest version".to_string(),
            "Enable JavaScript if disabled".to_string(),
            "Clear browser cache and cookies".to_string(),
        ]
    }

    fn clean_expired_errors(&mut self) {
        let current_time = get_timestamp();
        let expired_ids: Vec<String> = self
            .errors
            .iter()
            .filter(|(_, error)| error.is_expired(self.error_expiry_seconds))
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired_ids {
            self.errors.remove(&id);
        }
    }

    fn log_error_to_console(&self, error: &ApplicationError) {
        match error.severity {
            ErrorSeverity::Critical => {
                web_sys::console::error_1(&format!("CRITICAL: {}", error.message).into());
                if let Some(details) = &error.details {
                    web_sys::console::error_1(&format!("Details: {}", details).into());
                }
            }
            ErrorSeverity::Warning => {
                web_sys::console::warn_1(&format!("WARNING: {}", error.message).into());
                if let Some(details) = &error.details {
                    web_sys::console::warn_1(&format!("Details: {}", details).into());
                }
            }
            ErrorSeverity::Info => {
                web_sys::console::log_1(&format!("INFO: {}", error.message).into());
            }
        }

        // Log recommendations
        if !error.recommendations.is_empty() {
            web_sys::console::log_1(&"Recommendations:".into());
            for rec in &error.recommendations {
                web_sys::console::log_1(&format!("  - {}", rec).into());
            }
        }
    }

    pub fn enable_error_tracking(&mut self, enabled: bool) {
        self.error_tracking_enabled = enabled;
    }

    pub fn set_max_errors(&mut self, max_errors: usize) {
        self.max_errors = max_errors;
    }

    pub fn set_error_expiry(&mut self, expiry_seconds: u64) {
        self.error_expiry_seconds = expiry_seconds;
    }
}

impl ErrorManager {
    pub fn handle_permission_error(&mut self, dom_exception: &str, details: &str) {
        let error = match dom_exception {
            "NotAllowedError" => ApplicationError::microphone_permission_denied(details),
            "NotFoundError" => ApplicationError::new(
                ErrorCategory::DeviceAccess,
                ErrorSeverity::Warning,
                "No microphone device found".to_string(),
                Some(details.to_string()),
                RecoveryStrategy::UserGuidedRetry {
                    instructions: "Connect a microphone device and try again".to_string(),
                },
            ),
            "NotSupportedError" => ApplicationError::new(
                ErrorCategory::MediaDevicesSupport,
                ErrorSeverity::Critical,
                "getUserMedia not supported".to_string(),
                Some(details.to_string()),
                RecoveryStrategy::ApplicationReset {
                    reset_message: "Please use a supported browser".to_string(),
                },
            ),
            _ => ApplicationError::microphone_permission_denied(&format!("{}: {}", dom_exception, details)),
        };
        
        self.add_error(error);
    }

    pub fn handle_audio_context_error(&mut self, error_details: &str) {
        let error = ApplicationError::audio_context_creation_failed(error_details);
        self.add_error(error);
    }

    pub fn handle_wasm_loading_error(&mut self, error_details: &str) {
        let error = ApplicationError::wasm_loading_failed(error_details);
        self.add_error(error);
    }
} 