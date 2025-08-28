use super::pitch_detector::{PitchDetector, PitchDetectorConfig, PitchResult};

pub type PitchAnalysisError = String;

/// Real-time pitch analysis coordinator.
pub struct PitchAnalyzer {
    pitch_detector: PitchDetector,
    last_detection: Option<PitchResult>,
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
            last_detection: None,
            analysis_buffer,
        })
    }

    pub fn analyze_samples(&mut self, samples: &[f32]) -> Result<Option<PitchResult>, PitchAnalysisError> {
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
                self.last_detection = Some(result.clone());
                Ok(Some(result))
            }
            None => {
                self.last_detection = None;
                Ok(None)
            }
        }
    }

    pub fn get_latest_pitch_data(&self) -> Option<super::PitchData> {
        self.last_detection.as_ref().map(|result| {
            super::PitchData {
                frequency: result.frequency,
                clarity: result.clarity,
            }
        })
    }
}

