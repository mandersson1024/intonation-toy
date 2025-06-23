use yew::prelude::*;
use crate::hooks::use_microphone_permission::{use_microphone_permission, PermissionState};
use crate::services::error_manager::ApplicationError;

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

    let (status_icon, status_text, status_class, can_request) = match &permission_state {
        PermissionState::NotRequested => {
            ("🎤", "Microphone access not requested", "status-not-requested", true)
        }
        PermissionState::Requesting => {
            ("⏳", "Requesting microphone permission...", "status-requesting", false)
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
        <div class="microphone-permission">
            <div class="permission-status">
                <div class={classes!("status-display", status_class)}>
                    <span class="status-icon">{ status_icon }</span>
                    <span class="status-text">{ status_text }</span>
                </div>
            </div>

            <div class="permission-controls">
                if can_request {
                    <button 
                        class="permission-btn request-btn"
                        onclick={request_permission}
                        disabled={!can_request}
                    >
                        if matches!(permission_state, PermissionState::Denied) {
                            { "🔄 Retry Permission" }
                        } else {
                            { "🎤 Request Microphone Access" }
                        }
                    </button>
                }
            </div>

            if show_retry_info {
                <div class="permission-help retry-help">
                    <h4>{ "Permission Denied" }</h4>
                    <p>{ "To use this app, microphone access is required. You can:" }</p>
                    <ul>
                        <li>{ "Click the retry button to request permission again" }</li>
                        <li>{ "Check your browser's address bar for permission settings" }</li>
                        <li>{ "Reload the page and allow microphone access when prompted" }</li>
                    </ul>
                </div>
            }

            if show_browser_info {
                <div class="permission-help browser-help">
                    <h4>{ "Browser Not Supported" }</h4>
                    <p>{ "Your browser doesn't support microphone access. Please try:" }</p>
                    <ul>
                        <li>{ "Chrome 47+ or Firefox 36+" }</li>
                        <li>{ "Safari 11+ (with HTTPS)" }</li>
                        <li>{ "Edge 79+" }</li>
                    </ul>
                    <p><strong>{ "Note:" }</strong>{ " HTTPS is required for microphone access." }</p>
                </div>
            }

            if props.show_details {
                <div class="permission-details">
                    <h4>{ "Technical Details" }</h4>
                    <div class="detail-grid">
                        <div class="detail-item">
                            <span class="detail-label">{ "State:" }</span>
                            <span class="detail-value">{ permission_state.display_name() }</span>
                        </div>
                        if let Some(error) = &current_error {
                            <div class="detail-item">
                                <span class="detail-label">{ "Last Error:" }</span>
                                <span class="detail-value error">{ &error.message }</span>
                            </div>
                        }
                        <div class="detail-item">
                            <span class="detail-label">{ "getUserMedia Support:" }</span>
                            <span class="detail-value">{ 
                                if web_sys::window()
                                    .and_then(|w| w.navigator().media_devices().ok())
                                    .is_some() { "Yes" } else { "No" }
                            }</span>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
} 