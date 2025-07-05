// Permission Button Component - Standalone microphone permission management
//
// This component provides a standalone UI for requesting and managing microphone permissions.
// It can be used independently of the console and maintains user gesture context.

use yew::prelude::*;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

use crate::audio::AudioPermission;

/// Audio permission service trait for dependency injection
/// Note: This trait uses boxed futures to maintain dyn compatibility
/// Since this runs in a single-threaded WASM environment, Send + Sync are not required
pub trait AudioPermissionService {
    /// Request microphone permission
    fn request_permission(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AudioPermission, String>> + '_>>;
    
    /// Get current permission state
    fn get_current_permission(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = AudioPermission> + '_>>;
}

/// Properties for the PermissionButton component
#[derive(Properties)]
pub struct PermissionButtonProps {
    /// Audio service for permission management
    pub audio_service: Rc<dyn AudioPermissionService>,
    /// Callback for permission state changes
    pub on_permission_change: Callback<AudioPermission>,
}

impl PartialEq for PermissionButtonProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer equality since services are immutable after creation
        Rc::ptr_eq(&self.audio_service, &other.audio_service)
    }
}

/// State for the PermissionButton component
pub struct PermissionButton {
    /// Current permission state
    permission_state: AudioPermission,
    /// Error message if permission request fails
    error_message: Option<String>,
    /// Whether a permission request is in progress
    requesting: bool,
}

/// Messages for the PermissionButton component
#[derive(Debug)]
pub enum PermissionButtonMsg {
    /// Request microphone permission
    RequestPermission,
    /// Permission request completed successfully
    PermissionGranted(AudioPermission),
    /// Permission request failed
    PermissionError(String),
    /// Update permission state
    UpdatePermission(AudioPermission),
}

impl Component for PermissionButton {
    type Message = PermissionButtonMsg;
    type Properties = PermissionButtonProps;

    fn create(ctx: &Context<Self>) -> Self {
        let component = Self {
            permission_state: AudioPermission::Uninitialized,
            error_message: None,
            requesting: false,
        };

        // Check initial permission state from browser
        let link = ctx.link().clone();
        let audio_service = Rc::clone(&ctx.props().audio_service);
        spawn_local(async move {
            let permission = audio_service.get_current_permission().await;
            link.send_message(PermissionButtonMsg::UpdatePermission(permission));
        });

        component
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PermissionButtonMsg::RequestPermission => {
                if self.requesting {
                    return false;
                }

                self.requesting = true;
                self.error_message = None;
                self.permission_state = AudioPermission::Requesting;

                let link = ctx.link().clone();
                let audio_service = Rc::clone(&ctx.props().audio_service);
                spawn_local(async move {
                    match audio_service.request_permission().await {
                        Ok(permission) => {
                            link.send_message(PermissionButtonMsg::PermissionGranted(permission));
                        }
                        Err(error) => {
                            link.send_message(PermissionButtonMsg::PermissionError(error));
                        }
                    }
                });

                true
            }

            PermissionButtonMsg::PermissionGranted(permission) => {
                self.requesting = false;
                self.permission_state = permission;
                self.error_message = None;
                
                // Notify parent component
                ctx.props().on_permission_change.emit(self.permission_state.clone());
                
                true
            }

            PermissionButtonMsg::PermissionError(error) => {
                self.requesting = false;
                self.permission_state = AudioPermission::Denied;
                self.error_message = Some(error);
                
                // Notify parent component
                ctx.props().on_permission_change.emit(AudioPermission::Denied);
                
                true
            }

            PermissionButtonMsg::UpdatePermission(permission) => {
                self.permission_state = permission;
                self.requesting = false;
                
                // Notify parent component
                ctx.props().on_permission_change.emit(self.permission_state.clone());
                
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="permission-button-container">
                {self.render_permission_button(ctx)}
                {self.render_error_message()}
            </div>
        }
    }
}

impl PermissionButton {
    /// Render the main permission button
    fn render_permission_button(&self, ctx: &Context<Self>) -> Html {
        let (button_class, button_disabled) = match self.permission_state {
            AudioPermission::Uninitialized => (
                "permission-button permission-uninitialized",
                false,
            ),
            AudioPermission::Requesting => (
                "permission-button permission-requesting",
                true,
            ),
            AudioPermission::Granted => (
                "permission-button permission-granted",
                true,
            ),
            AudioPermission::Denied => (
                "permission-button permission-denied",
                false,
            ),
            AudioPermission::Unavailable => (
                "permission-button permission-unavailable",
                true,
            ),
        };

        html! {
            <button
                class={button_class}
                disabled={button_disabled}
                onclick={ctx.link().callback(|_| PermissionButtonMsg::RequestPermission)}
            >
                {"Request Audio Permission"}
            </button>
        }
    }

    /// Render button content with icon
    fn render_button_content(&self, text: &str) -> Html {
        let icon = match self.permission_state {
            AudioPermission::Uninitialized => "🎤",
            AudioPermission::Requesting => "⏳",
            AudioPermission::Granted => "✅",
            AudioPermission::Denied => "❌",
            AudioPermission::Unavailable => "⚠️",
        };

        html! {
            <div class="permission-button-content">
                <span class="permission-icon">{icon}</span>
                <span class="permission-text">{text}</span>
            </div>
        }
    }

    /// Render permission status display
    fn render_status_display(&self) -> Html {
        let (status_text, status_class) = match self.permission_state {
            AudioPermission::Uninitialized => ("Permission not requested", "status-uninitialized"),
            AudioPermission::Requesting => ("Requesting permission...", "status-requesting"),
            AudioPermission::Granted => ("Microphone access granted", "status-granted"),
            AudioPermission::Denied => ("Microphone access denied", "status-denied"),
            AudioPermission::Unavailable => ("Microphone unavailable", "status-unavailable"),
        };

        html! {
            <div class={format!("permission-status {}", status_class)}>
                {status_text}
            </div>
        }
    }

    /// Render error message if present
    fn render_error_message(&self) -> Html {
        if let Some(error) = &self.error_message {
            html! {
                <div class="permission-error">
                    <span class="error-icon">{"⚠️"}</span>
                    <span class="error-message">{error}</span>
                </div>
            }
        } else {
            html! {}
        }
    }
}