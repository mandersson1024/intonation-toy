//! Audio Event Types
//!
//! This module defines audio-specific events that can be published and subscribed to
//! by various components of the application. These events enable loose coupling between
//! the audio subsystem and other components like the console.

use crate::audio::{AudioPermission, AudioDevices, AudioContextState};

/// Audio-related events that can be published throughout the application
#[derive(Debug, Clone)]
pub enum AudioEvent {
    /// Audio device list has changed (devices added/removed)
    DeviceListChanged(AudioDevices),
    /// Audio permission state has changed
    PermissionChanged(AudioPermission),
    /// Audio context state has changed
    ContextStateChanged(AudioContextState),
}

impl AudioEvent {
    /// Get the event type as a string for subscription matching
    pub fn event_type(&self) -> &'static str {
        match self {
            AudioEvent::DeviceListChanged(_) => "device_list_changed",
            AudioEvent::PermissionChanged(_) => "permission_changed",
            AudioEvent::ContextStateChanged(_) => "context_state_changed",
        }
    }
    
    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            AudioEvent::DeviceListChanged(devices) => {
                format!("Audio devices changed: {} input, {} output", 
                    devices.input_devices.len(), 
                    devices.output_devices.len())
            }
            AudioEvent::PermissionChanged(permission) => {
                format!("Audio permission changed to: {}", permission)
            }
            AudioEvent::ContextStateChanged(state) => {
                format!("Audio context state changed to: {}", state)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioDevices;
    
    #[test]
    fn test_audio_event_types() {
        let devices = AudioDevices::new();
        let device_event = AudioEvent::DeviceListChanged(devices);
        assert_eq!(device_event.event_type(), "device_list_changed");
        
        let permission_event = AudioEvent::PermissionChanged(AudioPermission::Granted);
        assert_eq!(permission_event.event_type(), "permission_changed");
        
        let context_event = AudioEvent::ContextStateChanged(AudioContextState::Running);
        assert_eq!(context_event.event_type(), "context_state_changed");
    }
    
    #[test]
    fn test_audio_event_descriptions() {
        let devices = AudioDevices::new();
        let device_event = AudioEvent::DeviceListChanged(devices);
        assert!(device_event.description().contains("Audio devices changed"));
        
        let permission_event = AudioEvent::PermissionChanged(AudioPermission::Granted);
        assert!(permission_event.description().contains("Audio permission changed"));
        
        let context_event = AudioEvent::ContextStateChanged(AudioContextState::Running);
        assert!(context_event.description().contains("Audio context state changed"));
    }
}