//! # Use Microphone Permission Hook
//!
//! Placeholder for migrated microphone permission hook.
//! Will be implemented during hook migration task.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use web_sys::{MediaStream, MediaStreamConstraints, MediaDevices};
#[cfg(debug_assertions)]
use wasm_bindgen::prelude::*;
#[cfg(debug_assertions)]
use wasm_bindgen::JsValue;
#[cfg(debug_assertions)]
use gloo::console;

// Use modular error types instead of legacy
#[cfg(debug_assertions)]
use crate::legacy::active::services::error_manager::{ApplicationError, ErrorSeverity, ErrorCategory};
use crate::modules::application_core::RecoveryStrategy;

#[cfg(debug_assertions)]
#[derive(Clone, PartialEq)]
pub enum PermissionState {
    NotRequested,
    Requesting,
    Granted(MediaStream),
    Denied,
    Unsupported,
}

#[cfg(debug_assertions)]
pub fn use_microphone_permission() -> (UseStateHandle<PermissionState>, Callback<()>, UseStateHandle<Option<ApplicationError>>) {
    let permission_state = use_state(|| PermissionState::NotRequested);
    let error_state = use_state(|| None::<ApplicationError>);
    
    let request_permission = {
        let permission_state = permission_state.clone();
        let error_state = error_state.clone();
        
        Callback::from(move |_| {
            let permission_state = permission_state.clone();
            let error_state = error_state.clone();
            
            // Set state to requesting
            permission_state.set(PermissionState::Requesting);
            error_state.set(None);
            
            wasm_bindgen_futures::spawn_local(async move {
                match request_microphone_access().await {
                    Ok(stream) => {
                        console::log!("Microphone permission granted successfully");
                        permission_state.set(PermissionState::Granted(stream));
                    }
                    Err(error) => {
                        console::error!(&format!("Microphone permission failed: {}", error.message));
                        permission_state.set(PermissionState::Denied);
                        error_state.set(Some(error));
                    }
                }
            });
        })
    };
    
    (permission_state, request_permission, error_state)
}

#[cfg(debug_assertions)]
async fn request_microphone_access() -> Result<MediaStream, ApplicationError> {
    // Check if mediaDevices is supported
    let window = web_sys::window().ok_or_else(|| ApplicationError {
        id: "MEDIA_NO_WINDOW".to_string(),
        message: "Window object not available".to_string(),
        severity: ErrorSeverity::Critical,
        category: ErrorCategory::BrowserCompatibility,
        timestamp: js_sys::Date::now(),
        details: Some("Cannot access window object in this environment".to_string()),
        user_agent: get_user_agent(),
        can_continue: false,
        retry_count: 0,
        max_retries: 0,
        recommendations: vec![
            "Ensure this code is running in a browser environment".to_string(),
            "Check if the application is properly loaded".to_string(),
        ],
        recovery_strategy: RecoveryStrategy::ErrorEscalation {
            escalation_message: "Critical browser compatibility issue".to_string(),
        },
    })?;
    
    let navigator = window.navigator();
    let media_devices = navigator.media_devices().map_err(|_| ApplicationError {
        id: "MEDIA_DEVICES_UNSUPPORTED".to_string(),
        message: "MediaDevices API not supported in this browser".to_string(),
        severity: ErrorSeverity::Critical,
        category: ErrorCategory::MediaDevicesSupport,
        timestamp: js_sys::Date::now(),
        details: Some("The MediaDevices API is required for microphone access".to_string()),
        user_agent: get_user_agent(),
        can_continue: false,
        retry_count: 0,
        max_retries: 0,
        recommendations: vec![
            "Use a modern browser that supports WebRTC".to_string(),
            "Update your browser to the latest version".to_string(),
            "Enable microphone permissions in browser settings".to_string(),
        ],
        recovery_strategy: RecoveryStrategy::GracefulDegradation {
            fallback_description: "Microphone functionality unavailable".to_string(),
        },
    })?;
    
    // Create constraints for audio-only stream
    let mut constraints = MediaStreamConstraints::new();
    constraints.audio(&JsValue::from(true));
    constraints.video(&JsValue::from(false));
    
    // Request user media
    let promise = media_devices.get_user_media_with_constraints(&constraints).map_err(|e| {
        console::error!(&format!("Failed to call getUserMedia: {:?}", e));
        ApplicationError {
            id: "MEDIA_REQUEST_FAILED".to_string(),
            message: "Failed to request microphone access".to_string(),
            severity: ErrorSeverity::Warning,
            category: ErrorCategory::MicrophonePermission,
            timestamp: js_sys::Date::now(),
            details: Some(format!("getUserMedia call failed: {:?}", e)),
            user_agent: get_user_agent(),
            can_continue: true,
            retry_count: 0,
            max_retries: 3,
            recommendations: vec![
                "Check browser permissions for this site".to_string(),
                "Ensure microphone is connected and working".to_string(),
                "Try refreshing the page".to_string(),
            ],
            recovery_strategy: RecoveryStrategy::UserGuidedRetry {
                instructions: "Please check your microphone permissions and try again".to_string(),
            },
        }
    })?;
    
    let stream_js = wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|e| {
        let error_message = format!("{:?}", e);
        console::error!(&format!("getUserMedia promise rejected: {}", error_message));
        
        // Determine error type based on the error message
        if error_message.contains("NotAllowedError") || error_message.contains("PermissionDeniedError") {
            ApplicationError {
                id: "MICROPHONE_PERMISSION_DENIED".to_string(),
                message: "Microphone permission denied by user".to_string(),
                severity: ErrorSeverity::Warning,
                category: ErrorCategory::MicrophonePermission,
                timestamp: js_sys::Date::now(),
                details: Some("User explicitly denied microphone permission".to_string()),
                user_agent: get_user_agent(),
                can_continue: true,
                retry_count: 0,
                max_retries: 3,
                recommendations: vec![
                    "Click the microphone icon in the address bar to allow access".to_string(),
                    "Check browser settings for microphone permissions".to_string(),
                    "Reload the page and try again".to_string(),
                ],
                recovery_strategy: RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please allow microphone access when prompted".to_string(),
                },
            }
        } else if error_message.contains("NotFoundError") || error_message.contains("DevicesNotFoundError") {
            ApplicationError {
                id: "MICROPHONE_NOT_FOUND".to_string(),
                message: "No microphone device found".to_string(),
                severity: ErrorSeverity::Warning,
                category: ErrorCategory::DeviceAccess,
                timestamp: js_sys::Date::now(),
                details: Some("No audio input devices are available".to_string()),
                user_agent: get_user_agent(),
                can_continue: false,
                retry_count: 0,
                max_retries: 1,
                recommendations: vec![
                    "Connect a microphone to your computer".to_string(),
                    "Check that your microphone is working in other applications".to_string(),
                    "Try a different microphone if available".to_string(),
                ],
                recovery_strategy: RecoveryStrategy::GracefulDegradation {
                    fallback_description: "Audio input functionality disabled".to_string(),
                },
            }
        } else if error_message.contains("NotReadableError") || error_message.contains("TrackStartError") {
            ApplicationError {
                id: "MICROPHONE_HARDWARE_ERROR".to_string(),
                message: "Microphone hardware error".to_string(),
                severity: ErrorSeverity::Warning,
                category: ErrorCategory::DeviceAccess,
                timestamp: js_sys::Date::now(),
                details: Some("Hardware or driver issue with microphone".to_string()),
                user_agent: get_user_agent(),
                can_continue: true,
                retry_count: 0,
                max_retries: 2,
                recommendations: vec![
                    "Check microphone drivers are up to date".to_string(),
                    "Try restarting your browser".to_string(),
                    "Disconnect and reconnect your microphone".to_string(),
                ],
                recovery_strategy: RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please check your microphone hardware and try again".to_string(),
                },
            }
        } else {
            ApplicationError {
                id: "MICROPHONE_UNKNOWN_ERROR".to_string(),
                message: "Unknown microphone access error".to_string(),
                severity: ErrorSeverity::Warning,
                category: ErrorCategory::Unknown,
                timestamp: js_sys::Date::now(),
                details: Some(error_message),
                user_agent: get_user_agent(),
                can_continue: true,
                retry_count: 0,
                max_retries: 2,
                recommendations: vec![
                    "Try refreshing the page".to_string(),
                    "Check browser console for more details".to_string(),
                    "Try using a different browser".to_string(),
                ],
                recovery_strategy: RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please try again or contact support if the problem persists".to_string(),
                },
            }
        }
    })?;
    
    let stream: MediaStream = stream_js.dyn_into().map_err(|_| ApplicationError {
        id: "MEDIA_STREAM_CONVERSION_ERROR".to_string(),
        message: "Failed to convert media stream".to_string(),
        severity: ErrorSeverity::Critical,
        category: ErrorCategory::Unknown,
        timestamp: js_sys::Date::now(),
        details: Some("Media stream object conversion failed".to_string()),
        user_agent: get_user_agent(),
        can_continue: false,
        retry_count: 0,
        max_retries: 0,
        recommendations: vec![
            "This is likely a browser compatibility issue".to_string(),
            "Try using a different browser".to_string(),
        ],
        recovery_strategy: RecoveryStrategy::ErrorEscalation {
            escalation_message: "Critical media stream conversion failure".to_string(),
        },
    })?;
    
    console::log!("Successfully obtained microphone stream");
    Ok(stream)
}

#[cfg(debug_assertions)]
fn get_user_agent() -> String {
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .unwrap_or_else(|| "Unknown".to_string())
} 