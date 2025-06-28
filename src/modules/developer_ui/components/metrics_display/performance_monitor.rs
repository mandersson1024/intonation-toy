//! # Performance Monitor Component
//!
//! Placeholder for migrated performance monitor component.
//! Will be implemented during component migration task.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use web_sys::console;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;
#[cfg(debug_assertions)]
use gloo::timers::callback::Interval;

// Use modular services instead of legacy
#[cfg(debug_assertions)]
use crate::modules::audio_foundations::ModularAudioService;

#[cfg(debug_assertions)]
#[derive(Properties)]
pub struct PerformanceMonitorProps {
    pub audio_engine: Option<Rc<RefCell<ModularAudioService>>>,
    #[prop_or(1000)]
    pub update_interval_ms: u32,
    #[prop_or(true)]
    pub show_memory_stats: bool,
    #[prop_or(true)]
    pub show_processing_breakdown: bool,
    #[prop_or(true)]
    pub show_wasm_metrics: bool,
    #[prop_or(true)]
    pub show_performance_history: bool,
}

#[cfg(debug_assertions)]
impl PartialEq for PerformanceMonitorProps {
    fn eq(&self, other: &Self) -> bool {
        self.update_interval_ms == other.update_interval_ms 
            && self.show_memory_stats == other.show_memory_stats
            && self.show_processing_breakdown == other.show_processing_breakdown
            && self.show_wasm_metrics == other.show_wasm_metrics
            && self.show_performance_history == other.show_performance_history
            && self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr())
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug)]
pub struct MemoryStats {
    pub used_heap_mb: f64,
    pub total_heap_mb: f64,
    pub heap_utilization: f64,
    pub gc_count: u32,
    pub buffer_allocations: u32,
    pub timestamp: f64,
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug)]
pub struct ProcessingBreakdown {
    pub audio_processing_ms: f64,
    pub pitch_detection_ms: f64,
    pub fft_computation_ms: f64,
    pub buffer_management_ms: f64,
    pub total_processing_ms: f64,
    pub cpu_utilization: f64,
    pub timestamp: f64,
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug)]
pub struct WasmMetrics {
    pub compilation_time_ms: f64,
    pub instantiation_time_ms: f64,
    pub memory_pages: u32,
    pub memory_size_mb: f64,
    pub function_calls_per_sec: f64,
    pub wasm_cpu_usage: f64,
    pub timestamp: f64,
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug)]
pub struct PerformanceHistoryEntry {
    pub timestamp: f64,
    pub overall_score: f64,
    pub latency_ms: f64,
    pub throughput: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage: f64,
}

#[cfg(debug_assertions)]
#[function_component(PerformanceMonitor)]
pub fn performance_monitor(props: &PerformanceMonitorProps) -> Html {
    let memory_stats = use_state(|| None::<MemoryStats>);
    let processing_breakdown = use_state(|| None::<ProcessingBreakdown>);
    let wasm_metrics = use_state(|| None::<WasmMetrics>);
    let performance_history = use_state(|| Vec::<PerformanceHistoryEntry>::new());
    let _interval_handle = use_state(|| None::<Interval>);
    
    // Start monitoring automatically on mount
    {
        let memory_stats = memory_stats.clone();
        let processing_breakdown = processing_breakdown.clone();
        let wasm_metrics = wasm_metrics.clone();
        let performance_history = performance_history.clone();
        let interval_handle = _interval_handle.clone();
        let update_interval = props.update_interval_ms;
        use_effect_with(
            (),
            move |_| {
                let interval = Interval::new(update_interval, move || {
                    let current_time = js_sys::Date::now();
                    let mock_memory = MemoryStats {
                        used_heap_mb: 12.5 + (current_time / 5000.0).sin() as f64 * 2.0,
                        total_heap_mb: 32.0,
                        heap_utilization: 0.4 + (current_time / 8000.0).sin() as f64 * 0.1,
                        gc_count: ((current_time / 10000.0) as u32) % 50,
                        buffer_allocations: ((current_time / 1000.0) as u32) % 100,
                        timestamp: current_time,
                    };
                    let mock_processing = ProcessingBreakdown {
                        audio_processing_ms: 2.5 + (current_time / 3000.0).sin() as f64 * 0.5,
                        pitch_detection_ms: 1.8 + (current_time / 4000.0).cos() as f64 * 0.3,
                        fft_computation_ms: 1.2 + (current_time / 2500.0).sin() as f64 * 0.2,
                        buffer_management_ms: 0.8 + (current_time / 6000.0).cos() as f64 * 0.1,
                        total_processing_ms: 6.3 + (current_time / 3500.0).sin() as f64 * 1.0,
                        cpu_utilization: 0.25 + (current_time / 7000.0).sin() as f64 * 0.1,
                        timestamp: current_time,
                    };
                    let mock_wasm = WasmMetrics {
                        compilation_time_ms: 45.2,
                        instantiation_time_ms: 12.8,
                        memory_pages: 256,
                        memory_size_mb: 16.0,
                        function_calls_per_sec: 1200.0 + (current_time / 2000.0).sin() as f64 * 200.0,
                        wasm_cpu_usage: 0.15 + (current_time / 9000.0).cos() as f64 * 0.05,
                        timestamp: current_time,
                    };
                    memory_stats.set(Some(mock_memory.clone()));
                    processing_breakdown.set(Some(mock_processing.clone()));
                    wasm_metrics.set(Some(mock_wasm));
                    let mut history = (*performance_history).clone();
                    let overall_score = calculate_performance_score(&mock_memory, &mock_processing);
                    let history_entry = PerformanceHistoryEntry {
                        timestamp: current_time,
                        overall_score,
                        latency_ms: mock_processing.total_processing_ms,
                        throughput: 1000.0 / mock_processing.total_processing_ms,
                        memory_usage_mb: mock_memory.used_heap_mb,
                        cpu_usage: mock_processing.cpu_utilization,
                    };
                    history.push(history_entry);
                    if history.len() > 20 {
                        history.remove(0);
                    }
                    performance_history.set(history);
                });
                interval_handle.set(Some(interval));
                || ()
            },
        );
    }
    
    // Calculate performance grade
    let get_performance_grade = |score: f64| {
        if score >= 90.0 { ("🟢", "Excellent", "grade-excellent") }
        else if score >= 75.0 { ("🟡", "Good", "grade-good") }
        else if score >= 60.0 { ("🟠", "Fair", "grade-fair") }
        else { ("🔴", "Poor", "grade-poor") }
    };
    
    html! {
        <div class="performance-monitor">
            <div class="monitor-content">
                { if props.show_memory_stats {
                    html! {
                        <div class="monitor-section">
                            <h4>{ "💾 Memory Usage Statistics" }</h4>
                            { if let Some(memory) = memory_stats.as_ref() {
                                html! {
                                    <div class="monitor-section">
                                        <h4>{ "💾 Memory Usage Statistics" }</h4>
                                        <div class="memory-stats">
                                            <div class="stats-grid">
                                                <div class="stat-card">
                                                    <div class="stat-header">
                                                        <span class="stat-icon">{ "🧠" }</span>
                                                        <span class="stat-title">{ "Heap Usage" }</span>
                                                    </div>
                                                    <div class="stat-value">
                                                        { format!("{:.1} MB", memory.used_heap_mb) }
                                                    </div>
                                                    <div class="stat-detail">
                                                        { format!("of {:.1} MB ({:.1}%)", 
                                                            memory.total_heap_mb, 
                                                            memory.heap_utilization * 100.0) }
                                                    </div>
                                                    <div class="progress-bar">
                                                        <div 
                                                            class="progress-fill"
                                                            style={format!("width: {:.1}%", memory.heap_utilization * 100.0)}
                                                        ></div>
                                                    </div>
                                                </div>
                                                
                                                <div class="stat-card">
                                                    <div class="stat-header">
                                                        <span class="stat-icon">{ "🔄" }</span>
                                                        <span class="stat-title">{ "GC Collections" }</span>
                                                    </div>
                                                    <div class="stat-value">
                                                        { memory.gc_count }
                                                    </div>
                                                    <div class="stat-detail">{ "total collections" }</div>
                                                </div>
                                                
                                                <div class="stat-card">
                                                    <div class="stat-header">
                                                        <span class="stat-icon">{ "📦" }</span>
                                                        <span class="stat-title">{ "Buffer Allocations" }</span>
                                                    </div>
                                                    <div class="stat-value">
                                                        { memory.buffer_allocations }
                                                    </div>
                                                    <div class="stat-detail">{ "active buffers" }</div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="no-data">
                                        <p>{ "No memory statistics available" }</p>
                                        <p class="hint">{ "Start monitoring to see memory usage" }</p>
                                    </div>
                                }
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                { if props.show_processing_breakdown {
                    html! {
                        <div class="monitor-section">
                            <h4>{ "⚡ Processing Time Breakdown" }</h4>
                            { if let Some(processing) = processing_breakdown.as_ref() {
                                html! {
                                    <div class="processing-breakdown">
                                        <div class="breakdown-summary">
                                            <div class="total-time">
                                                <span class="label">{ "Total Processing Time:" }</span>
                                                <span class="value">{ format!("{:.2} ms", processing.total_processing_ms) }</span>
                                            </div>
                                            <div class="cpu-usage">
                                                <span class="label">{ "CPU Utilization:" }</span>
                                                <span class="value">{ format!("{:.1}%", processing.cpu_utilization * 100.0) }</span>
                                            </div>
                                        </div>
                                        
                                        <div class="breakdown-chart">
                                            <div class="chart-item">
                                                <div class="chart-bar">
                                                    <div class="bar-label">{ "Audio Processing" }</div>
                                                    <div class="bar-container">
                                                        <div 
                                                            class="bar-fill audio-processing"
                                                            style={format!("width: {:.1}%", (processing.audio_processing_ms / processing.total_processing_ms) * 100.0)}
                                                        ></div>
                                                    </div>
                                                    <div class="bar-value">{ format!("{:.2}ms", processing.audio_processing_ms) }</div>
                                                </div>
                                            </div>
                                            
                                            <div class="chart-item">
                                                <div class="chart-bar">
                                                    <div class="bar-label">{ "Pitch Detection" }</div>
                                                    <div class="bar-container">
                                                        <div 
                                                            class="bar-fill pitch-detection"
                                                            style={format!("width: {:.1}%", (processing.pitch_detection_ms / processing.total_processing_ms) * 100.0)}
                                                        ></div>
                                                    </div>
                                                    <div class="bar-value">{ format!("{:.2}ms", processing.pitch_detection_ms) }</div>
                                                </div>
                                            </div>
                                            
                                            <div class="chart-item">
                                                <div class="chart-bar">
                                                    <div class="bar-label">{ "FFT Computation" }</div>
                                                    <div class="bar-container">
                                                        <div 
                                                            class="bar-fill fft-computation"
                                                            style={format!("width: {:.1}%", (processing.fft_computation_ms / processing.total_processing_ms) * 100.0)}
                                                        ></div>
                                                    </div>
                                                    <div class="bar-value">{ format!("{:.2}ms", processing.fft_computation_ms) }</div>
                                                </div>
                                            </div>
                                            
                                            <div class="chart-item">
                                                <div class="chart-bar">
                                                    <div class="bar-label">{ "Buffer Management" }</div>
                                                    <div class="bar-container">
                                                        <div 
                                                            class="bar-fill buffer-management"
                                                            style={format!("width: {:.1}%", (processing.buffer_management_ms / processing.total_processing_ms) * 100.0)}
                                                        ></div>
                                                    </div>
                                                    <div class="bar-value">{ format!("{:.2}ms", processing.buffer_management_ms) }</div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="no-data">
                                        <p>{ "No processing breakdown available" }</p>
                                        <p class="hint">{ "Start monitoring to see processing metrics" }</p>
                                    </div>
                                }
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

#[cfg(debug_assertions)]
fn calculate_performance_score(memory: &MemoryStats, processing: &ProcessingBreakdown) -> f64 {
    // Performance calculation based on memory usage and processing efficiency
    let memory_score = (1.0 - memory.heap_utilization) * 50.0; // 50 points for memory efficiency
    let processing_score = (10.0 / processing.total_processing_ms) * 50.0; // 50 points for processing speed
    
    (memory_score + processing_score).min(100.0).max(0.0)
} 