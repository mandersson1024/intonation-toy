//! # Audio Control Panel Component
//!
//! Event-driven audio control panel component for debug builds only.
//! Provides audio engine control functionality with start/stop/test capabilities.
//! Now subscribes to audio events for real-time state updates.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use web_sys::console;
#[cfg(debug_assertions)]
use wasm_bindgen::prelude::*;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;

// Event system imports
#[cfg(debug_assertions)]
use crate::modules::developer_ui::hooks::use_event_subscription::use_event_subscription;
#[cfg(debug_assertions)]
use crate::modules::developer_ui::utils::debug_event_publisher::{DebugEventPublisher, DebugControlEvent};
#[cfg(debug_assertions)]
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
#[cfg(debug_assertions)]
use crate::modules::audio_foundations::audio_events::{
    AudioProcessingStateEvent, AudioSessionEvent, MicrophoneStateEvent, 
    AudioErrorEvent, DeviceListUpdatedEvent
};

// TODO: Update these imports once legacy services are migrated to modules
#[cfg(debug_assertions)]
use crate::legacy::active::services::audio_engine::{AudioEngineService, AudioEngineState, AudioDeviceInfo};
#[cfg(debug_assertions)]
use crate::legacy::active::services::error_manager::{ApplicationError, ErrorManager};

// JavaScript binding for getting audio output device name
#[cfg(debug_assertions)]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "getAudioOutputDeviceName")]
    async fn get_audio_output_device_name() -> JsValue;
}

#[cfg(debug_assertions)]
#[derive(Properties)]
pub struct AudioControlPanelProps {
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    #[prop_or(None)]
    pub error_manager: Option<Rc<RefCell<ErrorManager>>>,
    /// Event bus for subscribing to audio events and publishing debug events
    #[prop_or(None)]
    pub event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
}

#[cfg(debug_assertions)]
impl PartialEq for AudioControlPanelProps {
    fn eq(&self, other: &Self) -> bool {
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr()) &&
        self.error_manager.as_ref().map(|e| e.as_ptr()) == other.error_manager.as_ref().map(|e| e.as_ptr()) &&
        self.event_bus.as_ref().map(|e| e.as_ptr()) == other.event_bus.as_ref().map(|e| e.as_ptr())
    }
}

/// Event-driven audio engine control panel with start/stop/test functionality
#[cfg(debug_assertions)]
#[function_component(AudioControlPanel)]
pub fn audio_control_panel(props: &AudioControlPanelProps) -> Html {
    let engine_state = use_state(|| AudioEngineState::Uninitialized);
    let media_stream = use_state(|| None::<web_sys::MediaStream>);
    let output_device_name = use_state(|| None::<String>);
    let input_device_name = use_state(|| None::<String>);
    
    // Subscribe to audio processing state events
    let audio_state_event = use_event_subscription::<AudioProcessingStateEvent>(props.event_bus.clone());
    
    // Subscribe to microphone state events
    let microphone_event = use_event_subscription::<MicrophoneStateEvent>(props.event_bus.clone());
    
    // Subscribe to audio session events
    let session_event = use_event_subscription::<AudioSessionEvent>(props.event_bus.clone());
    
    // Subscribe to device list updates
    let device_list_event = use_event_subscription::<DeviceListUpdatedEvent>(props.event_bus.clone());
    
    // Subscribe to audio errors
    let audio_error_event = use_event_subscription::<AudioErrorEvent>(props.event_bus.clone());
    
    // Create debug event publisher for publishing control events
    let debug_publisher = use_state(|| {
        DebugEventPublisher::new(props.event_bus.clone())
    });
    
    // Event-driven state updates: React to audio processing state changes
    {
        let engine_state = engine_state.clone();
        use_effect_with(audio_state_event.clone(), move |event| {
            if let Some(state_event) = &**event {
                console::log_1(&format!("Audio state changed: {:?} -> {:?}", 
                    state_event.old_state, state_event.new_state).into());
                
                // Map audio foundation state to legacy state for compatibility
                match state_event.new_state {
                    crate::modules::audio_foundations::AudioEngineState::Idle => {
                        engine_state.set(AudioEngineState::Ready);
                    }
                    crate::modules::audio_foundations::AudioEngineState::Processing => {
                        engine_state.set(AudioEngineState::Processing);
                    }
                    crate::modules::audio_foundations::AudioEngineState::Error(ref msg) => {
                        engine_state.set(AudioEngineState::Error(msg.clone()));
                    }
                    _ => {
                        // Handle other states as needed
                    }
                }
            }
            || ()
        });
    }
    
    // Event-driven updates: React to microphone state changes
    {
        let input_device_name = input_device_name.clone();
        use_effect_with(microphone_event.clone(), move |event| {
            if let Some(mic_event) = &**event {
                if let Some(ref device_info) = mic_event.device_info {
                    console::log_1(&format!("Microphone device updated: {}", device_info.name).into());
                    input_device_name.set(Some(device_info.name.clone()));
                }
            }
            || ()
        });
    }
    
    // Event-driven updates: React to device list changes
    {
        let output_device_name = output_device_name.clone();
        use_effect_with(device_list_event.clone(), move |event| {
            if let Some(device_event) = &**event {
                // Find the default output device from the updated list
                if let Some(output_device) = device_event.devices.iter()
                    .find(|device| device.is_default && device.kind == crate::types::audio::AudioDeviceKind::AudioOutput) {
                    console::log_1(&format!("Output device updated: {}", output_device.name).into());
                    output_device_name.set(Some(output_device.name.clone()));
                }
            }
            || ()
        });
    }
    
    // Event-driven updates: React to audio errors
    {
        let error_manager = props.error_manager.clone();
        use_effect_with(audio_error_event.clone(), move |event| {
            if let Some(error_event) = &**event {
                console::error_1(&format!("Audio error received: {}", error_event.message).into());
                
                // Forward error to error manager if available
                if let Some(manager) = &error_manager {
                    if let Ok(mut manager_ref) = manager.try_borrow_mut() {
                        let app_error = crate::legacy::active::services::error_manager::ApplicationError {
                            message: error_event.message.clone(),
                            details: Some(error_event.context.clone()),
                            severity: crate::legacy::active::services::error_manager::ErrorSeverity::Medium,
                            component: "AudioControlPanel".to_string(),
                            timestamp: chrono::Utc::now().timestamp_millis() as u64,
                        };
                        manager_ref.add_error(app_error);
                    }
                }
            }
            || ()
        });
    }
    
    // Auto-initialize and start processing when component mounts
    {
        let engine_state = engine_state.clone();
        let media_stream = media_stream.clone();
        let output_device_name = output_device_name.clone();
        
        use_effect_with(
            props.audio_engine.clone(),
            move |audio_engine| {
                if let Some(engine) = audio_engine {
                    // Sync local state with actual engine state first
                    let current_engine_state = engine.borrow().get_state();
                    engine_state.set(current_engine_state.clone());
                    
                    // Auto-initialize if not already initialized
                    if matches!(current_engine_state, AudioEngineState::Uninitialized) {
                        console::log_1(&"Auto-initializing audio engine...".into());
                        let engine_clone = engine.clone();
                        let engine_state_clone = engine_state.clone();
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            // Use try_borrow_mut to avoid conflicts with other components
                            if let Ok(mut engine_ref) = engine_clone.try_borrow_mut() {
                                if let Ok(()) = engine_ref.initialize().await {
                                    console::log_1(&"Audio engine auto-initialized successfully".into());
                                    
                                    // Drop the mutable borrow before getting state
                                    drop(engine_ref);
                                    
                                    // Update local state to match engine state (separate scope to avoid borrow conflicts)
                                    {
                                        let actual_state = engine_clone.borrow().get_state();
                                        engine_state_clone.set(actual_state);
                                    }
                                    
                                    // Detect output device when engine is ready
                                    let output_device_clone = output_device_name.clone();
                                    wasm_bindgen_futures::spawn_local(async move {
                                        let device_result = get_audio_output_device_name().await;
                                        if let Some(name) = device_result.as_string() {
                                            console::log_1(&format!("Audio output device: {}", name).into());
                                            output_device_clone.set(Some(name));
                                        } else {
                                            console::warn_1(&"Could not detect audio output device".into());
                                        }
                                    });
                                    
                                    // Auto-start processing if media stream is available
                                    if media_stream.is_some() {
                                        console::log_1(&"Auto-starting audio processing...".into());
                                        {
                                            let processing_state = engine_clone.borrow().get_state();
                                            engine_state_clone.set(processing_state);
                                        }
                                    }
                                } else {
                                    console::error_1(&"Failed to auto-initialize audio engine".into());
                                }
                            } else {
                                console::warn_1(&"Could not borrow audio engine for initialization (busy)".into());
                            }
                        });
                    }
                }
                
                || ()
            },
        );
    }
    
    // Clear output device name when engine is not active
    {
        let output_device_name = output_device_name.clone();
        let engine_state = engine_state.clone();
        
        use_effect_with(
            engine_state.clone(),
            move |state| {
                match **state {
                    AudioEngineState::Uninitialized | 
                    AudioEngineState::Initializing | 
                    AudioEngineState::Error(_) => {
                        output_device_name.set(None);
                    }
                    _ => {}
                }
                || ()
            },
        );
    }

    
    // Handle input device name changes
    let handle_input_device_change = {
        let input_device_name = input_device_name.clone();
        
        Callback::from(move |name: Option<String>| {
            input_device_name.set(name);
        })
    };

    // Handle stream ready callback
    let handle_stream_ready = {
        let media_stream = media_stream.clone();
        let engine_state = engine_state.clone();
        let audio_engine = props.audio_engine.clone();
        
        Callback::from(move |stream: web_sys::MediaStream| {
            console::log_1(&"MediaStream ready, connecting to audio engine...".into());
            media_stream.set(Some(stream.clone()));
            
            // Auto-start processing when stream becomes available
            if let Some(engine) = &audio_engine {
                if matches!(*engine_state, AudioEngineState::Ready) {
                    console::log_1(&"Auto-starting audio processing with MediaStream...".into());
                    engine_state.set(AudioEngineState::Processing);
                    
                    let engine_clone = engine.clone();
                    let engine_state_clone = engine_state.clone();
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        // Use try_borrow_mut to avoid conflicts with other components
                        if let Ok(mut engine_ref) = engine_clone.try_borrow_mut() {
                            if let Ok(()) = engine_ref.connect_stream(stream).await {
                                console::log_1(&"Audio processing auto-started successfully".into());
                                
                                // Drop the mutable borrow before getting state
                                drop(engine_ref);
                                
                                // Update local state to match actual engine state (separate scope to avoid borrow conflicts)
                                {
                                    let actual_state = engine_clone.borrow().get_state();
                                    engine_state_clone.set(actual_state);
                                }
                            }
                        } else {
                            console::warn_1(&"Could not borrow audio engine for stream connection (busy)".into());
                        }
                    });
                }
            }
        })
    };
    
    // Handle permission errors
    let handle_permission_error = {
        let error_manager = props.error_manager.clone();
        Callback::from(move |error: ApplicationError| {
            console::error_1(&format!("Microphone permission error: {}", error.message).into());
            
            // Add error to error manager if available
            if let Some(manager) = &error_manager {
                if let Ok(mut manager_ref) = manager.try_borrow_mut() {
                    manager_ref.add_error(error);
                } else {
                    console::warn_1(&"Could not add error to manager (busy)".into());
                }
            }
        })
    };
    
    // Event publishing handlers for user interactions
    let handle_start_recording = {
        let debug_publisher = debug_publisher.clone();
        Callback::from(move |_| {
            let mut publisher = (*debug_publisher).clone();
            if let Err(e) = publisher.publish_control_event(DebugControlEvent::StartRecording) {
                console::error_1(&format!("Failed to publish start recording event: {}", e).into());
            } else {
                console::log_1(&"Published start recording event".into());
            }
        })
    };
    
    let handle_stop_recording = {
        let debug_publisher = debug_publisher.clone();
        Callback::from(move |_| {
            let mut publisher = (*debug_publisher).clone();
            if let Err(e) = publisher.publish_control_event(DebugControlEvent::StopRecording) {
                console::error_1(&format!("Failed to publish stop recording event: {}", e).into());
            } else {
                console::log_1(&"Published stop recording event".into());
            }
        })
    };
    
    let handle_reset_state = {
        let debug_publisher = debug_publisher.clone();
        Callback::from(move |_| {
            let mut publisher = (*debug_publisher).clone();
            if let Err(e) = publisher.publish_control_event(DebugControlEvent::ResetState) {
                console::error_1(&format!("Failed to publish reset state event: {}", e).into());
            } else {
                console::log_1(&"Published reset state event".into());
            }
        })
    };
    
    let is_engine_active = matches!(*engine_state, AudioEngineState::Ready | AudioEngineState::Processing);

    html! {
        <div class="audio-control-panel">
            <div class="engine-status">
                <h3>{ "Audio Engine" }</h3>
                <div class="status-info">
                    <span class="status-text">{ format!("State: {:?}", *engine_state) }</span>
                    if let Some(ref output_device) = *output_device_name {
                        <span class="device-info">{ format!("Output: {}", output_device) }</span>
                    }
                    if let Some(ref input_device) = *input_device_name {
                        <span class="device-info">{ format!("Input: {}", input_device) }</span>
                    }
                </div>
                
                // Event-driven status indicators
                <div class="event-status">
                    if audio_state_event.is_some() {
                        <span class="event-indicator audio-events">{ "🔊 Audio Events" }</span>
                    }
                    if microphone_event.is_some() {
                        <span class="event-indicator mic-events">{ "🎤 Mic Events" }</span>
                    }
                    if audio_error_event.is_some() {
                        <span class="event-indicator error-events">{ "⚠️ Error Events" }</span>
                    }
                </div>
            </div>
            
            <div class="event-controls">
                <h4>{ "Event-Driven Controls" }</h4>
                <div class="control-buttons">
                    <button 
                        class="debug-button start-recording"
                        onclick={handle_start_recording}
                        disabled={!is_engine_active}
                    >
                        { "▶️ Start Recording" }
                    </button>
                    <button 
                        class="debug-button stop-recording"
                        onclick={handle_stop_recording}
                        disabled={!is_engine_active}
                    >
                        { "⏹️ Stop Recording" }
                    </button>
                    <button 
                        class="debug-button reset-state"
                        onclick={handle_reset_state}
                    >
                        { "🔄 Reset State" }
                    </button>
                </div>
                
                // Display event publisher metrics
                if let Some(metrics) = debug_publisher.get_metrics() {
                    <div class="publisher-metrics">
                        <h5>{ "Publisher Metrics" }</h5>
                        <div class="metrics-grid">
                            <span class="metric">{ format!("Published: {}", metrics.total_published) }</span>
                            <span class="metric">{ format!("Errors: {}", metrics.total_errors) }</span>
                            <span class="metric">{ format!("Success Rate: {:.1}%", metrics.success_rate()) }</span>
                            <span class="metric">{ format!("Avg Time: {:.0}μs", metrics.avg_publish_time_us) }</span>
                            <span class={classes!("performance-indicator", 
                                if metrics.meets_performance_requirements() { "good" } else { "warning" })}
                            >
                                { if metrics.meets_performance_requirements() { "✅ Performance OK" } else { "⚠️ Performance Issue" } }
                            </span>
                        </div>
                    </div>
                }
            </div>
            
            <div class="microphone-section">
                <super::MicrophonePanel
                    on_stream_ready={handle_stream_ready}
                    on_error={Some(handle_permission_error)}
                    audio_engine={props.audio_engine.clone()}
                    media_stream={(*media_stream).clone()}
                    on_device_name_change={Some(handle_input_device_change)}
                />
            </div>
        </div>
    }
}

// Implementation re-exports would go here when components are fully migrated 