//! # Microphone Panel Component
//!
//! Placeholder for migrated microphone panel component.
//! Will be implemented during component migration task.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use web_sys::console;
#[cfg(debug_assertions)]
use wasm_bindgen::prelude::*;
#[cfg(debug_assertions)]
use js_sys::Float32Array;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;

// Use modular services instead of legacy
#[cfg(debug_assertions)]
use crate::modules::audio_foundations::ModularAudioService;
#[cfg(debug_assertions)]
use crate::modules::application_core::error_service::ApplicationError;
#[cfg(debug_assertions)]
use crate::modules::developer_ui::components::microphone_permission::MicrophonePermission;

#[cfg(debug_assertions)]
#[derive(Clone, PartialEq)]
pub struct AudioInputDevice {
    pub device_id: String,
    pub label: String,
    pub group_id: String,
}

// Helper function to load audio input devices
#[cfg(debug_assertions)]
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
#[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
#[derive(Properties)]
pub struct MicrophonePanelProps {
    /// Callback when MediaStream is ready
    pub on_stream_ready: Callback<web_sys::MediaStream>,
    /// Error handler callback
    #[prop_or(None)]
    pub on_error: Option<Callback<ApplicationError>>,
    /// Audio engine for accessing audio data
    #[prop_or(None)]
    pub audio_engine: Option<Rc<RefCell<ModularAudioService>>>,
    /// Current MediaStream for input level monitoring
    #[prop_or(None)]
    pub media_stream: Option<web_sys::MediaStream>,
    /// Callback when device name changes
    #[prop_or(None)]
    pub on_device_name_change: Option<Callback<Option<String>>>,
}

#[cfg(debug_assertions)]
impl PartialEq for MicrophonePanelProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer equality for audio engine
        self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr()) &&
        self.media_stream.as_ref().map(|s| s.id()) == other.media_stream.as_ref().map(|s| s.id())
    }
}

/// Microphone sub-panel containing permission controls and input monitoring
#[cfg(debug_assertions)]
#[function_component(MicrophonePanel)]
pub fn microphone_panel(props: &MicrophonePanelProps) -> Html {
    let input_level = use_state(|| 0.0f32);
    let is_active = use_state(|| false);
    let analyser_node = use_state(|| None::<JsValue>);
    let device_name = use_state(|| None::<String>);
    let available_devices = use_state(|| Vec::<AudioInputDevice>::new());
    let selected_device_id = use_state(|| None::<String>);
    let show_device_selector = use_state(|| false);
    
    // Load available input devices when we have permission
    {
        let available_devices = available_devices.clone();
        let media_stream = props.media_stream.clone();
        
        use_effect_with(media_stream, move |stream| {
            if stream.is_some() {
                let devices = available_devices.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let device_list = load_audio_input_devices(false).await;
                    devices.set(device_list);
                });
            }
            || ()
        });
    }

    html! {
        <div class="microphone-panel">
            <div class="permission-section">
                <MicrophonePermission
                    on_stream_ready={props.on_stream_ready.clone()}
                    on_error={props.on_error.clone()}
                />
            </div>
            
            if props.media_stream.is_some() {
                <div class="device-controls">
                    <div class="device-selector">
                        <label>{ "Input Device:" }</label>
                        <select>
                            { for available_devices.iter().map(|device| {
                                html! {
                                    <option value={device.device_id.clone()}>
                                        { &device.label }
                                    </option>
                                }
                            }) }
                        </select>
                    </div>
                    
                    <div class="input-level">
                        <label>{ "Input Level:" }</label>
                        <div class="level-meter">
                            <div 
                                class="level-bar" 
                                style={format!("width: {}%", *input_level * 100.0)}
                            ></div>
                        </div>
                        <span class="level-value">{ format!("{:.1}%", *input_level * 100.0) }</span>
                    </div>
                </div>
            }
        </div>
    }
} 