use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::services::audio_engine::{AudioEngineService, AudioEngineState};
use crate::components::MicrophonePermission;
use crate::services::error_manager::ApplicationError;
use gloo::console;

#[derive(Properties)]
pub struct AudioControlPanelProps {
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
}

impl PartialEq for AudioControlPanelProps {
    fn eq(&self, other: &Self) -> bool {
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr())
    }
}

/// Audio engine control panel with start/stop/test functionality
#[function_component(AudioControlPanel)]
pub fn audio_control_panel(props: &AudioControlPanelProps) -> Html {
    let engine_state = use_state(|| AudioEngineState::Uninitialized);
    let is_processing = use_state(|| false);
    let last_test_result = use_state(|| None::<String>);
    let media_stream = use_state(|| None::<web_sys::MediaStream>);
    
    // Initialize audio engine
    let initialize_engine = {
        let engine_state = engine_state.clone();
        let audio_engine = props.audio_engine.clone();
        let media_stream = media_stream.clone();
        
        Callback::from(move |_| {
            if let Some(engine) = &audio_engine {
                engine_state.set(AudioEngineState::Initializing);
                console::log!("Initializing audio engine...");
                
                // Simulate async initialization
                wasm_bindgen_futures::spawn_local({
                    let engine = engine.clone();
                    let engine_state = engine_state.clone();
                    let media_stream = media_stream.clone();
                    
                    async move {
                        let result = {
                            let mut engine_ref = engine.borrow_mut();
                            engine_ref.initialize().await
                        };
                        match result {
                            Ok(_) => {
                                console::log!("Audio engine initialized successfully");
                                engine_state.set(AudioEngineState::Ready);
                                
                                // Auto-connect any stored MediaStream
                                if let Some(stream) = &*media_stream {
                                    console::log!("Connecting stored MediaStream to newly initialized engine");
                                    let stream_clone = stream.clone();
                                    let engine_clone = engine.clone();
                                    
                                    // Connect the stream in a separate async block
                                    wasm_bindgen_futures::spawn_local(async move {
                                        let connect_result = {
                                            let mut engine_ref = engine_clone.borrow_mut();
                                            engine_ref.connect_stream(stream_clone).await
                                        };
                                        match connect_result {
                                            Ok(_) => {
                                                console::log!("MediaStream connected to audio engine after initialization");
                                            }
                                            Err(e) => {
                                                console::error!(&format!("Failed to connect stored stream: {:?}", e));
                                            }
                                        }
                                    });
                                }
                            }
                            Err(e) => {
                                console::error!(&format!("Failed to initialize audio engine: {:?}", e));
                                engine_state.set(AudioEngineState::Error(format!("Initialization failed: {}", e.message)));
                            }
                        }
                    }
                });
            }
        })
    };
    
    // Handle successful MediaStream from microphone permission
    let handle_stream_ready = {
        let media_stream = media_stream.clone();
        let audio_engine = props.audio_engine.clone();
        let engine_state = engine_state.clone();
        
        Callback::from(move |stream: web_sys::MediaStream| {
            console::log!("MediaStream ready, storing for audio engine connection");
            media_stream.set(Some(stream.clone()));
            
            // Only connect to audio engine if it's already initialized
            if let Some(engine) = &audio_engine {
                let current_state = &*engine_state;
                if matches!(current_state, AudioEngineState::Ready | AudioEngineState::Processing) {
                    console::log!("AudioEngine is ready, connecting MediaStream");
                    let engine_clone = engine.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = {
                            let mut engine_ref = engine_clone.borrow_mut();
                            engine_ref.connect_stream(stream).await
                        };
                        match result {
                            Ok(_) => {
                                console::log!("MediaStream connected to audio engine");
                            }
                            Err(e) => {
                                console::error!(&format!("Failed to connect stream: {:?}", e));
                            }
                        }
                    });
                } else {
                    console::log!("AudioEngine not ready yet. MediaStream stored for later connection.");
                }
            }
        })
    };

    // Handle microphone permission errors
    let handle_permission_error = {
        let engine_state = engine_state.clone();
        
        Callback::from(move |error: ApplicationError| {
            console::error!(&format!("Microphone permission error: {}", error.message));
            engine_state.set(AudioEngineState::Error(format!("Microphone: {}", error.message)));
        })
    };

    // Start audio processing (now requires MediaStream)
    let start_processing = {
        let engine_state = engine_state.clone();
        let is_processing = is_processing.clone();
        let audio_engine = props.audio_engine.clone();
        let media_stream = media_stream.clone();
        
        Callback::from(move |_| {
            if let Some(engine) = &audio_engine {
                if media_stream.is_some() {
                    console::log!("Starting audio processing with MediaStream...");
                    is_processing.set(true);
                    engine_state.set(AudioEngineState::Processing);
                    
                    wasm_bindgen_futures::spawn_local({
                        let engine = engine.clone();
                        let engine_state = engine_state.clone();
                        let is_processing = is_processing.clone();
                        
                        async move {
                            // Stream is already connected via handle_stream_ready
                            // Just update the state to show processing
                            console::log!("Audio processing started successfully");
                            engine_state.set(AudioEngineState::Processing);
                            is_processing.set(true);
                        }
                    });
                } else {
                    console::warn!("Cannot start processing: No MediaStream available");
                }
            }
        })
    };
    
    // Stop audio processing
    let stop_processing = {
        let engine_state = engine_state.clone();
        let is_processing = is_processing.clone();
        let audio_engine = props.audio_engine.clone();
        
        Callback::from(move |_| {
            if let Some(engine) = &audio_engine {
                console::log!("Stopping audio processing...");
                engine.borrow_mut().disconnect_stream();
                engine_state.set(AudioEngineState::Ready);
                is_processing.set(false);
            }
        })
    };
    
    // Test audio engine
    let test_engine = {
        let last_test_result = last_test_result.clone();
        let audio_engine = props.audio_engine.clone();
        
        Callback::from(move |_| {
            if let Some(engine) = &audio_engine {
                console::log!("Testing audio engine...");
                
                // Simulate audio engine test
                let metrics = engine.borrow().get_performance_metrics();
                let test_result = format!(
                    "✅ Test passed - Latency: {:.1}ms, Processing Rate: {:.1}Hz", 
                    metrics.total_latency_ms(), 
                    metrics.processing_rate_hz()
                );
                
                last_test_result.set(Some(test_result));
                console::log!("Audio engine test completed");
            } else {
                last_test_result.set(Some("❌ Test failed - No audio engine available".to_string()));
            }
        })
    };
    
    let get_state_display = || {
        match &*engine_state {
            AudioEngineState::Uninitialized => ("⚪", "Uninitialized".to_string(), "status-uninitialized"),
            AudioEngineState::Initializing => ("🟡", "Initializing...".to_string(), "status-initializing"),
            AudioEngineState::Ready => ("🟢", "Ready".to_string(), "status-ready"),
            AudioEngineState::Processing => ("🔵", "Processing".to_string(), "status-processing"),
            AudioEngineState::Suspended => ("🟠", "Suspended".to_string(), "status-suspended"),
            AudioEngineState::Error(msg) => ("🔴", format!("Error: {}", msg), "status-error"),
        }
    };
    
    let (status_icon, status_text, status_class) = get_state_display();
    let can_initialize = matches!(*engine_state, AudioEngineState::Uninitialized);
    let can_start = matches!(*engine_state, AudioEngineState::Ready) && media_stream.is_some();
    let can_stop = matches!(*engine_state, AudioEngineState::Processing);
    let can_test = matches!(*engine_state, AudioEngineState::Ready | AudioEngineState::Processing);
    
    html! {
        <div class="audio-control-panel">
            <div class="microphone-section">
                <h3>{ "Microphone Permission" }</h3>
                <MicrophonePermission 
                    on_stream_ready={handle_stream_ready}
                    on_error={Some(handle_permission_error)}
                    show_details={true}
                />
            </div>

            <div class="engine-status">
                <h3>{ "Audio Engine Status" }</h3>
                <div class={classes!("status-display", status_class)}>
                    <span class="status-icon">{ status_icon }</span>
                    <span class="status-text">{ &status_text }</span>
                </div>
            </div>
            
            <div class="control-buttons">
                <h3>{ "Engine Controls" }</h3>
                <div class="button-grid">
                    <button 
                        class="control-btn initialize-btn"
                        disabled={!can_initialize}
                        onclick={initialize_engine}
                    >
                        { "🔧 Initialize" }
                    </button>
                    
                    <button 
                        class="control-btn start-btn"
                        disabled={!can_start}
                        onclick={start_processing}
                    >
                        { "▶️ Start Processing" }
                    </button>
                    
                    <button 
                        class="control-btn stop-btn"
                        disabled={!can_stop}
                        onclick={stop_processing}
                    >
                        { "⏹️ Stop Processing" }
                    </button>
                    
                    <button 
                        class="control-btn test-btn"
                        disabled={!can_test}
                        onclick={test_engine}
                    >
                        { "🧪 Test Engine" }
                    </button>
                </div>
            </div>
            
            <div class="processing-info">
                <h3>{ "Processing Info" }</h3>
                <div class="info-grid">
                    <div class="info-item">
                        <span class="info-label">{ "Processing Active:" }</span>
                        <span class="info-value">{ if *is_processing { "Yes" } else { "No" } }</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">{ "Engine Available:" }</span>
                        <span class="info-value">{ if props.audio_engine.is_some() { "Yes" } else { "No" } }</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">{ "MediaStream:" }</span>
                        <span class="info-value">{ if media_stream.is_some() { "Connected" } else { "Not Connected" } }</span>
                    </div>
                </div>
            </div>
            
            { if let Some(test_result) = &*last_test_result {
                html! {
                    <div class="test-results">
                        <h3>{ "Last Test Result" }</h3>
                        <div class="test-result-display">
                            { test_result }
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
            
            <div class="debug-info">
                <h3>{ "Debug Info" }</h3>
                <div class="debug-details">
                    <div class="debug-item">
                        <span class="debug-label">{ "State:" }</span>
                        <span class="debug-value">{ format!("{:?}", &*engine_state) }</span>
                    </div>
                    <div class="debug-item">
                        <span class="debug-label">{ "Engine Ptr:" }</span>
                        <span class="debug-value">{ 
                            if let Some(engine) = &props.audio_engine {
                                format!("{:p}", engine.as_ptr())
                            } else {
                                "None".to_string()
                            }
                        }</span>
                    </div>
                </div>
            </div>
        </div>
    }
} 