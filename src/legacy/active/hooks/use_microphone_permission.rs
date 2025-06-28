use yew::prelude::*;
use web_sys::{MediaStreamConstraints, MediaDevices};
use wasm_bindgen::{JsValue, JsCast, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use gloo::console;
use crate::legacy::active::services::error_manager::{ApplicationError, ErrorCategory};

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

    // Note: Device change monitoring is now handled by the MicrophonePanel component
    // to avoid conflicts (only one ondevicechange listener can be active at a time).
    // The MicrophonePanel will handle both device list refresh and any necessary
    // reconnection coordination. Disconnection detection still happens via track.onended
    // events which are set up in the initialization effect below.

    // Initialize state based on browser support and check for existing permissions
    use_effect_with((), {
        let permission_state = permission_state.clone();
        let current_error = current_error.clone();
        move |_| {
            if !is_supported {
                permission_state.set(PermissionState::Unsupported);
                return;
            }
            
            // Check for existing microphone permissions on page load
            let permission_state_check = permission_state.clone();
            let current_error_check = current_error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(media_devices) = web_sys::window()
                    .and_then(|w| w.navigator().media_devices().ok())
                {
                    // Try to get MediaStream without user interaction to check if permission is already granted
                    let constraints = MediaStreamConstraints::new();
                    constraints.set_audio(&JsValue::from(true));
                    constraints.set_video(&JsValue::from(false));
                    
                    match media_devices.get_user_media_with_constraints(&constraints) {
                        Ok(promise) => {
                            match JsFuture::from(promise).await {
                                Ok(stream_js) => {
                                    // Permission was already granted!
                                    let stream: web_sys::MediaStream = stream_js.into();
                                    console::log!("🎤 Microphone permission already granted - auto-connecting");
                                    
                                    // Set up track monitoring for disconnection detection
                                    let tracks = stream.get_audio_tracks();
                                    for i in 0..tracks.length() {
                                        let track_js = tracks.get(i);
                                        if let Ok(track) = track_js.dyn_into::<web_sys::MediaStreamTrack>() {
                                            let current_error_track = current_error_check.clone();
                                            
                                            let onended = Closure::wrap(Box::new(move |_: web_sys::Event| {
                                                console::warn!("🎤 Microphone device disconnected - track ended");
                                                current_error_track.set(Some(
                                                    ApplicationError::microphone_device_disconnected(
                                                        "Device was physically disconnected or became unavailable. Reconnect your device - no additional permission needed."
                                                    )
                                                ));
                                            }) as Box<dyn Fn(_)>);
                                            
                                            track.set_onended(Some(onended.as_ref().unchecked_ref()));
                                            onended.forget();
                                        }
                                    }
                                    
                                    permission_state_check.set(PermissionState::Granted(stream));
                                }
                                Err(_) => {
                                    // Permission not granted yet, leave as NotRequested
                                    console::log!("🎤 No existing microphone permission - user will need to grant access");
                                }
                            }
                        }
                        Err(_) => {
                            // Permission not granted yet, leave as NotRequested
                            console::log!("🎤 No existing microphone permission - user will need to grant access");
                        }
                    }
                }
            });
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
                                
                                // Check if this is a reconnection (permission was previously granted)
                                if matches!(*permission_state, PermissionState::Granted(_)) {
                                    console::log!("🎤 Microphone reconnected successfully");
                                } else {
                                    console::log!("🎤 Microphone permission granted");
                                }
                                
                                // Clear any previous disconnection errors
                                current_error.set(None);
                                
                                // Monitor tracks for disconnection
                                let tracks = stream.get_audio_tracks();
                                for i in 0..tracks.length() {
                                    let track_js = tracks.get(i);
                                    if let Ok(track) = track_js.dyn_into::<web_sys::MediaStreamTrack>() {
                                        
                                        // Set up ended event listener
                                        let permission_state_clone = permission_state.clone();
                                        let current_error_clone = current_error.clone();
                                        
                                        let onended = Closure::wrap(Box::new(move |_: web_sys::Event| {
                                            console::warn!("🎤 Microphone device disconnected - track ended");
                                            // Keep permission as Granted - disconnection is a device issue, not permission issue
                                            current_error_clone.set(Some(
                                                ApplicationError::microphone_device_disconnected(
                                                    "Device was physically disconnected or became unavailable. Reconnect your device - no additional permission needed."
                                                )
                                            ));
                                        }) as Box<dyn Fn(_)>);
                                        
                                        track.set_onended(Some(onended.as_ref().unchecked_ref()));
                                        onended.forget(); // Keep the closure alive
                                    }
                                }
                                
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