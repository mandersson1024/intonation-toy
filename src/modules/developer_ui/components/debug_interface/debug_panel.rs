//! # Debug Panel Component
//!
//! Event-driven debug panel component with real-time performance and error monitoring.
//! Subscribes to performance and error events for automatic updates.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;
#[cfg(debug_assertions)]
use gloo::console;

// Event system imports
#[cfg(debug_assertions)]
use crate::modules::developer_ui::hooks::use_event_subscription::use_event_subscription;
#[cfg(debug_assertions)]
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
#[cfg(debug_assertions)]
use crate::modules::audio_foundations::audio_events::{
    AudioPerformanceMetricsEvent, AudioErrorEvent, PerformanceAlertEvent,
    LatencyViolationEvent, PerformanceRegressionEvent
};

// Use modular error types and services instead of legacy
#[cfg(debug_assertions)]
use crate::modules::application_core::{ApplicationError, ErrorSeverity, ErrorCategory};
use crate::modules::application_core::ModularErrorService;

#[cfg(debug_assertions)]
#[derive(Properties)]
pub struct DebugPanelProps {
    pub error_manager: Option<Rc<RefCell<ModularErrorService>>>,
    /// Event bus for subscribing to performance and error events
    #[prop_or(None)]
    pub event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
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
        self.error_manager.as_ref().map(|e| e.as_ptr()) == other.error_manager.as_ref().map(|e| e.as_ptr()) &&
        self.event_bus.as_ref().map(|e| e.as_ptr()) == other.event_bus.as_ref().map(|e| e.as_ptr())
    }
}

/// Event-driven debug panel component for error state visualization and performance monitoring
#[cfg(debug_assertions)]
#[function_component(DebugPanel)]
pub fn debug_panel(props: &DebugPanelProps) -> Html {
    let errors = use_state(|| Vec::<ApplicationError>::new());
    let selected_error = use_state(|| None::<usize>);
    let show_details = use_state(|| false);
    
    // Event-driven state for real-time performance monitoring
    let current_performance = use_state(|| None::<AudioPerformanceMetricsEvent>);
    let performance_alerts = use_state(|| Vec::<PerformanceAlertEvent>::new());
    let latency_violations = use_state(|| Vec::<LatencyViolationEvent>::new());
    
    // Subscribe to performance events
    let performance_event = use_event_subscription::<AudioPerformanceMetricsEvent>(props.event_bus.clone());
    let performance_alert_event = use_event_subscription::<PerformanceAlertEvent>(props.event_bus.clone());
    let latency_violation_event = use_event_subscription::<LatencyViolationEvent>(props.event_bus.clone());
    let performance_regression_event = use_event_subscription::<PerformanceRegressionEvent>(props.event_bus.clone());
    
    // Subscribe to audio error events
    let audio_error_event = use_event_subscription::<AudioErrorEvent>(props.event_bus.clone());
    
    // Event-driven updates: React to performance metrics
    {
        let current_performance = current_performance.clone();
        use_effect_with(performance_event.clone(), move |event| {
            if let Some(perf_event) = &**event {
                console::log!(&format!("Performance metrics updated: CPU {}%, Latency {}ms", 
                    perf_event.cpu_usage_percent, perf_event.end_to_end_latency_ms));
                current_performance.set(Some(perf_event.clone()));
            }
            || ()
        });
    }
    
    // Event-driven updates: React to performance alerts
    {
        let performance_alerts = performance_alerts.clone();
        use_effect_with(performance_alert_event.clone(), move |event| {
            if let Some(alert_event) = &**event {
                console::warn!(&format!("Performance alert: {} - {} ({})", 
                    alert_event.alert_type, alert_event.severity, alert_event.actual_value));
                
                let mut alerts = (*performance_alerts).clone();
                alerts.push(alert_event.clone());
                // Keep only the last 20 alerts
                if alerts.len() > 20 {
                    alerts.drain(0..alerts.len() - 20);
                }
                performance_alerts.set(alerts);
            }
            || ()
        });
    }
    
    // Event-driven updates: React to latency violations
    {
        let latency_violations = latency_violations.clone();
        use_effect_with(latency_violation_event.clone(), move |event| {
            if let Some(latency_event) = &**event {
                console::warn!(&format!("Latency violation: Expected {}ms, Got {}ms", 
                    latency_event.expected_latency_ms, latency_event.actual_latency_ms));
                
                let mut violations = (*latency_violations).clone();
                violations.push(latency_event.clone());
                // Keep only the last 10 violations
                if violations.len() > 10 {
                    violations.drain(0..violations.len() - 10);
                }
                latency_violations.set(violations);
            }
            || ()
        });
    }
    
    // Event-driven updates: React to audio errors
    {
        let errors = errors.clone();
        let error_manager = props.error_manager.clone();
        use_effect_with(audio_error_event.clone(), move |event| {
            if let Some(audio_error) = &**event {
                console::error!(&format!("Audio error received via event: {}", audio_error.message));
                
                // Convert audio error to application error and add to local state
                let app_error = ApplicationError {
                    id: format!("audio-event-{}", chrono::Utc::now().timestamp_millis()),
                    message: audio_error.message.clone(),
                    details: Some(format!("Type: {:?}, Context: {}", audio_error.error_type, audio_error.context)),
                    severity: match audio_error.error_type {
                        crate::modules::audio_foundations::audio_events::AudioErrorType::Critical => ErrorSeverity::Critical,
                        _ => ErrorSeverity::Warning,
                    },
                    category: ErrorCategory::WebAudioSupport,
                    component: "EventDrivenDebugPanel".to_string(),
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    retry_count: 0,
                    can_continue: audio_error.recovery_suggestion.is_some(),
                    recommendations: audio_error.recovery_suggestion.as_ref()
                        .map(|s| vec![s.clone()])
                        .unwrap_or_default(),
                };
                
                // Add to local errors list for immediate display
                let mut current_errors = (*errors).clone();
                current_errors.push(app_error.clone());
                errors.set(current_errors);
                
                // Also add to error manager if available
                if let Some(manager) = &error_manager {
                    if let Ok(mut manager_ref) = manager.try_borrow_mut() {
                        manager_ref.add_error(app_error);
                    }
                }
            }
            || ()
        });
    }
    
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
                <h3>{ "Event-Driven Debug Panel" }</h3>
                
                // Real-time performance section
                <div class="performance-section">
                    <h4>{ "Real-Time Performance" }</h4>
                    { if let Some(ref perf) = *current_performance {
                        html! {
                            <div class="performance-metrics">
                                <div class="metric-item">
                                    <span class="metric-label">{ "Latency:" }</span>
                                    <span class={classes!("metric-value", 
                                        if perf.end_to_end_latency_ms > 50.0 { "warning" } else { "good" })}
                                    >
                                        { format!("{:.1}ms", perf.end_to_end_latency_ms) }
                                    </span>
                                </div>
                                <div class="metric-item">
                                    <span class="metric-label">{ "CPU:" }</span>
                                    <span class={classes!("metric-value",
                                        if perf.cpu_usage_percent > 70.0 { "warning" } else { "good" })}
                                    >
                                        { format!("{:.1}%", perf.cpu_usage_percent) }
                                    </span>
                                </div>
                                <div class="metric-item">
                                    <span class="metric-label">{ "Memory:" }</span>
                                    <span class="metric-value">{ format!("{:.1}MB", perf.memory_usage_bytes as f64 / 1024.0 / 1024.0) }</span>
                                </div>
                                <div class="metric-item">
                                    <span class="metric-label">{ "Dropouts:" }</span>
                                    <span class={classes!("metric-value",
                                        if perf.dropout_count > 0 { "warning" } else { "good" })}
                                    >
                                        { perf.dropout_count }
                                    </span>
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="no-performance-data">
                                <span>{ "⏳ Waiting for performance data..." }</span>
                            </div>
                        }
                    }}
                </div>
                
                // Event indicators
                <div class="event-indicators">
                    <h4>{ "Event Status" }</h4>
                    <div class="indicator-grid">
                        <span class={classes!("event-indicator", if performance_event.is_some() { "active" } else { "inactive" })}>
                            { "📊 Performance" }
                        </span>
                        <span class={classes!("event-indicator", if audio_error_event.is_some() { "active" } else { "inactive" })}>
                            { "🚨 Audio Errors" }
                        </span>
                        <span class={classes!("event-indicator", 
                            if !performance_alerts.is_empty() { "alert" } else { "inactive" })}
                        >
                            { format!("⚠️ Alerts ({})", performance_alerts.len()) }
                        </span>
                        <span class={classes!("event-indicator",
                            if !latency_violations.is_empty() { "alert" } else { "inactive" })}
                        >
                            { format!("⏱️ Latency ({} violations)", latency_violations.len()) }
                        </span>
                    </div>
                </div>
                
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