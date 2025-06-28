//! # Error Toast Component
//!
//! Placeholder for migrated error toast component.
//! Will be implemented during component migration task.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use gloo::timers::callback::Timeout;

// Use modular error types instead of legacy
#[cfg(debug_assertions)]
use crate::modules::application_core::error_service::{ApplicationError, ErrorSeverity};

#[cfg(debug_assertions)]
#[derive(Properties, PartialEq)]
pub struct ErrorToastProps {
    pub error: ApplicationError,
    #[prop_or(5000)]
    pub auto_dismiss_ms: u32,
    #[prop_or(None)]
    pub on_dismiss: Option<Callback<String>>, // Passes error ID
    #[prop_or(None)]
    pub on_action: Option<Callback<String>>, // Passes error ID
    #[prop_or(None)]
    pub action_label: Option<String>,
    #[prop_or(false)]
    pub persistent: bool, // If true, won't auto-dismiss
}

#[cfg(debug_assertions)]
#[function_component(ErrorToastComponent)]
pub fn error_toast_component(props: &ErrorToastProps) -> Html {
    let visible = use_state(|| true);
    let timeout_handle = use_state(|| None::<Timeout>);
    
    // Auto-dismiss logic
    {
        let visible = visible.clone();
        let timeout_handle = timeout_handle.clone();
        let on_dismiss = props.on_dismiss.clone();
        let error_id = props.error.id.clone();
        let auto_dismiss_ms = props.auto_dismiss_ms;
        let persistent = props.persistent;
        
        use_effect_with((), move |_| {
            if !persistent {
                let timeout = Timeout::new(auto_dismiss_ms, move || {
                    visible.set(false);
                    if let Some(callback) = on_dismiss {
                        callback.emit(error_id);
                    }
                });
                timeout_handle.set(Some(timeout));
            }
            
            move || {
                // Clean up effect - we can't easily cancel the timeout here
                // The timeout will complete naturally
            }
        });
    }
    
    let handle_dismiss = {
        let visible = visible.clone();
        let on_dismiss = props.on_dismiss.clone();
        let error_id = props.error.id.clone();
        let timeout_handle = timeout_handle.clone();
        
        Callback::from(move |_: MouseEvent| {
            // Cancel auto-dismiss timeout by setting timeout_handle to None
            timeout_handle.set(None);
            
            visible.set(false);
            if let Some(callback) = on_dismiss.clone() {
                callback.emit(error_id.clone());
            }
        })
    };
    
    let handle_action = {
        let on_action = props.on_action.clone();
        let error_id = props.error.id.clone();
        
        Callback::from(move |_: MouseEvent| {
            if let Some(callback) = on_action.clone() {
                callback.emit(error_id.clone());
            }
        })
    };
    
    if !*visible {
        return html! {};
    }
    
    let toast_class = match props.error.severity {
        ErrorSeverity::Critical => "error-toast critical",
        ErrorSeverity::Warning => "error-toast warning", 
        ErrorSeverity::Info => "error-toast info",
    };
    
    let error_icon = match props.error.severity {
        ErrorSeverity::Critical => "🚨",
        ErrorSeverity::Warning => "⚠️",
        ErrorSeverity::Info => "ℹ️",
    };
    
    html! {
        <div class={classes!(toast_class, if props.persistent { Some("persistent") } else { None })}>
            <div class="toast-content">
                <div class="toast-icon">
                    {error_icon}
                </div>
                <div class="toast-message">
                    <div class="toast-title">
                        {&props.error.message}
                    </div>
                    if let Some(ref details) = props.error.details {
                        <div class="toast-details">
                            {details}
                        </div>
                    }
                </div>
                <div class="toast-actions">
                    if let Some(ref action_label) = props.action_label {
                        <button class="toast-action-button" onclick={handle_action}>
                            {action_label}
                        </button>
                    }
                    <button class="toast-dismiss-button" onclick={handle_dismiss} aria-label="Dismiss">
                        {"×"}
                    </button>
                </div>
            </div>
            
            if !props.persistent {
                <div class="toast-progress">
                    <div 
                        class="toast-progress-bar"
                        style={format!("animation-duration: {}ms", props.auto_dismiss_ms)}
                    ></div>
                </div>
            }
        </div>
    }
}

// Toast container component for managing multiple toasts
#[cfg(debug_assertions)]
#[derive(Properties, PartialEq)]
pub struct ErrorToastContainerProps {
    pub errors: Vec<ApplicationError>,
    #[prop_or(None)]
    pub on_dismiss: Option<Callback<String>>,
    #[prop_or(None)]
    pub on_action: Option<Callback<String>>,
    #[prop_or(5)]
    pub max_toasts: usize,
    #[prop_or(String::from("top-right"))]
    pub position: String, // "top-right", "top-left", "bottom-right", "bottom-left"
}

#[cfg(debug_assertions)]
#[function_component(ErrorToastContainer)]
pub fn error_toast_container(props: &ErrorToastContainerProps) -> Html {
    // Filter to only show non-critical errors (critical errors should use fallback UI)
    let toast_errors: Vec<&ApplicationError> = props.errors
        .iter()
        .filter(|e| !matches!(e.severity, ErrorSeverity::Critical))
        .take(props.max_toasts)
        .collect();
    
    if toast_errors.is_empty() {
        return html! {};
    }
    
    let container_class = format!("error-toast-container {}", props.position);
    
    html! {
        <div class={container_class}>
            { for toast_errors.into_iter().map(|error| {
                let action_label = match error.severity {
                    ErrorSeverity::Warning if error.can_retry() => Some("Retry".to_string()),
                    _ => None,
                };
                
                html! {
                    <ErrorToastComponent
                        key={error.id.clone()}
                        error={error.clone()}
                        on_dismiss={props.on_dismiss.clone()}
                        on_action={props.on_action.clone()}
                        action_label={action_label}
                        persistent={matches!(error.severity, ErrorSeverity::Warning) && error.retry_count > 0}
                    />
                }
            }) }
        </div>
    }
} 