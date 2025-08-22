use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector as PitchDetectorTrait};
use crate::app_config::{CLARITY_THRESHOLD, POWER_THRESHOLD};

use crate::app_config::BUFFER_SIZE;

pub type PitchDetectionError = String;

#[derive(Debug, Clone)]
pub struct PitchResult {
    pub frequency: f32,
    pub clarity: f32,
}


#[derive(Debug, Clone)]
pub struct PitchDetectorConfig {
    pub sample_window_size: usize,
    pub power_threshold: f32,
    pub clarity_threshold: f32,
    pub padding_size: usize,
}

impl Default for PitchDetectorConfig {
    fn default() -> Self {
        Self {
            sample_window_size: BUFFER_SIZE,
            power_threshold: POWER_THRESHOLD,
            clarity_threshold: CLARITY_THRESHOLD,
            padding_size: BUFFER_SIZE / 2,
        }
    }
}

pub struct PitchDetector {
    config: PitchDetectorConfig,
    detector: McLeodDetector<f32>,
    sample_rate: u32,
}

impl PitchDetector {
    pub fn new(config: PitchDetectorConfig, sample_rate: u32) -> Result<Self, PitchDetectionError> {
        if config.sample_window_size % 128 != 0 {
            return Err(format!(
                "Sample window size must be a multiple of 128, got {}",
                config.sample_window_size
            ));
        }

        if config.sample_window_size == 0 {
            return Err("Sample window size cannot be zero".to_string());
        }

        if sample_rate == 0 {
            return Err(format!("Sample rate must be positive, got {}", sample_rate));
        }

        if config.power_threshold <= 0.0 {
            return Err(format!(
                "Power threshold must be positive, got {}",
                config.power_threshold
            ));
        }

        if config.clarity_threshold < 0.0 || config.clarity_threshold > 1.0 {
            return Err(format!(
                "Clarity threshold must be between 0.0 and 1.0, got {}",
                config.clarity_threshold
            ));
        }

        if config.padding_size > config.sample_window_size {
            return Err(format!(
                "Padding size ({}) cannot be larger than sample window size ({})",
                config.padding_size, config.sample_window_size
            ));
        }


        let mcleod_detector = McLeodDetector::new(config.sample_window_size, config.padding_size);

        Ok(Self {
            config,
            detector: mcleod_detector,
            sample_rate,
        })
    }

    pub fn analyze(&mut self, samples: &[f32]) -> Result<Option<PitchResult>, PitchDetectionError> {
        if samples.len() != self.config.sample_window_size {
            return Err(format!(
                "Expected {} samples, got {}",
                self.config.sample_window_size,
                samples.len()
            ));
        }

        let result = self.detector.get_pitch(samples, self.sample_rate as usize, self.config.power_threshold, self.config.clarity_threshold);
        
        Ok(result.map(|pitch_info| PitchResult {
            frequency: pitch_info.frequency,
            clarity: pitch_info.clarity,
        }))
    }
}

