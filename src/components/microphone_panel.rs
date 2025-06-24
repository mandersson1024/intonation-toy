use yew::prelude::*;
use web_sys::console;
use wasm_bindgen::prelude::*;
use js_sys::Float32Array;
use std::rc::Rc;
use std::cell::RefCell;
use crate::components::microphone_permission::MicrophonePermission;
use crate::services::audio_engine::AudioEngineService;
use crate::services::error_manager::{ApplicationError, ErrorManager};

#[derive(Clone, PartialEq)]
pub struct AudioInputDevice {
    pub device_id: String,
    pub label: String,
    pub group_id: String,
}

// Helper function to load audio input devices
async fn load_audio_input_devices(is_refresh: bool) -> Vec<AudioInputDevice> {
    let devices_result = get_audio_input_devices().await;
    if let Ok(devices_array) = devices_result.dyn_into::<js_sys::Array>() {
        let mut device_list = Vec::new();
        
        let header = if is_refresh { 
            "=== REFRESHED AUDIO INPUT DEVICES ===" 
        } else { 
            "=== AUDIO INPUT DEVICES FROM BROWSER API ===" 
        };
        console::log_1(&header.into());
        
        for i in 0..devices_array.length() {
            if let Ok(device_obj) = devices_array.get(i).dyn_into::<js_sys::Object>() {
                let device_id = js_sys::Reflect::get(&device_obj, &"deviceId".into())
                    .unwrap_or_else(|_| JsValue::from_str(""))
                    .as_string()
                    .unwrap_or_default();
                let label = js_sys::Reflect::get(&device_obj, &"label".into())
                    .unwrap_or_else(|_| JsValue::from_str("Unknown Device"))
                    .as_string()
                    .unwrap_or_else(|| "Unknown Device".to_string());
                let group_id = js_sys::Reflect::get(&device_obj, &"groupId".into())
                    .unwrap_or_else(|_| JsValue::from_str(""))
                    .as_string()
                    .unwrap_or_default();
                
                console::log_1(&format!("Device {}: ID='{}', Label='{}', GroupID='{}'", 
                    i + 1, device_id, label, group_id).into());
                
                device_list.push(AudioInputDevice {
                    device_id,
                    label,
                    group_id,
                });
            }
        }
        
        let footer = if is_refresh { 
            "=== END REFRESHED DEVICE LIST ===" 
        } else { 
            "=== END DEVICE LIST ===" 
        };
        console::log_1(&footer.into());
        
        let message = if is_refresh {
            format!("Refreshed to {} audio input devices", device_list.len())
        } else {
            format!("Found {} audio input devices total", device_list.len())
        };
        console::log_1(&message.into());
        
        device_list
    } else {
        console::warn_1(&"Failed to get audio input devices".into());
        Vec::new()
    }
}

// External JavaScript bindings for Web Audio API
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setupMicrophoneAnalysis")]
    fn setup_microphone_analysis(stream: &web_sys::MediaStream) -> JsValue;
    
    #[wasm_bindgen(js_name = "getMicrophoneLevel")]
    fn get_microphone_level(analyser: &JsValue) -> f32;
    
    #[wasm_bindgen(js_name = "cleanupMicrophoneAnalysis")]
    fn cleanup_microphone_analysis(analyser: &JsValue);
    
    #[wasm_bindgen(js_name = "getMicrophoneDeviceName")]
    async fn get_microphone_device_name(stream: &web_sys::MediaStream) -> JsValue;
    
    #[wasm_bindgen(js_name = "getAudioInputDevices")]
    async fn get_audio_input_devices() -> JsValue;
    
    #[wasm_bindgen(js_name = "switchToInputDevice")]
    async fn switch_to_input_device(device_id: &str) -> JsValue;
}

#[derive(Properties)]
pub struct MicrophonePanelProps {
    /// Callback when MediaStream is ready
    pub on_stream_ready: Callback<web_sys::MediaStream>,
    /// Error handler callback
    #[prop_or(None)]
    pub on_error: Option<Callback<ApplicationError>>,
    /// Audio engine for accessing audio data
    #[prop_or(None)]
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    /// Current MediaStream for input level monitoring
    #[prop_or(None)]
    pub media_stream: Option<web_sys::MediaStream>,
    /// Callback when device name changes
    #[prop_or(None)]
    pub on_device_name_change: Option<Callback<Option<String>>>,
}

impl PartialEq for MicrophonePanelProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer equality for audio engine
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr()) &&
        self.media_stream.as_ref().map(|s| s.id()) == other.media_stream.as_ref().map(|s| s.id())
    }
}

/// Microphone sub-panel containing permission controls and input monitoring
#[function_component(MicrophonePanel)]
pub fn microphone_panel(props: &MicrophonePanelProps) -> Html {
    let input_level = use_state(|| 0.0f32);
    let is_active = use_state(|| false);
    let analyser_node = use_state(|| None::<JsValue>);
    let device_name = use_state(|| None::<String>);
    let available_devices = use_state(|| Vec::<AudioInputDevice>::new());
    let selected_device_id = use_state(|| None::<String>);
    let show_device_selector = use_state(|| false);
    
    // Load available input devices and set up device change monitoring
    // Note: Only load devices when we have a media stream (permission granted)
    {
        let available_devices = available_devices.clone();
        
        {
            let devices = available_devices.clone();
            let on_stream_ready = props.on_stream_ready.clone();
            let selected_device_id_for_effect = selected_device_id.clone();
            let media_stream = props.media_stream.clone();
            
            use_effect_with(media_stream, move |stream| {
                // Load devices when we have permission (media stream available)
                if stream.is_some() {
                    let devices_initial = devices.clone();
                    let selected_device_id_initial = selected_device_id_for_effect.clone();
                    let stream_for_detection = stream.clone();
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        let device_list = load_audio_input_devices(false).await;
                        devices_initial.set(device_list);
                        
                        // If we have a stream, detect the actual device ID and sync with dropdown
                        if let Some(stream) = stream_for_detection {
                            let audio_tracks = stream.get_audio_tracks();
                            if audio_tracks.length() > 0 {
                                if let Ok(track) = audio_tracks.get(0).dyn_into::<web_sys::MediaStreamTrack>() {
                                    // Get the actual device ID from the stream
                                    let settings_result = js_sys::Reflect::get(&track, &"getSettings".into());
                                    if let Ok(get_settings_fn) = settings_result {
                                        if let Ok(settings) = js_sys::Reflect::apply(
                                            &get_settings_fn.dyn_into::<js_sys::Function>().unwrap(),
                                            &track,
                                            &js_sys::Array::new()
                                        ) {
                                            if let Ok(device_id_js) = js_sys::Reflect::get(&settings, &"deviceId".into()) {
                                                if let Some(actual_device_id) = device_id_js.as_string() {
                                                    console::log_1(&format!("🔍 Initial device detection: stream using device ID: {}", actual_device_id).into());
                                                    selected_device_id_initial.set(Some(actual_device_id));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
                
                // Set up device change listener to refresh the list when devices are added/removed
                // Note: This will be the primary device change listener and will handle both
                // device list refresh and potential reconnection scenarios
                // Only set up when we have permission (media stream available)
                if stream.is_some() {
                    if let Some(media_devices) = web_sys::window()
                        .and_then(|w| w.navigator().media_devices().ok())
                    {
                    let devices_for_change = devices.clone();
                    let on_stream_ready_change = on_stream_ready.clone();
                    let selected_device_id_change = selected_device_id_for_effect.clone();
                    let ondevicechange = Closure::wrap(Box::new(move |_: web_sys::Event| {
                        console::log_1(&"🔄 Audio devices changed - refreshing device list and checking for reconnection".into());
                        
                        // Reload devices when change detected
                        let devices_for_reload = devices_for_change.clone();
                        let on_stream_ready_reconnect = on_stream_ready_change.clone();
                        let selected_device_id_reconnect = selected_device_id_change.clone();
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            let device_list = load_audio_input_devices(true).await;
                            
                            // Check if we should attempt automatic reconnection before moving device_list
                            let should_reconnect = !device_list.is_empty();
                            let reconnect_device_id = if should_reconnect {
                                // Simple approach: use the first available device
                                console::log_1(&format!("🎯 Using first available device for reconnection: {}", device_list[0].label).into());
                                Some(device_list[0].device_id.clone())
                            } else {
                                None
                            };
                            
                            // Check if current selected device is still in the list
                            // If not, clear the selection so the UI shows no device selected
                            let current_selection = (*selected_device_id_reconnect).clone();
                            if let Some(ref selected_id) = current_selection {
                                let device_still_exists = device_list.iter().any(|d| &d.device_id == selected_id);
                                if !device_still_exists {
                                    console::log_1(&format!("⚠️ Previously selected device {} no longer available - clearing selection", selected_id).into());
                                    selected_device_id_reconnect.set(None);
                                }
                            }
                            
                            // Set the device list (moves device_list)
                            devices_for_reload.set(device_list);
                            
                            // Attempt reconnection if we have devices
                            if let Some(device_id) = reconnect_device_id {
                                console::log_1(&format!("🎤 Attempting reconnection to first available device: {}", device_id).into());
                                
                                // Try to connect to the first available device
                                let stream_result = switch_to_input_device(&device_id).await;
                                
                                if stream_result.is_undefined() || stream_result.is_null() {
                                    console::error_1(&format!("Failed to auto-reconnect to device: {}", device_id).into());
                                } else if let Ok(stream) = stream_result.dyn_into::<web_sys::MediaStream>() {
                                    console::log_1(&format!("✅ Successfully auto-reconnected to first available device: {}", device_id).into());
                                    
                                    // Update the dropdown selection to match the reconnected device
                                    console::log_1(&format!("🎯 Setting dropdown selection to auto-reconnected device: {}", device_id).into());
                                    selected_device_id_reconnect.set(Some(device_id.clone()));
                                    
                                    // Emit the new stream to update the UI and Device Configuration
                                    on_stream_ready_reconnect.emit(stream);
                                } else {
                                    console::error_1(&"Failed to convert reconnected stream object".into());
                                }
                            }
                        });
                    }) as Box<dyn Fn(_)>);
                    
                        media_devices.set_ondevicechange(Some(ondevicechange.as_ref().unchecked_ref()));
                        ondevicechange.forget(); // Keep the closure alive
                        console::log_1(&"🔍 Primary device change listener set up for device list refresh and auto-reconnection".into());
                    }
                }
                
                || ()
            });
        }
    }
    
    // Set up real-time audio analysis when MediaStream is available
    {
        let input_level = input_level.clone();
        let is_active = is_active.clone();
        let analyser_node = analyser_node.clone();
        let device_name = device_name.clone();
        let selected_device_id = selected_device_id.clone();
        let media_stream = props.media_stream.clone();
        let device_name_callback = props.on_device_name_change.clone();
        
        use_effect_with(media_stream, move |stream| {
            if let Some(stream) = stream {
                is_active.set(true);
                
                // Get and update device name and ID
                let device_name_clone = device_name.clone();
                let selected_device_id_clone = selected_device_id.clone();
                let stream_clone = stream.clone();
                let device_callback = device_name_callback.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let name_result = get_microphone_device_name(&stream_clone).await;
                    if let Some(name) = name_result.as_string() {
                        console::log_1(&format!("Microphone device name: {}", name).into());
                        device_name_clone.set(Some(name.clone()));
                        
                        // Update the dropdown selection to match the actual connected device
                        // This ensures UI consistency between dropdown and Device Configuration
                        let audio_tracks = stream_clone.get_audio_tracks();
                        if audio_tracks.length() > 0 {
                            if let Ok(track) = audio_tracks.get(0).dyn_into::<web_sys::MediaStreamTrack>() {
                                // Get the actual device ID from the stream
                                let settings_result = js_sys::Reflect::get(&track, &"getSettings".into());
                                if let Ok(get_settings_fn) = settings_result {
                                    if let Ok(settings) = js_sys::Reflect::apply(
                                        &get_settings_fn.dyn_into::<js_sys::Function>().unwrap(),
                                        &track,
                                        &js_sys::Array::new()
                                    ) {
                                        if let Ok(device_id_js) = js_sys::Reflect::get(&settings, &"deviceId".into()) {
                                            if let Some(actual_device_id) = device_id_js.as_string() {
                                                console::log_1(&format!("📺 Stream connected with actual device ID: {}", actual_device_id).into());
                                                console::log_1(&format!("🔍 Current dropdown selection: {:?}", *selected_device_id_clone).into());
                                                
                                                // Always update dropdown to match the actual connected device
                                                if selected_device_id_clone.as_ref() != Some(&actual_device_id) {
                                                    console::log_1(&format!("🔄 Updating dropdown to match actual connected device: {}", actual_device_id).into());
                                                    selected_device_id_clone.set(Some(actual_device_id));
                                                } else {
                                                    console::log_1(&"✅ Dropdown already matches connected device".into());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        if let Some(callback) = device_callback {
                            callback.emit(Some(name));
                        }
                    } else {
                        console::warn_1(&"Could not get microphone device name".into());
                        device_name_clone.set(None);
                        if let Some(callback) = device_callback {
                            callback.emit(None);
                        }
                    }
                });
                
                // Set up Web Audio API for real microphone analysis
                let analyser = setup_microphone_analysis(stream);
                
                if !analyser.is_null() && !analyser.is_undefined() {
                    analyser_node.set(Some(analyser.clone()));
                    
                    let input_level_clone = input_level.clone();
                    let is_active_clone = is_active.clone();
                    let analyser_clone = analyser.clone();
                    
                    // Monitor real audio levels
                    let interval = gloo::timers::callback::Interval::new(50, move || {
                        let level = get_microphone_level(&analyser_clone);
                        input_level_clone.set(level);
                    });
                    
                    Box::new(move || {
                        is_active_clone.set(false);
                        cleanup_microphone_analysis(&analyser);
                        drop(interval);
                    }) as Box<dyn FnOnce()>
                } else {
                    console::error_1(&"Failed to set up microphone analysis".into());
                    is_active.set(false);
                    Box::new(move || {}) as Box<dyn FnOnce()>
                }
            } else {
                is_active.set(false);
                input_level.set(0.0);
                analyser_node.set(None);
                device_name.set(None);
                selected_device_id.set(None);
                if let Some(callback) = device_name_callback.clone() {
                    callback.emit(None);
                }
                Box::new(move || {}) as Box<dyn FnOnce()>
            }
        });
    }
    
    // Handle device selection
    let handle_device_change = {
        let selected_device_id_handle = selected_device_id.clone();
        let on_stream_ready = props.on_stream_ready.clone();
        
        Callback::from(move |device_id: String| {
            selected_device_id_handle.set(Some(device_id.clone()));
            
            let on_stream_ready = on_stream_ready.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let stream_result = switch_to_input_device(&device_id).await;
                if stream_result.is_undefined() || stream_result.is_null() {
                    console::error_1(&format!("Failed to switch device: {}", device_id).into());
                } else if let Ok(stream) = stream_result.dyn_into::<web_sys::MediaStream>() {
                    console::log_1(&format!("Successfully switched to device: {}", device_id).into());
                    on_stream_ready.emit(stream);
                } else {
                    console::error_1(&"Failed to convert stream object".into());
                }
            });
        })
    };

    // Toggle device selector visibility
    let toggle_device_selector = {
        let show_device_selector = show_device_selector.clone();
        
        Callback::from(move |_| {
            show_device_selector.set(!*show_device_selector);
        })
    };

    // Format input level as percentage
    let level_percentage = (*input_level * 100.0) as u32;
    let level_class = if level_percentage > 70 {
        "input-level-high"
    } else if level_percentage > 30 {
        "input-level-medium"
    } else {
        "input-level-low"
    };

    html! {
        <div class="microphone-panel">
            <div class="microphone-header">
                <h4 class="microphone-title">{ "🎤 Microphone" }</h4>
                <span class={if *is_active { "microphone-status active" } else { "microphone-status inactive" }}>
                    { if *is_active { "● ACTIVE" } else { "○ INACTIVE" } }
                </span>
            </div>
            
            <div class="microphone-content">
                // Permission Component
                <div class="microphone-permission-section">
                    <MicrophonePermission 
                        on_stream_ready={props.on_stream_ready.clone()}
                        on_error={props.on_error.clone()}
                        show_details={false}
                    />
                </div>
                
                // Device Selection Section
                { if !available_devices.is_empty() {
                    html! {
                        <div class="microphone-device-section">
                            <div class="device-selector-header">
                                <span class="device-selector-label">{ "Input Device:" }</span>
                                <button class="device-selector-toggle" onclick={toggle_device_selector.clone()}>
                                    { if *show_device_selector { "▲" } else { "▼" } }
                                </button>
                            </div>
                            
                            { if *show_device_selector {
                                html! {
                                    <div class="device-selector-dropdown">
                                        { for available_devices.iter().enumerate().map(|(index, device)| {
                                            let device_id = device.device_id.clone();
                                            let is_selected = selected_device_id.as_ref() == Some(&device.device_id);
                                            let handle_click = {
                                                let handle_device_change = handle_device_change.clone();
                                                let device_id = device_id.clone();
                                                Callback::from(move |_| {
                                                    handle_device_change.emit(device_id.clone());
                                                })
                                            };
                                            
                                            html! {
                                                <div 
                                                    key={index}
                                                    class={if is_selected { "device-option selected" } else { "device-option" }}
                                                    onclick={handle_click}
                                                >
                                                    <span class="device-option-label">{ &device.label }</span>
                                                    { if is_selected {
                                                        html! { <span class="device-option-check">{ "✓" }</span> }
                                                    } else {
                                                        html! {}
                                                    }}
                                                </div>
                                            }
                                        })}
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}

                // Input Level Display
                <div class="microphone-input-section">
                    <div class="input-level-container">
                        <div class="input-level-header">
                            <span class="input-level-label">{ "Input Level:" }</span>
                            <span class={format!("input-level-value {}", level_class)}>
                                { format!("{}%", level_percentage) }
                            </span>
                        </div>
                        
                        <div class="input-level-bar-container">
                            <div class="input-level-bar-background">
                                <div 
                                    class={format!("input-level-bar {}", level_class)}
                                    style={format!("width: {}%", level_percentage.min(100))}
                                ></div>
                            </div>
                        </div>
                        
                        <div class="input-level-indicators">
                            <span class="level-indicator low">{ "0%" }</span>
                            <span class="level-indicator medium">{ "50%" }</span>
                            <span class="level-indicator high">{ "100%" }</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/*
 * JAVASCRIPT IMPLEMENTATION REQUIRED:
 * 
 * Add this JavaScript code to your index.html or a separate JS file:
 * 
 * window.setupMicrophoneAnalysis = function(stream) {
 *     try {
 *         const audioContext = new (window.AudioContext || window.webkitAudioContext)();
 *         const analyser = audioContext.createAnalyser();
 *         analyser.fftSize = 256;
 *         analyser.smoothingTimeConstant = 0.3;
 *         
 *         const source = audioContext.createMediaStreamSource(stream);
 *         source.connect(analyser);
 *         
 *         console.log('Real-time microphone analysis set up successfully');
 *         return analyser;
 *     } catch (error) {
 *         console.error('Failed to set up microphone analysis:', error);
 *         return null;
 *     }
 * };
 * 
 * window.getMicrophoneLevel = function(analyser) {
 *     if (!analyser) return 0.0;
 *     
 *     const bufferLength = analyser.fftSize;
 *     const dataArray = new Uint8Array(bufferLength);
 *     analyser.getByteTimeDomainData(dataArray);
 *     
 *     // Calculate RMS level
 *     let sum = 0;
 *     for (let i = 0; i < dataArray.length; i++) {
 *         const sample = (dataArray[i] - 128) / 128;
 *         sum += sample * sample;
 *     }
 *     
 *     const rms = Math.sqrt(sum / dataArray.length);
 *     return Math.min(rms * 5.0, 1.0); // Amplify sensitivity and cap at 1.0
 * };
 * 
 * window.cleanupMicrophoneAnalysis = function(analyser) {
 *     if (analyser && analyser.context) {
 *         analyser.context.close();
 *     }
 * };
 */ 