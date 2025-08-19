/// Encapsulates audio analysis state from the engine
pub struct AudioAnalysis {
    pub pitch_detected: bool,
    pub cents_offset: f32,
    pub volume_peak: bool,
}

impl AudioAnalysis {
    /// Creates a new AudioAnalysis with default values
    pub fn new() -> Self {
        Self {
            pitch_detected: false,
            cents_offset: 0.0,
            volume_peak: false,
        }
    }
    
    /// Updates pitch detection state
    pub fn update_pitch(&mut self, pitch_detected: bool, cents_offset: f32) {
        self.pitch_detected = pitch_detected;
        self.cents_offset = cents_offset;
    }
    
    /// Updates volume peak state
    pub fn update_volume_peak(&mut self, volume_peak: bool) {
        self.volume_peak = volume_peak;
    }
}