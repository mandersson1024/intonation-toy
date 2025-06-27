//! # Audio Control Panel Component
//!
//! Migrated audio control panel component for debug builds only.
//! Provides audio engine control functionality with start/stop/test capabilities.

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
}

#[cfg(debug_assertions)]
impl PartialEq for AudioControlPanelProps {
    fn eq(&self, other: &Self) -> bool {
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr()) &&
        self.error_manager.as_ref().map(|e| e.as_ptr()) == other.error_manager.as_ref().map(|e| e.as_ptr())
    }
}

/// Audio engine control panel with start/stop/test functionality
#[cfg(debug_assertions)]
#[function_component(AudioControlPanel)]
pub fn audio_control_panel(props: &AudioControlPanelProps) -> Html {
    let engine_state = use_state(|| AudioEngineState::Uninitialized);
    let media_stream = use_state(|| None::<web_sys::MediaStream>);
    let output_device_name = use_state(|| None::<String>);
    let input_device_name = use_state(|| None::<String>);
    
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

#[cfg(debug_assertions)]
pub use implementation::*; 