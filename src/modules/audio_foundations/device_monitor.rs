// Device Monitor Implementation - STORY-014
// Monitors audio device state changes and handles device events

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, Event, MediaDevices, Navigator, Window};
use super::audio_events::*;
use super::device_manager::{AudioDevice, DeviceError};
use crate::modules::application_core::event_bus::EventBus;
use crate::modules::application_core::typed_event_bus::TypedEventBus;

/// Trait for monitoring device state changes
pub trait DeviceMonitor: Send + Sync {
    /// Start monitoring device changes
    fn start_monitoring(&mut self) -> Result<(), DeviceMonitorError>;
    
    /// Stop monitoring device changes
    fn stop_monitoring(&mut self) -> Result<(), DeviceMonitorError>;
    
    /// Check if monitoring is active
    fn is_monitoring(&self) -> bool;
    
    /// Get the current device monitoring state
    fn get_monitoring_state(&self) -> DeviceMonitoringState;
    
    /// Handle device connection event
    fn handle_device_connected(&mut self, device: AudioDevice);
    
    /// Handle device disconnection event
    fn handle_device_disconnected(&mut self, device_id: &str);
    
    /// Handle device error event
    fn handle_device_error(&mut self, device_id: &str, error: String);
}

/// Device monitoring errors
#[derive(Debug, Clone)]
pub enum DeviceMonitorError {
    BrowserNotSupported,
    MonitoringAlreadyActive,
    MonitoringNotActive,
    EventListenerError(String),
    InternalError(String),
}

impl fmt::Display for DeviceMonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceMonitorError::BrowserNotSupported => {
                write!(f, "Browser does not support device monitoring")
            }
            DeviceMonitorError::MonitoringAlreadyActive => {
                write!(f, "Device monitoring is already active")
            }
            DeviceMonitorError::MonitoringNotActive => {
                write!(f, "Device monitoring is not active")
            }
            DeviceMonitorError::EventListenerError(msg) => {
                write!(f, "Error setting up event listener: {}", msg)
            }
            DeviceMonitorError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
        }
    }
}

impl Error for DeviceMonitorError {}

/// Device monitoring state
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceMonitoringState {
    Inactive,
    Starting,
    Active,
    Stopping,
    Error(String),
}

/// Web-based device monitor implementation
pub struct WebDeviceMonitor {
    media_devices: MediaDevices,
    monitoring_state: DeviceMonitoringState,
    known_devices: HashMap<String, AudioDevice>,
    event_bus: Option<Arc<TypedEventBus>>,
    device_change_closure: Option<Closure<dyn FnMut(Event)>>,
}

impl WebDeviceMonitor {
    /// Create a new web device monitor
    pub fn new() -> Result<Self, DeviceMonitorError> {
        let window = web_sys::window().ok_or(DeviceMonitorError::BrowserNotSupported)?;
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| DeviceMonitorError::BrowserNotSupported)?;
        
        Ok(Self {
            media_devices,
            monitoring_state: DeviceMonitoringState::Inactive,
            known_devices: HashMap::new(),
            event_bus: None,
            device_change_closure: None,
        })
    }
    
    /// Set event bus for publishing device events
    pub fn set_event_bus(&mut self, event_bus: Arc<TypedEventBus>) {
        self.event_bus = Some(event_bus);
    }
    
    /// Initialize the known devices list
    pub async fn initialize_device_list(&mut self) -> Result<(), DeviceMonitorError> {
        // This would typically enumerate current devices and store them
        // For now, we'll just clear the known devices list
        self.known_devices.clear();
        
        // In a real implementation, you'd call enumerate_devices() here
        // and populate the known_devices HashMap
        
        Ok(())
    }
    
    /// Set up device change event listener
    fn setup_device_change_listener(&mut self) -> Result<(), DeviceMonitorError> {
        // Create a closure that will handle device change events
        let event_bus = self.event_bus.clone();
        let mut known_devices = self.known_devices.clone();
        
        let closure = Closure::wrap(Box::new(move |_event: Event| {
            // Device change detected - we need to re-enumerate devices
            // and compare with known devices to detect changes
            web_sys::console::log_1(&"Device change detected".into());
            
            // In a real implementation, you would:
            // 1. Re-enumerate devices
            // 2. Compare with known_devices
            // 3. Identify added/removed devices
            // 4. Publish appropriate events
            
            if let Some(ref event_bus) = event_bus {
                let event = DeviceMonitoringEvent {
                    event_type: DeviceMonitoringEventType::DeviceListChanged,
                    message: "Device list has changed".to_string(),
                    timestamp_ns: get_timestamp_ns(),
                };
                
                if let Err(e) = event_bus.publish(event) {
                    web_sys::console::warn_1(&format!("Failed to publish device monitoring event: {:?}", e).into());
                }
            }
        }) as Box<dyn FnMut(Event)>);
        
        // Add event listener to media devices
        let event_target: &EventTarget = self.media_devices.as_ref();
        event_target.add_event_listener_with_callback("devicechange", closure.as_ref().unchecked_ref())
            .map_err(|e| DeviceMonitorError::EventListenerError(format!("Failed to add devicechange listener: {:?}", e)))?;
        
        // Store the closure to prevent it from being dropped
        self.device_change_closure = Some(closure);
        
        Ok(())
    }
    
    /// Remove device change event listener
    fn remove_device_change_listener(&mut self) -> Result<(), DeviceMonitorError> {
        if let Some(ref closure) = self.device_change_closure {
            let event_target: &EventTarget = self.media_devices.as_ref();
            event_target.remove_event_listener_with_callback("devicechange", closure.as_ref().unchecked_ref())
                .map_err(|e| DeviceMonitorError::EventListenerError(format!("Failed to remove devicechange listener: {:?}", e)))?;
        }
        
        self.device_change_closure = None;
        Ok(())
    }
    
    /// Publish device monitoring event
    fn publish_monitoring_event(&self, event_type: DeviceMonitoringEventType, message: String) {
        if let Some(ref event_bus) = self.event_bus {
            let event = DeviceMonitoringEvent {
                event_type,
                message,
                timestamp_ns: get_timestamp_ns(),
            };
            
            if let Err(e) = event_bus.publish(event) {
                web_sys::console::warn_1(&format!("Failed to publish device monitoring event: {:?}", e).into());
            }
        }
    }
    
    /// Publish device state change event
    fn publish_device_state_event(&self, device_id: &str, state: DeviceState, device_info: Option<AudioDevice>) {
        if let Some(ref event_bus) = self.event_bus {
            let event = MicrophoneStateEvent {
                state,
                device_info,
                permissions: super::permission_manager::PermissionState::Granted, // Placeholder
                timestamp_ns: get_timestamp_ns(),
            };
            
            if let Err(e) = event_bus.publish(event) {
                web_sys::console::warn_1(&format!("Failed to publish device state event: {:?}", e).into());
            }
        }
    }
    
    /// Handle graceful recovery from device changes during recording
    pub fn handle_device_change_during_recording(&mut self, device_id: &str) -> Result<DeviceRecoveryAction, DeviceMonitorError> {
        // Check if the device that changed is currently in use
        if let Some(device) = self.known_devices.get(device_id) {
            // Device is known - determine recovery action based on device type and current state
            let recovery_action = match device.device_type {
                super::device_manager::AudioDeviceType::Input => {
                    // Input device changed - need to handle recording interruption
                    DeviceRecoveryAction::SwitchToDefaultDevice
                }
                super::device_manager::AudioDeviceType::Output => {
                    // Output device changed - less critical
                    DeviceRecoveryAction::ContinueWithWarning
                }
                super::device_manager::AudioDeviceType::InputOutput => {
                    // Dual-purpose device - handle as input device
                    DeviceRecoveryAction::SwitchToDefaultDevice
                }
            };
            
            // Publish recovery event
            self.publish_monitoring_event(
                DeviceMonitoringEventType::RecoveryActionRequired,
                format!("Device {} changed during recording, recovery action: {:?}", device_id, recovery_action)
            );
            
            Ok(recovery_action)
        } else {
            // Unknown device - safe to ignore
            Ok(DeviceRecoveryAction::NoActionRequired)
        }
    }
}

impl DeviceMonitor for WebDeviceMonitor {
    fn start_monitoring(&mut self) -> Result<(), DeviceMonitorError> {
        if self.monitoring_state == DeviceMonitoringState::Active {
            return Err(DeviceMonitorError::MonitoringAlreadyActive);
        }
        
        self.monitoring_state = DeviceMonitoringState::Starting;
        
        // Set up device change event listener
        self.setup_device_change_listener()?;
        
        self.monitoring_state = DeviceMonitoringState::Active;
        
        // Publish monitoring started event
        self.publish_monitoring_event(
            DeviceMonitoringEventType::MonitoringStarted,
            "Device monitoring started".to_string()
        );
        
        web_sys::console::log_1(&"Device monitoring started successfully".into());
        Ok(())
    }
    
    fn stop_monitoring(&mut self) -> Result<(), DeviceMonitorError> {
        if self.monitoring_state != DeviceMonitoringState::Active {
            return Err(DeviceMonitorError::MonitoringNotActive);
        }
        
        self.monitoring_state = DeviceMonitoringState::Stopping;
        
        // Remove device change event listener
        self.remove_device_change_listener()?;
        
        self.monitoring_state = DeviceMonitoringState::Inactive;
        
        // Publish monitoring stopped event
        self.publish_monitoring_event(
            DeviceMonitoringEventType::MonitoringStopped,
            "Device monitoring stopped".to_string()
        );
        
        web_sys::console::log_1(&"Device monitoring stopped successfully".into());
        Ok(())
    }
    
    fn is_monitoring(&self) -> bool {
        self.monitoring_state == DeviceMonitoringState::Active
    }
    
    fn get_monitoring_state(&self) -> DeviceMonitoringState {
        self.monitoring_state.clone()
    }
    
    fn handle_device_connected(&mut self, device: AudioDevice) {
        let device_id = device.device_id.clone();
        
        // Add device to known devices
        self.known_devices.insert(device_id.clone(), device.clone());
        
        // Publish device connected event
        self.publish_device_state_event(&device_id, DeviceState::Connected, Some(device));
        
        // Publish monitoring event
        self.publish_monitoring_event(
            DeviceMonitoringEventType::DeviceConnected,
            format!("Device connected: {}", device_id)
        );
        
        web_sys::console::log_1(&format!("Device connected: {}", device_id).into());
    }
    
    fn handle_device_disconnected(&mut self, device_id: &str) {
        // Get device info before removing
        let device_info = self.known_devices.remove(device_id);
        
        // Publish device disconnected event
        self.publish_device_state_event(device_id, DeviceState::Disconnected, device_info);
        
        // Publish monitoring event
        self.publish_monitoring_event(
            DeviceMonitoringEventType::DeviceDisconnected,
            format!("Device disconnected: {}", device_id)
        );
        
        web_sys::console::log_1(&format!("Device disconnected: {}", device_id).into());
    }
    
    fn handle_device_error(&mut self, device_id: &str, error: String) {
        if let Some(device) = self.known_devices.get(device_id) {
            // Publish device error event
            self.publish_device_state_event(device_id, DeviceState::Error(error.clone()), Some(device.clone()));
            
            // Log error details
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&format!(
                "Device error on {}: {}", device_id, error
            ).into());
        }
        
        // Publish monitoring event
        self.publish_monitoring_event(
            DeviceMonitoringEventType::DeviceError,
            format!("Device error on {}: {}", device_id, error)
        );
    }
}

/// Actions to take when device changes occur during recording
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceRecoveryAction {
    NoActionRequired,
    SwitchToDefaultDevice,
    PauseRecording,
    StopRecording,
    ContinueWithWarning,
    RequestUserInput,
}

// Utility function to get current timestamp
fn get_timestamp_ns() -> u64 {
    if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
        (performance.now() * 1_000_000.0) as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::device_manager::{AudioDevice, AudioDeviceType};
    
    #[test]
    fn test_device_monitoring_state() {
        let state = DeviceMonitoringState::Active;
        assert_eq!(state, DeviceMonitoringState::Active);
        
        let state = DeviceMonitoringState::Error("Test error".to_string());
        match state {
            DeviceMonitoringState::Error(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Error state"),
        }
    }
    
    #[test]
    fn test_device_recovery_action() {
        let action = DeviceRecoveryAction::SwitchToDefaultDevice;
        assert_eq!(action, DeviceRecoveryAction::SwitchToDefaultDevice);
    }
    
    #[test]
    fn test_device_monitor_error_display() {
        let error = DeviceMonitorError::BrowserNotSupported;
        assert_eq!(error.to_string(), "Browser does not support device monitoring");
        
        let error = DeviceMonitorError::EventListenerError("Test error".to_string());
        assert_eq!(error.to_string(), "Error setting up event listener: Test error");
    }
}