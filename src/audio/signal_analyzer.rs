use wasm_bindgen::prelude::*;

/// Detailed audio signal analysis
/// Provides enhanced signal analysis beyond basic real-time processing
#[wasm_bindgen]
pub struct AudioAnalysis {
    signal_detected: bool,
    peak_amplitude: f32,
    rms_level: f32,
    signal_stable: bool,
    stability_percentage: f32,
    zero_crossing_rate: f32,
}

#[wasm_bindgen]
impl AudioAnalysis {
    #[wasm_bindgen(getter)]
    pub fn signal_detected(&self) -> bool { self.signal_detected }
    
    #[wasm_bindgen(getter)]
    pub fn peak_amplitude(&self) -> f32 { self.peak_amplitude }
    
    #[wasm_bindgen(getter)]
    pub fn rms_level(&self) -> f32 { self.rms_level }
    
    #[wasm_bindgen(getter)]
    pub fn signal_stable(&self) -> bool { self.signal_stable }
    
    #[wasm_bindgen(getter)]
    pub fn stability_percentage(&self) -> f32 { self.stability_percentage }
    
    #[wasm_bindgen(getter)]
    pub fn zero_crossing_rate(&self) -> f32 { self.zero_crossing_rate }
}

impl Default for AudioAnalysis {
    fn default() -> Self {
        Self {
            signal_detected: false,
            peak_amplitude: 0.0,
            rms_level: 0.0,
            signal_stable: false,
            stability_percentage: 0.0,
            zero_crossing_rate: 0.0,
        }
    }
}

/// Smart buffer configuration
#[wasm_bindgen]
pub struct BufferConfig {
    optimal_buffer_size: usize,
    expected_latency_ms: f32,
    recommended_sample_rate: f32,
}

#[wasm_bindgen]
impl BufferConfig {
    #[wasm_bindgen(getter)]
    pub fn optimal_buffer_size(&self) -> usize { self.optimal_buffer_size }
    
    #[wasm_bindgen(getter)]
    pub fn expected_latency_ms(&self) -> f32 { self.expected_latency_ms }
    
    #[wasm_bindgen(getter)]
    pub fn recommended_sample_rate(&self) -> f32 { self.recommended_sample_rate }
}

/// Buffer configuration constraints
#[wasm_bindgen]
pub struct BufferConstraints {
    max_latency_ms: f32,
    min_sample_rate: f32,
    max_sample_rate: f32,
    prefer_low_latency: bool,
}

#[wasm_bindgen]
impl BufferConstraints {
    #[wasm_bindgen(constructor)]
    pub fn new(max_latency_ms: f32, min_sample_rate: f32, max_sample_rate: f32, prefer_low_latency: bool) -> Self {
        Self {
            max_latency_ms,
            min_sample_rate,
            max_sample_rate,
            prefer_low_latency,
        }
    }
}

/// Detailed audio signal analysis
#[wasm_bindgen]
pub struct SignalAnalyzer {
    sample_rate: f32,
    analysis_window_size: usize,
    signal_history_buffer: Vec<f32>,
    history_index: usize,
    spectral_centroid_history: Vec<f32>,
    amplitude_history: Vec<f32>,
    noise_floor: f32,
}

impl SignalAnalyzer {
    pub fn new(sample_rate: f32, analysis_window_size: usize) -> Self {
        Self {
            sample_rate,
            analysis_window_size,
            signal_history_buffer: vec![0.0; analysis_window_size * 4], // 4x window for history
            history_index: 0,
            spectral_centroid_history: Vec::with_capacity(100),
            amplitude_history: Vec::with_capacity(100),
            noise_floor: 0.00001, // Adaptive noise floor estimation
        }
    }
    
    /// Comprehensive audio signal analysis  
    /// Replaces and enhances JavaScript detectAudioSignal function
    pub fn analyze_audio_signal(&mut self, buffer: &[f32]) -> AudioAnalysis {
        // Update signal history
        self.update_signal_history(buffer);
        
        // Basic amplitude analysis
        let (peak_amplitude, rms_level) = self.calculate_amplitude_metrics(buffer);
        
        // Zero crossing analysis
        let zero_crossing_rate = self.calculate_zero_crossing_rate(buffer);
        
        // Enhanced signal detection with adaptive thresholds
        let signal_detected = self.detect_signal_adaptive(peak_amplitude, rms_level);
        
        // Stability analysis
        let stability_percentage = self.calculate_signal_stability(buffer);
        let signal_stable = stability_percentage > 0.7; // 70% stability threshold
        
        // Update history for future analysis
        self.amplitude_history.push(rms_level);
        if self.amplitude_history.len() > 100 {
            self.amplitude_history.remove(0);
        }
        
        AudioAnalysis {
            signal_detected,
            peak_amplitude,
            rms_level,
            signal_stable,
            stability_percentage,
            zero_crossing_rate,
        }
    }
    
    /// Optimize buffer configuration based on constraints
    /// Provides intelligent buffer size recommendations
    pub fn optimize_buffer_configuration(&self, constraints: &BufferConstraints) -> BufferConfig {
        let mut optimal_buffer_size = 1024; // Default
        let mut optimal_sample_rate = self.sample_rate;
        
        // Calculate latency for different buffer sizes
        let buffer_sizes = vec![128, 256, 512, 1024, 2048, 4096];
        
        for &buffer_size in &buffer_sizes {
            let buffer_latency = (buffer_size as f32 / optimal_sample_rate) * 1000.0;
            
            if buffer_latency <= constraints.max_latency_ms {
                if constraints.prefer_low_latency {
                    // Choose smallest valid buffer size
                    if buffer_size < optimal_buffer_size {
                        optimal_buffer_size = buffer_size;
                    }
                } else {
                    // Choose largest valid buffer size for stability
                    if buffer_size > optimal_buffer_size {
                        optimal_buffer_size = buffer_size;
                    }
                }
            }
        }
        
        // Optimize sample rate if needed
        if optimal_sample_rate < constraints.min_sample_rate {
            optimal_sample_rate = constraints.min_sample_rate;
        } else if optimal_sample_rate > constraints.max_sample_rate {
            optimal_sample_rate = constraints.max_sample_rate;
        }
        
        let expected_latency_ms = (optimal_buffer_size as f32 / optimal_sample_rate) * 1000.0;
        
        BufferConfig {
            optimal_buffer_size,
            expected_latency_ms,
            recommended_sample_rate: optimal_sample_rate,
        }
    }
    
    /// Update signal history buffer for temporal analysis
    fn update_signal_history(&mut self, buffer: &[f32]) {
        for &sample in buffer {
            self.signal_history_buffer[self.history_index] = sample;
            self.history_index = (self.history_index + 1) % self.analysis_window_size;
        }
    }
    
    /// Calculate amplitude metrics (peak and RMS)
    fn calculate_amplitude_metrics(&self, buffer: &[f32]) -> (f32, f32) {
        let mut peak_amplitude: f32 = 0.0;
        let mut rms_sum: f32 = 0.0;
        
        for &sample in buffer {
            let abs_sample = sample.abs();
            peak_amplitude = peak_amplitude.max(abs_sample);
            rms_sum += sample * sample;
        }
        
        let rms_level = (rms_sum / buffer.len() as f32).sqrt();
        
        (peak_amplitude, rms_level)
    }
    
    /// Calculate zero crossing rate
    fn calculate_zero_crossing_rate(&self, buffer: &[f32]) -> f32 {
        let mut zero_crossings = 0;
        
        for i in 1..buffer.len() {
            if (buffer[i] >= 0.0) != (buffer[i-1] >= 0.0) {
                zero_crossings += 1;
            }
        }
        
        zero_crossings as f32 / buffer.len() as f32
    }
    
    /// Adaptive signal detection with noise floor estimation
    fn detect_signal_adaptive(&mut self, peak_amplitude: f32, rms_level: f32) -> bool {
        // Update noise floor estimation
        if peak_amplitude < self.noise_floor * 2.0 && rms_level < self.noise_floor {
            // Slowly adapt noise floor downward during quiet periods
            self.noise_floor = self.noise_floor * 0.999 + peak_amplitude * 0.001;
        }
        
        // Adaptive threshold based on noise floor
        let adaptive_threshold = self.noise_floor * 10.0;
        
        peak_amplitude > adaptive_threshold || rms_level > adaptive_threshold * 0.5
    }
    
    /// Calculate signal stability over time
    fn calculate_signal_stability(&self, _buffer: &[f32]) -> f32 {
        if self.amplitude_history.len() < 5 {
            return 0.0;
        }
        
        // Calculate coefficient of variation for recent amplitude history
        let recent_history = &self.amplitude_history[self.amplitude_history.len().saturating_sub(10)..];
        
        let mean: f32 = recent_history.iter().sum::<f32>() / recent_history.len() as f32;
        
        if mean == 0.0 {
            return 0.0;
        }
        
        let variance: f32 = recent_history.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / recent_history.len() as f32;
        
        let std_dev = variance.sqrt();
        let coefficient_of_variation = std_dev / mean;
        
        // Convert to stability percentage (lower variation = higher stability)
        (1.0 - coefficient_of_variation.min(1.0)).max(0.0)
    }
    
    /// Get current noise floor for debugging
    pub fn get_noise_floor(&self) -> f32 {
        self.noise_floor
    }
    
    /// Reset analysis state
    pub fn reset_analysis(&mut self) {
        self.signal_history_buffer.fill(0.0);
        self.history_index = 0;
        self.spectral_centroid_history.clear();
        self.amplitude_history.clear();
        self.noise_floor = 0.00001;
    }
    
    /// Get analysis window info
    pub fn get_analysis_info(&self) -> (usize, f32, usize) {
        (
            self.analysis_window_size,
            self.sample_rate,
            self.signal_history_buffer.len()
        )
    }
} 