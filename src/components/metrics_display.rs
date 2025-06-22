use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::services::audio_engine::AudioEngineService;
use crate::audio::performance_monitor::PerformanceMetrics;
use gloo::console;
use gloo::timers::callback::Interval;

#[derive(Properties)]
pub struct MetricsDisplayProps {
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    #[prop_or(1000)]
    pub update_interval_ms: u32,
}

impl PartialEq for MetricsDisplayProps {
    fn eq(&self, other: &Self) -> bool {
        self.update_interval_ms == other.update_interval_ms &&
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr())
    }
}

/// Real-time metrics display component showing audio processing performance
#[function_component(MetricsDisplay)]
pub fn metrics_display(props: &MetricsDisplayProps) -> Html {
    let metrics = use_state(|| None::<PerformanceMetrics>);
    let last_update = use_state(|| None::<String>);
    let update_count = use_state(|| 0u32);
    let is_monitoring = use_state(|| false);
    
    // Set up automatic metrics updates
    {
        let metrics = metrics.clone();
        let last_update = last_update.clone();
        let update_count = update_count.clone();
        let is_monitoring = is_monitoring.clone();
        let audio_engine = props.audio_engine.clone();
        let update_interval_ms = props.update_interval_ms;
        
        use_effect_with(
            update_interval_ms, // Only track interval changes, not the engine itself
            move |interval| {
                let interval_handle = if let Some(engine) = &audio_engine {
                    is_monitoring.set(true);
                    let engine_clone = engine.clone();
                    
                    let interval = Interval::new(*interval, move || {
                        if let Ok(engine_ref) = engine_clone.try_borrow() {
                            let current_metrics = engine_ref.get_performance_metrics();
                            let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default();
                            
                            metrics.set(Some(current_metrics));
                            last_update.set(Some(timestamp));
                            update_count.set(*update_count + 1);
                            
                            console::log!("Updated audio metrics");
                        } else {
                            console::warn!("Could not borrow audio engine for metrics update");
                        }
                    });
                    
                    Some(interval)
                } else {
                    is_monitoring.set(false);
                    metrics.set(None);
                    last_update.set(None);
                    None
                };
                
                move || {
                    if let Some(interval) = interval_handle {
                        drop(interval);
                    }
                    is_monitoring.set(false);
                }
            },
        );
    }
    
    // Manual refresh callback
    let refresh_metrics = {
        let metrics = metrics.clone();
        let last_update = last_update.clone();
        let update_count = update_count.clone();
        let audio_engine = props.audio_engine.clone();
        
        Callback::from(move |_| {
            if let Some(engine) = &audio_engine {
                if let Ok(engine_ref) = engine.try_borrow() {
                    let current_metrics = engine_ref.get_performance_metrics();
                    let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default();
                    
                    metrics.set(Some(current_metrics));
                    last_update.set(Some(timestamp));
                    update_count.set(*update_count + 1);
                    
                    console::log!("Manually refreshed audio metrics");
                } else {
                    console::warn!("Could not borrow audio engine for manual metrics refresh");
                }
            }
        })
    };
    
    let format_latency = |latency_ms: f32| {
        if latency_ms < 10.0 {
            format!("{:.1}ms ✅", latency_ms)
        } else if latency_ms < 50.0 {
            format!("{:.1}ms ⚠️", latency_ms)
        } else {
            format!("{:.1}ms ❌", latency_ms)
        }
    };
    
    let format_rate = |rate_hz: f32| {
        if rate_hz > 40.0 {
            format!("{:.1}Hz ✅", rate_hz)
        } else if rate_hz > 20.0 {
            format!("{:.1}Hz ⚠️", rate_hz)
        } else {
            format!("{:.1}Hz ❌", rate_hz)
        }
    };
    
    html! {
        <div class="metrics-display">
            <div class="metrics-header">
                <h3>{ "Real-time Metrics" }</h3>
                <div class="metrics-controls">
                    <button 
                        class="refresh-btn"
                        onclick={refresh_metrics}
                        title="Manually refresh metrics"
                    >
                        { "🔄 Refresh" }
                    </button>
                    <div class="monitoring-status">
                        { if *is_monitoring { 
                            html! { <span class="status-active">{ "● LIVE" }</span> }
                        } else { 
                            html! { <span class="status-inactive">{ "○ STOPPED" }</span> }
                        }}
                    </div>
                </div>
            </div>
            
            { if let Some(current_metrics) = &*metrics {
                html! {
                    <div class="metrics-content">
                        <div class="metrics-grid">
                            <div class="metric-card latency-card">
                                <div class="metric-header">
                                    <span class="metric-icon">{ "⏱️" }</span>
                                    <span class="metric-title">{ "Total Latency" }</span>
                                </div>
                                <div class="metric-value">
                                    { format_latency(current_metrics.total_latency_ms()) }
                                </div>
                                <div class="metric-details">
                                    <div class="metric-breakdown">
                                        <span>{ format!("Buffer: {:.1}ms", current_metrics.buffer_latency_ms()) }</span>
                                        <span>{ format!("Processing: {:.1}ms", current_metrics.processing_latency_ms()) }</span>
                                    </div>
                                </div>
                            </div>
                            
                            <div class="metric-card rate-card">
                                <div class="metric-header">
                                    <span class="metric-icon">{ "📊" }</span>
                                    <span class="metric-title">{ "Processing Rate" }</span>
                                </div>
                                <div class="metric-value">
                                    { format_rate(current_metrics.processing_rate_hz()) }
                                </div>
                                <div class="metric-details">
                                    <span>{ format!("Target: >40Hz") }</span>
                                </div>
                            </div>
                            
                            <div class="metric-card compliance-card">
                                <div class="metric-header">
                                    <span class="metric-icon">{ "🎯" }</span>
                                    <span class="metric-title">{ "Latency Compliance" }</span>
                                </div>
                                <div class="metric-value">
                                    { if current_metrics.latency_compliant() {
                                        html! { <span class="compliance-pass">{ "✅ PASS" }</span> }
                                    } else {
                                        html! { <span class="compliance-fail">{ "❌ FAIL" }</span> }
                                    }}
                                </div>
                                <div class="metric-details">
                                    <span>{ format!("Target: <{:.0}ms", current_metrics.target_latency_ms()) }</span>
                                </div>
                            </div>
                            
                            <div class="metric-card info-card">
                                <div class="metric-header">
                                    <span class="metric-icon">{ "ℹ️" }</span>
                                    <span class="metric-title">{ "Update Info" }</span>
                                </div>
                                <div class="metric-value">
                                    { format!("#{}", *update_count) }
                                </div>
                                <div class="metric-details">
                                    <span>{ format!("Interval: {}ms", props.update_interval_ms) }</span>
                                    { if let Some(timestamp) = &*last_update {
                                        html! { <span class="last-update">{ format!("Last: {}", &timestamp[11..19]) }</span> }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            </div>
                        </div>
                        
                        <div class="metrics-summary">
                            <h4>{ "Performance Summary" }</h4>
                            <div class="summary-grid">
                                <div class="summary-item">
                                    <span class="summary-label">{ "Overall Status:" }</span>
                                    <span class="summary-value">
                                        { if current_metrics.latency_compliant() && current_metrics.processing_rate_hz() > 40.0 {
                                            "🟢 Excellent"
                                        } else if current_metrics.latency_compliant() || current_metrics.processing_rate_hz() > 20.0 {
                                            "🟡 Good"
                                        } else {
                                            "🔴 Poor"
                                        }}
                                    </span>
                                </div>
                                <div class="summary-item">
                                    <span class="summary-label">{ "Monitoring:" }</span>
                                    <span class="summary-value">
                                        { if *is_monitoring { "Active" } else { "Inactive" } }
                                    </span>
                                </div>
                                <div class="summary-item">
                                    <span class="summary-label">{ "Engine Available:" }</span>
                                    <span class="summary-value">
                                        { if props.audio_engine.is_some() { "Yes" } else { "No" } }
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div class="metrics-placeholder">
                        <div class="placeholder-content">
                            <span class="placeholder-icon">{ "📊" }</span>
                            <p>{ "No metrics available" }</p>
                            <p class="placeholder-hint">
                                { if props.audio_engine.is_some() {
                                    "Initialize and start the audio engine to see metrics"
                                } else {
                                    "Audio engine not available"
                                }}
                            </p>
                        </div>
                    </div>
                }
            }}
        </div>
    }
} 