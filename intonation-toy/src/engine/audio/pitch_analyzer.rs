use super::pitch_detector::{PitchDetector, PitchDetectorConfig};

pub type PitchAnalysisError = String;

/// Real-time pitch analysis coordinator.
pub struct PitchAnalyzer {
    pitch_detector: PitchDetector,
    analysis_buffer: Vec<f32>,
}

impl PitchAnalyzer {
    pub fn new(
        config: PitchDetectorConfig,
        sample_rate: u32,
    ) -> Result<Self, PitchAnalysisError> {
        let pitch_detector = PitchDetector::new(config.clone(), sample_rate)
            .map_err(|e| format!("Failed to create pitch detector: {}", e))?;
        
        let analysis_buffer = vec![0.0; config.sample_window_size];
        
        Ok(Self {
            pitch_detector,
            analysis_buffer,
        })
    }

    pub fn analyze_samples(&mut self, samples: &[f32]) -> Result<Option<super::PitchData>, PitchAnalysisError> {
        if samples.len() != self.analysis_buffer.len() {
            return Err(format!("Expected {} samples, got {}", self.analysis_buffer.len(), samples.len()));
        }

        self.analysis_buffer.copy_from_slice(samples);
        
        let pitch_result = crate::profile!("pitch_detector.analyze", 
            self.pitch_detector.analyze(&self.analysis_buffer)
        );

        let pitch_result = pitch_result
            .map_err(|e| format!("Pitch detection failed: {}", e))?;

        match pitch_result {
            Some(result) => {
                Ok(Some(super::PitchData {
                    frequency: result.frequency,
                    clarity: result.clarity,
                }))
            }
            None => {
                Ok(None)
            }
        }
    }

}

