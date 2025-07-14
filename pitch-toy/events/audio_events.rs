//! Audio Event Types
//!
//! This module defines audio-specific events that can be published and subscribed to
//! by various components of the application. These events enable loose coupling between
//! the audio subsystem and other components like the console.

use crate::audio::{MusicalNote, VolumeLevel};
use event_dispatcher::{Event, SharedEventDispatcher, create_shared_dispatcher};

/// Audio-related events that can be published throughout the application
#[derive(Debug, Clone)]
pub enum AudioEvent {
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

impl Event for AudioEvent {
    fn event_type(&self) -> &'static str {
        self.event_type()
    }
    
    fn description(&self) -> String {
        self.description()
    }
}

/// Convenience type alias for audio event dispatcher
pub type AudioEventDispatcher = SharedEventDispatcher<AudioEvent>;

/// Creates a shared audio event dispatcher for audio subsystem communication.
/// This creates an audio-specific dispatcher that should be distributed to all 
/// components that need to publish or subscribe to audio events.
pub fn create_shared_audio_dispatcher() -> AudioEventDispatcher {
    create_shared_dispatcher::<AudioEvent>()
}

#[cfg(test)]
mod tests {
    use super::*;
     use wasm_bindgen_test::wasm_bindgen_test;
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_types() {
        
        
        let buffer_event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        assert_eq!(buffer_event.event_type(), "buffer_filled");
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_descriptions() {
        
        
        let buffer_event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        assert!(buffer_event.description().contains("Buffer 0 filled"));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
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

    #[allow(dead_code)]
    #[wasm_bindgen_test]
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

    #[allow(dead_code)]
    #[wasm_bindgen_test]
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
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_event_dispatcher_integration() {
        use event_dispatcher::EventDispatcher;
        use std::rc::Rc;
        use std::cell::RefCell;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
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
        dispatcher.publish(&pitch_event);
        
        // Verify event was received
        let received = events_received.borrow();
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0], AudioEvent::PitchDetected { .. }));
        
        // Verify subscriber count
        assert_eq!(dispatcher.subscriber_count("pitch_detected"), 1);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]  
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

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_shared_audio_dispatcher() {
        let shared_dispatcher = create_shared_audio_dispatcher();
        
        // Subscribe through shared dispatcher
        shared_dispatcher.borrow_mut().subscribe("buffer_filled", |_| {});
        
        assert_eq!(shared_dispatcher.borrow().subscriber_count("buffer_filled"), 1);
        
        // Publish through shared dispatcher
        let event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        shared_dispatcher.borrow().publish(&event);
    }

    // Generic EventDispatcher tests with AudioEvent
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_dispatcher_creation() {
        use event_dispatcher::EventDispatcher;
        
        let dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        assert_eq!(dispatcher.subscriber_count("test_event"), 0);
        assert!(dispatcher.subscribed_event_types().is_empty());
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_subscription() {
        use event_dispatcher::EventDispatcher;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        
        // Subscribe to buffer filled events
        dispatcher.subscribe("buffer_filled", |event| {
            match event {
                AudioEvent::BufferFilled { .. } => {
                    // Test callback received the right event
                }
                _ => panic!("Wrong event type received"),
            }
        });
        
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 1);
        assert!(dispatcher.subscribed_event_types().contains(&"buffer_filled"));
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_publishing() {
        use event_dispatcher::EventDispatcher;
                use std::rc::Rc;
        use std::cell::RefCell;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        let received_events = Rc::new(RefCell::new(Vec::new()));
        
        // Subscribe to buffer filled events
        let received_events_clone = received_events.clone();
        dispatcher.subscribe("buffer_filled", move |event| {
            received_events_clone.borrow_mut().push(event);
        });
        
        // Publish an event
        let event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        dispatcher.publish(&event);
        
        // Verify the event was received
        assert_eq!(received_events.borrow().len(), 1);
        let events = received_events.borrow();
        match &events[0] {
            AudioEvent::BufferFilled { .. } => {
                // Test passed
            }
            _ => panic!("Wrong event type received"),
        }
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_multiple_subscribers() {
        use event_dispatcher::EventDispatcher;
                use std::rc::Rc;
        use std::cell::RefCell;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        let call_count = Rc::new(RefCell::new(0));
        
        // Subscribe multiple callbacks to the same event
        let call_count_clone1 = call_count.clone();
        dispatcher.subscribe("buffer_filled", move |_| {
            *call_count_clone1.borrow_mut() += 1;
        });
        
        let call_count_clone2 = call_count.clone();
        dispatcher.subscribe("buffer_filled", move |_| {
            *call_count_clone2.borrow_mut() += 1;
        });
        
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 2);
        
        // Publish an event
        let event = AudioEvent::BufferFilled { buffer_index: 0, length: 1024 };
        dispatcher.publish(&event);
        
        // Both callbacks should have been called
        assert_eq!(*call_count.borrow(), 2);
    }
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_event_clear_subscribers() {
        use event_dispatcher::EventDispatcher;
        
        let mut dispatcher: EventDispatcher<AudioEvent> = EventDispatcher::new();
        
        dispatcher.subscribe("buffer_filled", |_| {});
        dispatcher.subscribe("audioworklet_status_changed", |_| {});
        
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 1);
        assert_eq!(dispatcher.subscriber_count("audioworklet_status_changed"), 1);
        
        dispatcher.clear_subscribers("buffer_filled");
        assert_eq!(dispatcher.subscriber_count("buffer_filled"), 0);
        assert_eq!(dispatcher.subscriber_count("audioworklet_status_changed"), 1);
        
        dispatcher.clear_all_subscribers();
        assert_eq!(dispatcher.subscriber_count("audioworklet_status_changed"), 0);
    }
}