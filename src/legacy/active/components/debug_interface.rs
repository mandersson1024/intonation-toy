use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::legacy::active::services::{AudioEngineService};
use crate::legacy::active::services::audio_engine::AudioEngineState;
use crate::legacy::active::services::error_manager::ErrorManager;
use crate::legacy::active::components::{AudioControlPanel, MetricsDisplay, DebugPanel, TestSignalGenerator};

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
    let state_check_counter = use_state(|| 0);
    let generator_is_generating = use_state(|| false);
    
    // Force re-check of engine state periodically
    {
        let state_check_counter = state_check_counter.clone();
        use_effect_with(
            props.audio_engine.clone(),
            move |_| {
                // Set up interval to periodically force re-render to check state
                let counter_clone = state_check_counter.clone();
                let interval = gloo::timers::callback::Interval::new(1000, move || {
                    counter_clone.set(*counter_clone + 1);
                });
                
                move || drop(interval)
            },
        );
    }
    
    // Get detailed engine state for display
    let (status_icon, status_text, status_class) = if let Some(audio_engine) = &props.audio_engine {
        if let Ok(engine_ref) = audio_engine.try_borrow() {
            let current_state = engine_ref.get_state();
            
            match current_state {
                AudioEngineState::Uninitialized => ("●", "UNINITIALIZED", "status-inactive"),
                AudioEngineState::Initializing => ("●", "INITIALIZING", "status-warning"),
                AudioEngineState::Ready => ("●", "READY", "status-active"),
                AudioEngineState::Processing => ("●", "PROCESSING", "status-active"),
                AudioEngineState::Suspended => ("●", "SUSPENDED", "status-warning"),
                AudioEngineState::Error(ref msg) => ("●", "ERROR", "status-error"),
            }
        } else {
            web_sys::console::log_1(&"🔍 Debug: Cannot borrow AudioEngine".into());
            ("○", "UNAVAILABLE", "status-error")
        }
    } else {
        web_sys::console::log_1(&"🔍 Debug: No AudioEngine available".into());
        ("○", "NO ENGINE", "status-inactive")
    };

    // Callback for test signal generator state changes
    let on_generation_state_change = {
        let generator_is_generating = generator_is_generating.clone();
        Callback::from(move |is_generating: bool| {
            generator_is_generating.set(is_generating);
        })
    };
    
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
                    <div class="metrics-header">
                        <h3 class="metrics-title">{ "AUDIO ENGINE" }</h3>
                        <span class={format!("{} status-indicator", status_class)}>
                            { format!("{} {}", status_icon, status_text) }
                        </span>
                    </div>
                    <AudioControlPanel 
                        audio_engine={props.audio_engine.clone()}
                        error_manager={props.error_manager.clone()}
                    />
                </div>
                
                <div class="debug-section metrics-display">
                    <MetricsDisplay 
                        audio_engine={props.audio_engine.clone()}
                        update_interval_ms={props.update_interval_ms}
                    />
                </div>
                
                <div class="debug-section error-panel">
                    <h2>{ "Errors" }</h2>
                    <DebugPanel 
                        error_manager={props.error_manager.clone()}
                        update_interval_ms={props.update_interval_ms}
                        auto_refresh={true}
                    />
                </div>
                

                <div class="debug-section test-signal-generator-section">
                    <div class="metrics-header">
                        <h3 class="metrics-title">{ "TEST SIGNAL GENERATOR" }</h3>
                        <span class={if *generator_is_generating { "status-active status-indicator" } else { "status-inactive status-indicator" }}>
                            { if *generator_is_generating { "● GENERATING" } else { "○ READY" } }
                        </span>
                    </div>
                    <TestSignalGenerator 
                        audio_engine={props.audio_engine.clone()}
                        on_generation_state_change={Some(on_generation_state_change)}
                    />
                </div>
            </div>
        </div>
    }
} 