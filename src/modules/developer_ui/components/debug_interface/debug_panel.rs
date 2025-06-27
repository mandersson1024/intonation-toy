//! # Debug Panel Component
//!
//! Placeholder for migrated debug panel component.
//! Will be implemented during component migration task.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;
#[cfg(debug_assertions)]
use gloo::console;

// TODO: Update these imports once legacy services are migrated to modules
#[cfg(debug_assertions)]
use crate::legacy::active::services::error_manager::{ErrorManager, ApplicationError, ErrorSeverity, ErrorCategory};

#[cfg(debug_assertions)]
#[derive(Properties)]
pub struct DebugPanelProps {
    pub error_manager: Option<Rc<RefCell<ErrorManager>>>,
    #[prop_or(false)]
    pub show_all_errors: bool,
    #[prop_or(1000)]
    pub update_interval_ms: u32,
    #[prop_or(true)]
    pub auto_refresh: bool,
}

#[cfg(debug_assertions)]
impl PartialEq for DebugPanelProps {
    fn eq(&self, other: &Self) -> bool {
        self.show_all_errors == other.show_all_errors &&
        self.update_interval_ms == other.update_interval_ms &&
        self.auto_refresh == other.auto_refresh &&
        self.error_manager.as_ref().map(|e| e.as_ptr()) == other.error_manager.as_ref().map(|e| e.as_ptr())
    }
}

/// Debug panel component for error state visualization and debugging
#[cfg(debug_assertions)]
#[function_component(DebugPanel)]
pub fn debug_panel(props: &DebugPanelProps) -> Html {
    let errors = use_state(|| Vec::<ApplicationError>::new());
    let selected_error = use_state(|| None::<usize>);
    let show_details = use_state(|| false);
    
    // Load errors from error manager
    let refresh_errors = {
        let errors = errors.clone();
        let error_manager = props.error_manager.clone();
        
        Callback::from(move |_: web_sys::MouseEvent| {
            if let Some(manager) = &error_manager {
                if let Ok(manager_ref) = manager.try_borrow() {
                    let all_errors = manager_ref.get_all_errors();
                    let error_count = all_errors.len();
                    let error_list: Vec<ApplicationError> = all_errors.into_iter().cloned().collect();
                    
                    errors.set(error_list);
                    console::log!(&format!("Refreshed error list: {} errors", error_count));
                } else {
                    console::warn!("Could not borrow error manager for refresh");
                }
            }
        })
    };
    
    // Auto-refresh errors on mount
    {
        let error_manager = props.error_manager.clone();
        let errors = errors.clone();
        use_effect_with((), move |_| {
            if let Some(manager) = &error_manager {
                if let Ok(manager_ref) = manager.try_borrow() {
                    let all_errors = manager_ref.get_all_errors();
                    let error_list: Vec<ApplicationError> = all_errors.into_iter().cloned().collect();
                    errors.set(error_list);
                }
            }
            || ()
        });
    }
    
    // Continuous auto-refresh with interval (if enabled)
    {
        let error_manager = props.error_manager.clone();
        let errors = errors.clone();
        let auto_refresh = props.auto_refresh;
        let update_interval_ms = props.update_interval_ms;
        
        use_effect_with(
            (auto_refresh, update_interval_ms),
            move |(should_auto_refresh, interval_ms)| {
                if *should_auto_refresh {
                    let error_manager = error_manager.clone();
                    let errors = errors.clone();
                    
                    let interval = gloo::timers::callback::Interval::new(*interval_ms, move || {
                        if let Some(manager) = &error_manager {
                            if let Ok(manager_ref) = manager.try_borrow() {
                                let all_errors = manager_ref.get_all_errors();
                                let error_list: Vec<ApplicationError> = all_errors.into_iter().cloned().collect();
                                errors.set(error_list);
                            }
                        }
                    });
                    
                    Box::new(move || drop(interval)) as Box<dyn FnOnce()>
                } else {
                    Box::new(move || {}) as Box<dyn FnOnce()>
                }
            },
        );
    }
    
    // Toggle error details view
    let toggle_details = {
        let show_details = show_details.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            show_details.set(!*show_details);
        })
    };
    
    // Select an error for detailed view
    let select_error = {
        let selected_error = selected_error.clone();
        Callback::from(move |index: usize| {
            selected_error.set(Some(index));
        })
    };
    
    // Clear selected error
    let clear_selection = {
        let selected_error = selected_error.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            selected_error.set(None);
        })
    };
    
    // Format error severity with color
    let format_severity = |severity: &ErrorSeverity| {
        match severity {
            ErrorSeverity::Critical => ("🔴", "Critical", "severity-critical"),
            ErrorSeverity::Warning => ("🟡", "Warning", "severity-warning"),
            ErrorSeverity::Info => ("🔵", "Info", "severity-info"),
        }
    };
    
    // Format error category
    let format_category = |category: &ErrorCategory| {
        match category {
            ErrorCategory::BrowserCompatibility => ("🌐", "Browser"),
            ErrorCategory::WebAssemblySupport => ("⚙️", "WASM"),
            ErrorCategory::WebAudioSupport => ("🎵", "WebAudio"),
            ErrorCategory::MediaDevicesSupport => ("🎤", "Media"),
            ErrorCategory::AudioContextCreation => ("🔊", "AudioCtx"),
            ErrorCategory::AudioWorkletLoading => ("🔗", "Worklet"),
            ErrorCategory::PitchDetection => ("🎯", "Pitch"),
            ErrorCategory::MicrophonePermission => ("🔒", "MicPerm"),
            ErrorCategory::DeviceAccess => ("📱", "Device"),
            ErrorCategory::WasmLoading => ("📦", "WasmLoad"),
            ErrorCategory::NetworkConnectivity => ("📡", "Network"),
            ErrorCategory::MemoryAllocation => ("💾", "Memory"),
            ErrorCategory::ProcessingTimeout => ("⏰", "Timeout"),
            ErrorCategory::ComponentRender => ("🖥️", "Render"),
            ErrorCategory::StateManagement => ("📊", "State"),
            ErrorCategory::Unknown => ("❓", "Unknown"),
        }
    };
    
    html! {
        <div class="debug-panel">
            <div class="debug-panel-header">
                <h3>{ "Error Debug Panel" }</h3>
                <div class="panel-controls">
                    { if !props.auto_refresh {
                        html! {
                            <button 
                                class="refresh-btn"
                                onclick={refresh_errors}
                                title="Refresh error list"
                            >
                                { "🔄 Refresh" }
                            </button>
                        }
                    } else {
                        html! {}
                    }}
                    <button 
                        class="details-toggle"
                        onclick={toggle_details}
                        disabled={selected_error.is_none()}
                        title={if selected_error.is_some() { "Toggle detailed view" } else { "Select an error to view details" }}
                    >
                        { if *show_details { "📋 Simple" } else { "🔍 Details" } }
                    </button>
                </div>
            </div>
            
            <div class="error-list">
                { if errors.is_empty() {
                    html! {
                        <div class="no-errors">
                            <div class="no-errors-content">
                                <span class="no-errors-icon">{ "✅" }</span>
                                <p>{ "No errors found" }</p>
                                <p class="no-errors-hint">{ "All systems operating normally" }</p>
                            </div>
                        </div>
                    }
                } else {
                    html! {
                        <div class="errors-container">
                            { for errors.iter().enumerate().map(|(index, error)| {
                                let (severity_icon, severity_text, severity_class) = format_severity(&error.severity);
                                let (category_icon, category_text) = format_category(&error.category);
                                let is_selected = *selected_error == Some(index);
                                let select_callback = select_error.reform(move |_| index);
                                
                                html! {
                                    <div 
                                        class={classes!("error-item", if is_selected { "selected" } else { "" }, severity_class)}
                                        onclick={select_callback}
                                    >
                                        <div class="error-summary">
                                            <div class="error-indicators">
                                                <span class="severity-indicator" title={severity_text}>{ severity_icon }</span>
                                                <span class="category-indicator" title={category_text}>{ category_icon }</span>
                                            </div>
                                            <div class="error-info">
                                                <div class="error-title">{ &error.message }</div>
                                                <div class="error-meta">
                                                    <span class="error-id">{ format!("ID: {}", &error.id) }</span>
                                                    <span class="error-timestamp">{ format!("Time: {}", error.timestamp) }</span>
                                                </div>
                                            </div>
                                        </div>
                                        
                                        { if is_selected && *show_details {
                                            html! {
                                                <div class="error-details">
                                                    <div class="detail-section">
                                                        <h4>{ "Error Details" }</h4>
                                                        <div class="detail-grid">
                                                            <div class="detail-item">
                                                                <span class="detail-label">{ "Severity:" }</span>
                                                                <span class="detail-value">{ severity_text }</span>
                                                            </div>
                                                            <div class="detail-item">
                                                                <span class="detail-label">{ "Category:" }</span>
                                                                <span class="detail-value">{ category_text }</span>
                                                            </div>
                                                            <div class="detail-item">
                                                                <span class="detail-label">{ "Retry Count:" }</span>
                                                                <span class="detail-value">{ error.retry_count }</span>
                                                            </div>
                                                            <div class="detail-item">
                                                                <span class="detail-label">{ "Can Continue:" }</span>
                                                                <span class="detail-value">{ if error.can_continue { "Yes" } else { "No" } }</span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                    
                                                    { if let Some(ref details) = error.details {
                                                        html! {
                                                            <div class="detail-section">
                                                                <h4>{ "Additional Details" }</h4>
                                                                <p class="error-detail-text">{ details }</p>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }}
                                                    
                                                    { if !error.recommendations.is_empty() {
                                                        html! {
                                                            <div class="detail-section">
                                                                <h4>{ "Recommendations" }</h4>
                                                                <ul class="recommendations-list">
                                                                    { for error.recommendations.iter().map(|rec| html! {
                                                                        <li>{ rec }</li>
                                                                    }) }
                                                                </ul>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }}
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                    </div>
                                }
                            }) }
                        </div>
                    }
                }}
            </div>
        </div>
    }
} 