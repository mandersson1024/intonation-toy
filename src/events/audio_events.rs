//! Audio Event Types
//!
//! This module defines audio-specific events that can be published and subscribed to
//! by various components of the application. These events enable loose coupling between
//! the audio subsystem and other components like the console.

use crate::audio::{AudioPermission, AudioDevices, AudioContextState, MusicalNote, VolumeLevel};

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
    /// Pitch successfully detected
    PitchDetected {
        frequency: f32,
        confidence: f32,
        note: MusicalNote,
        clarity: f32,
        timestamp: f64,
    },
    /// Pitch detection lost (below threshold)
    PitchLost {
        last_frequency: f32,
        timestamp: f64,
    },
    /// Confidence level changed significantly
    ConfidenceChanged {
        frequency: f32,
        confidence: f32,
        timestamp: f64,
    },
    /// Volume level detected from audio input
    VolumeDetected {
        rms_db: f32,
        peak_db: f32,
        peak_fast_db: f32,
        peak_slow_db: f32,
        level: VolumeLevel,
        confidence_weight: f32,
        timestamp: f64,
    },
    /// Volume level changed significantly
    VolumeChanged {
        previous_rms_db: f32,
        current_rms_db: f32,
        change_db: f32,
        timestamp: f64,
    },
    /// Volume warning for problematic levels
    VolumeWarning {
        level: VolumeLevel,
        rms_db: f32,
        message: String,
        timestamp: f64,
    },
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
            AudioEvent::PitchDetected { .. } => "pitch_detected",
            AudioEvent::PitchLost { .. } => "pitch_lost",
            AudioEvent::ConfidenceChanged { .. } => "pitch_confidence_changed",
            AudioEvent::VolumeDetected { .. } => "volume_detected",
            AudioEvent::VolumeChanged { .. } => "volume_changed",
            AudioEvent::VolumeWarning { .. } => "volume_warning",
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
            AudioEvent::PitchDetected { frequency, confidence, note, .. } => {
                format!("Pitch detected: {:.2}Hz ({}) confidence={:.2}", frequency, note, confidence)
            }
            AudioEvent::PitchLost { last_frequency, .. } => {
                format!("Pitch lost (was {:.2}Hz)", last_frequency)
            }
            AudioEvent::ConfidenceChanged { frequency, confidence, .. } => {
                format!("Confidence changed: {:.2}Hz confidence={:.2}", frequency, confidence)
            }
            AudioEvent::VolumeDetected { rms_db, peak_db, level, confidence_weight, .. } => {
                format!("Volume detected: RMS={:.1}dB, Peak={:.1}dB, Level={}, Confidence={:.2}", 
                    rms_db, peak_db, level, confidence_weight)
            }
            AudioEvent::VolumeChanged { previous_rms_db, current_rms_db, change_db, .. } => {
                format!("Volume changed: {:.1}dB → {:.1}dB (Δ{:.1}dB)", 
                    previous_rms_db, current_rms_db, change_db)
            }
            AudioEvent::VolumeWarning { level, rms_db, message, .. } => {
                format!("Volume warning: {} ({:.1}dB) - {}", level, rms_db, message)
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

    #[test]
    fn test_pitch_event_types_and_descriptions() {
        use crate::audio::{NoteName, MusicalNote};

        let note = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        let detected = AudioEvent::PitchDetected {
            frequency: 440.0,
            confidence: 0.9,
            note,
            clarity: 0.8,
            timestamp: 1000.0,
        };
        assert_eq!(detected.event_type(), "pitch_detected");
        assert!(detected.description().contains("440.00Hz"));
        assert!(detected.description().contains("A4"));
        assert!(detected.description().contains("confidence=0.90"));

        let lost = AudioEvent::PitchLost {
            last_frequency: 440.0,
            timestamp: 1000.0,
        };
        assert_eq!(lost.event_type(), "pitch_lost");
        assert!(lost.description().contains("was 440.00Hz"));

        let confidence_changed = AudioEvent::ConfidenceChanged {
            frequency: 440.0,
            confidence: 0.7,
            timestamp: 1000.0,
        };
        assert_eq!(confidence_changed.event_type(), "pitch_confidence_changed");
        assert!(confidence_changed.description().contains("440.00Hz"));
        assert!(confidence_changed.description().contains("confidence=0.70"));
    }

    #[test]
    fn test_volume_event_types_and_descriptions() {
        use crate::audio::VolumeLevel;

        let detected = AudioEvent::VolumeDetected {
            rms_db: -12.0,
            peak_db: -6.0,
            peak_fast_db: -8.0,
            peak_slow_db: -10.0,
            level: VolumeLevel::Optimal,
            confidence_weight: 0.8,
            timestamp: 1000.0,
        };
        assert_eq!(detected.event_type(), "volume_detected");
        assert!(detected.description().contains("Volume detected"));
        assert!(detected.description().contains("RMS=-12.0dB"));
        assert!(detected.description().contains("Peak=-6.0dB"));
        assert!(detected.description().contains("Level=Optimal"));
        assert!(detected.description().contains("Confidence=0.80"));

        let changed = AudioEvent::VolumeChanged {
            previous_rms_db: -15.0,
            current_rms_db: -12.0,
            change_db: 3.0,
            timestamp: 1000.0,
        };
        assert_eq!(changed.event_type(), "volume_changed");
        assert!(changed.description().contains("Volume changed"));
        assert!(changed.description().contains("-15.0dB → -12.0dB"));
        assert!(changed.description().contains("Δ3.0dB"));

        let warning = AudioEvent::VolumeWarning {
            level: VolumeLevel::Clipping,
            rms_db: 2.0,
            message: "Input level too high".to_string(),
            timestamp: 1000.0,
        };
        assert_eq!(warning.event_type(), "volume_warning");
        assert!(warning.description().contains("Volume warning"));
        assert!(warning.description().contains("Clipping"));
        assert!(warning.description().contains("2.0dB"));
        assert!(warning.description().contains("Input level too high"));
    }

    // Event Publishing and Subscription Integration Tests (Task 8 Requirements)
    
    #[test]
    fn test_event_dispatcher_integration() {
        use super::super::event_dispatcher::EventDispatcher;
        use std::rc::Rc;
        use std::cell::RefCell;
        
        let mut dispatcher = EventDispatcher::new();
        let events_received = Rc::new(RefCell::new(Vec::new()));
        
        // Subscribe to pitch detected events
        let events_clone = events_received.clone();
        dispatcher.subscribe("pitch_detected", move |event| {
            events_clone.borrow_mut().push(event.clone());
        });
        
        // Create and publish a pitch event
        let pitch_event = AudioEvent::PitchDetected {
            frequency: 440.0,
            confidence: 0.8,
            note: crate::audio::MusicalNote::new(
                crate::audio::NoteName::A,
                4,
                0.0,
                440.0
            ),
            clarity: 0.7,
            timestamp: 1000.0,
        };
        
        // Publish event
        dispatcher.publish(pitch_event.clone());
        
        // Verify event was received
        let received = events_received.borrow();
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0], AudioEvent::PitchDetected { .. }));
        
        // Verify subscriber count
        assert_eq!(dispatcher.subscriber_count("pitch_detected"), 1);
    }

    #[test]  
    fn test_event_types_integration() {
        // Test that all event types work correctly
        let pitch_detected = AudioEvent::PitchDetected {
            frequency: 440.0,
            confidence: 0.8,
            note: crate::audio::MusicalNote::new(
                crate::audio::NoteName::A,
                4,
                0.0,
                440.0
            ),
            clarity: 0.7,
            timestamp: 1000.0,
        };
        assert_eq!(pitch_detected.event_type(), "pitch_detected");
        
        let pitch_lost = AudioEvent::PitchLost {
            last_frequency: 440.0,
            timestamp: 2000.0,
        };
        assert_eq!(pitch_lost.event_type(), "pitch_lost");
        
        let confidence_changed = AudioEvent::ConfidenceChanged {
            frequency: 440.0,
            confidence: 0.6,
            timestamp: 3000.0,
        };
        assert_eq!(confidence_changed.event_type(), "pitch_confidence_changed");
    }
}