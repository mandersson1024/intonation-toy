use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::services::{AudioEngineService};
use crate::services::error_manager::ErrorManager;
use crate::components::{AudioControlPanel, MetricsDisplay, DebugPanel, AudioInspector, PerformanceMonitor, TestSignalGenerator};

#[derive(Properties)]
pub struct DebugInterfaceProps {
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    pub error_manager: Option<Rc<RefCell<ErrorManager>>>,
    #[prop_or(1000)]
    pub update_interval_ms: u32,
}

impl PartialEq for DebugInterfaceProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare based on update interval only for simplicity
        // Audio engine and error manager are compared by pointer equality
        self.update_interval_ms == other.update_interval_ms &&
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr()) &&
        self.error_manager.as_ref().map(|e| e.as_ptr()) == other.error_manager.as_ref().map(|e| e.as_ptr())
    }
}

/// Main developer debug interface providing audio processing tools and monitoring
#[function_component(DebugInterface)]
pub fn debug_interface(props: &DebugInterfaceProps) -> Html {
    let interface_active = use_state(|| true);
    
    html! {
        <div class="debug-interface">
            <header class="debug-header">
                <h1>{ "🔧 Developer Debug Interface" }</h1>
                <div class="debug-status">
                    <span class="status-indicator active">{ "● ACTIVE" }</span>
                    <span class="update-rate">{ format!("Update: {}ms", props.update_interval_ms) }</span>
                </div>
            </header>
            
            <div class="debug-layout">
                <div class="debug-section audio-controls">
                    <h2>{ "Audio Engine Controls" }</h2>
                    <AudioControlPanel audio_engine={props.audio_engine.clone()} />
                </div>
                
                <div class="debug-section metrics-display">
                    <h2>{ "Real-time Metrics" }</h2>
                    <MetricsDisplay 
                        audio_engine={props.audio_engine.clone()}
                        update_interval_ms={props.update_interval_ms}
                    />
                </div>
                
                <div class="debug-section error-panel">
                    <h2>{ "Error & Debug States" }</h2>
                    <DebugPanel error_manager={props.error_manager.clone()} />
                </div>
                
                <div class="debug-section audio-inspector">
                    <h2>{ "Audio Data Inspector" }</h2>
                    <AudioInspector 
                        audio_engine={props.audio_engine.clone()}
                        update_interval_ms={props.update_interval_ms}
                        show_raw_buffers={false}
                        show_frequency_data={true}
                        show_pitch_data={true}
                    />
                </div>
                
                <div class="debug-section performance-monitor">
                    <h2>{ "Performance Monitor" }</h2>
                    <PerformanceMonitor 
                        audio_engine={props.audio_engine.clone()}
                        update_interval_ms={props.update_interval_ms}
                        show_memory_stats={true}
                        show_processing_breakdown={true}
                        show_wasm_metrics={true}
                        show_performance_history={true}
                    />
                </div>
                
                <div class="debug-section test-signal-generator-section">
                    <h2>{ "Test Signal Generator" }</h2>
                    <TestSignalGenerator />
                </div>
            </div>
        </div>
    }
} 