use super::pitch_detector::{PitchDetector, PitchDetectorConfig, PitchResult};
use super::buffer_analyzer::{BufferAnalyzer, BufferProcessor};
use super::buffer::CircularBuffer;
use super::volume_detector::VolumeAnalysis;

pub type PitchAnalysisError = String;

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
/// let mut analyzer = PitchAnalyzer::new(config, sample_rate)?;
/// 
/// // Analyze samples and get immediate result
/// if let Some(result) = analyzer.analyze_samples(&samples)? {
///     println!("Detected pitch: {} Hz", result.frequency);
/// }
/// 
/// // Get latest detection for data collection
/// if let Some(pitch_data) = analyzer.get_latest_pitch_data() {
///     println!("Latest pitch: {} Hz", pitch_data.frequency);
/// }
/// ```
pub struct PitchAnalyzer {
    pitch_detector: PitchDetector,
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
        if cfg!(feature = "profiling") {
            crate::web::profiling::profiled("pitch_analyze_samples", || {
                self.analyze_samples_impl(samples)
            })
        } else {
            self.analyze_samples_impl(samples)
        }
    }
    
    fn analyze_samples_impl(&mut self, samples: &[f32]) -> Result<Option<PitchResult>, PitchAnalysisError> {
        // Validate input size
        if samples.len() != self.analysis_buffer.len() {
            return Err(format!("Expected {} samples, got {}", self.analysis_buffer.len(), samples.len()));
        }

        // Copy samples to pre-allocated buffer (minimal allocation)
        self.analysis_buffer.copy_from_slice(samples);
        
        let pitch_result = match self.pitch_detector.analyze(&self.analysis_buffer) {
            Ok(result) => result,
            Err(e) => {
                return Err(format!("Pitch detection failed: {}", e));
            }
        };

        // Process the result and publish events
        match pitch_result {
            Some(result) => {
                self.handle_pitch_detected(result.clone())?;
                self.last_detection = Some(result.clone());
                Ok(Some(result))
            }
            None => {
                self.handle_pitch_lost()?;
                Ok(None)
            }
        }
    }

    /// Analyze audio data from a BufferAnalyzer using zero-allocation processing
    /// 
    /// This method integrates with the existing BufferAnalyzer for sequential processing.
    /// It uses the pre-allocated buffer to avoid heap allocations during steady-state.
    pub fn analyze_from_buffer_analyzer(
        &mut self, 
        buffer_analyzer: &mut BufferAnalyzer
    ) -> Result<Option<PitchResult>, PitchAnalysisError> {
        if cfg!(feature = "profiling") {
            crate::web::profiling::profiled("pitch_analyze_from_buffer", || {
                self.analyze_from_buffer_analyzer_impl(buffer_analyzer)
            })
        } else {
            self.analyze_from_buffer_analyzer_impl(buffer_analyzer)
        }
    }
    
    fn analyze_from_buffer_analyzer_impl(
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


        // Use zero-allocation processing
        let success = buffer_analyzer.process_next_into(&mut self.analysis_buffer);
        if !success {
            return Ok(None); // Not enough data available
        }

        let pitch_result = match self.pitch_detector.analyze(&self.analysis_buffer) {
            Ok(result) => result,
            Err(e) => {
                return Err(format!("Pitch detection failed: {}", e));
            }
        };

        // Process the result and publish events
        match pitch_result {
            Some(result) => {
                self.handle_pitch_detected(result.clone())?;
                self.last_detection = Some(result.clone());
                Ok(Some(result))
            }
            None => {
                self.handle_pitch_lost()?;
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
        // Metrics are now analyzed through proper profiling tools
        // No console logging needed
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

