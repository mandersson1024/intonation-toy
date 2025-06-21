use wasm_bindgen::prelude::*;
use js_sys;

/// Comprehensive real-time processing result
/// Replaces complex JavaScript logic with single optimized Rust computation
#[wasm_bindgen]
pub struct RealtimeProcessingResult {
    // Audio processing results
    audio_processed: bool,
    output_buffer: Vec<f32>,
    
    // Pitch detection results  
    pitch_detected: bool,
    detected_frequency: f32,
    pitch_confidence: f32,
    
    // Performance metrics
    processing_time_ms: f32,
    buffer_latency_ms: f32,
}

#[wasm_bindgen]
impl RealtimeProcessingResult {
    #[wasm_bindgen(getter)]
    pub fn audio_processed(&self) -> bool { self.audio_processed }
    
    #[wasm_bindgen(getter)]
    pub fn output_buffer(&self) -> Vec<f32> { self.output_buffer.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn pitch_detected(&self) -> bool { self.pitch_detected }
    
    #[wasm_bindgen(getter)]
    pub fn detected_frequency(&self) -> f32 { self.detected_frequency }
    
    #[wasm_bindgen(getter)]
    pub fn pitch_confidence(&self) -> f32 { self.pitch_confidence }
    
    #[wasm_bindgen(getter)]
    pub fn processing_time_ms(&self) -> f32 { self.processing_time_ms }
    
    #[wasm_bindgen(getter)]
    pub fn buffer_latency_ms(&self) -> f32 { self.buffer_latency_ms }
}

impl Default for RealtimeProcessingResult {
    fn default() -> Self {
        Self {
            audio_processed: false,
            output_buffer: Vec::new(),
            pitch_detected: false,
            detected_frequency: 0.0,
            pitch_confidence: 0.0,
            processing_time_ms: 0.0,
            buffer_latency_ms: 0.0,
        }
    }
}

/// Real-time audio processor
/// Consolidates all audio processing logic from JavaScript into optimized Rust
pub struct RealtimeProcessor {
    sample_rate: f32,
    buffer_size: usize,
    
    // Performance tracking
    process_count: u64,
    total_processing_time: f64,
    
    // Signal analysis state
    signal_history: Vec<bool>,
    signal_index: usize,
    
    // Audio analysis buffers
    last_peak: f32,
    last_rms: f32,
    last_zero_crossings: usize,
}

impl RealtimeProcessor {
    pub fn new(sample_rate: f32, buffer_size: usize) -> Self {
        Self {
            sample_rate,
            buffer_size,
            process_count: 0,
            total_processing_time: 0.0,
            signal_history: vec![false; 10], // 10-sample history like the JS version
            signal_index: 0,
            last_peak: 0.0,
            last_rms: 0.0,
            last_zero_crossings: 0,
        }
    }
    
    /// Main real-time processing function
    /// Replaces ~200 lines of JavaScript logic with optimized Rust
    pub fn process_audio_buffer(&mut self, input_buffer: &[f32]) -> RealtimeProcessingResult {
        let start_time = js_sys::Date::now();
        
        // 1. Analyze audio signal (replaces detectAudioSignal JS function)
        let signal_analysis = self.analyze_signal(input_buffer);
        
        // 2. Update signal stability history
        self.update_signal_history(signal_analysis.signal_detected);
        
        // 3. Perform pitch detection (replaces zero-crossing JS logic)
        let pitch_result = self.detect_pitch(input_buffer);
        
        // 4. Process audio output (simple passthrough for now)
        let output_buffer = self.process_output(input_buffer);
        
        // 5. Calculate performance metrics
        let end_time = js_sys::Date::now();
        let processing_time = (end_time - start_time) as f32;
        
        self.process_count += 1;
        self.total_processing_time += processing_time as f64;
        
        // Calculate buffer latency
        let buffer_latency_ms = (self.buffer_size as f32 / self.sample_rate) * 1000.0;
        
        RealtimeProcessingResult {
            audio_processed: true,
            output_buffer,
            pitch_detected: pitch_result.0,
            detected_frequency: pitch_result.1,
            pitch_confidence: pitch_result.2,
            processing_time_ms: processing_time,
            buffer_latency_ms,
        }
    }
    
    /// Audio signal analysis - replaces JavaScript detectAudioSignal function
    fn analyze_signal(&mut self, buffer: &[f32]) -> SignalAnalysis {
        let mut peak_amplitude: f32 = 0.0;
        let mut rms_sum: f32 = 0.0;
        
        // Calculate peak and RMS in single pass (more efficient than JS)
        for &sample in buffer {
            let abs_sample = sample.abs();
            peak_amplitude = peak_amplitude.max(abs_sample);
            rms_sum += sample * sample;
        }
        
        let rms_level = (rms_sum / buffer.len() as f32).sqrt();
        
        // Enhanced sensitivity thresholds (matching JS version)
        let signal_detected = peak_amplitude > 0.00001 || rms_level > 0.000005;
        
        // Calculate stability
        let _stability = self.calculate_signal_stability();
        
        self.last_peak = peak_amplitude;
        self.last_rms = rms_level;
        
        SignalAnalysis {
            signal_detected,
        }
    }
    
    /// Zero-crossing pitch detection - replaces JavaScript algorithm
    fn detect_pitch(&mut self, buffer: &[f32]) -> (bool, f32, f32) {
        let mut zero_crossings = 0;
        
        // Count zero crossings
        for i in 1..buffer.len() {
            if (buffer[i] >= 0.0) != (buffer[i-1] >= 0.0) {
                zero_crossings += 1;
            }
        }
        
        self.last_zero_crossings = zero_crossings;
        
        // Estimate frequency from zero crossings
        let estimated_frequency = (zero_crossings as f32 * self.sample_rate) / (2.0 * buffer.len() as f32);
        
        // Validate frequency range (matching JS logic)
        let is_valid = (80.0..=2000.0).contains(&estimated_frequency);
        let confidence = if is_valid { 
            // Simple confidence based on signal strength
            (self.last_rms * 1000.0).min(1.0)
        } else { 
            0.0 
        };
        
        (is_valid, if is_valid { estimated_frequency } else { 0.0 }, confidence)
    }
    
    /// Signal stability calculation
    fn calculate_signal_stability(&self) -> f32 {
        let detected_count = self.signal_history.iter().filter(|&&x| x).count();
        detected_count as f32 / self.signal_history.len() as f32
    }
    
    /// Update signal history for stability tracking
    fn update_signal_history(&mut self, signal_detected: bool) {
        self.signal_history[self.signal_index] = signal_detected;
        self.signal_index = (self.signal_index + 1) % self.signal_history.len();
    }
    
    /// Audio output processing (passthrough for now)
    fn process_output(&self, input_buffer: &[f32]) -> Vec<f32> {
        // Simple passthrough with gain (matching existing JS logic)
        input_buffer.iter().map(|&sample| sample * 0.8).collect()
    }
    
    /// Get current processing statistics
    pub fn get_processing_stats(&self) -> (f64, f32, f32, usize) {
        let avg_processing_time = if self.process_count > 0 {
            self.total_processing_time / self.process_count as f64
        } else {
            0.0
        };
        
        (avg_processing_time, self.last_peak, self.last_rms, self.last_zero_crossings)
    }
}

/// Internal signal analysis result
#[derive(Debug, Clone)]
struct SignalAnalysis {
    signal_detected: bool,
} 