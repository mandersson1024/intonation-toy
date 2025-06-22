use yew::prelude::*;
use web_sys::MediaStream;
use gloo::console;
use std::rc::Rc;
use std::cell::RefCell;

use crate::services::audio_engine::{AudioEngineService, AudioEngineState, AudioData};
use crate::audio::performance_monitor::PerformanceMetrics;
use crate::services::error_manager::ApplicationError;

/// Properties for AudioEngine component
#[derive(Properties, PartialEq)]
pub struct AudioEngineProps {
    /// MediaStream from microphone permission
    pub media_stream: Option<MediaStream>,
    
    /// Callback for audio data updates (pitch, confidence, etc.)
    pub on_audio_data: Callback<AudioData>,
    
    /// Callback for performance metrics updates
    #[prop_or(Callback::noop())]
    pub on_performance_update: Callback<PerformanceMetrics>,
    
    /// Callback for error handling
    #[prop_or(Callback::noop())]
    pub on_error: Callback<ApplicationError>,
    
    /// Callback for state changes
    #[prop_or(Callback::noop())]
    pub on_state_change: Callback<AudioEngineState>,
    
    /// Target latency in milliseconds
    #[prop_or(10.0)]
    pub target_latency_ms: f32,
    
    /// Enable/disable audio processing
    #[prop_or(true)]
    pub enabled: bool,
    
    /// Auto-initialize when component mounts
    #[prop_or(true)]
    pub auto_initialize: bool,
}

/// AudioEngine component for real-time audio processing
#[function_component(AudioEngineComponent)]
pub fn audio_engine_component(props: &AudioEngineProps) -> Html {
    let audio_engine = use_state(|| Rc::new(RefCell::new(AudioEngineService::new())));
    let current_state = use_state(|| AudioEngineState::Uninitialized);
    let last_performance_update = use_state(|| 0.0);
    let processing_active = use_state(|| false);

    // Initialize audio engine on mount
    {
        let audio_engine = audio_engine.clone();
        let current_state = current_state.clone();
        let auto_initialize = props.auto_initialize;
        let target_latency = props.target_latency_ms;
        let on_error = props.on_error.clone();
        let on_state_change = props.on_state_change.clone();

        use_effect_with((), move |_| {
            if auto_initialize {
                let audio_engine = audio_engine.clone();
                let current_state = current_state.clone();
                let on_error = on_error.clone();
                let on_state_change = on_state_change.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    // Setup callbacks
                    {
                        let mut engine = audio_engine.borrow_mut();
                        engine.set_target_latency(target_latency);
                        engine.set_on_error(on_error);
                        engine.set_on_state_change(on_state_change.clone());
                    }

                    // Initialize
                    match audio_engine.borrow_mut().initialize().await {
                        Ok(()) => {
                            console::log!("AudioEngine initialized successfully");
                            current_state.set(AudioEngineState::Ready);
                            on_state_change.emit(AudioEngineState::Ready);
                        }
                        Err(error) => {
                            console::error!(&format!("AudioEngine initialization failed: {}", error.message));
                            current_state.set(AudioEngineState::Error(error.message.clone()));
                            on_state_change.emit(AudioEngineState::Error(error.message));
                        }
                    }
                });
            }
            || ()
        });
    }

    // Handle MediaStream changes
    {
        let audio_engine = audio_engine.clone();
        let current_state = current_state.clone();
        let processing_active = processing_active.clone();
        let media_stream = props.media_stream.clone();
        let on_audio_data = props.on_audio_data.clone();
        let on_performance_update = props.on_performance_update.clone();
        let on_error = props.on_error.clone();
        let on_state_change = props.on_state_change.clone();

        use_effect_with(media_stream, move |stream| {
            if let Some(stream) = stream {
                let audio_engine = audio_engine.clone();
                let current_state = current_state.clone();
                let processing_active = processing_active.clone();
                let stream = stream.clone();
                let on_audio_data = on_audio_data.clone();
                let on_performance_update = on_performance_update.clone();
                let on_error = on_error.clone();
                let on_state_change = on_state_change.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    // Setup callbacks before connecting
                    {
                        let mut engine = audio_engine.borrow_mut();
                        engine.set_on_audio_data(on_audio_data);
                        engine.set_on_performance_update(on_performance_update);
                        engine.set_on_error(on_error);
                        engine.set_on_state_change(on_state_change.clone());
                    }

                    // Connect stream
                    match audio_engine.borrow_mut().connect_stream(stream).await {
                        Ok(()) => {
                            console::log!("MediaStream connected to AudioEngine");
                            processing_active.set(true);
                            current_state.set(AudioEngineState::Processing);
                            on_state_change.emit(AudioEngineState::Processing);
                        }
                        Err(error) => {
                            console::error!(&format!("Failed to connect MediaStream: {}", error.message));
                            current_state.set(AudioEngineState::Error(error.message.clone()));
                            on_state_change.emit(AudioEngineState::Error(error.message));
                        }
                    }
                });
            } else {
                // Disconnect stream if no longer available
                audio_engine.borrow_mut().disconnect_stream();
                processing_active.set(false);
                current_state.set(AudioEngineState::Ready);
                on_state_change.emit(AudioEngineState::Ready);
            }

            move || {
                // Cleanup on unmount
                audio_engine.borrow_mut().disconnect_stream();
            }
        });
    }

    // Handle enabled state changes
    {
        let audio_engine = audio_engine.clone();
        let enabled = props.enabled;

        use_effect_with(enabled, move |&enabled| {
            audio_engine.borrow_mut().set_enabled(enabled);
            console::log!(&format!("AudioEngine processing enabled: {}", enabled));
            || ()
        });
    }

    // Handle target latency changes
    {
        let audio_engine = audio_engine.clone();
        let target_latency = props.target_latency_ms;

        use_effect_with(target_latency, move |&latency| {
            audio_engine.borrow_mut().set_target_latency(latency);
            console::log!(&format!("AudioEngine target latency updated: {}ms", latency));
            || ()
        });
    }

    // Performance monitoring timer
    {
        let audio_engine = audio_engine.clone();
        let last_performance_update = last_performance_update.clone();
        let on_performance_update = props.on_performance_update.clone();
        let processing_active = *processing_active;

        use_effect_with(processing_active, move |&active| {
            if active {
                let interval = gloo::timers::callback::Interval::new(1000, move || {
                    let now = js_sys::Date::now();
                    if now - *last_performance_update > 1000.0 {
                        let metrics = audio_engine.borrow().get_performance_metrics();
                        on_performance_update.emit(metrics);
                        last_performance_update.set(now);
                    }
                });

                // Keep interval alive
                Box::leak(Box::new(interval));
            }
            || ()
        });
    }

    // Render component UI
    let state_display = match &*current_state {
        AudioEngineState::Uninitialized => "Uninitialized",
        AudioEngineState::Initializing => "Initializing...",
        AudioEngineState::Ready => "Ready",
        AudioEngineState::Processing => "Processing",
        AudioEngineState::Error(msg) => msg,
        AudioEngineState::Suspended => "Suspended",
    };

    let state_class = match &*current_state {
        AudioEngineState::Processing => "audio-engine-processing",
        AudioEngineState::Ready => "audio-engine-ready",
        AudioEngineState::Error(_) => "audio-engine-error",
        AudioEngineState::Initializing => "audio-engine-initializing",
        _ => "audio-engine-inactive",
    };

    html! {
        <div class={classes!("audio-engine-component", state_class)}>
            <div class="audio-engine-status">
                <div class="status-indicator">
                    <span class="status-dot"></span>
                    <span class="status-text">{state_display}</span>
                </div>
                
                if *processing_active {
                    <div class="processing-info">
                        <span class="latency-target">
                            {format!("Target: {}ms", props.target_latency_ms)}
                        </span>
                        <span class="processing-enabled">
                            {if props.enabled { "Enabled" } else { "Disabled" }}
                        </span>
                    </div>
                }
            </div>

            if let AudioEngineState::Error(error_msg) = &*current_state {
                <div class="audio-engine-error-details">
                    <span class="error-message">{error_msg}</span>
                </div>
            }

            // Debug information (only in development)
            if cfg!(debug_assertions) {
                <div class="audio-engine-debug">
                    <details>
                        <summary>{"Debug Info"}</summary>
                        <div class="debug-content">
                            <p>{format!("State: {:?}", *current_state)}</p>
                            <p>{format!("Processing Active: {}", *processing_active)}</p>
                            <p>{format!("Target Latency: {}ms", props.target_latency_ms)}</p>
                            <p>{format!("Enabled: {}", props.enabled)}</p>
                            <p>{format!("Has MediaStream: {}", props.media_stream.is_some())}</p>
                        </div>
                    </details>
                </div>
            }
        </div>
    }
}

/// Hook for managing AudioEngine state in components
#[hook]
pub fn use_audio_engine() -> (
    AudioEngineState,
    Callback<MediaStream>,
    Callback<()>,
    Callback<f32>,
    Callback<bool>,
) {
    let audio_engine = use_state(|| Rc::new(RefCell::new(AudioEngineService::new())));
    let state = use_state(|| AudioEngineState::Uninitialized);

    // Initialize callback
    let initialize = {
        let audio_engine = audio_engine.clone();
        let state = state.clone();

        Callback::from(move |_: ()| {
            let audio_engine = audio_engine.clone();
            let state = state.clone();

            wasm_bindgen_futures::spawn_local(async move {
                state.set(AudioEngineState::Initializing);
                match audio_engine.borrow_mut().initialize().await {
                    Ok(()) => state.set(AudioEngineState::Ready),
                    Err(error) => state.set(AudioEngineState::Error(error.message)),
                }
            });
        })
    };

    // Connect stream callback
    let connect_stream = {
        let audio_engine = audio_engine.clone();
        let state = state.clone();

        Callback::from(move |stream: MediaStream| {
            let audio_engine = audio_engine.clone();
            let state = state.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match audio_engine.borrow_mut().connect_stream(stream).await {
                    Ok(()) => state.set(AudioEngineState::Processing),
                    Err(error) => state.set(AudioEngineState::Error(error.message)),
                }
            });
        })
    };

    // Disconnect callback
    let disconnect = {
        let audio_engine = audio_engine.clone();
        let state = state.clone();

        Callback::from(move |_: ()| {
            audio_engine.borrow_mut().disconnect_stream();
            state.set(AudioEngineState::Ready);
        })
    };

    // Set target latency callback
    let set_target_latency = {
        let audio_engine = audio_engine.clone();

        Callback::from(move |latency: f32| {
            audio_engine.borrow_mut().set_target_latency(latency);
        })
    };

    // Set enabled callback
    let set_enabled = {
        let audio_engine = audio_engine.clone();

        Callback::from(move |enabled: bool| {
            audio_engine.borrow_mut().set_enabled(enabled);
        })
    };

    (
        (*state).clone(),
        connect_stream,
        disconnect,
        set_target_latency,
        set_enabled,
    )
} 