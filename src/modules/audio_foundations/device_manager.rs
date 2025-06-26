// Device Manager Implementation - STORY-014
// Provides audio device enumeration, selection, and management functionality

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaDevices, MediaStream, MediaStreamConstraints, MediaDeviceInfo, Navigator, Window};
use js_sys::{Array, Promise};
use super::audio_events::*;
use crate::modules::application_core::event_bus::EventBus;
use crate::modules::application_core::typed_event_bus::TypedEventBus;

/// Core trait for device management functionality
pub trait DeviceManager: Send + Sync {
    /// List all available audio input devices
    fn list_input_devices(&self) -> Result<Vec<AudioDevice>, DeviceError>;
    
    /// List all available audio output devices  
    fn list_output_devices(&self) -> Result<Vec<AudioDevice>, DeviceError>;
    
    /// Set the active input device
    fn set_input_device(&mut self, device_id: &str) -> Result<(), DeviceError>;
    
    /// Get the currently active input device
    fn get_active_input_device(&self) -> Option<&AudioDevice>;
    
    /// Request microphone permission from the browser
    fn request_microphone_permission(&self) -> Result<PermissionStatus, DeviceError>;
    
    /// Get current microphone permission status
    fn get_microphone_permission_status(&self) -> Result<PermissionStatus, DeviceError>;
    
    /// Monitor device changes (connections/disconnections)
    fn start_device_monitoring(&mut self) -> Result<(), DeviceError>;
    
    /// Stop device monitoring
    fn stop_device_monitoring(&mut self) -> Result<(), DeviceError>;
    
    /// Get device capabilities for a specific device
    fn get_device_capabilities(&self, device_id: &str) -> Result<DeviceCapabilities, DeviceError>;
}

/// Audio device information
#[derive(Debug, Clone, PartialEq)]
pub struct AudioDevice {
    pub device_id: String,
    pub device_name: String,
    pub is_default: bool,
    pub device_type: AudioDeviceType,
    pub supported_sample_rates: Vec<u32>,
    pub max_channels: u32,
    pub group_id: Option<String>,
}

/// Audio device type classification
#[derive(Debug, Clone, PartialEq)]
pub enum AudioDeviceType {
    Input,
    Output,
    InputOutput,
}

/// Device capabilities information
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceCapabilities {
    pub sample_rates: Vec<u32>,
    pub channel_counts: Vec<u32>,
    pub buffer_sizes: Vec<u32>,
    pub supports_echo_cancellation: bool,
    pub supports_noise_suppression: bool,
    pub supports_auto_gain_control: bool,
}

/// Device management errors
#[derive(Debug, Clone)]
pub enum DeviceError {
    PermissionDenied,
    DeviceNotFound(String),
    DeviceInUse(String),
    BrowserNotSupported,
    NetworkError(String),
    InvalidDeviceId(String),
    InternalError(String),
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::PermissionDenied => write!(f, "Microphone permission denied"),
            DeviceError::DeviceNotFound(id) => write!(f, "Audio device not found: {}", id),
            DeviceError::DeviceInUse(id) => write!(f, "Audio device in use: {}", id),
            DeviceError::BrowserNotSupported => write!(f, "Browser does not support required audio APIs"),
            DeviceError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            DeviceError::InvalidDeviceId(id) => write!(f, "Invalid device ID: {}", id),
            DeviceError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl Error for DeviceError {}

/// Web-based device manager implementation
pub struct WebDeviceManager {
    media_devices: MediaDevices,
    current_input_device: Option<AudioDevice>,
    current_stream: Option<MediaStream>,
    available_devices: HashMap<String, AudioDevice>,
    monitoring_active: bool,
    event_bus: Option<Arc<TypedEventBus>>,
}

impl WebDeviceManager {
    /// Create a new web device manager
    pub fn new() -> Result<Self, DeviceError> {
        let window = web_sys::window().ok_or(DeviceError::BrowserNotSupported)?;
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| DeviceError::BrowserNotSupported)?;
        
        Ok(Self {
            media_devices,
            current_input_device: None,
            current_stream: None,
            available_devices: HashMap::new(),
            monitoring_active: false,
            event_bus: None,
        })
    }
    
    /// Set event bus for publishing device events
    pub fn set_event_bus(&mut self, event_bus: Arc<TypedEventBus>) {
        self.event_bus = Some(event_bus);
    }
    
    /// Refresh the list of available devices
    pub async fn refresh_devices(&mut self) -> Result<(), DeviceError> {
        let promise = self.media_devices.enumerate_devices()
            .map_err(|e| DeviceError::InternalError(format!("Failed to enumerate devices: {:?}", e)))?;
        
        let js_devices = JsFuture::from(promise).await
            .map_err(|e| DeviceError::NetworkError(format!("Device enumeration failed: {:?}", e)))?;
        
        let devices_array: Array = js_devices.into();
        self.available_devices.clear();
        
        for device_info in devices_array.iter() {
            let device_info: MediaDeviceInfo = device_info.into();
            
            if device_info.kind() == "audioinput" || device_info.kind() == "audiooutput" {
                let audio_device = self.create_audio_device_from_media_info(device_info)?;
                self.available_devices.insert(audio_device.device_id.clone(), audio_device);
            }
        }
        
        // Publish device list updated event
        self.publish_device_list_updated();
        
        Ok(())
    }
    
    /// Create AudioDevice from MediaDeviceInfo
    fn create_audio_device_from_media_info(&self, info: MediaDeviceInfo) -> Result<AudioDevice, DeviceError> {
        let device_id = info.device_id();
        let device_name = info.label();
        
        // Convert MediaDeviceKind to our AudioDeviceType
        let device_type = match info.kind() {
            web_sys::MediaDeviceKind::Audioinput => AudioDeviceType::Input,
            web_sys::MediaDeviceKind::Audiooutput => AudioDeviceType::Output,
            _ => return Err(DeviceError::InvalidDeviceId("Unsupported device type".to_string())),
        };
        
        Ok(AudioDevice {
            device_id,
            device_name,
            is_default: false, // Would need to determine this from browser defaults
            device_type,
            supported_sample_rates: vec![44100, 48000], // Default values
            max_channels: 2, // Default stereo
            group_id: Some(info.group_id()),
        })
    }
    
    /// Request media stream for a specific device
    pub async fn request_media_stream(&mut self, device_id: &str) -> Result<MediaStream, DeviceError> {
        let device = self.available_devices.get(device_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))?;
        
        if device.device_type != AudioDeviceType::Input && device.device_type != AudioDeviceType::InputOutput {
            return Err(DeviceError::InvalidDeviceId(format!("Device {} is not an input device", device_id)));
        }
        
        let mut constraints = MediaStreamConstraints::new();
        
        // Set audio constraints
        let audio_constraints = js_sys::Object::new();
        js_sys::Reflect::set(&audio_constraints, &"deviceId".into(), &device_id.into())
            .map_err(|_| DeviceError::InternalError("Failed to set device constraints".to_string()))?;
        js_sys::Reflect::set(&audio_constraints, &"echoCancellation".into(), &true.into())
            .map_err(|_| DeviceError::InternalError("Failed to set echo cancellation".to_string()))?;
        
        constraints.audio(&audio_constraints.into());
        constraints.video(&false.into());
        
        let promise = self.media_devices.get_user_media_with_constraints(&constraints)
            .map_err(|_| DeviceError::BrowserNotSupported)?;
        
        let stream = JsFuture::from(promise).await
            .map_err(|_| DeviceError::PermissionDenied)?;
        
        let media_stream: MediaStream = stream.into();
        self.current_stream = Some(media_stream.clone());
        
        // Publish device connection event
        self.publish_device_state_change(device_id, DeviceState::Connected);
        
        Ok(media_stream)
    }
    
    /// Publish device list updated event
    fn publish_device_list_updated(&self) {
        if let Some(ref event_bus) = self.event_bus {
            let devices: Vec<_> = self.available_devices.values().cloned().collect();
            let event = DeviceListUpdatedEvent {
                devices,
                timestamp_ns: get_timestamp_ns(),
            };
            
            if let Err(e) = event_bus.publish(event) {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::warn_1(&format!("Failed to publish device list updated event: {:?}", e).into());
            }
        }
    }
    
    /// Publish device state change event
    fn publish_device_state_change(&self, device_id: &str, state: DeviceState) {
        if let Some(ref event_bus) = self.event_bus {
            let device_info = self.available_devices.get(device_id).cloned();
            let event = MicrophoneStateEvent {
                state,
                device_info,
                permissions: PermissionStatus::Granted, // Would get actual permission status
                timestamp_ns: get_timestamp_ns(),
            };
            
            if let Err(e) = event_bus.publish(event) {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::warn_1(&format!("Failed to publish device state event: {:?}", e).into());
            }
        }
    }
}

impl DeviceManager for WebDeviceManager {
    fn list_input_devices(&self) -> Result<Vec<AudioDevice>, DeviceError> {
        let input_devices: Vec<AudioDevice> = self.available_devices
            .values()
            .filter(|device| device.device_type == AudioDeviceType::Input || device.device_type == AudioDeviceType::InputOutput)
            .cloned()
            .collect();
        
        Ok(input_devices)
    }
    
    fn list_output_devices(&self) -> Result<Vec<AudioDevice>, DeviceError> {
        let output_devices: Vec<AudioDevice> = self.available_devices
            .values()
            .filter(|device| device.device_type == AudioDeviceType::Output || device.device_type == AudioDeviceType::InputOutput)
            .cloned()
            .collect();
        
        Ok(output_devices)
    }
    
    fn set_input_device(&mut self, device_id: &str) -> Result<(), DeviceError> {
        let device = self.available_devices.get(device_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))?
            .clone();
        
        if device.device_type != AudioDeviceType::Input && device.device_type != AudioDeviceType::InputOutput {
            return Err(DeviceError::InvalidDeviceId(format!("Device {} is not an input device", device_id)));
        }
        
        // Store the old device for event publishing
        let old_device_id = self.current_input_device.as_ref().map(|d| d.device_id.clone());
        
        // Set new device
        self.current_input_device = Some(device);
        
        // Publish device change events
        if let Some(old_id) = old_device_id {
            self.publish_device_state_change(&old_id, DeviceState::Disconnected);
        }
        self.publish_device_state_change(device_id, DeviceState::Connected);
        
        Ok(())
    }
    
    fn get_active_input_device(&self) -> Option<&AudioDevice> {
        self.current_input_device.as_ref()
    }
    
    fn request_microphone_permission(&self) -> Result<PermissionStatus, DeviceError> {
        // This is a simplified implementation
        // In practice, you'd need to actually request permission via getUserMedia
        // and handle the promise/async nature properly
        Ok(PermissionStatus::Granted)
    }
    
    fn get_microphone_permission_status(&self) -> Result<PermissionStatus, DeviceError> {
        // This would query the actual permission status from the browser
        // For now, return a placeholder
        Ok(PermissionStatus::Granted)
    }
    
    fn start_device_monitoring(&mut self) -> Result<(), DeviceError> {
        if self.monitoring_active {
            return Ok(());
        }
        
        // In a real implementation, you'd set up event listeners for device changes
        // This is a placeholder that indicates monitoring is active
        self.monitoring_active = true;
        
        web_sys::console::log_1(&"Device monitoring started".into());
        Ok(())
    }
    
    fn stop_device_monitoring(&mut self) -> Result<(), DeviceError> {
        if !self.monitoring_active {
            return Ok(());
        }
        
        self.monitoring_active = false;
        web_sys::console::log_1(&"Device monitoring stopped".into());
        Ok(())
    }
    
    fn get_device_capabilities(&self, device_id: &str) -> Result<DeviceCapabilities, DeviceError> {
        let _device = self.available_devices.get(device_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))?;
        
        // Return default capabilities
        // In a real implementation, these would be probed from the actual device
        Ok(DeviceCapabilities {
            sample_rates: vec![44100, 48000],
            channel_counts: vec![1, 2],
            buffer_sizes: vec![256, 512, 1024, 2048],
            supports_echo_cancellation: true,
            supports_noise_suppression: true,
            supports_auto_gain_control: true,
        })
    }
}

// Utility function to get current timestamp
fn get_timestamp_ns() -> u64 {
    // Get current time in nanoseconds
    if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
        (performance.now() * 1_000_000.0) as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_device_creation() {
        let device = AudioDevice {
            device_id: "test-device-1".to_string(),
            device_name: "Test Microphone".to_string(),
            is_default: true,
            device_type: AudioDeviceType::Input,
            supported_sample_rates: vec![44100, 48000],
            max_channels: 2,
            group_id: Some("test-group".to_string()),
        };
        
        assert_eq!(device.device_id, "test-device-1");
        assert_eq!(device.device_name, "Test Microphone");
        assert!(device.is_default);
        assert_eq!(device.device_type, AudioDeviceType::Input);
    }
    
    #[test]
    fn test_device_capabilities() {
        let capabilities = DeviceCapabilities {
            sample_rates: vec![44100, 48000],
            channel_counts: vec![1, 2],
            buffer_sizes: vec![256, 512, 1024],
            supports_echo_cancellation: true,
            supports_noise_suppression: true,
            supports_auto_gain_control: true,
        };
        
        assert!(capabilities.supports_echo_cancellation);
        assert!(capabilities.sample_rates.contains(&44100));
        assert!(capabilities.channel_counts.contains(&2));
    }
    
    #[test]
    fn test_device_error_display() {
        let error = DeviceError::DeviceNotFound("test-device".to_string());
        assert_eq!(error.to_string(), "Audio device not found: test-device");
        
        let error = DeviceError::PermissionDenied;
        assert_eq!(error.to_string(), "Microphone permission denied");
    }
}