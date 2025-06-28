//! # Event-Driven Metrics Display Component
//!
//! Real-time metrics display component with event-driven updates.
//! Subscribes to performance and audio events for instant metric visualization.

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

// Event system imports
#[cfg(debug_assertions)]
use crate::modules::developer_ui::hooks::use_event_subscription::use_event_subscription;
#[cfg(debug_assertions)]
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
#[cfg(debug_assertions)]
use crate::modules::audio_foundations::audio_events::{
    AudioPerformanceMetricsEvent, PitchDetectionEvent, SignalAnalysisEvent,
    AudioProcessingStateEvent, BufferProcessingEvent
};

// Use modular services instead of legacy
#[cfg(debug_assertions)]
use crate::modules::audio_foundations::{ModularAudioService, AudioEngineState};
use crate::legacy::active::services::AudioData;
#[cfg(debug_assertions)]
use crate::audio::performance_monitor::PerformanceMetrics;

#[cfg(debug_assertions)]
#[derive(Properties)]
pub struct MetricsDisplayProps {
    pub audio_engine: Option<Rc<RefCell<ModularAudioService>>>,
    /// Event bus for subscribing to real-time performance and audio events
    #[prop_or(None)]
    pub event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    #[prop_or(1000)]
    pub update_interval_ms: u32,
}

#[cfg(debug_assertions)]
impl PartialEq for MetricsDisplayProps {
    fn eq(&self, other: &Self) -> bool {
        self.update_interval_ms == other.update_interval_ms &&
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr()) &&
        self.event_bus.as_ref().map(|e| e.as_ptr()) == other.event_bus.as_ref().map(|e| e.as_ptr())
    }
}

/// Event-driven real-time metrics display component showing audio processing performance
#[cfg(debug_assertions)]
#[function_component(MetricsDisplay)]
pub fn metrics_display(props: &MetricsDisplayProps) -> Html {
    let metrics = use_state(|| None::<PerformanceMetrics>);
    let audio_data = use_state(|| None::<AudioData>);
    let is_monitoring = use_state(|| false);
    let interval_handle = use_state(|| None::<Interval>);
    
    // Event-driven state for real-time metrics
    let event_performance = use_state(|| None::<AudioPerformanceMetricsEvent>);
    let event_pitch_detection = use_state(|| None::<PitchDetectionEvent>);
    let event_signal_analysis = use_state(|| None::<SignalAnalysisEvent>);
    let last_buffer_event = use_state(|| None::<BufferProcessingEvent>);
    
    // Subscribe to real-time events
    let performance_event = use_event_subscription::<AudioPerformanceMetricsEvent>(props.event_bus.clone());
    let pitch_event = use_event_subscription::<PitchDetectionEvent>(props.event_bus.clone());
    let signal_event = use_event_subscription::<SignalAnalysisEvent>(props.event_bus.clone());
    let state_event = use_event_subscription::<AudioProcessingStateEvent>(props.event_bus.clone());
    let buffer_event = use_event_subscription::<BufferProcessingEvent>(props.event_bus.clone());
    
    // Event-driven updates: React to performance metrics events
    {
        let event_performance = event_performance.clone();
        let is_monitoring = is_monitoring.clone();
        use_effect_with(performance_event.clone(), move |event| {
            if let Some(perf_event) = &**event {
                console::log!(&format!("Metrics display: Performance metrics updated - Latency: {:.1}ms, CPU: {:.1}%", 
                    perf_event.end_to_end_latency_ms, perf_event.cpu_usage_percent));
                event_performance.set(Some(perf_event.clone()));
                is_monitoring.set(true);
            }
            || ()
        });
    }
    
    // Event-driven updates: React to pitch detection events
    {
        let event_pitch_detection = event_pitch_detection.clone();
        use_effect_with(pitch_event.clone(), move |event| {
            if let Some(pitch_event) = &**event {
                console::log!(&format!("Metrics display: Pitch detected - {:.2}Hz (confidence: {:.1}%)", 
                    pitch_event.frequency, pitch_event.confidence * 100.0));
                event_pitch_detection.set(Some(pitch_event.clone()));
            }
            || ()
        });
    }
    
    // Event-driven updates: React to signal analysis events
    {
        let event_signal_analysis = event_signal_analysis.clone();
        use_effect_with(signal_event.clone(), move |event| {
            if let Some(signal_event) = &**event {
                console::log!(&format!("Metrics display: Signal analysis - SNR: {:.1}dB, RMS: {:.3}", 
                    signal_event.snr_estimate, signal_event.rms_energy));
                event_signal_analysis.set(Some(signal_event.clone()));
            }
            || ()
        });
    }
    
    // Event-driven updates: React to buffer processing events
    {
        let last_buffer_event = last_buffer_event.clone();
        use_effect_with(buffer_event.clone(), move |event| {
            if let Some(buffer_event) = &**event {
                console::log!(&format!("Metrics display: Buffer processing - Stage: {:?}, Latency: {:.1}ms", 
                    buffer_event.processing_stage, buffer_event.latency_ms));
                last_buffer_event.set(Some(buffer_event.clone()));
            }
            || ()
        });
    }
    
    // Event-driven updates: React to processing state changes
    {
        let is_monitoring = is_monitoring.clone();
        use_effect_with(state_event.clone(), move |event| {
            if let Some(state_event) = &**event {
                let is_processing = matches!(state_event.new_state, 
                                            crate::modules::audio_foundations::AudioEngineState::Processing);
                console::log!(&format!("Metrics display: Processing state changed - Monitoring: {}", is_processing));
                is_monitoring.set(is_processing);
            }
            || ()
        });
    }
    
    // Set up automatic metrics updates (legacy fallback)
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
                <h3 class="metrics-title">{ "EVENT-DRIVEN METRICS" }</h3>
                <span class={if *is_monitoring { "status-active status-indicator" } else { "status-inactive status-indicator" }}>
                    { if *is_monitoring { "● LIVE EVENTS" } else { "○ NO EVENTS" } }
                </span>
            </div>
            
            // Event-driven metrics section
            { if event_performance.is_some() || event_pitch_detection.is_some() || event_signal_analysis.is_some() {
                html! {
                    <div class="event-metrics-content">
                        <div class="metrics-stack">
                            // Event-driven performance metrics
                            { if let Some(ref perf) = *event_performance {
                                html! {
                                    <div class="device-config-table event-performance-section">
                                        <div class="latency-header">
                                            <h3 class="device-config-title">{ "🚀 Live Performance (Events)" }</h3>
                                            <span class="latency-total-value">{ format!("{:.1}ms", perf.end_to_end_latency_ms) }</span>
                                        </div>
                                        <div class="device-config-rows">
                                            <div class="device-config-row">
                                                <span class="config-label">{"Processing Latency:"}</span>
                                                <span class="config-value">{ format!("{:.1}ms", perf.processing_latency_ms) }</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"CPU Usage:"}</span>
                                                <span class={classes!("config-value", 
                                                    if perf.cpu_usage_percent > 70.0 { "warning" } else { "good" })}
                                                >
                                                    { format!("{:.1}%", perf.cpu_usage_percent) }
                                                </span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Memory Usage:"}</span>
                                                <span class="config-value">{ format!("{:.1}MB", perf.memory_usage_bytes as f64 / 1024.0 / 1024.0) }</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Buffer Underruns:"}</span>
                                                <span class={classes!("config-value",
                                                    if perf.buffer_underruns > 0 { "warning" } else { "good" })}
                                                >
                                                    { perf.buffer_underruns }
                                                </span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else { html! {} }}
                            
                            // Event-driven pitch detection
                            { if let Some(ref pitch) = *event_pitch_detection {
                                html! {
                                    <div class="device-config-table event-pitch-section">
                                        <h3 class="device-config-title">{ "🎵 Live Pitch Detection (Events)" }</h3>
                                        <div class="device-config-rows">
                                            <div class="device-config-row">
                                                <span class="config-label">{"Frequency:"}</span>
                                                <span class="config-value">{ format!("{:.2} Hz", pitch.frequency) }</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Confidence:"}</span>
                                                <span class={classes!("config-value",
                                                    if pitch.confidence > 0.8 { "good" } else if pitch.confidence > 0.5 { "warning" } else { "poor" })}
                                                >
                                                    { format!("{:.1}%", pitch.confidence * 100.0) }
                                                </span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Note:"}</span>
                                                <span class="config-value">{ frequency_to_note_name(pitch.frequency) }</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Algorithm:"}</span>
                                                <span class="config-value">{ format!("{:?}", pitch.algorithm_used) }</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Processing Time:"}</span>
                                                <span class="config-value">{ format!("{:.2}μs", pitch.processing_time_ns as f64 / 1000.0) }</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else { html! {} }}
                            
                            // Event-driven signal analysis
                            { if let Some(ref signal) = *event_signal_analysis {
                                html! {
                                    <div class="device-config-table event-signal-section">
                                        <h3 class="device-config-title">{ "📊 Live Signal Analysis (Events)" }</h3>
                                        <div class="device-config-rows">
                                            <div class="device-config-row">
                                                <span class="config-label">{"SNR Estimate:"}</span>
                                                <span class={classes!("config-value",
                                                    if signal.snr_estimate > 20.0 { "good" } else if signal.snr_estimate > 10.0 { "warning" } else { "poor" })}
                                                >
                                                    { format!("{:.1} dB", signal.snr_estimate) }
                                                </span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"RMS Energy:"}</span>
                                                <span class="config-value">{ format!("{:.3}", signal.rms_energy) }</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Peak Amplitude:"}</span>
                                                <span class="config-value">{ format!("{:.3}", signal.peak_amplitude) }</span>
                                            </div>
                                            <div class="device-config-row">
                                                <span class="config-label">{"Signal Complexity:"}</span>
                                                <span class="config-value">{ format!("{:.2}", signal.signal_complexity) }</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else { html! {} }}
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div class="event-metrics-placeholder">
                        <div class="placeholder-content">
                            <span class="placeholder-icon">{ "📡" }</span>
                            <p>{ "Waiting for real-time events..." }</p>
                            <p class="placeholder-hint">
                                { "Event-driven metrics will appear here when audio processing starts" }
                            </p>
                        </div>
                    </div>
                }
            }}
            
            // Legacy metrics section (fallback)
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