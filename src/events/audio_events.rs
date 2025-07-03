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
    /// Circular buffer has been filled (ready for processing)
    BufferFilled { buffer_index: usize, length: usize },
    /// Circular buffer experienced overflow
    BufferOverflow { buffer_index: usize, overflow_count: usize },
    /// Buffer pool metrics update (periodic)
    BufferMetrics { total_buffers: usize, total_overflows: usize, memory_bytes: usize },
}

impl AudioEvent {
    /// Get the event type as a string for subscription matching
    pub fn event_type(&self) -> &'static str {
        match self {
            AudioEvent::DeviceListChanged(_) => "device_list_changed",
            AudioEvent::PermissionChanged(_) => "permission_changed",
            AudioEvent::ContextStateChanged(_) => "context_state_changed",
            AudioEvent::BufferFilled { .. } => "buffer_filled",
            AudioEvent::BufferOverflow { .. } => "buffer_overflow",
            AudioEvent::BufferMetrics { .. } => "buffer_metrics",
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
            AudioEvent::BufferFilled { buffer_index, length } => {
                format!("Buffer {} filled ({} samples)", buffer_index, length)
            }
            AudioEvent::BufferOverflow { buffer_index, overflow_count } => {
                format!("Buffer {} overflow (count = {})", buffer_index, overflow_count)
            }
            AudioEvent::BufferMetrics { total_buffers, total_overflows, memory_bytes } => {
                format!("Buffer metrics: {} buffers, {} overflows, {:.2} MB", total_buffers, total_overflows, *memory_bytes as f64 / 1_048_576.0)
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

    #[test]
    fn test_buffer_event_types_and_descriptions() {
        let filled = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        assert_eq!(filled.event_type(), "buffer_filled");
        assert!(filled.description().contains("Buffer 0 filled"));

        let overflow = AudioEvent::BufferOverflow { buffer_index: 1, overflow_count: 3 };
        assert_eq!(overflow.event_type(), "buffer_overflow");
        assert!(overflow.description().contains("overflow"));

        let metrics = AudioEvent::BufferMetrics { total_buffers: 8, total_overflows: 5, memory_bytes: 32768 };
        assert_eq!(metrics.event_type(), "buffer_metrics");
        assert!(metrics.description().contains("8 buffers"));
    }
}