use crate::app_config::STANDARD_SAMPLE_RATE;

/// Volume analysis result from processing an audio buffer
#[derive(Debug, Clone)]
pub struct VolumeAnalysis {
    /// RMS (Root Mean Square) level as amplitude (0.0 to 1.0)
    pub rms_amplitude: f32,
    /// Peak level as amplitude (0.0 to 1.0)
    pub peak_amplitude: f32,
}

impl VolumeAnalysis {
    /// Create a new volume analysis result
    pub fn new(rms_amplitude: f32, peak_amplitude: f32) -> Self {
        Self {
            rms_amplitude,
            peak_amplitude,
        }
    }

}

/// Configuration for volume detection
#[derive(Debug, Clone)]
pub struct VolumeDetectorConfig {
    /// Input gain adjustment in dB (-60 to +60)
    pub input_gain_db: f32,
    /// Noise floor threshold in dB (-80 to -20)
    pub noise_floor_db: f32,
    /// Sample rate for calculations
    pub sample_rate: u32,
}

impl VolumeDetectorConfig {
    /// Create new configuration with default values
    pub fn new() -> Self {
        Self {
            input_gain_db: 0.0,
            noise_floor_db: -60.0,
            sample_rate: STANDARD_SAMPLE_RATE,
        }
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.input_gain_db < -60.0 || self.input_gain_db > 60.0 {
            return Err(format!("Input gain must be between -60 and 60 dB, got {}", self.input_gain_db));
        }
        
        if self.noise_floor_db < -80.0 || self.noise_floor_db > -20.0 {
            return Err(format!("Noise floor must be between -80 and -20 dB, got {}", self.noise_floor_db));
        }
        
        if self.sample_rate == 0 {
            return Err("Sample rate must be positive".to_string());
        }
        
        Ok(())
    }

}

impl Default for VolumeDetectorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Real-time volume detector for audio processing
#[derive(Clone)]
pub struct VolumeDetector {
    config: VolumeDetectorConfig,
}

impl VolumeDetector {
    /// Create new volume detector with configuration
    pub fn new(config: VolumeDetectorConfig) -> Result<Self, String> {
        config.validate()?;
        
        Ok(Self {
            config,
        })
    }

    /// Create new volume detector with default configuration
    pub fn new_default() -> Self {
        Self::new(VolumeDetectorConfig::default()).unwrap()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: VolumeDetectorConfig) -> Result<(), String> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &VolumeDetectorConfig {
        &self.config
    }

    /// Process audio buffer and return volume analysis
    pub fn process_buffer(&mut self, samples: &[f32]) -> VolumeAnalysis {
        if samples.is_empty() {
            return VolumeAnalysis::new(0.0, 0.0);
        }

        // Apply input gain
        let gain_linear = self.db_to_linear(self.config.input_gain_db);
        
        // Calculate RMS and peak values
        let (rms_linear, peak_linear) = self.calculate_rms_and_peak(samples, gain_linear);
        
        // Store as amplitude values
        let rms_amplitude = rms_linear;
        let peak_amplitude = peak_linear;
        
        VolumeAnalysis::new(rms_amplitude, peak_amplitude)
    }

    /// Calculate RMS and peak values from audio samples with zero allocation
    fn calculate_rms_and_peak(&self, samples: &[f32], gain: f32) -> (f32, f32) {
        let mut sum_squares = 0.0f32;
        let mut peak = 0.0f32;
        
        for &sample in samples {
            let scaled_sample = sample * gain;
            
            // Handle NaN and infinity values
            if scaled_sample.is_finite() {
                let abs_sample = scaled_sample.abs();
                sum_squares += scaled_sample * scaled_sample;
                peak = peak.max(abs_sample);
            }
        }
        
        let rms = if sum_squares > 0.0 {
            (sum_squares / samples.len() as f32).sqrt()
        } else {
            0.0
        };
        
        (rms, peak)
    }

    /// Convert dB to linear amplitude
    fn db_to_linear(&self, db: f32) -> f32 {
        if db == -f32::INFINITY {
            0.0
        } else {
            10.0_f32.powf(db / 20.0)
        }
    }
}

