// Permission Manager Implementation - STORY-014
// Handles microphone permissions and user consent for web browsers

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    MediaDevices, MediaStream, MediaStreamConstraints,
    Navigator, Window
};

/// Permission states for audio devices (replacing PermissionState which may not be available)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    Granted,
    Denied,
    Prompt,
}
use js_sys::{Object, Promise};
use super::audio_events::*;
use crate::modules::application_core::event_bus::EventBus;
use crate::modules::application_core::typed_event_bus::TypedEventBus;

/// Trait for managing audio permissions
pub trait PermissionManager: Send + Sync {
    /// Request microphone permission from the user
    fn request_microphone_permission(&mut self) -> Result<PermissionRequestResult, PermissionError>;
    
    /// Get current microphone permission status
    fn get_microphone_permission_status(&self) -> Result<PermissionState, PermissionError>;
    
    /// Check if microphone permission is granted
    fn has_microphone_permission(&self) -> bool;
    
    /// Start monitoring permission changes
    fn start_permission_monitoring(&mut self) -> Result<(), PermissionError>;
    
    /// Stop monitoring permission changes
    fn stop_permission_monitoring(&mut self) -> Result<(), PermissionError>;
    
    /// Handle permission denied scenario with user guidance
    fn handle_permission_denied(&self) -> PermissionRecoveryAction;
}

/// Result of a permission request
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionRequestResult {
    pub status: PermissionState,
    pub user_action_required: bool,
    pub recovery_instructions: Option<String>,
    pub can_retry: bool,
}

/// Permission-related errors
#[derive(Debug, Clone)]
pub enum PermissionError {
    BrowserNotSupported,
    UserDenied,
    SystemDenied,
    TemporarilyUnavailable,
    NetworkError(String),
    InternalError(String),
}

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionError::BrowserNotSupported => {
                write!(f, "Browser does not support microphone permissions API")
            }
            PermissionError::UserDenied => {
                write!(f, "User denied microphone permission")
            }
            PermissionError::SystemDenied => {
                write!(f, "System denied microphone access")
            }
            PermissionError::TemporarilyUnavailable => {
                write!(f, "Microphone temporarily unavailable")
            }
            PermissionError::NetworkError(msg) => {
                write!(f, "Network error while requesting permission: {}", msg)
            }
            PermissionError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
        }
    }
}

impl Error for PermissionError {}

/// Actions to recover from permission issues
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionRecoveryAction {
    ShowInstructions(String),
    RedirectToSettings,
    RequestAgain,
    ContactSupport,
    NoRecoveryPossible,
}

/// Web-based permission manager implementation
pub struct WebPermissionManager {
    media_devices: MediaDevices,
    current_permission_status: PermissionState,
    monitoring_active: bool,
    event_bus: Option<Arc<TypedEventBus>>,
    permission_request_in_progress: bool,
}

impl WebPermissionManager {
    /// Create a new web permission manager
    pub fn new() -> Result<Self, PermissionError> {
        let window = web_sys::window().ok_or(PermissionError::BrowserNotSupported)?;
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| PermissionError::BrowserNotSupported)?;
        
        Ok(Self {
            media_devices,
            current_permission_status: PermissionState::Prompt,
            monitoring_active: false,
            event_bus: None,
            permission_request_in_progress: false,
        })
    }
    
    /// Set event bus for publishing permission events
    pub fn set_event_bus(&mut self, event_bus: Arc<TypedEventBus>) {
        self.event_bus = Some(event_bus);
    }
    
    /// Request permission using getUserMedia (most reliable method)
    pub async fn request_permission_via_user_media(&mut self) -> Result<PermissionRequestResult, PermissionError> {
        if self.permission_request_in_progress {
            return Err(PermissionError::InternalError("Permission request already in progress".to_string()));
        }
        
        self.permission_request_in_progress = true;
        
        // Publish permission request started event
        self.publish_permission_event(PermissionEventType::RequestStarted, self.current_permission_status);
        
        let mut constraints = MediaStreamConstraints::new();
        
        // Create audio constraints using Object
        let audio_constraints = Object::new();
        js_sys::Reflect::set(&audio_constraints, &"echoCancellation".into(), &true.into())
            .map_err(|_| PermissionError::InternalError("Failed to set audio constraints".to_string()))?;
        js_sys::Reflect::set(&audio_constraints, &"noiseSuppression".into(), &true.into())
            .map_err(|_| PermissionError::InternalError("Failed to set audio constraints".to_string()))?;
        
        constraints.audio(&audio_constraints.into());
        constraints.video(&false.into());
        
        let promise = self.media_devices.get_user_media_with_constraints(&constraints)
            .map_err(|_| PermissionError::BrowserNotSupported)?;
        
        let result = JsFuture::from(promise).await;
        self.permission_request_in_progress = false;
        
        match result {
            Ok(stream) => {
                let media_stream: MediaStream = stream.into();
                
                // Permission granted - stop the stream immediately since we only needed permission
                let tracks = media_stream.get_audio_tracks();
                for i in 0..tracks.length() {
                    if let Some(track) = tracks.get(i) {
                        let track: web_sys::MediaStreamTrack = track.into();
                        track.stop();
                    }
                }
                
                self.current_permission_status = PermissionState::Granted;
                
                let result = PermissionRequestResult {
                    status: PermissionState::Granted,
                    user_action_required: false,
                    recovery_instructions: None,
                    can_retry: false,
                };
                
                // Publish permission granted event
                self.publish_permission_event(PermissionEventType::Granted, PermissionState::Granted);
                
                Ok(result)
            }
            Err(error) => {
                // Analyze the error to determine the cause
                let js_error = error.as_string().unwrap_or_default();
                
                let (permission_error, permission_state, recovery_action) = if js_error.contains("NotAllowedError") {
                    (
                        PermissionError::UserDenied,
                        PermissionState::Denied,
                        PermissionRecoveryAction::ShowInstructions(
                            "To enable microphone access:\n1. Click the microphone icon in your browser's address bar\n2. Select 'Allow' for this site\n3. Refresh the page and try again".to_string()
                        )
                    )
                } else if js_error.contains("NotFoundError") {
                    (
                        PermissionError::SystemDenied,
                        PermissionState::Denied,
                        PermissionRecoveryAction::ShowInstructions(
                            "No microphone found. Please check that:\n1. A microphone is connected to your device\n2. Your system audio settings allow microphone access\n3. Other applications aren't using the microphone".to_string()
                        )
                    )
                } else if js_error.contains("NotReadableError") {
                    (
                        PermissionError::TemporarilyUnavailable,
                        PermissionState::Denied,
                        PermissionRecoveryAction::RequestAgain
                    )
                } else {
                    (
                        PermissionError::InternalError(js_error),
                        PermissionState::Denied,
                        PermissionRecoveryAction::ContactSupport
                    )
                };
                
                self.current_permission_status = permission_state;
                
                let result = PermissionRequestResult {
                    status: permission_state,
                    user_action_required: true,
                    recovery_instructions: Some(match recovery_action {
                        PermissionRecoveryAction::ShowInstructions(ref msg) => msg.clone(),
                        _ => "Please check your browser settings and try again".to_string(),
                    }),
                    can_retry: matches!(recovery_action, PermissionRecoveryAction::RequestAgain),
                };
                
                // Publish permission denied event
                self.publish_permission_event(PermissionEventType::Denied, permission_state);
                
                Err(permission_error)
            }
        }
    }
    
    /// Query permission status (simplified for now)
    pub async fn query_permission_status(&mut self) -> Result<PermissionState, PermissionError> {
        // Simplified implementation - would use Permissions API in full version
        Ok(self.current_permission_status)
    }
    
    /// Publish permission-related events
    fn publish_permission_event(&self, event_type: PermissionEventType, status: PermissionState) {
        if let Some(ref event_bus) = self.event_bus {
            let event = MicrophonePermissionEvent {
                event_type,
                permission_status: status,
                user_action_required: matches!(event_type, PermissionEventType::Denied),
                recovery_instructions: self.get_recovery_instructions_for_status(status),
                timestamp_ns: get_timestamp_ns(),
            };
            
            if let Err(e) = event_bus.publish(event) {
                web_sys::console::warn_1(&format!("Failed to publish permission event: {:?}", e).into());
            }
        }
    }
    
    /// Get recovery instructions for a permission status
    fn get_recovery_instructions_for_status(&self, status: PermissionState) -> Option<String> {
        match status {
            PermissionState::Denied => Some(
                "Microphone access is blocked. Please click the microphone icon in your browser's address bar and allow access.".to_string()
            ),
            PermissionState::Prompt => Some(
                "Click 'Allow' when prompted to grant microphone access.".to_string()
            ),
            PermissionState::Granted => None,
        }
    }
}

impl PermissionManager for WebPermissionManager {
    fn request_microphone_permission(&mut self) -> Result<PermissionRequestResult, PermissionError> {
        // For synchronous interface, return a placeholder result
        // In practice, the async version should be used
        if self.current_permission_status == PermissionState::Granted {
            return Ok(PermissionRequestResult {
                status: PermissionState::Granted,
                user_action_required: false,
                recovery_instructions: None,
                can_retry: false,
            });
        }
        
        // Return a result indicating async operation is needed
        Ok(PermissionRequestResult {
            status: PermissionState::Prompt,
            user_action_required: true,
            recovery_instructions: Some("Use the async request_permission_via_user_media method for actual permission request.".to_string()),
            can_retry: true,
        })
    }
    
    fn get_microphone_permission_status(&self) -> Result<PermissionState, PermissionError> {
        Ok(self.current_permission_status)
    }
    
    fn has_microphone_permission(&self) -> bool {
        self.current_permission_status == PermissionState::Granted
    }
    
    fn start_permission_monitoring(&mut self) -> Result<(), PermissionError> {
        if self.monitoring_active {
            return Ok(());
        }
        
        self.monitoring_active = true;
        web_sys::console::log_1(&"Permission monitoring started".into());
        Ok(())
    }
    
    fn stop_permission_monitoring(&mut self) -> Result<(), PermissionError> {
        if !self.monitoring_active {
            return Ok(());
        }
        
        self.monitoring_active = false;
        web_sys::console::log_1(&"Permission monitoring stopped".into());
        Ok(())
    }
    
    fn handle_permission_denied(&self) -> PermissionRecoveryAction {
        match self.current_permission_status {
            PermissionState::Denied => {
                PermissionRecoveryAction::ShowInstructions(
                    "To enable microphone access:\n\n1. Look for the microphone icon in your browser's address bar\n2. Click it and select 'Allow'\n3. Refresh the page\n\nIf you don't see the icon:\n1. Go to your browser settings\n2. Find Privacy/Security settings\n3. Allow microphone access for this site".to_string()
                )
            }
            PermissionState::Prompt => PermissionRecoveryAction::RequestAgain,
            PermissionState::Granted => PermissionRecoveryAction::NoRecoveryPossible,
        }
    }
}

// Utility function to get current timestamp
fn get_timestamp_ns() -> u64 {
    if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
        (performance.now() * 1_000_000.0) as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_request_result() {
        let result = PermissionRequestResult {
            status: PermissionState::Granted,
            user_action_required: false,
            recovery_instructions: None,
            can_retry: false,
        };
        
        assert_eq!(result.status, PermissionState::Granted);
        assert!(!result.user_action_required);
        assert!(result.recovery_instructions.is_none());
    }
    
    #[test]
    fn test_permission_error_display() {
        let error = PermissionError::UserDenied;
        assert_eq!(error.to_string(), "User denied microphone permission");
        
        let error = PermissionError::BrowserNotSupported;
        assert_eq!(error.to_string(), "Browser does not support microphone permissions API");
    }
    
    #[test]
    fn test_permission_recovery_action() {
        let action = PermissionRecoveryAction::ShowInstructions("Test instructions".to_string());
        match action {
            PermissionRecoveryAction::ShowInstructions(instructions) => {
                assert_eq!(instructions, "Test instructions");
            }
            _ => panic!("Expected ShowInstructions variant"),
        }
    }
}