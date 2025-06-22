use crate::browser_compat::{BrowserInfo, CompatibilityLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{console, window};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    UnsupportedBrowser,
    WebAssemblyNotSupported,
    WebAudioNotSupported,
    MediaDevicesNotSupported,
    FeatureDegradation,
    BrowserUpgradeRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserError {
    pub error_type: ErrorType,
    pub message: String,
    pub details: Option<String>,
    pub recommendations: Vec<String>,
    pub severity: ErrorSeverity,
    pub can_continue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,    // App cannot function
    Warning,     // App can function with degraded experience
    Info,        // Informational only
}

pub struct ErrorManager {
    browser_info: Option<BrowserInfo>,
    errors: Vec<BrowserError>,
    error_tracking_enabled: bool,
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            browser_info: None,
            errors: Vec::new(),
            error_tracking_enabled: true,
        }
    }

    pub fn initialize(&mut self, browser_info: BrowserInfo) {
        self.browser_info = Some(browser_info.clone());
        self.analyze_compatibility(&browser_info);
    }

    fn analyze_compatibility(&mut self, browser_info: &BrowserInfo) {
        self.errors.clear();

        match browser_info.compatibility_level {
            CompatibilityLevel::Unsupported => {
                self.add_critical_errors(browser_info);
            }
            CompatibilityLevel::PartiallySupported => {
                self.add_warning_errors(browser_info);
            }
            CompatibilityLevel::MostlySupported => {
                self.add_info_messages(browser_info);
            }
            CompatibilityLevel::FullySupported => {
                // No errors needed
            }
        }

        if self.error_tracking_enabled {
            self.log_errors_to_console();
        }
    }

    fn add_critical_errors(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_wasm {
            self.errors.push(BrowserError {
                error_type: ErrorType::WebAssemblyNotSupported,
                message: "WebAssembly is not supported by your browser".to_string(),
                details: Some("This application requires WebAssembly to function. Please upgrade your browser or use a different one.".to_string()),
                recommendations: self.get_browser_upgrade_recommendations(),
                severity: ErrorSeverity::Critical,
                can_continue: false,
            });
        }

        if !browser_info.capabilities.supports_audio_context {
            self.errors.push(BrowserError {
                error_type: ErrorType::WebAudioNotSupported,
                message: "Web Audio API is not supported by your browser".to_string(),
                details: Some("This application requires Web Audio API for pitch detection. Please upgrade your browser.".to_string()),
                recommendations: self.get_browser_upgrade_recommendations(),
                severity: ErrorSeverity::Critical,
                can_continue: false,
            });
        }

        if !self.is_version_supported(browser_info) {
            self.errors.push(BrowserError {
                error_type: ErrorType::BrowserUpgradeRequired,
                message: format!("Your {} browser version is not supported", browser_info.browser_name),
                details: Some(self.get_version_requirement_message(browser_info)),
                recommendations: self.get_browser_upgrade_recommendations(),
                severity: ErrorSeverity::Critical,
                can_continue: false,
            });
        }
    }

    fn add_warning_errors(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_audio_worklet {
            self.errors.push(BrowserError {
                error_type: ErrorType::FeatureDegradation,
                message: "AudioWorklet is not supported - audio latency may be higher".to_string(),
                details: Some("The application will use ScriptProcessorNode as a fallback, which may result in higher audio latency.".to_string()),
                recommendations: vec![
                    "Consider upgrading your browser for better audio performance".to_string(),
                    "Use headphones for better audio quality".to_string(),
                ],
                severity: ErrorSeverity::Warning,
                can_continue: true,
            });
        }

        if !browser_info.capabilities.supports_wasm_streaming {
            self.errors.push(BrowserError {
                error_type: ErrorType::FeatureDegradation,
                message: "WebAssembly streaming is not supported - slower loading times expected".to_string(),
                details: Some("The application will load WebAssembly modules without streaming, which may take longer.".to_string()),
                recommendations: vec![
                    "Upgrade your browser for faster loading times".to_string(),
                ],
                severity: ErrorSeverity::Warning,
                can_continue: true,
            });
        }

        if !browser_info.capabilities.supports_media_devices {
            self.errors.push(BrowserError {
                error_type: ErrorType::MediaDevicesNotSupported,
                message: "MediaDevices API is not supported - microphone access may be limited".to_string(),
                details: Some("The application may not be able to access your microphone properly.".to_string()),
                recommendations: vec![
                    "Upgrade your browser for full microphone support".to_string(),
                    "Ensure your browser has microphone permissions enabled".to_string(),
                ],
                severity: ErrorSeverity::Warning,
                can_continue: true,
            });
        }
    }

    fn add_info_messages(&mut self, browser_info: &BrowserInfo) {
        if !browser_info.capabilities.supports_shared_array_buffer {
            self.errors.push(BrowserError {
                error_type: ErrorType::FeatureDegradation,
                message: "SharedArrayBuffer is not available - some advanced features disabled".to_string(),
                details: Some("Multithreaded WebAssembly features are not available.".to_string()),
                recommendations: vec![
                    "This is normal for most browsers due to security requirements".to_string(),
                ],
                severity: ErrorSeverity::Info,
                can_continue: true,
            });
        }
    }

    fn is_version_supported(&self, browser_info: &BrowserInfo) -> bool {
        match (browser_info.browser_name.as_str(), &browser_info.browser_version) {
            ("Chrome", Some(v)) => v.major >= 69,
            ("Firefox", Some(v)) => v.major >= 76,
            ("Safari", Some(v)) => v.major >= 14 && (v.major > 14 || v.minor >= 1),
            ("Edge", Some(v)) => v.major >= 79,
            _ => false,
        }
    }

    fn get_version_requirement_message(&self, browser_info: &BrowserInfo) -> String {
        let current_version = if let Some(ref v) = browser_info.browser_version {
            format!("{}.{}.{}", v.major, v.minor, v.patch)
        } else {
            "Unknown".to_string()
        };

        let required_version = match browser_info.browser_name.as_str() {
            "Chrome" => "69.0.0",
            "Firefox" => "76.0.0",
            "Safari" => "14.1.0",
            "Edge" => "79.0.0",
            _ => "Unknown",
        };

        format!(
            "Current version: {}, Required version: {} or later",
            current_version, required_version
        )
    }

    fn get_browser_upgrade_recommendations(&self) -> Vec<String> {
        vec![
            "Upgrade to Chrome 69 or later: https://www.google.com/chrome/".to_string(),
            "Upgrade to Firefox 76 or later: https://www.mozilla.org/firefox/".to_string(),
            "Upgrade to Safari 14.1 or later (macOS only)".to_string(),
            "Upgrade to Edge 79 or later: https://www.microsoft.com/edge/".to_string(),
        ]
    }

    fn log_errors_to_console(&self) {
        for error in &self.errors {
            let message = format!(
                "[BrowserCompatibility] {}: {}",
                match error.severity {
                    ErrorSeverity::Critical => "ERROR",
                    ErrorSeverity::Warning => "WARN",
                    ErrorSeverity::Info => "INFO",
                },
                error.message
            );

            match error.severity {
                ErrorSeverity::Critical => console::error_1(&message.into()),
                ErrorSeverity::Warning => console::warn_1(&message.into()),
                ErrorSeverity::Info => console::log_1(&message.into()),
            }
        }
    }

    pub fn get_errors(&self) -> &[BrowserError] {
        &self.errors
    }

    pub fn get_critical_errors(&self) -> Vec<&BrowserError> {
        self.errors
            .iter()
            .filter(|e| matches!(e.severity, ErrorSeverity::Critical))
            .collect()
    }

    pub fn get_warnings(&self) -> Vec<&BrowserError> {
        self.errors
            .iter()
            .filter(|e| matches!(e.severity, ErrorSeverity::Warning))
            .collect()
    }

    pub fn can_app_continue(&self) -> bool {
        !self.errors.iter().any(|e| !e.can_continue)
    }

    pub fn get_fallback_ui_message(&self) -> Option<String> {
        if !self.can_app_continue() {
            let critical_errors: Vec<String> = self
                .get_critical_errors()
                .iter()
                .map(|e| e.message.clone())
                .collect();

            Some(format!(
                "This application cannot run in your current browser:\n\n{}\n\nPlease upgrade your browser to continue.",
                critical_errors.join("\n")
            ))
        } else {
            None
        }
    }

    pub fn track_error(&mut self, error_type: ErrorType, details: String) {
        if self.error_tracking_enabled {
            // Send error to analytics/logging service
            let error_data = format!(
                "ErrorType: {:?}, Browser: {}, Details: {}",
                error_type,
                self.browser_info
                    .as_ref()
                    .map(|b| b.browser_name.as_str())
                    .unwrap_or("Unknown"),
                details
            );

            console::log_1(&format!("[ErrorTracking] {}", error_data).into());
        }
    }

    pub fn enable_error_tracking(&mut self, enabled: bool) {
        self.error_tracking_enabled = enabled;
    }
}

// JavaScript-callable functions for error management
#[wasm_bindgen]
pub struct JsErrorManager {
    inner: ErrorManager,
}

#[wasm_bindgen]
impl JsErrorManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ErrorManager::new(),
        }
    }

    #[wasm_bindgen(js_name = canAppContinue)]
    pub fn can_app_continue(&self) -> bool {
        self.inner.can_app_continue()
    }

    #[wasm_bindgen(js_name = getFallbackMessage)]
    pub fn get_fallback_message(&self) -> Option<String> {
        self.inner.get_fallback_ui_message()
    }

    #[wasm_bindgen(js_name = getErrorCount)]
    pub fn get_error_count(&self) -> usize {
        self.inner.get_errors().len()
    }

    #[wasm_bindgen(js_name = getCriticalErrorCount)]
    pub fn get_critical_error_count(&self) -> usize {
        self.inner.get_critical_errors().len()
    }
} 