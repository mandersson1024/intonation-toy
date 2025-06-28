use yew::prelude::*;
use crate::legacy::active::hooks::use_microphone_permission::{use_microphone_permission, PermissionState};
use crate::legacy::active::services::error_manager::ApplicationError;

#[derive(Properties, PartialEq)]
pub struct MicrophonePermissionProps {
    /// Callback when MediaStream is successfully obtained
    pub on_stream_ready: Callback<web_sys::MediaStream>,
    /// Callback when permission request fails
    pub on_error: Option<Callback<ApplicationError>>,
    /// Whether to show detailed status information
    #[prop_or(false)]
    pub show_details: bool,
}

/// Microphone permission request component with status display and user controls
#[function_component(MicrophonePermission)]
pub fn microphone_permission(props: &MicrophonePermissionProps) -> Html {
    let (permission_state, request_permission, current_error) = use_microphone_permission();

    // Handle successful stream acquisition
    use_effect_with(permission_state.clone(), {
        let on_stream_ready = props.on_stream_ready.clone();
        move |state| {
            if let PermissionState::Granted(stream) = state {
                on_stream_ready.emit(stream.clone());
            }
        }
    });

    // Handle errors
    use_effect_with(current_error.clone(), {
        let on_error = props.on_error.clone();
        move |error| {
            if let (Some(error), Some(callback)) = (error, &on_error) {
                callback.emit(error.clone());
            }
        }
    });

    // Check if we have a device disconnection error (permission granted but device unavailable)
    let has_device_error = current_error.as_ref()
        .map(|err| err.message.contains("device disconnected") || err.message.contains("Device was physically disconnected"))
        .unwrap_or(false);

    let (status_icon, status_text, status_class, can_request) = match &permission_state {
        PermissionState::NotRequested => {
            ("", "", "status-not-requested", true)
        }
        PermissionState::Requesting => {
            ("⏳", "Requesting microphone permission...", "status-requesting", false)
        }
        PermissionState::Granted(_) if has_device_error => {
            ("⚠️", "Microphone device disconnected", "status-device-error", false)
        }
        PermissionState::Granted(_) => {
            ("✅", "Microphone access granted", "status-granted", false)
        }
        PermissionState::Denied => {
            ("❌", "Microphone access denied", "status-denied", true)
        }
        PermissionState::Unsupported => {
            ("⚠️", "Microphone not supported in this browser", "status-unsupported", false)
        }
    };

    let show_retry_info = matches!(permission_state, PermissionState::Denied);
    let show_browser_info = matches!(permission_state, PermissionState::Unsupported);

    html! {
        if matches!(permission_state, PermissionState::Granted(_)) {
            <div class={classes!("microphone-status", "status-granted")}>
                <span class="status-icon">{"✅"}</span>
                <span class="status-text">{"🎤 Microphone access granted"}</span>
            </div>
        } else if can_request {
            <button 
                class="microphone-btn request-btn"
                onclick={request_permission}
                disabled={!can_request}
            >
                if matches!(permission_state, PermissionState::Denied) {
                    { "🔄 Retry Microphone Permission" }
                } else {
                    { "🎤 Request Microphone Access" }
                }
            </button>
        } else {
            <div class={classes!("microphone-status", status_class)}>
                <span class="status-icon">{ status_icon }</span>
                <span class="status-text">{ status_text }</span>
            </div>
        }
    }
} 