use yew::prelude::*;
use web_sys::{MediaStreamConstraints, MediaDevices};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use gloo::console;
use crate::services::error_manager::{ApplicationError, ErrorCategory};

/// Microphone permission states
#[derive(Clone, Debug, PartialEq)]
pub enum PermissionState {
    /// Permission has not been requested yet
    NotRequested,
    /// Currently requesting permission (showing browser dialog)
    Requesting,
    /// Permission granted and MediaStream available
    Granted(web_sys::MediaStream),
    /// User denied permission
    Denied,
    /// Browser doesn't support getUserMedia
    Unsupported,
}

impl Default for PermissionState {
    fn default() -> Self {
        Self::NotRequested
    }
}

impl PermissionState {
    /// Returns a clean display string for the permission state
    pub fn display_name(&self) -> &'static str {
        match self {
            PermissionState::NotRequested => "Not Requested",
            PermissionState::Requesting => "Requesting",
            PermissionState::Granted(_) => "Granted",
            PermissionState::Denied => "Denied",
            PermissionState::Unsupported => "Unsupported",
        }
    }
}

/// Hook for managing microphone permissions and MediaStream access
/// Returns: (permission_state, request_callback, current_error)
#[hook]
pub fn use_microphone_permission() -> (
    PermissionState,
    Callback<web_sys::MouseEvent>,
    Option<ApplicationError>,
) {
    let permission_state = use_state(|| PermissionState::NotRequested);
    let current_error = use_state(|| None::<ApplicationError>);

    // Check if getUserMedia is supported in current browser
    let is_supported = {
        web_sys::window()
            .and_then(|w| w.navigator().media_devices().ok())
            .is_some()
    };

    // Initialize state based on browser support
    use_effect_with((), {
        let permission_state = permission_state.clone();
        move |_| {
            if !is_supported {
                permission_state.set(PermissionState::Unsupported);
            }
        }
    });

    // Request microphone permission callback
    let request_permission = {
        let permission_state = permission_state.clone();
        let current_error = current_error.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let permission_state = permission_state.clone();
            let current_error = current_error.clone();

            // Clear any previous errors
            current_error.set(None);

            // Check browser support first
            let media_devices = match web_sys::window()
                .and_then(|w| w.navigator().media_devices().ok())
            {
                Some(devices) => devices,
                None => {
                    permission_state.set(PermissionState::Unsupported);
                    current_error.set(Some(
                        ApplicationError::microphone_permission_denied(
                            "Browser does not support getUserMedia"
                        )
                    ));
                    return;
                }
            };

            permission_state.set(PermissionState::Requesting);
            console::log!("Requesting microphone permission...");

            wasm_bindgen_futures::spawn_local(async move {
                // Configure audio-only constraints
                let constraints = MediaStreamConstraints::new();
                constraints.set_audio(&JsValue::from(true));
                constraints.set_video(&JsValue::from(false));

                match media_devices.get_user_media_with_constraints(&constraints) {
                    Ok(promise) => {
                        match JsFuture::from(promise).await {
                            Ok(stream_js) => {
                                // Successfully got MediaStream
                                let stream: web_sys::MediaStream = stream_js.into();
                                console::log!("Microphone permission granted");
                                permission_state.set(PermissionState::Granted(stream));
                            }
                            Err(js_error) => {
                                // Permission denied or other getUserMedia error
                                let error_msg = js_error
                                    .as_string()
                                    .unwrap_or_else(|| "Unknown getUserMedia error".to_string());
                                
                                console::error!(&format!("getUserMedia failed: {}", error_msg));
                                
                                permission_state.set(PermissionState::Denied);
                                current_error.set(Some(
                                    ApplicationError::microphone_permission_denied(&error_msg)
                                ));
                            }
                        }
                    }
                    Err(js_error) => {
                        // Error creating getUserMedia promise
                        let error_msg = js_error
                            .as_string()
                            .unwrap_or_else(|| "Failed to create getUserMedia request".to_string());
                        
                        console::error!(&format!("getUserMedia request failed: {}", error_msg));
                        
                        permission_state.set(PermissionState::Denied);
                        current_error.set(Some(
                            ApplicationError::microphone_permission_denied(&error_msg)
                        ));
                    }
                }
            });
        })
    };

    ((*permission_state).clone(), request_permission, (*current_error).clone())
} 