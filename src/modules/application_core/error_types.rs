//! Pure Modular Error Types
//!
//! This module provides clean, modular error types without any legacy dependencies.
//! These types replace the legacy ApplicationError and ErrorSeverity with pure
//! implementations designed for the modular architecture.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::console;

/// WASM-compatible timestamp function
fn get_timestamp() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Date;
        (Date::now() / 1000.0) as u64
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Comprehensive error categorization for modular architecture
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

/// Error severity levels for modular architecture
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,    // App cannot function
    Warning,     // App can function with degraded experience
    Info,        // Informational only
}

/// Recovery strategy for handling errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    AutomaticRetry { max_attempts: u32, delay_ms: u64 },
    UserGuidedRetry { instructions: String },
    GracefulDegradation { fallback_description: String },
    ErrorEscalation { escalation_message: String },
    ApplicationReset { reset_message: String },
    None,
}

/// Pure modular application error type
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
    /// Create a new application error
    pub fn new(
        category: ErrorCategory,
        severity: ErrorSeverity,
        message: String,
        details: Option<String>,
        recovery_strategy: RecoveryStrategy,
    ) -> Self {
        let timestamp = get_timestamp();
        
        let user_agent = web_sys::window()
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
    
    /// Add recommendations to the error
    pub fn with_recommendations(mut self, recommendations: Vec<String>) -> Self {
        self.recommendations = recommendations;
        self
    }
    
    /// Set maximum retry attempts
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
    
    /// Check if error can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
    
    /// Check if error has expired
    pub fn is_expired(&self, max_age_seconds: u64) -> bool {
        let now = get_timestamp();
        now - self.timestamp > max_age_seconds
    }
}

/// Factory methods for common error scenarios
impl ApplicationError {
    /// Create browser unsupported error
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
    
    /// Create microphone permission denied error
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
    
    /// Create audio context creation failed error
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
    
    /// Create WASM loading failed error
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
    
    /// Create component render error
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

    /// Create microphone device disconnected error
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

/// Convert from JsValue to ApplicationError
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