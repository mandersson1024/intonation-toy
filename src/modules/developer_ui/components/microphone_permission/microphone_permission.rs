//! # Event-Driven Microphone Permission Component
//!
//! Microphone permission component with real-time event-driven status updates.
//! Subscribes to microphone permission events for instant status synchronization.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;

// Event system imports
#[cfg(debug_assertions)]
use crate::modules::developer_ui::hooks::use_event_subscription::use_event_subscription;
#[cfg(debug_assertions)]
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
#[cfg(debug_assertions)]
use crate::modules::audio_foundations::audio_events::{
    MicrophonePermissionEvent, MicrophoneStateEvent, DeviceListUpdatedEvent
};

// Use modular services instead of legacy
#[cfg(debug_assertions)]
use crate::modules::developer_ui::hooks::use_microphone_permission::{use_microphone_permission, PermissionState};
#[cfg(debug_assertions)]
use crate::modules::application_core::error_service::ApplicationError;

#[cfg(debug_assertions)]
#[derive(Properties, PartialEq)]
pub struct MicrophonePermissionProps {
    /// Callback when MediaStream is successfully obtained
    pub on_stream_ready: Callback<web_sys::MediaStream>,
    /// Callback when permission request fails
    pub on_error: Option<Callback<ApplicationError>>,
    /// Event bus for subscribing to microphone permission events
    #[prop_or(None)]
    pub event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    /// Whether to show detailed status information
    #[prop_or(false)]
    pub show_details: bool,
}

/// Event-driven microphone permission request component with status display and user controls
#[cfg(debug_assertions)]
#[function_component(MicrophonePermission)]
pub fn microphone_permission(props: &MicrophonePermissionProps) -> Html {
    let (permission_state, request_permission, current_error) = use_microphone_permission();
    
    // Event-driven state for real-time permission updates
    let event_permission_status = use_state(|| None::<MicrophonePermissionEvent>);
    let event_microphone_state = use_state(|| None::<MicrophoneStateEvent>);
    let last_device_update = use_state(|| None::<DeviceListUpdatedEvent>);
    
    // Subscribe to microphone permission events
    let permission_event = use_event_subscription::<MicrophonePermissionEvent>(props.event_bus.clone());
    let microphone_state_event = use_event_subscription::<MicrophoneStateEvent>(props.event_bus.clone());
    let device_list_event = use_event_subscription::<DeviceListUpdatedEvent>(props.event_bus.clone());

    // Event-driven updates: React to permission status changes
    {
        let event_permission_status = event_permission_status.clone();
        use_effect_with(permission_event.clone(), move |event| {
            if let Some(perm_event) = &**event {
                web_sys::console::log_1(&format!("Permission status updated: {:?} - Action required: {}", 
                    perm_event.permission_status, perm_event.user_action_required).into());
                event_permission_status.set(Some(perm_event.clone()));
            }
            || ()
        });
    }
    
    // Event-driven updates: React to microphone state changes
    {
        let event_microphone_state = event_microphone_state.clone();
        use_effect_with(microphone_state_event.clone(), move |event| {
            if let Some(mic_event) = &**event {
                web_sys::console::log_1(&format!("Microphone state updated: {:?}", mic_event.state).into());
                event_microphone_state.set(Some(mic_event.clone()));
            }
            || ()
        });
    }
    
    // Event-driven updates: React to device list changes
    {
        let last_device_update = last_device_update.clone();
        use_effect_with(device_list_event.clone(), move |event| {
            if let Some(device_event) = &**event {
                let mic_count = device_event.devices.iter()
                    .filter(|d| d.kind == crate::modules::audio_foundations::audio_events::AudioDeviceKind::AudioInput)
                    .count();
                web_sys::console::log_1(&format!("Device list updated: {} microphones available", mic_count).into());
                last_device_update.set(Some(device_event.clone()));
            }
            || ()
        });
    }

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
        <div class="microphone-permission-container">
            // Main permission status
            { if matches!(permission_state, PermissionState::Granted(_)) {
                html! {
                    <div class={classes!("microphone-status", "status-granted")}>
                        <span class="status-icon">{"✅"}</span>
                        <span class="status-text">{"🎤 Microphone access granted"}</span>
                    </div>
                }
            } else if can_request {
                html! {
                    <button 
                        class="microphone-btn request-btn"
                        onclick={request_permission}
                        disabled={!can_request}
                    >
                        { if matches!(permission_state, PermissionState::Denied) {
                            "🔄 Retry Microphone Permission"
                        } else {
                            "🎤 Request Microphone Access"
                        }}
                    </button>
                }
            } else {
                html! {
                    <div class={classes!("microphone-status", status_class)}>
                        <span class="status-icon">{ status_icon }</span>
                        <span class="status-text">{ status_text }</span>
                    </div>
                }
            }}
            
            // Event-driven status information
            { if props.show_details {
                html! {
                    <div class="event-driven-status">
                        <h4>{ "📡 Real-Time Status (Events)" }</h4>
                        
                        // Permission event status
                        { if let Some(ref perm_event) = *event_permission_status {
                            html! {
                                <div class="event-status-item">
                                    <span class="event-label">{ "Permission Event:" }</span>
                                    <span class={classes!("event-value",
                                        match perm_event.permission_status {
                                            crate::modules::audio_foundations::audio_events::PermissionStatus::Granted => "good",
                                            crate::modules::audio_foundations::audio_events::PermissionStatus::Denied => "warning",
                                            _ => "neutral"
                                        })}
                                    >
                                        { format!("{:?}", perm_event.permission_status) }
                                    </span>
                                    { if perm_event.user_action_required {
                                        html! { <span class="action-required">{ " (Action Required)" }</span> }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            }
                        } else {
                            html! {
                                <div class="event-status-item">
                                    <span class="event-label">{ "Permission Event:" }</span>
                                    <span class="event-value neutral">{ "No events received" }</span>
                                </div>
                            }
                        }}
                        
                        // Microphone state event status
                        { if let Some(ref mic_event) = *event_microphone_state {
                            html! {
                                <div class="event-status-item">
                                    <span class="event-label">{ "Microphone State:" }</span>
                                    <span class={classes!("event-value",
                                        match mic_event.state {
                                            crate::modules::audio_foundations::audio_events::DeviceState::Active => "good",
                                            crate::modules::audio_foundations::audio_events::DeviceState::Inactive => "neutral",
                                            crate::modules::audio_foundations::audio_events::DeviceState::Error => "warning",
                                            _ => "neutral"
                                        })}
                                    >
                                        { format!("{:?}", mic_event.state) }
                                    </span>
                                    { if let Some(ref device) = mic_event.device_info {
                                        html! {
                                            <span class="device-name">{ format!(" ({})", device.name) }</span>
                                        }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            }
                        } else {
                            html! {
                                <div class="event-status-item">
                                    <span class="event-label">{ "Microphone State:" }</span>
                                    <span class="event-value neutral">{ "No state events received" }</span>
                                </div>
                            }
                        }}
                        
                        // Device list status
                        { if let Some(ref device_event) = *last_device_update {
                            let mic_count = device_event.devices.iter()
                                .filter(|d| d.kind == crate::modules::audio_foundations::audio_events::AudioDeviceKind::AudioInput)
                                .count();
                            html! {
                                <div class="event-status-item">
                                    <span class="event-label">{ "Available Microphones:" }</span>
                                    <span class={classes!("event-value", if mic_count > 0 { "good" } else { "warning" })}>
                                        { format!("{} devices", mic_count) }
                                    </span>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="event-status-item">
                                    <span class="event-label">{ "Device Events:" }</span>
                                    <span class="event-value neutral">{ "No device events received" }</span>
                                </div>
                            }
                        }}
                        
                        // Event indicators
                        <div class="event-indicators">
                            <span class={classes!("event-indicator",
                                if permission_event.is_some() { "active" } else { "inactive" })}
                            >
                                { "🔐 Permission Events" }
                            </span>
                            <span class={classes!("event-indicator",
                                if microphone_state_event.is_some() { "active" } else { "inactive" })}
                            >
                                { "🎤 State Events" }
                            </span>
                            <span class={classes!("event-indicator",
                                if device_list_event.is_some() { "active" } else { "inactive" })}
                            >
                                { "📱 Device Events" }
                            </span>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
} 