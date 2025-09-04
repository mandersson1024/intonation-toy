use super::pitch_detector::{PitchDetector, PitchDetectorConfig, PitchResult};

pub type PitchAnalysisError = String;

/// Real-time pitch analysis coordinator.
pub struct PitchAnalyzer {
    pitch_detector: PitchDetector,
    analysis_buffer: Vec<f32>,
}

impl PitchAnalyzer {
    pub fn new(sample_rate: u32) -> Result<Self, PitchAnalysisError> {
        let config = PitchDetectorConfig::default();
        let sample_window_size = config.sample_window_size;
        let pitch_detector = PitchDetector::new(config, sample_rate)
            .map_err(|e| format!("Failed to create pitch detector: {}", e))?;
        
        Ok(Self {
            pitch_detector,
            analysis_buffer: vec![0.0; sample_window_size],
        })
    }

    pub fn analyze_samples(&mut self, samples: &[f32]) -> Option<PitchResult> {
        assert_eq!(samples.len(), self.analysis_buffer.len(), 
                   "Expected {} samples, got {}", self.analysis_buffer.len(), samples.len());

        self.analysis_buffer.copy_from_slice(samples);
        
        crate::profile!("pitch_detector.analyze", 
            self.pitch_detector.analyze(&self.analysis_buffer)
        )
    }

}

