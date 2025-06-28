//! # Error Display Component
//!
//! Placeholder for migrated error display component.
//! Will be implemented during component migration task.

#[cfg(debug_assertions)]
use yew::prelude::*;

// Use modular error types instead of legacy
#[cfg(debug_assertions)]
use crate::modules::application_core::{ApplicationError, ErrorSeverity, RecoveryStrategy};

#[cfg(debug_assertions)]
#[derive(Properties, PartialEq)]
pub struct ErrorDisplayProps {
    pub error: ApplicationError,
    #[prop_or(None)]
    pub on_retry: Option<Callback<()>>,
    #[prop_or(None)]
    pub on_dismiss: Option<Callback<()>>,
    #[prop_or(true)]
    pub show_recovery_options: bool,
    #[prop_or(false)]
    pub compact_mode: bool,
}

#[cfg(debug_assertions)]
#[function_component(ErrorDisplayComponent)]
pub fn error_display_component(props: &ErrorDisplayProps) -> Html {
    let expanded = use_state(|| false);
    
    let toggle_details = {
        let expanded = expanded.clone();
        Callback::from(move |_: MouseEvent| {
            expanded.set(!*expanded);
        })
    };
    
    let handle_retry = {
        let on_retry = props.on_retry.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref callback) = on_retry {
                callback.emit(());
            }
        })
    };
    
    let handle_dismiss = {
        let on_dismiss = props.on_dismiss.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref callback) = on_dismiss {
                callback.emit(());
            }
        })
    };
    
    let error_class = match props.error.severity {
        ErrorSeverity::Critical => "error-display critical",
        ErrorSeverity::Warning => "error-display warning",
        ErrorSeverity::Info => "error-display info",
    };
    
    let error_icon = match props.error.severity {
        ErrorSeverity::Critical => "❌",
        ErrorSeverity::Warning => "⚠️",
        ErrorSeverity::Info => "ℹ️",
    };
    
    if props.compact_mode {
        return html! {
            <div class={classes!(error_class, "compact")}>
                <div class="error-compact-content">
                    <span class="error-icon">{error_icon}</span>
                    <span class="error-message">{&props.error.message}</span>
                    if let Some(_) = props.on_dismiss {
                        <button class="dismiss-button" onclick={handle_dismiss}>
                            {"×"}
                        </button>
                    }
                </div>
            </div>
        };
    }
    
    html! {
        <div class={error_class}>
            <div class="error-header">
                <div class="error-title">
                    <span class="error-icon">{error_icon}</span>
                    <h4 class="error-message">{&props.error.message}</h4>
                </div>
                <div class="error-actions">
                    if props.error.details.is_some() || !props.error.recommendations.is_empty() {
                        <button 
                            class="toggle-details-button"
                            onclick={toggle_details}
                            aria-expanded={(*expanded).to_string()}
                        >
                            { if *expanded { "Less Details" } else { "More Details" } }
                        </button>
                    }
                    if let Some(_) = props.on_dismiss {
                        <button class="dismiss-button" onclick={handle_dismiss}>
                            {"Dismiss"}
                        </button>
                    }
                </div>
            </div>
            
            if *expanded {
                <div class="error-details">
                    if let Some(ref details) = props.error.details {
                        <div class="error-description">
                            <h5>{"Details:"}</h5>
                            <p>{details}</p>
                        </div>
                    }
                    
                    if !props.error.recommendations.is_empty() {
                        <div class="error-recommendations">
                            <h5>{"Recommendations:"}</h5>
                            <ul>
                                { for props.error.recommendations.iter().map(|rec| html! {
                                    <li>{rec}</li>
                                }) }
                            </ul>
                        </div>
                    }
                    
                    <div class="error-metadata">
                        <small>
                            {"Error ID: "}{&props.error.id}
                            {" | Category: "}{format!("{:?}", props.error.category)}
                            {" | Retries: "}{props.error.retry_count}{"/"}{props.error.max_retries}
                        </small>
                    </div>
                </div>
            }
            
            if props.show_recovery_options {
                <div class="error-recovery">
                    { match &props.error.recovery_strategy {
                        RecoveryStrategy::AutomaticRetry { max_attempts, delay_ms } => {
                            html! {
                                <div class="recovery-automatic">
                                    <p>{"🔄 Automatic retry in progress..."}</p>
                                    <small>{format!("Max attempts: {}, Delay: {}ms", max_attempts, delay_ms)}</small>
                                </div>
                            }
                        },
                        RecoveryStrategy::UserGuidedRetry { instructions } => {
                            html! {
                                <div class="recovery-user-guided">
                                    <p><strong>{"What you can do:"}</strong></p>
                                    <p>{instructions}</p>
                                    if props.error.can_retry() && props.on_retry.is_some() {
                                        <button class="retry-button" onclick={handle_retry}>
                                            {"Try Again"}
                                        </button>
                                    }
                                </div>
                            }
                        },
                        RecoveryStrategy::GracefulDegradation { fallback_description } => {
                            html! {
                                <div class="recovery-degradation">
                                    <p><strong>{"Fallback mode:"}</strong></p>
                                    <p>{fallback_description}</p>
                                </div>
                            }
                        },
                        RecoveryStrategy::ErrorEscalation { escalation_message } => {
                            html! {
                                <div class="recovery-escalation">
                                    <p><strong>{"Action required:"}</strong></p>
                                    <p>{escalation_message}</p>
                                </div>
                            }
                        },
                        RecoveryStrategy::ApplicationReset { reset_message } => {
                            html! {
                                <div class="recovery-reset">
                                    <p><strong>{"Application reset required:"}</strong></p>
                                    <p>{reset_message}</p>
                                    <button 
                                        class="reset-button"
                                        onclick={Callback::from(|_| {
                                            if let Some(window) = web_sys::window() {
                                                let _ = window.location().reload();
                                            }
                                        })}
                                    >
                                        {"Refresh Page"}
                                    </button>
                                </div>
                            }
                        },
                        RecoveryStrategy::None => {
                            html! {
                                <div class="recovery-none">
                                    <p>{"No automatic recovery available."}</p>
                                </div>
                            }
                        }
                    }}
                </div>
            }
        </div>
    }
} 