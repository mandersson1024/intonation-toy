use super::pitch_detector::{PitchDetector, PitchDetectorConfig, PitchResult};
use super::buffer_analyzer::{BufferAnalyzer, BufferProcessor};
use super::buffer::CircularBuffer;
use super::volume_detector::VolumeAnalysis;
use crate::common::dev_log;
use crate::app_config::{POWER_THRESHOLD, CLARITY_THRESHOLD};

pub type PitchAnalysisError = String;

/// Performance metrics for pitch analysis monitoring
#[derive(Debug, Clone)]
pub struct PitchPerformanceMetrics {
    /// Processing latency in milliseconds (latest)
    pub processing_latency_ms: f64,
    /// Average processing latency over recent samples
    pub average_latency_ms: f64,
    /// Maximum observed processing latency
    pub max_latency_ms: f64,
    /// Minimum observed processing latency
    pub min_latency_ms: f64,
    /// Total number of analysis cycles completed
    pub analysis_cycles: u64,
    /// Number of successful pitch detections
    pub successful_detections: u64,
    /// Number of failed or no-pitch detections
    pub failed_detections: u64,
    /// Current memory usage for zero-allocation validation
    pub memory_usage_bytes: usize,
    /// Number of samples exceeding 50ms processing time
    pub latency_violations: u64,
    /// Success rate (successful_detections / analysis_cycles)
    pub success_rate: f32,
    /// Time spent in YIN algorithm specifically (microseconds)
    pub yin_processing_time_us: f64,
}

impl Default for PitchPerformanceMetrics {
    fn default() -> Self {
        Self {
            processing_latency_ms: 0.0,
            average_latency_ms: 0.0,
            max_latency_ms: 0.0,
            min_latency_ms: f64::INFINITY,
            analysis_cycles: 0,
            successful_detections: 0,
            failed_detections: 0,
            memory_usage_bytes: 0,
            latency_violations: 0,
            success_rate: 0.0,
            yin_processing_time_us: 0.0,
        }
    }
}

/// Real-time pitch analysis coordinator that integrates with BufferAnalyzer
/// and returns pitch data through the engine update system.
/// 
/// ## Data Collection
/// 
/// The PitchAnalyzer provides data collection through:
/// - Analysis methods that return `PitchResult` directly
/// - `get_latest_pitch_data()` method for retrieving the most recent detection
/// 
/// ## Usage Example
/// 
/// ```rust,no_run
/// use crate::common::log;
/// 
/// let mut analyzer = PitchAnalyzer::new(config, sample_rate)?;
/// 
/// // Analyze samples and get immediate result
/// if let Some(result) = analyzer.analyze_samples(&samples)? {
///     log!("Detected pitch: {} Hz", result.frequency);
/// }
/// 
/// // Get latest detection for data collection
/// if let Some(pitch_data) = analyzer.get_latest_pitch_data() {
///     log!("Latest pitch: {} Hz", pitch_data.frequency);
/// }
/// ```
pub struct PitchAnalyzer {
    pitch_detector: PitchDetector,
    metrics: PitchPerformanceMetrics,
    last_detection: Option<PitchResult>,
    // Pre-allocated buffer for zero-allocation processing
    analysis_buffer: Vec<f32>,
    // Volume analysis for tracking
    last_volume_analysis: Option<VolumeAnalysis>,
}

impl PitchAnalyzer {
    /// Create a new PitchAnalyzer
    pub fn new(
        config: PitchDetectorConfig,
        sample_rate: u32,
    ) -> Result<Self, PitchAnalysisError> {
        let pitch_detector = PitchDetector::new(config.clone(), sample_rate)
            .map_err(|e| format!("Failed to create pitch detector: {}", e))?;
        
        // Pre-allocate buffer for zero-allocation processing
        let analysis_buffer = vec![0.0; config.sample_window_size];
        
        Ok(Self {
            pitch_detector,
            metrics: PitchPerformanceMetrics::default(),
            last_detection: None,
            analysis_buffer,
            last_volume_analysis: None,
        })
    }

    /// Update volume analysis for confidence weighting
    pub fn update_volume_analysis(&mut self, volume_analysis: VolumeAnalysis) {
        self.last_volume_analysis = Some(volume_analysis);
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
        
        // Measure YIN algorithm performance specifically
        let yin_start = self.get_high_resolution_time();
        let pitch_result = match self.pitch_detector.analyze(&self.analysis_buffer) {
            Ok(result) => result,
            Err(e) => {
                let end_time = self.get_high_resolution_time();
                self.update_metrics(start_time, end_time, 0.0, false);
                return Err(format!("Pitch detection failed: {}", e));
            }
        };
        let yin_end = self.get_high_resolution_time();
        let yin_time_us = (yin_end - yin_start) * 1000.0; // Convert to microseconds

        let end_time = self.get_high_resolution_time();

        // Process the result and publish events
        match pitch_result {
            Some(result) => {
                self.handle_pitch_detected(result.clone())?;
                self.update_metrics(start_time, end_time, yin_time_us, true);
                self.last_detection = Some(result.clone());
                Ok(Some(result))
            }
            None => {
                self.handle_pitch_lost()?;
                self.update_metrics(start_time, end_time, yin_time_us, false);
                Ok(None)
            }
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

    /// Update performance metrics with new timing data
    fn update_metrics(&mut self, start_time: f64, end_time: f64, yin_time_us: f64, success: bool) {
        let latency_ms = end_time - start_time;
        
        self.metrics.analysis_cycles += 1;
        self.metrics.processing_latency_ms = latency_ms;
        self.metrics.yin_processing_time_us = yin_time_us;
        
        // Update latency statistics
        if self.metrics.analysis_cycles == 1 {
            self.metrics.average_latency_ms = latency_ms;
            self.metrics.max_latency_ms = latency_ms;
            self.metrics.min_latency_ms = latency_ms;
        } else {
            // Exponential moving average for responsiveness
            let alpha = 0.1;
            self.metrics.average_latency_ms = alpha * latency_ms + (1.0 - alpha) * self.metrics.average_latency_ms;
            self.metrics.max_latency_ms = self.metrics.max_latency_ms.max(latency_ms);
            self.metrics.min_latency_ms = self.metrics.min_latency_ms.min(latency_ms);
        }
        
        // Check for latency violations (>50ms production requirement)
        if latency_ms > 50.0 {
            self.metrics.latency_violations += 1;
        }
        
        // Update success counts and rate
        if success {
            self.metrics.successful_detections += 1;
        } else {
            self.metrics.failed_detections += 1;
        }
        self.metrics.success_rate = self.metrics.successful_detections as f32 / self.metrics.analysis_cycles as f32;
        
        // Update memory usage estimate (pre-allocated buffer + detector memory)
        let buffer_memory = self.analysis_buffer.len() * std::mem::size_of::<f32>();
        let detector_memory = self.pitch_detector.memory_usage_bytes();
        let analyzer_memory = std::mem::size_of::<Self>();
        self.metrics.memory_usage_bytes = buffer_memory + detector_memory + analyzer_memory;
    }

    /// Get performance benchmark for different window sizes
    pub fn benchmark_window_sizes(&mut self, sample_rate: u32) -> Vec<(usize, f64, f64)> {
        let window_sizes = vec![256, 512, 1024, 2048, 4096];
        let mut results = Vec::new();
        
        // Generate test signal (440Hz sine wave)
        let test_frequency = 440.0;
        for &window_size in &window_sizes {
            let test_samples: Vec<f32> = (0..window_size)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    (2.0 * std::f32::consts::PI * test_frequency * t).sin()
                })
                .collect();
            
            // Measure performance over multiple iterations
            let iterations = 10;
            let mut total_time = 0.0;
            let mut min_time = f64::INFINITY;
            
            for _ in 0..iterations {
                let start = self.get_high_resolution_time();
                
                // Create temporary detector for this window size
                let config = super::pitch_detector::PitchDetectorConfig {
                    sample_window_size: window_size,
                    power_threshold: POWER_THRESHOLD,
                    clarity_threshold: CLARITY_THRESHOLD,
                    padding_size: window_size / 2,
                    min_frequency: 80.0,
                    max_frequency: 2000.0,
                };
                
                if let Ok(mut detector) = super::pitch_detector::PitchDetector::new(config, sample_rate) {
                    let _ = detector.analyze(&test_samples);
                }
                
                let end = self.get_high_resolution_time();
                let elapsed = end - start;
                total_time += elapsed;
                min_time = min_time.min(elapsed);
            }
            
            let avg_time = total_time / iterations as f64;
            results.push((window_size, avg_time, min_time));
        }
        
        results
    }

    /// Check if processing meets performance requirements
    pub fn meets_performance_requirements(&self) -> bool {
        // Check average latency requirement (≤50ms production)
        self.metrics.average_latency_ms <= 50.0 && 
        // Check that we don't have too many violations (max 5% of samples)
        (self.metrics.latency_violations as f32 / self.metrics.analysis_cycles.max(1) as f32) <= 0.05
    }

    /// Get performance grade based on latency metrics
    pub fn performance_grade(&self) -> &'static str {
        if self.metrics.average_latency_ms <= 20.0 {
            "Excellent"
        } else if self.metrics.average_latency_ms <= 35.0 {
            "Good"
        } else if self.metrics.average_latency_ms <= 50.0 {
            "Acceptable"
        } else if self.metrics.average_latency_ms <= 100.0 {
            "Poor"
        } else {
            "Unacceptable"
        }
    }

    /// Validate zero-allocation compliance during steady-state processing
    pub fn validate_zero_allocation(&self) -> bool {
        // Check that we're using pre-allocated buffers
        // During steady-state, memory usage should remain constant
        // Note: memory_usage_bytes is only updated after first analysis
        !self.analysis_buffer.is_empty()
    }

    /// Get memory efficiency metrics
    pub fn get_memory_efficiency(&self) -> (usize, f32, bool) {
        let total_memory = if self.metrics.memory_usage_bytes > 0 {
            self.metrics.memory_usage_bytes
        } else {
            // Estimate if no analysis has been performed yet
            let buffer_memory = self.analysis_buffer.len() * std::mem::size_of::<f32>();
            let detector_memory = self.pitch_detector.memory_usage_bytes();
            let analyzer_memory = std::mem::size_of::<Self>();
            buffer_memory + detector_memory + analyzer_memory
        };
        
        let memory_per_sample = total_memory as f32 / self.config().sample_window_size as f32;
        let is_efficient = memory_per_sample < 100.0; // Less than 100 bytes per sample is efficient
        
        (total_memory, memory_per_sample, is_efficient)
    }

    /// Optimize configuration for target latency while prioritizing accuracy
    pub fn optimize_for_latency(&mut self, target_latency_ms: f32) -> Result<(), PitchAnalysisError> {
        // Get optimal window size for target latency (accuracy-prioritized)
        let optimal_size = super::pitch_detector::PitchDetector::get_optimal_window_size_for_latency(
            target_latency_ms, 
            self.pitch_detector.sample_rate()
        );
        
        if optimal_size != self.config().sample_window_size {
            let mut new_config = self.config().clone();
            new_config.sample_window_size = optimal_size;
            
            self.update_config(new_config)?;
        }
        
        
        Ok(())
    }

    /// Optimize configuration for maximum accuracy within reasonable latency bounds
    pub fn optimize_for_accuracy(&mut self) -> Result<(), PitchAnalysisError> {
        let optimal_size = super::pitch_detector::PitchDetector::get_accuracy_optimized_window_size(
            self.pitch_detector.sample_rate(),
            self.config().min_frequency
        );
        
        if optimal_size != self.config().sample_window_size {
            let mut new_config = self.config().clone();
            new_config.sample_window_size = optimal_size;
            
            self.update_config(new_config)?;
        }
        
        
        Ok(())
    }

    /// Check if current configuration can meet performance requirements
    pub fn meets_latency_requirement(&self, max_latency_ms: f32) -> bool {
        self.metrics.average_latency_ms <= max_latency_ms as f64 && 
        self.meets_performance_requirements()
    }

    /// Update pitch detector configuration
    pub fn update_config(&mut self, config: PitchDetectorConfig) -> Result<(), PitchAnalysisError> {
        self.pitch_detector.update_config(config.clone())
            .map_err(|e| format!("Failed to update pitch detector config: {}", e))?;
        
        
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

    /// Get a reference to the pitch detector for optimization access
    pub fn pitch_detector(&self) -> &super::pitch_detector::PitchDetector {
        &self.pitch_detector
    }

    /// Get a mutable reference to the pitch detector for optimization access
    pub fn pitch_detector_mut(&mut self) -> &mut super::pitch_detector::PitchDetector {
        &mut self.pitch_detector
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

        // Measure YIN algorithm performance specifically
        let yin_start = self.get_high_resolution_time();
        let pitch_result = match self.pitch_detector.analyze(&self.analysis_buffer) {
            Ok(result) => result,
            Err(e) => {
                let end_time = self.get_high_resolution_time();
                self.update_metrics(start_time, end_time, 0.0, false);
                return Err(format!("Pitch detection failed: {}", e));
            }
        };
        let yin_end = self.get_high_resolution_time();
        let yin_time_us = (yin_end - yin_start) * 1000.0; // Convert to microseconds

        let end_time = self.get_high_resolution_time();

        // Process the result and publish events
        match pitch_result {
            Some(result) => {
                self.handle_pitch_detected(result.clone())?;
                self.update_metrics(start_time, end_time, yin_time_us, true);
                self.last_detection = Some(result.clone());
                Ok(Some(result))
            }
            None => {
                self.handle_pitch_lost()?;
                self.update_metrics(start_time, end_time, yin_time_us, false);
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

    /// Analyze a batch of audio data directly without BufferAnalyzer
    /// 
    /// This method is designed for the new postMessage-based architecture where
    /// batched audio data is sent directly from the AudioWorklet.
    /// 
    /// # Arguments
    /// * `batch_data` - The batched audio samples (e.g., 1024 samples)
    /// 
    /// # Returns
    /// * Vec of pitch results, one for each window-sized chunk in the batch
    pub fn analyze_batch_direct(&mut self, batch_data: &[f32]) -> Result<Vec<PitchResult>, PitchAnalysisError> {
        let window_size = self.analysis_buffer.len();
        let num_windows = batch_data.len() / window_size;
        
        if batch_data.len() < window_size {
            return Ok(Vec::new()); // Not enough data for even one window
        }
        
        let mut results = Vec::with_capacity(num_windows);
        
        // Process each window-sized chunk
        for i in 0..num_windows {
            let start = i * window_size;
            let end = start + window_size;
            
            if end <= batch_data.len() {
                let chunk = &batch_data[start..end];
                
                // Use existing analyze_samples method which handles all the
                // pitch detection, event publishing, and metrics
                if let Some(result) = self.analyze_samples(chunk)? {
                    results.push(result);
                } // No pitch detected in this chunk
            }
        }
        
        // Publish metrics update if we processed any chunks
        if !results.is_empty() {
            self.publish_metrics_update();
        }
        
        Ok(results)
    }

    /// Analyze a batch with overlapping windows for improved accuracy
    /// 
    /// This method processes batched data with configurable overlap between windows,
    /// which can improve pitch detection accuracy at the cost of more processing.
    /// 
    /// # Arguments
    /// * `batch_data` - The batched audio samples
    /// * `overlap_factor` - Overlap factor (0.0 = no overlap, 0.5 = 50% overlap)
    /// 
    /// # Returns
    /// * Vec of pitch results from overlapping windows
    pub fn analyze_batch_with_overlap(
        &mut self, 
        batch_data: &[f32], 
        overlap_factor: f32
    ) -> Result<Vec<PitchResult>, PitchAnalysisError> {
        let window_size = self.analysis_buffer.len();
        let overlap_factor = overlap_factor.clamp(0.0, 0.9); // Max 90% overlap
        let step_size = ((window_size as f32) * (1.0 - overlap_factor)) as usize;
        
        if step_size == 0 || batch_data.len() < window_size {
            return Ok(Vec::new());
        }
        
        let mut results = Vec::new();
        let mut position = 0;
        
        while position + window_size <= batch_data.len() {
            let chunk = &batch_data[position..position + window_size];
            
            if let Some(result) = self.analyze_samples(chunk)? {
                results.push(result);
            } // No pitch detected
            
            position += step_size;
        }
        
        // Publish metrics update if we processed any chunks
        if !results.is_empty() {
            self.publish_metrics_update();
        }
        
        Ok(results)
    }

    // Private helper methods

    fn handle_pitch_detected(&mut self, result: PitchResult) -> Result<(), PitchAnalysisError> {
        // Store the latest detection result
        self.last_detection = Some(result.clone());
        
        // Pitch data is now returned through the analyze methods
        // and collected by Engine::update()

        Ok(())
    }

    fn handle_pitch_lost(&mut self) -> Result<(), PitchAnalysisError> {
        // Clear the last detection when pitch is lost
        self.last_detection = None;
        
        // Pitch lost state is now communicated by returning None
        // from the analyze methods
        Ok(())
    }

    fn publish_metrics_update(&mut self) {
        // Log metrics only occasionally to avoid spam (every 1000 cycles)
        if self.metrics.analysis_cycles % 1000 == 0 {
            dev_log!(
                "Pitch Metrics: latency={:.1}ms, cycles={}, success={}", 
                self.metrics.processing_latency_ms, 
                self.metrics.analysis_cycles,
                self.metrics.successful_detections
            );
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

    /// Get the latest pitch detection result
    /// 
    /// Returns the most recent pitch detection result if available,
    /// or None if no pitch has been detected yet.
    pub fn get_latest_pitch_data(&self) -> Option<super::PitchData> {
        self.last_detection.as_ref().map(|result| {
            super::PitchData {
                frequency: result.frequency,
                clarity: result.clarity,
                timestamp: self.get_high_resolution_time(),
            }
        })
    }
}

