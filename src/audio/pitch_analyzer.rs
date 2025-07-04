use super::pitch_detector::{PitchDetector, PitchDetectorConfig, PitchResult};
use super::note_mapper::NoteMapper;
use super::buffer_analyzer::{BufferAnalyzer, BufferProcessor};
use super::buffer::CircularBuffer;
use crate::events::SharedEventDispatcher;
use crate::events::audio_events::AudioEvent;

pub type PitchAnalysisError = String;

/// Performance metrics for pitch analysis monitoring
#[derive(Debug, Clone)]
pub struct PitchPerformanceMetrics {
    /// Processing latency in milliseconds
    pub processing_latency_ms: f64,
    /// Total number of analysis cycles completed
    pub analysis_cycles: u64,
    /// Number of successful pitch detections
    pub successful_detections: u64,
    /// Number of failed or no-pitch detections
    pub failed_detections: u64,
    /// Average confidence over recent detections
    pub average_confidence: f32,
    /// Current memory usage for zero-allocation validation
    pub memory_usage_bytes: usize,
}

impl Default for PitchPerformanceMetrics {
    fn default() -> Self {
        Self {
            processing_latency_ms: 0.0,
            analysis_cycles: 0,
            successful_detections: 0,
            failed_detections: 0,
            average_confidence: 0.0,
            memory_usage_bytes: 0,
        }
    }
}


/// Real-time pitch analysis coordinator that integrates with BufferAnalyzer
/// and publishes PitchEvents through the Event Dispatcher
pub struct PitchAnalyzer {
    pitch_detector: PitchDetector,
    note_mapper: NoteMapper,
    event_dispatcher: Option<SharedEventDispatcher>,
    metrics: PitchPerformanceMetrics,
    last_detection: Option<PitchResult>,
    confidence_threshold_for_events: f32,
    // Pre-allocated buffer for zero-allocation processing
    analysis_buffer: Vec<f32>,
}

impl PitchAnalyzer {
    /// Create a new PitchAnalyzer
    pub fn new(
        config: PitchDetectorConfig,
        sample_rate: f32,
    ) -> Result<Self, PitchAnalysisError> {
        let pitch_detector = PitchDetector::new(config.clone(), sample_rate)
            .map_err(|e| format!("Failed to create pitch detector: {}", e))?;
        
        let note_mapper = NoteMapper::new(config.tuning_system.clone());
        
        // Pre-allocate buffer for zero-allocation processing
        let analysis_buffer = vec![0.0; config.sample_window_size];
        
        Ok(Self {
            pitch_detector,
            note_mapper,
            event_dispatcher: None,
            metrics: PitchPerformanceMetrics::default(),
            last_detection: None,
            confidence_threshold_for_events: 0.1, // Threshold for confidence change events
            analysis_buffer,
        })
    }

    /// Set the event dispatcher for publishing pitch events
    pub fn set_event_dispatcher(&mut self, dispatcher: SharedEventDispatcher) {
        self.event_dispatcher = Some(dispatcher);
    }

    /// Set the confidence threshold for confidence change events
    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold_for_events = threshold.clamp(0.0, 1.0);
    }

    /// Analyze audio samples and publish pitch events
    /// 
    /// This is the main processing function that should be called with new audio data.
    /// It performs pitch detection and publishes appropriate events.
    pub fn analyze_samples(&mut self, samples: &[f32]) -> Result<Option<PitchResult>, PitchAnalysisError> {
        let start_time = self.get_high_resolution_time();
        
        // Validate input size
        if samples.len() != self.analysis_buffer.len() {
            return Err(format!("Expected {} samples, got {}", self.analysis_buffer.len(), samples.len()));
        }

        // Copy samples to pre-allocated buffer (minimal allocation)
        self.analysis_buffer.copy_from_slice(samples);
        
        // Perform pitch detection
        let pitch_result = match self.pitch_detector.analyze(&self.analysis_buffer) {
            Ok(result) => result,
            Err(e) => {
                self.metrics.failed_detections += 1;
                return Err(format!("Pitch detection failed: {}", e));
            }
        };

        // Update metrics
        self.metrics.analysis_cycles += 1;
        let end_time = self.get_high_resolution_time();
        self.metrics.processing_latency_ms = end_time - start_time;

        // Process the result and publish events
        match pitch_result {
            Some(result) => {
                self.handle_pitch_detected(result.clone())?;
                self.metrics.successful_detections += 1;
                self.last_detection = Some(result.clone());
                Ok(Some(result))
            }
            None => {
                self.handle_pitch_lost()?;
                self.metrics.failed_detections += 1;
                Ok(None)
            }
        }
    }

    /// Process BufferAnalyzer events for event-driven pitch detection
    pub fn process_buffer_event(&mut self, event: &AudioEvent) -> Result<(), PitchAnalysisError> {
        match event {
            AudioEvent::BufferFilled { .. } => {
                // Buffer is filled and ready for analysis
                // In a real implementation, this would retrieve the buffer data
                // For now, we'll just update metrics to show we're listening
                self.publish_metrics_update();
                Ok(())
            }
            _ => Ok(()), // Ignore other audio events
        }
    }

    /// Get current performance metrics
    pub fn metrics(&self) -> &PitchPerformanceMetrics {
        &self.metrics
    }

    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = PitchPerformanceMetrics::default();
    }

    /// Update pitch detector configuration
    pub fn update_config(&mut self, config: PitchDetectorConfig) -> Result<(), PitchAnalysisError> {
        self.pitch_detector.update_config(config.clone())
            .map_err(|e| format!("Failed to update pitch detector config: {}", e))?;
        
        // Update note mapper tuning system
        self.note_mapper.set_tuning_system(config.tuning_system);
        
        // Resize analysis buffer if needed
        if config.sample_window_size != self.analysis_buffer.len() {
            self.analysis_buffer.resize(config.sample_window_size, 0.0);
        }

        Ok(())
    }

    /// Get current pitch detector configuration
    pub fn config(&self) -> &PitchDetectorConfig {
        self.pitch_detector.config()
    }

    /// Check if the analyzer is ready for processing
    pub fn is_ready(&self) -> bool {
        !self.analysis_buffer.is_empty()
    }

    /// Analyze audio data from a BufferAnalyzer using zero-allocation processing
    /// 
    /// This method integrates with the existing BufferAnalyzer for sequential processing.
    /// It uses the pre-allocated buffer to avoid heap allocations during steady-state.
    pub fn analyze_from_buffer_analyzer(
        &mut self, 
        buffer_analyzer: &mut BufferAnalyzer
    ) -> Result<Option<PitchResult>, PitchAnalysisError> {
        // Check if buffer has enough data for analysis
        if !buffer_analyzer.can_process() {
            return Ok(None);
        }

        // Validate that the buffer analyzer block size matches our window size
        if buffer_analyzer.block_size() != self.analysis_buffer.len() {
            return Err(format!(
                "BufferAnalyzer block size ({}) does not match pitch window size ({})",
                buffer_analyzer.block_size(),
                self.analysis_buffer.len()
            ));
        }

        let start_time = self.get_high_resolution_time();

        // Use zero-allocation processing
        let success = buffer_analyzer.process_next_into(&mut self.analysis_buffer);
        if !success {
            return Ok(None); // Not enough data available
        }

        // Perform pitch detection on the filled buffer
        let pitch_result = match self.pitch_detector.analyze(&self.analysis_buffer) {
            Ok(result) => result,
            Err(e) => {
                self.metrics.failed_detections += 1;
                return Err(format!("Pitch detection failed: {}", e));
            }
        };

        // Update metrics
        self.metrics.analysis_cycles += 1;
        let end_time = self.get_high_resolution_time();
        self.metrics.processing_latency_ms = end_time - start_time;

        // Process the result and publish events
        match pitch_result {
            Some(result) => {
                self.handle_pitch_detected(result.clone())?;
                self.metrics.successful_detections += 1;
                self.last_detection = Some(result.clone());
                Ok(Some(result))
            }
            None => {
                self.handle_pitch_lost()?;
                self.metrics.failed_detections += 1;
                Ok(None)
            }
        }
    }

    /// Process multiple blocks from a BufferAnalyzer in a continuous loop
    /// 
    /// This method processes all available blocks from the buffer analyzer,
    /// ensuring real-time processing without blocking.
    pub fn process_continuous_from_buffer(
        &mut self,
        buffer_analyzer: &mut BufferAnalyzer
    ) -> Result<Vec<PitchResult>, PitchAnalysisError> {
        let mut results = Vec::new();
        
        // Process all available blocks
        while buffer_analyzer.can_process() {
            match self.analyze_from_buffer_analyzer(buffer_analyzer)? {
                Some(result) => results.push(result),
                None => break, // No more data available
            }
        }

        // Publish metrics update if we processed any blocks
        if !results.is_empty() {
            self.publish_metrics_update();
        }

        Ok(results)
    }

    /// Create a complete pitch analysis pipeline with CircularBuffer and BufferAnalyzer
    /// 
    /// This is a convenience method that creates a BufferAnalyzer for the given CircularBuffer
    /// and performs pitch analysis. This demonstrates the full integration pipeline.
    pub fn analyze_from_circular_buffer(
        &mut self,
        buffer: &mut CircularBuffer<f32>,
        window_function: super::buffer_analyzer::WindowFunction,
    ) -> Result<Vec<PitchResult>, PitchAnalysisError> {
        // Create a BufferAnalyzer for sequential processing
        let mut buffer_analyzer = BufferAnalyzer::new(
            buffer, 
            self.analysis_buffer.len(), 
            window_function
        ).map_err(|e| format!("Failed to create BufferAnalyzer: {}", e))?;

        // Process all available blocks
        self.process_continuous_from_buffer(&mut buffer_analyzer)
    }

    // Private helper methods

    fn handle_pitch_detected(&mut self, result: PitchResult) -> Result<(), PitchAnalysisError> {
        // Convert frequency to musical note
        let note = self.note_mapper.frequency_to_note(result.frequency);

        // Check if this is a significant confidence change
        let confidence_changed = if let Some(ref last) = self.last_detection {
            (result.confidence - last.confidence).abs() > self.confidence_threshold_for_events
        } else {
            true // First detection is always a change
        };

        // Publish pitch detected event
        let pitch_event = AudioEvent::PitchDetected {
            frequency: result.frequency,
            confidence: result.confidence,
            note,
            clarity: result.clarity,
            timestamp: result.timestamp,
        };
        self.publish_event(pitch_event);

        // Publish confidence change event if significant
        if confidence_changed {
            let confidence_event = AudioEvent::ConfidenceChanged {
                frequency: result.frequency,
                confidence: result.confidence,
                timestamp: result.timestamp,
            };
            self.publish_event(confidence_event);
        }

        // Update average confidence (simple moving average over last few samples)
        self.update_average_confidence(result.confidence);

        Ok(())
    }

    fn handle_pitch_lost(&mut self) -> Result<(), PitchAnalysisError> {
        if let Some(ref last) = self.last_detection {
            let pitch_lost_event = AudioEvent::PitchLost {
                last_frequency: last.frequency,
                timestamp: self.get_high_resolution_time(),
            };
            self.publish_event(pitch_lost_event);
        }
        Ok(())
    }

    fn publish_event(&self, event: AudioEvent) {
        if let Some(ref dispatcher) = self.event_dispatcher {
            // Publish the event through the Event Dispatcher
            dispatcher.borrow().publish(event);
        } else {
            // Fallback: log the event if no dispatcher is available
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(&format!("PitchEvent: {}", event.description()).into());
            }
            
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!("PitchEvent: {}", event.description());
            }
        }
    }

    fn publish_metrics_update(&mut self) {
        // For now, we'll log metrics updates rather than create a new event type
        // In the future, this could be extended to include performance metrics in AudioEvent
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&format!(
                "Pitch Metrics: latency={:.1}ms, cycles={}, success={}", 
                self.metrics.processing_latency_ms, 
                self.metrics.analysis_cycles,
                self.metrics.successful_detections
            ).into());
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!(
                "Pitch Metrics: latency={:.1}ms, cycles={}, success={}", 
                self.metrics.processing_latency_ms, 
                self.metrics.analysis_cycles,
                self.metrics.successful_detections
            );
        }
    }

    fn update_average_confidence(&mut self, new_confidence: f32) {
        // Simple exponential moving average
        let alpha = 0.1; // Smoothing factor
        if self.metrics.average_confidence == 0.0 {
            self.metrics.average_confidence = new_confidence;
        } else {
            self.metrics.average_confidence = alpha * new_confidence + (1.0 - alpha) * self.metrics.average_confidence;
        }
    }

    fn get_high_resolution_time(&self) -> f64 {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(performance) = window.performance() {
                    return performance.now();
                }
            }
            0.0
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::pitch_detector::{TuningSystem, PitchDetectorConfig};

    fn create_test_config() -> PitchDetectorConfig {
        PitchDetectorConfig {
            sample_window_size: 1024,
            threshold: 0.15,
            tuning_system: TuningSystem::EqualTemperament { reference_pitch: 440.0 },
            min_frequency: 80.0,
            max_frequency: 2000.0,
        }
    }

    #[test]
    fn test_pitch_analyzer_creation() {
        let config = create_test_config();
        let analyzer = PitchAnalyzer::new(config, 48000.0);
        assert!(analyzer.is_ok());

        let analyzer = analyzer.unwrap();
        assert!(analyzer.is_ready());
        assert_eq!(analyzer.config().sample_window_size, 1024);
    }

    #[test]
    fn test_pitch_analyzer_config_update() {
        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        let mut new_config = create_test_config();
        new_config.sample_window_size = 2048;
        new_config.threshold = 0.2;

        let result = analyzer.update_config(new_config);
        assert!(result.is_ok());
        assert_eq!(analyzer.config().sample_window_size, 2048);
        assert_eq!(analyzer.config().threshold, 0.2);
    }

    #[test]
    fn test_pitch_analyzer_invalid_sample_size() {
        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Try to analyze with wrong sample size
        let samples = vec![0.0; 512]; // Wrong size, expected 1024
        let result = analyzer.analyze_samples(&samples);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 1024 samples"));
    }

    #[test]
    fn test_pitch_analyzer_silence() {
        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        let samples = vec![0.0; 1024]; // Silence
        let result = analyzer.analyze_samples(&samples);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No pitch detected in silence

        // Check metrics were updated
        assert_eq!(analyzer.metrics().analysis_cycles, 1);
        assert_eq!(analyzer.metrics().failed_detections, 1);
    }

    #[test]
    fn test_pitch_analyzer_sine_wave() {
        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Generate 440Hz sine wave
        let frequency = 440.0;
        let sample_rate = 48000.0;
        let samples: Vec<f32> = (0..1024)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();

        let result = analyzer.analyze_samples(&samples);
        assert!(result.is_ok());

        if let Some(pitch_result) = result.unwrap() {
            // Should detect close to 440Hz
            assert!((pitch_result.frequency - 440.0).abs() < 50.0);
            assert!(pitch_result.confidence > 0.5);
            
            // Check metrics were updated
            assert_eq!(analyzer.metrics().analysis_cycles, 1);
            assert_eq!(analyzer.metrics().successful_detections, 1);
            assert!(analyzer.metrics().processing_latency_ms >= 0.0);
        }
    }

    #[test]
    fn test_pitch_analyzer_confidence_threshold() {
        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Test setting confidence threshold
        analyzer.set_confidence_threshold(0.2);
        assert_eq!(analyzer.confidence_threshold_for_events, 0.2);

        // Test clamping
        analyzer.set_confidence_threshold(-0.5);
        assert_eq!(analyzer.confidence_threshold_for_events, 0.0);

        analyzer.set_confidence_threshold(1.5);
        assert_eq!(analyzer.confidence_threshold_for_events, 1.0);
    }

    #[test]
    fn test_pitch_analyzer_metrics_reset() {
        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Generate some metrics
        let samples = vec![0.0; 1024];
        let _ = analyzer.analyze_samples(&samples);
        assert!(analyzer.metrics().analysis_cycles > 0);

        // Reset metrics
        analyzer.reset_metrics();
        assert_eq!(analyzer.metrics().analysis_cycles, 0);
        assert_eq!(analyzer.metrics().successful_detections, 0);
        assert_eq!(analyzer.metrics().failed_detections, 0);
    }

    #[test]
    fn test_pitch_event_integration() {
        // Test that pitch events are properly integrated with AudioEvent
        // This is tested more thoroughly in the events module
        use crate::events::audio_events::AudioEvent;
        use crate::audio::pitch_detector::{NoteName, MusicalNote};

        let note = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        let detected_event = AudioEvent::PitchDetected {
            frequency: 440.0,
            confidence: 0.9,
            note,
            clarity: 0.8,
            timestamp: 1000.0,
        };
        assert_eq!(detected_event.event_type(), "pitch_detected");
        assert!(detected_event.description().contains("440.00Hz"));

        let lost_event = AudioEvent::PitchLost {
            last_frequency: 440.0,
            timestamp: 1000.0,
        };
        assert_eq!(lost_event.event_type(), "pitch_lost");
        assert!(lost_event.description().contains("440.00Hz"));

        let confidence_event = AudioEvent::ConfidenceChanged {
            frequency: 440.0,
            confidence: 0.8,
            timestamp: 1000.0,
        };
        assert_eq!(confidence_event.event_type(), "pitch_confidence_changed");
        assert!(confidence_event.description().contains("confidence=0.80"));
    }

    #[test]
    fn test_pitch_analyzer_buffer_event_processing() {
        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Test processing buffer filled event
        let buffer_event = AudioEvent::BufferFilled {
            buffer_index: 0,
            length: 1024,
        };

        let result = analyzer.process_buffer_event(&buffer_event);
        assert!(result.is_ok());

        // Test processing other events (should be ignored)
        let permission_event = AudioEvent::PermissionChanged(
            crate::audio::AudioPermission::Granted
        );
        let result = analyzer.process_buffer_event(&permission_event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PitchPerformanceMetrics::default();
        assert_eq!(metrics.processing_latency_ms, 0.0);
        assert_eq!(metrics.analysis_cycles, 0);
        assert_eq!(metrics.successful_detections, 0);
        assert_eq!(metrics.failed_detections, 0);
        assert_eq!(metrics.average_confidence, 0.0);
        assert_eq!(metrics.memory_usage_bytes, 0);
    }

    #[test]
    fn test_pitch_analyzer_multiple_detections() {
        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Multiple analysis cycles
        for i in 0..5 {
            let frequency = 440.0 + (i as f32 * 10.0); // Varying frequency
            let samples: Vec<f32> = (0..1024)
                .map(|j| {
                    let t = j as f32 / 48000.0;
                    (2.0 * std::f32::consts::PI * frequency * t).sin()
                })
                .collect();

            let _ = analyzer.analyze_samples(&samples);
        }

        // Check metrics accumulated
        assert_eq!(analyzer.metrics().analysis_cycles, 5);
        assert!(analyzer.metrics().average_confidence > 0.0);
    }

    #[test]
    fn test_pitch_analyzer_buffer_analyzer_integration() {
        use crate::audio::buffer::{CircularBuffer, PRODUCTION_BUFFER_SIZE};
        use crate::audio::buffer_analyzer::{BufferAnalyzer, WindowFunction};

        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Create a circular buffer and fill it with test data
        let mut buffer = CircularBuffer::new(PRODUCTION_BUFFER_SIZE).unwrap();
        
        // Generate 440Hz sine wave
        let frequency = 440.0;
        let sample_rate = 48000.0;
        let samples: Vec<f32> = (0..2048) // More than one block
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();

        // Write samples to buffer
        buffer.write_chunk(&samples);

        // Create BufferAnalyzer
        let mut buffer_analyzer = BufferAnalyzer::new(&mut buffer, 1024, WindowFunction::None).unwrap();

        // Analyze from buffer analyzer
        let result = analyzer.analyze_from_buffer_analyzer(&mut buffer_analyzer);
        assert!(result.is_ok());

        if let Some(pitch_result) = result.unwrap() {
            // Should detect close to 440Hz
            assert!((pitch_result.frequency - 440.0).abs() < 50.0);
            assert!(pitch_result.confidence > 0.5);
        }

        // Check metrics were updated
        assert_eq!(analyzer.metrics().analysis_cycles, 1);
    }

    #[test]
    fn test_pitch_analyzer_buffer_analyzer_insufficient_data() {
        use crate::audio::buffer::{CircularBuffer, PRODUCTION_BUFFER_SIZE};
        use crate::audio::buffer_analyzer::{BufferAnalyzer, WindowFunction};

        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Create a circular buffer with insufficient data
        let mut buffer = CircularBuffer::new(PRODUCTION_BUFFER_SIZE).unwrap();
        let samples = vec![0.0; 512]; // Less than required 1024
        buffer.write_chunk(&samples);

        // Create BufferAnalyzer
        let mut buffer_analyzer = BufferAnalyzer::new(&mut buffer, 1024, WindowFunction::None).unwrap();

        // Should return None due to insufficient data
        let result = analyzer.analyze_from_buffer_analyzer(&mut buffer_analyzer);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_pitch_analyzer_buffer_analyzer_size_mismatch() {
        use crate::audio::buffer::{CircularBuffer, PRODUCTION_BUFFER_SIZE};
        use crate::audio::buffer_analyzer::{BufferAnalyzer, WindowFunction};

        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Create a circular buffer
        let mut buffer = CircularBuffer::new(PRODUCTION_BUFFER_SIZE).unwrap();
        let samples = vec![0.0; 2048];
        buffer.write_chunk(&samples);

        // Create BufferAnalyzer with different block size
        let mut buffer_analyzer = BufferAnalyzer::new(&mut buffer, 512, WindowFunction::None).unwrap();

        // Should return error due to size mismatch
        let result = analyzer.analyze_from_buffer_analyzer(&mut buffer_analyzer);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not match pitch window size"));
    }

    #[test]
    fn test_pitch_analyzer_continuous_processing() {
        use crate::audio::buffer::{CircularBuffer, PRODUCTION_BUFFER_SIZE};
        use crate::audio::buffer_analyzer::{BufferAnalyzer, WindowFunction};

        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Create a circular buffer with multiple blocks of data
        let mut buffer = CircularBuffer::new(PRODUCTION_BUFFER_SIZE).unwrap();
        
        // Generate enough data for 3 blocks (need extra to ensure buffer has enough)
        let frequency = 440.0;
        let sample_rate = 48000.0;
        let samples: Vec<f32> = (0..4096) // 4 blocks worth to ensure enough data
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();

        buffer.write_chunk(&samples);

        // Create BufferAnalyzer
        let mut buffer_analyzer = BufferAnalyzer::new(&mut buffer, 1024, WindowFunction::None).unwrap();

        // Process all available blocks
        let results = analyzer.process_continuous_from_buffer(&mut buffer_analyzer);
        assert!(results.is_ok());
        
        let pitch_results = results.unwrap();
        let num_results = pitch_results.len();
        assert!(num_results >= 1); // Should process at least 1 block
        assert!(num_results <= 4); // But not more than 4

        // Check that all results are valid
        for result in &pitch_results {
            assert!((result.frequency - 440.0).abs() < 50.0);
            assert!(result.confidence > 0.5);
        }

        // Check metrics (should match number of results)
        assert_eq!(analyzer.metrics().analysis_cycles, num_results as u64);
        assert_eq!(analyzer.metrics().successful_detections, num_results as u64);
    }

    #[test]
    fn test_pitch_analyzer_circular_buffer_integration() {
        use crate::audio::buffer::{CircularBuffer, PRODUCTION_BUFFER_SIZE};
        use crate::audio::buffer_analyzer::WindowFunction;

        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Create a circular buffer with test data
        let mut buffer = CircularBuffer::new(PRODUCTION_BUFFER_SIZE).unwrap();
        
        // Generate 440Hz sine wave for 2 blocks (with extra to ensure enough data)
        let frequency = 440.0;
        let sample_rate = 48000.0;
        let samples: Vec<f32> = (0..3072) // 3 blocks worth to ensure enough data
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();

        buffer.write_chunk(&samples);

        // Use the convenience method for full pipeline
        let results = analyzer.analyze_from_circular_buffer(&mut buffer, WindowFunction::Hamming);
        assert!(results.is_ok());
        
        let pitch_results = results.unwrap();
        assert!(pitch_results.len() >= 1); // Should process at least 1 block
        assert!(pitch_results.len() <= 3); // But not more than 3

        // Check that results are valid
        for result in &pitch_results {
            assert!((result.frequency - 440.0).abs() < 50.0);
            assert!(result.confidence > 0.5);
        }
    }

    #[test]
    fn test_pitch_analyzer_windowing_functions() {
        use crate::audio::buffer::{CircularBuffer, PRODUCTION_BUFFER_SIZE};
        use crate::audio::buffer_analyzer::WindowFunction;

        let config = create_test_config();
        let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();

        // Test different windowing functions
        let window_functions = [WindowFunction::None, WindowFunction::Hamming, WindowFunction::Blackman];

        for window_fn in window_functions.iter() {
            // Create fresh buffer for each test
            let mut buffer = CircularBuffer::new(PRODUCTION_BUFFER_SIZE).unwrap();
            
            // Generate test signal
            let frequency = 440.0;
            let sample_rate = 48000.0;
            let samples: Vec<f32> = (0..1024)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    (2.0 * std::f32::consts::PI * frequency * t).sin()
                })
                .collect();

            buffer.write_chunk(&samples);

            // Test with different window function
            let results = analyzer.analyze_from_circular_buffer(&mut buffer, *window_fn);
            assert!(results.is_ok());
            
            let pitch_results = results.unwrap();
            assert!(!pitch_results.is_empty());

            // Reset analyzer for next test
            analyzer.reset_metrics();
        }
    }
}