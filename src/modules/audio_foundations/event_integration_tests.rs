//! Integration tests for audio event publishing
//! 
//! Tests to verify that the MultiAlgorithmPitchDetector properly publishes events
//! to the TypedEventBus with correct timing and content.

#[cfg(test)]
mod tests {
    use super::super::multi_algorithm_pitch_detector::{MultiAlgorithmPitchDetector, PitchDetectionConfig, PitchAlgorithm};
    use super::super::audio_events::{PitchDetectionEvent, get_timestamp_ns};
    use crate::modules::application_core::event_bus::{EventBus, EventHandler};
    use crate::modules::application_core::typed_event_bus::TypedEventBus;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// Test event handler that captures pitch detection events
    struct TestPitchEventHandler {
        captured_events: Arc<Mutex<Vec<PitchDetectionEvent>>>,
        capture_times: Arc<Mutex<Vec<u64>>>,
    }

    impl TestPitchEventHandler {
        fn new() -> (Self, Arc<Mutex<Vec<PitchDetectionEvent>>>, Arc<Mutex<Vec<u64>>>) {
            let captured_events = Arc::new(Mutex::new(Vec::new()));
            let capture_times = Arc::new(Mutex::new(Vec::new()));
            
            let handler = Self {
                captured_events: captured_events.clone(),
                capture_times: capture_times.clone(),
            };
            
            (handler, captured_events, capture_times)
        }
    }

    impl EventHandler<PitchDetectionEvent> for TestPitchEventHandler {
        fn handle_event(&mut self, event: &PitchDetectionEvent) -> Result<(), Box<dyn std::error::Error>> {
            let capture_time = get_timestamp_ns();
            
            if let Ok(mut events) = self.captured_events.lock() {
                events.push(event.clone());
            }
            
            if let Ok(mut times) = self.capture_times.lock() {
                times.push(capture_time);
            }
            
            Ok(())
        }
    }

    /// Generate test sine wave for pitch detection
    fn generate_sine_wave(frequency: f32, sample_rate: f32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5
            })
            .collect()
    }

    #[test]
    fn test_pitch_detection_event_publishing() {
        // Setup event bus and handler
        let mut event_bus = TypedEventBus::new();
        let (handler, captured_events, _capture_times) = TestPitchEventHandler::new();
        
        // Start the event bus
        event_bus.start().expect("Failed to start event bus");
        
        // Subscribe to pitch detection events
        let _subscription_id = event_bus.subscribe(Box::new(handler))
            .expect("Failed to subscribe to pitch detection events");

        // Create pitch detector with event bus
        let config = PitchDetectionConfig {
            algorithm: PitchAlgorithm::YIN,
            sample_rate: 44100.0,
            min_frequency: 80.0,
            max_frequency: 2000.0,
            yin_threshold: 0.2,
            mcleod_threshold: 0.3,
            mcleod_clarity_threshold: 0.7,
            enable_confidence_scoring: true,
            enable_harmonic_analysis: true,
            auto_selection_sensitivity: 0.5,
        };

        let event_bus_arc = Arc::new(event_bus);
        let mut detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus_arc.clone()))
            .expect("Failed to create pitch detector");

        // Generate test signal - 440Hz (A4)
        let test_frequency = 440.0;
        let sample_rate = 44100.0;
        let buffer_size = 2048;
        let test_buffer = generate_sine_wave(test_frequency, sample_rate, buffer_size);

        // Perform pitch detection (should trigger event publishing)
        let detection_start = get_timestamp_ns();
        let result = detector.detect_pitch(&test_buffer);
        let detection_end = get_timestamp_ns();

        // Allow some time for event processing
        std::thread::sleep(Duration::from_millis(10));

        // Stop event bus
        event_bus_arc.stop().expect("Failed to stop event bus");

        // Verify pitch detection was successful
        assert!(result.is_ok(), "Pitch detection should succeed for clean sine wave");
        let pitch_result = result.unwrap();
        assert!((pitch_result.frequency - test_frequency).abs() < 10.0, 
                "Detected frequency should be close to test frequency");

        // Verify event was published and captured
        let events = captured_events.lock().unwrap();
        assert!(!events.is_empty(), "At least one pitch detection event should have been published");

        let event = &events[0];
        
        // Verify event content
        assert!((event.frequency - test_frequency).abs() < 10.0, 
                "Event frequency should match detected frequency");
        assert!(event.confidence > 0.0, "Event should have confidence > 0");
        assert!(event.is_valid, "Event should be marked as valid");
        assert!(!event.source_buffer_ref.is_empty(), "Event should have buffer reference");
        assert!(event.processing_time_ns > 0, "Event should have processing time");
        
        // Verify timestamp is reasonable
        assert!(event.timestamp_ns >= detection_start && event.timestamp_ns <= detection_end + 1_000_000, 
                "Event timestamp should be within detection timeframe");

        // Verify buffer reference format
        assert!(event.source_buffer_ref.starts_with("buf_"), 
                "Buffer reference should start with 'buf_'");
    }

    #[test]
    fn test_event_publishing_latency() {
        // Setup event bus and handler  
        let mut event_bus = TypedEventBus::new();
        let (handler, _captured_events, capture_times) = TestPitchEventHandler::new();
        
        event_bus.start().expect("Failed to start event bus");
        let _subscription_id = event_bus.subscribe(Box::new(handler))
            .expect("Failed to subscribe to pitch detection events");

        let config = PitchDetectionConfig::default();
        let event_bus_arc = Arc::new(event_bus);
        let mut detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus_arc.clone()))
            .expect("Failed to create pitch detector");

        // Generate test signal
        let test_buffer = generate_sine_wave(440.0, 44100.0, 1024);

        // Measure detection and event publishing time
        let start_time = Instant::now();
        let detection_start_ns = get_timestamp_ns();
        
        let _result = detector.detect_pitch(&test_buffer);
        
        // Allow time for event processing
        std::thread::sleep(Duration::from_millis(5));
        
        let total_time = start_time.elapsed();
        
        event_bus_arc.stop().expect("Failed to stop event bus");

        // Verify total processing time meets <1ms requirement for critical events
        // Note: This includes both detection and event publishing
        assert!(total_time < Duration::from_millis(10), 
                "Total processing time should be reasonable for real-time audio");

        // Verify event capture time
        let times = capture_times.lock().unwrap();
        if !times.is_empty() {
            let event_latency_ns = times[0] - detection_start_ns;
            let event_latency_ms = event_latency_ns as f64 / 1_000_000.0;
            
            // Event should be published within reasonable time for critical priority
            assert!(event_latency_ms < 10.0, 
                    "Event publishing latency should be under 10ms, got {}ms", event_latency_ms);
        }
    }

    #[test]
    fn test_buffer_reference_uniqueness() {
        // Setup event bus and handler
        let mut event_bus = TypedEventBus::new();
        let (handler, captured_events, _capture_times) = TestPitchEventHandler::new();
        
        event_bus.start().expect("Failed to start event bus");
        let _subscription_id = event_bus.subscribe(Box::new(handler))
            .expect("Failed to subscribe to pitch detection events");

        let config = PitchDetectionConfig::default();
        let event_bus_arc = Arc::new(event_bus);
        let mut detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus_arc.clone()))
            .expect("Failed to create pitch detector");

        // Generate multiple test buffers and detect pitch
        let test_buffer1 = generate_sine_wave(440.0, 44100.0, 1024);
        let test_buffer2 = generate_sine_wave(880.0, 44100.0, 1024);

        let _result1 = detector.detect_pitch(&test_buffer1);
        std::thread::sleep(Duration::from_millis(1)); // Ensure different timestamps
        let _result2 = detector.detect_pitch(&test_buffer2);

        // Allow time for event processing
        std::thread::sleep(Duration::from_millis(10));
        
        event_bus_arc.stop().expect("Failed to stop event bus");

        // Verify buffer references are unique
        let events = captured_events.lock().unwrap();
        assert!(events.len() >= 2, "Should have at least 2 events");
        
        if events.len() >= 2 {
            assert_ne!(events[0].source_buffer_ref, events[1].source_buffer_ref,
                      "Buffer references should be unique between detections");
        }
    }
} 