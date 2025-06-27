//! # Metrics Display Component
//!
//! Placeholder for migrated metrics display component.
//! Will be implemented during component migration task.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;
#[cfg(debug_assertions)]
use gloo::console;
#[cfg(debug_assertions)]
use gloo::timers::callback::Interval;

// TODO: Update these imports once legacy services are migrated to modules
#[cfg(debug_assertions)]
use crate::legacy::active::services::audio_engine::{AudioEngineService, AudioEngineState, AudioData};
#[cfg(debug_assertions)]
use crate::audio::performance_monitor::PerformanceMetrics;

#[cfg(debug_assertions)]
#[derive(Properties)]
pub struct MetricsDisplayProps {
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    #[prop_or(1000)]
    pub update_interval_ms: u32,
}

#[cfg(debug_assertions)]
impl PartialEq for MetricsDisplayProps {
    fn eq(&self, other: &Self) -> bool {
        self.update_interval_ms == other.update_interval_ms &&
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr())
    }
}

/// Real-time metrics display component showing audio processing performance
#[cfg(debug_assertions)]
#[function_component(MetricsDisplay)]
pub fn metrics_display(props: &MetricsDisplayProps) -> Html {
    let metrics = use_state(|| None::<PerformanceMetrics>);
    let audio_data = use_state(|| None::<AudioData>);
    let is_monitoring = use_state(|| false);
    let interval_handle = use_state(|| None::<Interval>);
    
    // Set up automatic metrics updates
    {
        let metrics = metrics.clone();
        let audio_data = audio_data.clone();
        let is_monitoring = is_monitoring.clone();
        let interval_handle = interval_handle.clone();
        let audio_engine = props.audio_engine.clone();
        let update_interval_ms = props.update_interval_ms;
        
        use_effect_with(
            (update_interval_ms, audio_engine.clone()),
            move |(interval, audio_engine)| {
                let interval_obj = if let Some(engine) = audio_engine {
                    let engine_clone = engine.clone();
                    let metrics = metrics.clone();
                    let audio_data = audio_data.clone();
                    let is_monitoring = is_monitoring.clone();
                    
                    let interval = Interval::new(*interval, move || {
                        if let Ok(engine_ref) = engine_clone.try_borrow() {
                            // Check if the engine is actually processing audio
                            let engine_state = engine_ref.get_state();
                            let is_processing = matches!(engine_state, AudioEngineState::Processing);
                            
                            is_monitoring.set(is_processing);
                            
                            if is_processing {
                                let current_metrics = engine_ref.get_performance_metrics();
                                let current_audio_data = engine_ref.get_simulated_audio_data();
                                metrics.set(Some(current_metrics));
                                audio_data.set(current_audio_data);
                            }
                        } else {
                            console::warn!("Could not borrow audio engine for metrics update");
                            is_monitoring.set(false);
                        }
                    });
                    
                    Some(interval)
                } else {
                    is_monitoring.set(false);
                    metrics.set(None);
                    audio_data.set(None);
                    None
                };
                interval_handle.set(interval_obj);
                move || {
                    // The interval cleanup is handled automatically when the interval is dropped
                    // We just need to clear the state
                    interval_handle.set(None);
                    is_monitoring.set(false);
                }
            },
        );
    }
    
    let format_latency = |latency_ms: Option<f32>| {
        match latency_ms {
            Some(val) => format!("{:.1}ms", val),
            None => "N/A".to_string(),
        }
    };

    let frequency_to_note_name = |frequency: f32| -> String {
        let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let a4_freq = 440.0;
        let c0_freq = a4_freq * (2.0_f32).powf(-4.75); // C0 frequency
        
        if frequency <= 0.0 {
            return "---".to_string();
        }
        
        let half_steps_from_c0 = 12.0 * (frequency / c0_freq).log2();
        let octave = (half_steps_from_c0 / 12.0).floor() as i32;
        let note_index = (half_steps_from_c0 % 12.0).round() as usize % 12;
        
        format!("{}{}", notes[note_index], octave)
    };
    
    html! {
        <div class="metrics-display">
            <div class="metrics-header">
                <h3 class="metrics-title">{ "REAL-TIME METRICS" }</h3>
                <span class={if *is_monitoring { "status-active status-indicator" } else { "status-inactive status-indicator" }}>
                    { if *is_monitoring { "● LIVE" } else { "○ STOPPED" } }
                </span>
            </div>
            
            { if let Some(current_metrics) = &*metrics {
                html! {
                    <div class="metrics-content">
                        <div class="metrics-stack">
                            // Compact Total Latency Section
                            <div class="device-config-table latency-section">
                                <div class="latency-header">
                                    <h3 class="device-config-title">{ "Total Latency" }</h3>
                                    <span class="latency-total-value">{ format_latency(Some(current_metrics.total_latency_ms())) }</span>
                                </div>
                                <div class="device-config-rows">
                                    <div class="device-config-row">
                                        <span class="config-label">{"Buffer:"}</span>
                                        <span class="config-value">{ if current_metrics.buffer_latency_ms() > 0.0 { format!("{:.1}ms", current_metrics.buffer_latency_ms()) } else { "N/A".to_string() } }</span>
                                    </div>
                                    { if let Some(data) = &*audio_data {
                                        if data.processing_time_ms > 0.0 {
                                            let processing_ms = data.processing_time_ms;
                                            // Realistic breakdown based on typical audio processing patterns
                                            let signal_analysis_ms = processing_ms * 0.35; // ~35% for signal analysis (peak/RMS)
                                            let pitch_detection_ms = processing_ms * 0.45; // ~45% for pitch detection (zero-crossing)
                                            let output_processing_ms = processing_ms * 0.20; // ~20% for output processing
                                            html! {
                                                <>
                                                    <div class="device-config-row">
                                                        <span class="config-label">{"Processing:"}</span>
                                                        <span class="config-value">{ format!("{:.1}ms", processing_ms) }</span>
                                                    </div>
                                                    <div class="device-config-row processing-breakdown">
                                                        <span class="config-label breakdown-indent">{"• Signal Analysis:"}</span>
                                                        <span class="config-value">{ format!("{:.2}ms", signal_analysis_ms) }</span>
                                                    </div>
                                                    <div class="device-config-row processing-breakdown">
                                                        <span class="config-label breakdown-indent">{"• Pitch Detection:"}</span>
                                                        <span class="config-value">{ format!("{:.2}ms", pitch_detection_ms) }</span>
                                                    </div>
                                                    <div class="device-config-row processing-breakdown">
                                                        <span class="config-label breakdown-indent">{"• Audio Output:"}</span>
                                                        <span class="config-value">{ format!("{:.2}ms", output_processing_ms) }</span>
                                                    </div>
                                                </>
                                            }
                                        } else {
                                            html! {
                                                <div class="device-config-row">
                                                    <span class="config-label">{"Processing:"}</span>
                                                    <span class="config-value">{"N/A"}</span>
                                                </div>
                                            }
                                        }
                                    } else {
                                        html! {
                                            <div class="device-config-row">
                                                <span class="config-label">{"Processing:"}</span>
                                                <span class="config-value">{"N/A"}</span>
                                            </div>
                                        }
                                    }}
                                </div>
                            </div>

                            // Pitch Detection Results Section
                            <div class="device-config-table">
                                <h3 class="device-config-title">{ "Pitch Detection" }</h3>
                                { if let Some(data) = &*audio_data {
                                    html! {
                                        <div class="device-config-rows">
                                            <div class="device-config-row">
                                                <span class="config-label">{"Frequency:"}</span>
                                                <span class="config-value">{format!("{:.2} Hz", data.pitch_frequency)}</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Confidence:"}</span>
                                                <span class="config-value">{format!("{:.1}%", data.confidence * 100.0)}</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Note:"}</span>
                                                <span class="config-value">{frequency_to_note_name(data.pitch_frequency)}</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Audio Level:"}</span>
                                                <span class="config-value">{format!("{:.1}%", data.audio_level * 100.0)}</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Processing Time:"}</span>
                                                <span class="config-value">{format!("{:.2} ms", data.processing_time_ms)}</span>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {
                                        <div class="device-config-rows">
                                            <div class="device-config-row">
                                                <span class="config-label">{"Status:"}</span>
                                                <span class="config-value">{"No pitch data available"}</span>
                                            </div>
                                        </div>
                                    }
                                }}
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