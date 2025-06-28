use yew::prelude::*;
use web_sys::console;
use std::rc::Rc;
use std::cell::RefCell;
use crate::legacy::active::services::audio_engine::AudioEngineService;
use gloo::timers::callback::Interval;

#[derive(Properties)]
pub struct PerformanceMonitorProps {
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
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

#[derive(Clone, Debug)]
pub struct MemoryStats {
    pub used_heap_mb: f64,
    pub total_heap_mb: f64,
    pub heap_utilization: f64,
    pub gc_count: u32,
    pub buffer_allocations: u32,
    pub timestamp: f64,
}

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

#[derive(Clone, Debug)]
pub struct PerformanceHistoryEntry {
    pub timestamp: f64,
    pub overall_score: f64,
    pub latency_ms: f64,
    pub throughput: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage: f64,
}

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
                                                            style={format!("width: {:.1}%", 
                                                                (processing.audio_processing_ms / processing.total_processing_ms) * 100.0)}
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
                                                            style={format!("width: {:.1}%", 
                                                                (processing.pitch_detection_ms / processing.total_processing_ms) * 100.0)}
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
                                                            style={format!("width: {:.1}%", 
                                                                (processing.fft_computation_ms / processing.total_processing_ms) * 100.0)}
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
                                                            style={format!("width: {:.1}%", 
                                                                (processing.buffer_management_ms / processing.total_processing_ms) * 100.0)}
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
                                        <p class="hint">{ "Start monitoring to see processing times" }</p>
                                    </div>
                                }
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                { if props.show_wasm_metrics {
                    html! {
                        <div class="monitor-section">
                            <h4>{ "🦀 WASM Performance Metrics" }</h4>
                            { if let Some(wasm) = wasm_metrics.as_ref() {
                                html! {
                                    <div class="wasm-metrics">
                                        <div class="metrics-grid">
                                            <div class="metric-group">
                                                <h5>{ "Initialization" }</h5>
                                                <div class="metric-item">
                                                    <span class="metric-label">{ "Compilation:" }</span>
                                                    <span class="metric-value">{ format!("{:.1} ms", wasm.compilation_time_ms) }</span>
                                                </div>
                                                <div class="metric-item">
                                                    <span class="metric-label">{ "Instantiation:" }</span>
                                                    <span class="metric-value">{ format!("{:.1} ms", wasm.instantiation_time_ms) }</span>
                                                </div>
                                            </div>
                                            
                                            <div class="metric-group">
                                                <h5>{ "Memory" }</h5>
                                                <div class="metric-item">
                                                    <span class="metric-label">{ "Pages:" }</span>
                                                    <span class="metric-value">{ wasm.memory_pages }</span>
                                                </div>
                                                <div class="metric-item">
                                                    <span class="metric-label">{ "Size:" }</span>
                                                    <span class="metric-value">{ format!("{:.1} MB", wasm.memory_size_mb) }</span>
                                                </div>
                                            </div>
                                            
                                            <div class="metric-group">
                                                <h5>{ "Performance" }</h5>
                                                <div class="metric-item">
                                                    <span class="metric-label">{ "Function Calls:" }</span>
                                                    <span class="metric-value">{ format!("{:.0}/s", wasm.function_calls_per_sec) }</span>
                                                </div>
                                                <div class="metric-item">
                                                    <span class="metric-label">{ "CPU Usage:" }</span>
                                                    <span class="metric-value">{ format!("{:.1}%", wasm.wasm_cpu_usage * 100.0) }</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="no-data">
                                        <p>{ "No WASM metrics available" }</p>
                                        <p class="hint">{ "Start monitoring to see WASM performance" }</p>
                                    </div>
                                }
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                { if props.show_performance_history {
                    html! {
                        <div class="monitor-section">
                            <h4>{ "📊 Performance History" }</h4>
                            { if !performance_history.is_empty() {
                                let latest_entry = performance_history.last().unwrap();
                                let (grade_icon, grade_text, grade_class) = get_performance_grade(latest_entry.overall_score);
                                
                                html! {
                                    <div class="performance-history">
                                        <div class="history-summary">
                                            <div class="current-score">
                                                <span class="score-icon">{ grade_icon }</span>
                                                <span class="score-text">{ format!("Overall Score: {:.1}", latest_entry.overall_score) }</span>
                                                <span class={classes!("score-grade", grade_class)}>{ grade_text }</span>
                                            </div>
                                        </div>
                                        
                                        <div class="history-list">
                                            <div class="history-header">
                                                <span>{ "Time" }</span>
                                                <span>{ "Score" }</span>
                                                <span>{ "Latency" }</span>
                                                <span>{ "Memory" }</span>
                                                <span>{ "CPU" }</span>
                                            </div>
                                            { for performance_history.iter().rev().take(10).map(|entry| {
                                                let (entry_icon, _, entry_class) = get_performance_grade(entry.overall_score);
                                                html! {
                                                    <div class="history-entry">
                                                        <span class="entry-time">
                                                            { format!("{:.0}ms", entry.timestamp % 100000.0) }
                                                        </span>
                                                        <span class={classes!("entry-score", entry_class)}>
                                                            { format!("{} {:.1}", entry_icon, entry.overall_score) }
                                                        </span>
                                                        <span class="entry-latency">
                                                            { format!("{:.1}ms", entry.latency_ms) }
                                                        </span>
                                                        <span class="entry-memory">
                                                            { format!("{:.1}MB", entry.memory_usage_mb) }
                                                        </span>
                                                        <span class="entry-cpu">
                                                            { format!("{:.1}%", entry.cpu_usage * 100.0) }
                                                        </span>
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="no-data">
                                        <p>{ "No performance history available" }</p>
                                        <p class="hint">{ "Start monitoring to track performance over time" }</p>
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

// Helper function to calculate performance score
fn calculate_performance_score(memory: &MemoryStats, processing: &ProcessingBreakdown) -> f64 {
    let memory_score = (1.0 - memory.heap_utilization) * 100.0;
    let latency_score = if processing.total_processing_ms <= 5.0 { 100.0 } 
                       else if processing.total_processing_ms <= 10.0 { 80.0 }
                       else { 60.0 - (processing.total_processing_ms - 10.0) * 2.0 };
    let cpu_score = (1.0 - processing.cpu_utilization) * 100.0;
    
    // Weighted average: 30% memory, 50% latency, 20% CPU
    (memory_score * 0.3 + latency_score * 0.5 + cpu_score * 0.2).max(0.0).min(100.0)
} 