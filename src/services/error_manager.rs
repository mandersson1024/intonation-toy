use crate::browser_compat::{BrowserInfo, CompatibilityLevel};
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

// Comprehensive error categorization for Yew application
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorCategory {
    // Browser compatibility errors
    BrowserCompatibility,
    WebAssemblySupport,
    WebAudioSupport,
    MediaDevicesSupport,
    
    // Audio processing errors
    AudioContextCreation,
    AudioWorkletLoading,
    PitchDetection,
    
    // Permission errors
    MicrophonePermission,
    DeviceAccess,
    
    // Network and loading errors
    WasmLoading,
    NetworkConnectivity,
    
    // Runtime errors
    MemoryAllocation,
    ProcessingTimeout,
    
    // UI/Component errors
    ComponentRender,
    StateManagement,
    
    // Unknown/generic errors
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,    // App cannot function
    Warning,     // App can function with degraded experience
    Info,        // Informational only
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    AutomaticRetry { max_attempts: u32, delay_ms: u64 },
    UserGuidedRetry { instructions: String },
    GracefulDegradation { fallback_description: String },
    ErrorEscalation { escalation_message: String },
    ApplicationReset { reset_message: String },
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationError {
    pub id: String,
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub message: String,
    pub details: Option<String>,
    pub recommendations: Vec<String>,
    pub recovery_strategy: RecoveryStrategy,
    pub timestamp: u64,
    pub user_agent: String,
    pub can_continue: bool,
    pub retry_count: u32,
    pub max_retries: u32,
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
            RecoveryStrategy::AutomaticRetry {
                max_attempts: 2,
                delay_ms: 2000,
            },
        ).with_recommendations(vec![
            "Check your internet connection".to_string(),
            "Try refreshing the page".to_string(),
            "Ensure your browser supports WebAssembly".to_string(),
        ])
    }
    
    pub fn component_render_error(component_name: &str, error_details: &str) -> Self {
        Self::new(
            ErrorCategory::ComponentRender,
            ErrorSeverity::Warning,
            format!("Component {} failed to render", component_name),
            Some(error_details.to_string()),
            RecoveryStrategy::AutomaticRetry {
                max_attempts: 2,
                delay_ms: 500,
            },
        ).with_recommendations(vec![
            "Try refreshing the page".to_string(),
            "Check browser console for more details".to_string(),
        ])
    }

    /// Create error for microphone device disconnection
    pub fn microphone_device_disconnected(error_details: &str) -> Self {
        Self::new(
            ErrorCategory::DeviceAccess,
            ErrorSeverity::Warning,
            "Microphone device disconnected".to_string(),
            Some(error_details.to_string()),
            RecoveryStrategy::UserGuidedRetry {
                instructions: "Please reconnect your microphone and request access again".to_string(),
            },
        ).with_recommendations(vec![
            "Check that your microphone is properly connected".to_string(),
            "Try unplugging and reconnecting your microphone".to_string(),
            "Click 'Retry Microphone Permission' after reconnecting".to_string(),
        ])
    }
}

impl From<JsValue> for ApplicationError {
    fn from(js_value: JsValue) -> Self {
        let message = js_value
            .as_string()
            .unwrap_or_else(|| format!("{:?}", js_value));
        
        ApplicationError::new(
            ErrorCategory::Unknown,
            ErrorSeverity::Warning,
            "JavaScript error occurred".to_string(),
            Some(message),
            RecoveryStrategy::AutomaticRetry {
                max_attempts: 2,
                delay_ms: 1000,
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
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
            max_errors: 50,
            error_expiry_seconds: 300, // 5 minutes
        }
    }
    
    pub fn initialize(&mut self, browser_info: BrowserInfo) {
        self.browser_info = Some(browser_info.clone());
        self.analyze_browser_compatibility(&browser_info);
        self.clean_expired_errors();
    }
    
    pub fn add_error(&mut self, error: ApplicationError) {
        // Check if we already have this error type
        let existing_key = self.errors.keys()
            .find(|k| k.starts_with(&format!("{:?}_", error.category)))
            .cloned();
        
        if let Some(key) = existing_key {
            if let Some(existing_error) = self.errors.get_mut(&key) {
                existing_error.increment_retry();
                existing_error.timestamp = error.timestamp;
                if self.error_tracking_enabled {
                    // Clone the error to avoid borrow checker issues
                    let error_for_logging = existing_error.clone();
                    self.log_error_to_console(&error_for_logging);
                }
                return;
            }
        }
        
        // Clean up old errors if we're at capacity
        if self.errors.len() >= self.max_errors {
            self.clean_expired_errors();
        }
        
        if self.error_tracking_enabled {
            self.log_error_to_console(&error);
        }
        
        self.errors.insert(error.id.clone(), error);
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
            .filter(|e| e.category == category)
            .collect()
    }
    
    pub fn get_errors_by_severity(&self, severity: ErrorSeverity) -> Vec<&ApplicationError> {
        self.errors
            .values()
            .filter(|e| e.severity == severity)
            .collect()
    }
    
    pub fn get_critical_errors(&self) -> Vec<&ApplicationError> {
        self.get_errors_by_severity(ErrorSeverity::Critical)
    }
    
    pub fn get_warnings(&self) -> Vec<&ApplicationError> {
        self.get_errors_by_severity(ErrorSeverity::Warning)
    }
    
    pub fn can_app_continue(&self) -> bool {
        !self.errors.values().any(|e| !e.can_continue)
    }
    
    pub fn get_fallback_message(&self) -> Option<String> {
        let critical_errors = self.get_critical_errors();
        if critical_errors.is_empty() {
            return None;
        }
        
        let messages: Vec<String> = critical_errors
            .iter()
            .map(|e| format!("• {}", e.message))
            .collect();
        
        Some(format!(
            "This application cannot run due to the following issues:\n\n{}\n\nPlease resolve these issues and refresh the page.",
            messages.join("\n")
        ))
    }
    
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
    
    fn add_critical_browser_errors(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_wasm {
            let error = ApplicationError::new(
                ErrorCategory::WebAssemblySupport,
                ErrorSeverity::Critical,
                "WebAssembly is not supported".to_string(),
                Some("This application requires WebAssembly to function properly.".to_string()),
                RecoveryStrategy::ApplicationReset {
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
                RecoveryStrategy::ApplicationReset {
                    reset_message: "Please upgrade your browser to continue".to_string(),
                },
            ).with_recommendations(self.get_browser_upgrade_recommendations());
            self.add_error(error);
        }
        
        if !self.is_browser_version_supported(browser_info) {
            let current_version = self.format_browser_version(browser_info);
            let required_version = self.get_required_browser_version(&browser_info.browser_name);
            let error = ApplicationError::browser_unsupported(
                &browser_info.browser_name,
                &current_version,
                &required_version,
            );
            self.add_error(error);
        }
    }
    
    fn add_warning_browser_errors(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_audio_worklet {
            let error = ApplicationError::new(
                ErrorCategory::AudioWorkletLoading,
                ErrorSeverity::Warning,
                "AudioWorklet not supported - using fallback audio processing".to_string(),
                Some("Audio latency may be higher than optimal.".to_string()),
                RecoveryStrategy::GracefulDegradation {
                    fallback_description: "Using ScriptProcessorNode for audio processing".to_string(),
                },
            ).with_recommendations(vec![
                "Consider upgrading your browser for better performance".to_string(),
                "Use headphones for better audio quality".to_string(),
            ]);
            self.add_error(error);
        }
        
        if !browser_info.capabilities.supports_media_devices {
            let error = ApplicationError::new(
                ErrorCategory::MediaDevicesSupport,
                ErrorSeverity::Warning,
                "MediaDevices API not fully supported".to_string(),
                Some("Microphone access may be limited.".to_string()),
                RecoveryStrategy::GracefulDegradation {
                    fallback_description: "Limited microphone functionality available".to_string(),
                },
            ).with_recommendations(vec![
                "Upgrade your browser for full microphone support".to_string(),
                "Ensure microphone permissions are enabled".to_string(),
            ]);
            self.add_error(error);
        }
    }
    
    fn add_info_browser_messages(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_shared_array_buffer {
            let error = ApplicationError::new(
                ErrorCategory::BrowserCompatibility,
                ErrorSeverity::Info,
                "SharedArrayBuffer not available".to_string(),
                Some("Advanced multithreading features are disabled.".to_string()),
                RecoveryStrategy::None,
            ).with_recommendations(vec![
                "This is normal for most browsers due to security requirements".to_string(),
            ]);
            self.add_error(error);
        }
    }
    
    fn is_browser_version_supported(&self, browser_info: &BrowserInfo) -> bool {
        match (browser_info.browser_name.as_str(), &browser_info.browser_version) {
            ("Chrome", Some(v)) => v.major >= 69,
            ("Firefox", Some(v)) => v.major >= 76,
            ("Safari", Some(v)) => v.major >= 14 && (v.major > 14 || v.minor >= 1),
            ("Edge", Some(v)) => v.major >= 79,
            _ => false,
        }
    }
    
    fn format_browser_version(&self, browser_info: &BrowserInfo) -> String {
        if let Some(ref v) = browser_info.browser_version {
            format!("{}.{}.{}", v.major, v.minor, v.patch)
        } else {
            "Unknown".to_string()
        }
    }
    
    fn get_required_browser_version(&self, browser_name: &str) -> String {
        match browser_name {
            "Chrome" => "69.0.0".to_string(),
            "Firefox" => "76.0.0".to_string(),
            "Safari" => "14.1.0".to_string(),
            "Edge" => "79.0.0".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    
    fn get_browser_upgrade_recommendations(&self) -> Vec<String> {
        vec![
            "Upgrade to Chrome 69+: https://www.google.com/chrome/".to_string(),
            "Upgrade to Firefox 76+: https://www.mozilla.org/firefox/".to_string(),
            "Upgrade to Safari 14.1+ (macOS only)".to_string(),
            "Upgrade to Edge 79+: https://www.microsoft.com/edge/".to_string(),
        ]
    }
    
    fn clean_expired_errors(&mut self) {
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
        let log_message = format!(
            "[ErrorManager] {}: {} (Category: {:?}, Severity: {:?})",
            error.id, error.message, error.category, error.severity
        );
        
        match error.severity {
            ErrorSeverity::Critical => console::error_1(&log_message.into()),
            ErrorSeverity::Warning => console::warn_1(&log_message.into()),
            ErrorSeverity::Info => console::log_1(&log_message.into()),
        }
        
        if let Some(ref details) = error.details {
            console::log_1(&format!("  Details: {}", details).into());
        }
        
        if !error.recommendations.is_empty() {
            console::log_1(&format!("  Recommendations: {}", error.recommendations.join(", ")).into());
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

// Integration helpers for existing components
impl ErrorManager {
    pub fn handle_permission_error(&mut self, dom_exception: &str, details: &str) {
        let error = match dom_exception {
            "NotAllowedError" => ApplicationError::microphone_permission_denied(details),
            "NotFoundError" => ApplicationError::new(
                ErrorCategory::DeviceAccess,
                ErrorSeverity::Warning,
                "No microphone found".to_string(),
                Some(details.to_string()),
                RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please connect a microphone and try again".to_string(),
                },
            ),
            "NotReadableError" => ApplicationError::new(
                ErrorCategory::DeviceAccess,
                ErrorSeverity::Warning,
                "Microphone is being used by another application".to_string(),
                Some(details.to_string()),
                RecoveryStrategy::UserGuidedRetry {
                    instructions: "Close other applications using your microphone and try again".to_string(),
                },
            ),
            _ => ApplicationError::new(
                ErrorCategory::MicrophonePermission,
                ErrorSeverity::Warning,
                format!("Microphone access error: {}", dom_exception),
                Some(details.to_string()),
                RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please check your microphone permissions and try again".to_string(),
                },
            ),
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