#[derive(Debug, Clone)]
pub struct VolumeAnalysis {
    pub rms_amplitude: f32,
    pub peak_amplitude: f32,
}

#[derive(Clone)]
pub struct VolumeDetector {}

impl VolumeDetector {
    pub fn new_default() -> Self {
        Self {}
    }

    pub fn process_buffer(&mut self, samples: &[f32]) -> VolumeAnalysis {
        if samples.is_empty() {
            return VolumeAnalysis { rms_amplitude: 0.0, peak_amplitude: 0.0 };
        }

        let (rms_amplitude, peak_amplitude) = self.calculate_rms_and_peak(samples);
        VolumeAnalysis { rms_amplitude, peak_amplitude }
    }

    fn calculate_rms_and_peak(&self, samples: &[f32]) -> (f32, f32) {
        let mut sum_squares = 0.0f32;
        let mut peak = 0.0f32;
        
        for &sample in samples {
            if sample.is_finite() {
                let abs_sample = sample.abs();
                sum_squares += sample * sample;
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
}

