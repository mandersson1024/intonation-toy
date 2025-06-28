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
use crate::modules::application_core::{ApplicationError, ErrorSeverity, ErrorCategory, RecoveryStrategy};

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
    let window = web_sys::window().ok_or_else(|| {
        ApplicationError::new(
            ErrorCategory::BrowserCompatibility,
            ErrorSeverity::Critical,
            "Window object not available".to_string(),
            Some("Cannot access window object in this environment".to_string()),
            RecoveryStrategy::ErrorEscalation {
                escalation_message: "Critical browser compatibility issue".to_string(),
            },
        ).with_recommendations(vec![
            "Ensure this code is running in a browser environment".to_string(),
            "Check if the application is properly loaded".to_string(),
        ])
    })?;
    
    let navigator = window.navigator();
    let media_devices = navigator.media_devices().map_err(|_| {
        ApplicationError::new(
            ErrorCategory::MediaDevicesSupport,
            ErrorSeverity::Critical,
            "MediaDevices API not supported in this browser".to_string(),
            Some("The MediaDevices API is required for microphone access".to_string()),
            RecoveryStrategy::GracefulDegradation {
                fallback_description: "Microphone functionality unavailable".to_string(),
            },
        ).with_recommendations(vec![
            "Use a modern browser that supports WebRTC".to_string(),
            "Update your browser to the latest version".to_string(),
            "Enable microphone permissions in browser settings".to_string(),
        ])
    })?;
    
    // Create constraints for audio-only stream
    let mut constraints = MediaStreamConstraints::new();
    constraints.audio(&JsValue::from(true));
    constraints.video(&JsValue::from(false));
    
    // Request user media
    let promise = media_devices.get_user_media_with_constraints(&constraints).map_err(|e| {
        console::error!(&format!("Failed to call getUserMedia: {:?}", e));
        ApplicationError::new(
            ErrorCategory::MicrophonePermission,
            ErrorSeverity::Warning,
            "Failed to request microphone access".to_string(),
            Some(format!("getUserMedia call failed: {:?}", e)),
            RecoveryStrategy::UserGuidedRetry {
                instructions: "Please check your microphone permissions and try again".to_string(),
            },
        ).with_recommendations(vec![
            "Check browser permissions for this site".to_string(),
            "Ensure microphone is connected and working".to_string(),
            "Try refreshing the page".to_string(),
        ]).with_max_retries(3)
    })?;
    
    let stream_js = wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|e| {
        let error_message = format!("{:?}", e);
        console::error!(&format!("getUserMedia promise rejected: {}", error_message));
        
        // Determine error type based on the error message
        if error_message.contains("NotAllowedError") || error_message.contains("PermissionDeniedError") {
            ApplicationError::microphone_permission_denied("User explicitly denied microphone permission")
                .with_recommendations(vec![
                    "Click the microphone icon in the address bar to allow access".to_string(),
                    "Check browser settings for microphone permissions".to_string(),
                    "Reload the page and try again".to_string(),
                ])
        } else if error_message.contains("NotFoundError") || error_message.contains("DevicesNotFoundError") {
            ApplicationError::new(
                ErrorCategory::DeviceAccess,
                ErrorSeverity::Warning,
                "No microphone device found".to_string(),
                Some("No audio input devices are available".to_string()),
                RecoveryStrategy::GracefulDegradation {
                    fallback_description: "Audio input functionality disabled".to_string(),
                },
            ).with_recommendations(vec![
                "Connect a microphone to your computer".to_string(),
                "Check that your microphone is working in other applications".to_string(),
                "Try a different microphone if available".to_string(),
            ]).with_max_retries(1)
        } else if error_message.contains("NotReadableError") || error_message.contains("TrackStartError") {
            ApplicationError::new(
                ErrorCategory::DeviceAccess,
                ErrorSeverity::Warning,
                "Microphone hardware error".to_string(),
                Some("Hardware or driver issue with microphone".to_string()),
                RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please check your microphone hardware and try again".to_string(),
                },
            ).with_recommendations(vec![
                "Check microphone drivers are up to date".to_string(),
                "Try restarting your browser".to_string(),
                "Disconnect and reconnect your microphone".to_string(),
            ]).with_max_retries(2)
        } else {
            ApplicationError::new(
                ErrorCategory::Unknown,
                ErrorSeverity::Warning,
                "Unknown microphone access error".to_string(),
                Some(error_message),
                RecoveryStrategy::UserGuidedRetry {
                    instructions: "Please try again or contact support if the problem persists".to_string(),
                },
            ).with_recommendations(vec![
                "Try refreshing the page".to_string(),
                "Check browser console for more details".to_string(),
                "Try using a different browser".to_string(),
            ]).with_max_retries(2)
        }
    })?;
    
    let stream: MediaStream = stream_js.dyn_into().map_err(|_| {
        ApplicationError::new(
            ErrorCategory::Unknown,
            ErrorSeverity::Critical,
            "Failed to convert media stream".to_string(),
            Some("Media stream object conversion failed".to_string()),
            RecoveryStrategy::ErrorEscalation {
                escalation_message: "Critical media stream conversion failure".to_string(),
            },
        ).with_recommendations(vec![
            "This is likely a browser compatibility issue".to_string(),
            "Try using a different browser".to_string(),
        ])
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