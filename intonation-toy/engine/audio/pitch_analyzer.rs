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
        })
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

