use wasm_bindgen::prelude::*;
use js_sys;

/// Comprehensive performance metrics
/// Replaces complex JavaScript performance calculations with optimized Rust
#[wasm_bindgen]
pub struct PerformanceMetrics {
    processing_rate_hz: f32,
    total_latency_ms: f32,
    buffer_latency_ms: f32,
    processing_latency_ms: f32,
    latency_compliant: bool,
    target_latency_ms: f32,
}

#[wasm_bindgen]
impl PerformanceMetrics {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            processing_rate_hz: 0.0,
            total_latency_ms: 0.0,
            buffer_latency_ms: 0.0,
            processing_latency_ms: 0.0,
            latency_compliant: false,
            target_latency_ms: 10.0,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn processing_rate_hz(&self) -> f32 { self.processing_rate_hz }
    
    #[wasm_bindgen(getter)]
    pub fn total_latency_ms(&self) -> f32 { self.total_latency_ms }
    
    #[wasm_bindgen(getter)]
    pub fn buffer_latency_ms(&self) -> f32 { self.buffer_latency_ms }
    
    #[wasm_bindgen(getter)]
    pub fn processing_latency_ms(&self) -> f32 { self.processing_latency_ms }
    
    #[wasm_bindgen(getter)]
    pub fn latency_compliant(&self) -> bool { self.latency_compliant }
    
    #[wasm_bindgen(getter)]
    pub fn target_latency_ms(&self) -> f32 { self.target_latency_ms }
}

/// Pipeline health and validation status
#[wasm_bindgen]
pub struct PipelineStatus {
    pipeline_healthy: bool,
    wasm_connected: bool,
    audio_flowing: bool,
    configuration_valid: bool,
    sample_rate: f32,
    buffer_size: usize,
}

#[wasm_bindgen]
impl PipelineStatus {
    #[wasm_bindgen(getter)]
    pub fn pipeline_healthy(&self) -> bool { self.pipeline_healthy }
    
    #[wasm_bindgen(getter)]
    pub fn wasm_connected(&self) -> bool { self.wasm_connected }
    
    #[wasm_bindgen(getter)]
    pub fn audio_flowing(&self) -> bool { self.audio_flowing }
    
    #[wasm_bindgen(getter)]
    pub fn configuration_valid(&self) -> bool { self.configuration_valid }
    
    #[wasm_bindgen(getter)]
    pub fn sample_rate(&self) -> f32 { self.sample_rate }
    
    #[wasm_bindgen(getter)]
    pub fn buffer_size(&self) -> usize { self.buffer_size }
}

/// Performance monitoring system
/// Consolidates all performance calculation logic from JavaScript into optimized Rust
pub struct PerformanceMonitor {
    // Configuration
    sample_rate: f32,
    buffer_size: usize,
    target_latency_ms: f32,
    
    // Performance tracking
    process_count: u64,
    processing_times: Vec<f32>,
    max_history: usize,
    
    // Latency components
    audio_context_latency: f32,
    output_latency: f32,
    
    // Pipeline status
    is_wasm_connected: bool,
    is_audio_flowing: bool,
    last_audio_signal_time: f64,
    
    // Timing
    last_performance_report: f64,
    performance_report_interval: f64,
}

impl PerformanceMonitor {
    pub fn new(sample_rate: f32, buffer_size: usize, target_latency_ms: f32) -> Self {
        Self {
            sample_rate,
            buffer_size,
            target_latency_ms,
            process_count: 0,
            processing_times: Vec::with_capacity(1000), // Keep last 1000 measurements
            max_history: 1000,
            audio_context_latency: 0.0,
            output_latency: 0.0,
            is_wasm_connected: false,
            is_audio_flowing: false,
            last_audio_signal_time: 0.0,
            last_performance_report: 0.0,
            performance_report_interval: 1000.0, // Report every second
        }
    }
    
    /// Record a processing cycle
    /// Replaces JavaScript latency accumulation logic
    pub fn record_processing_cycle(&mut self, processing_time_ms: f32, has_audio_signal: bool) {
        self.process_count += 1;
        
        // Add processing time to history
        self.processing_times.push(processing_time_ms);
        if self.processing_times.len() > self.max_history {
            self.processing_times.remove(0);
        }
        
        // Update audio flow status
        if has_audio_signal {
            self.last_audio_signal_time = js_sys::Date::now();
            self.is_audio_flowing = true;
        } else {
            // Consider audio stopped if no signal for 5 seconds
            let current_time = js_sys::Date::now();
            if current_time - self.last_audio_signal_time > 5000.0 {
                self.is_audio_flowing = false;
            }
        }
    }
    
    /// Set WASM connection status
    pub fn set_wasm_connected(&mut self, connected: bool) {
        self.is_wasm_connected = connected;
    }
    
    /// Update latency components
    /// Replaces JavaScript latency calculation logic from app.js
    pub fn update_latency_components(&mut self, audio_context_latency: f32, output_latency: f32) {
        self.audio_context_latency = audio_context_latency;
        self.output_latency = output_latency;
    }
    
    /// Get comprehensive performance metrics
    /// Replaces multiple JavaScript performance calculation functions
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        // Calculate processing rate (Hz)
        let processing_rate_hz = self.calculate_processing_rate();
        
        // Calculate average processing latency
        let processing_latency_ms = self.calculate_average_processing_latency();
        
        // Calculate buffer latency
        let buffer_latency_ms = (self.buffer_size as f32 / self.sample_rate) * 1000.0;
        
        // Calculate total latency
        let total_latency_ms = self.audio_context_latency + self.output_latency + 
                              processing_latency_ms + buffer_latency_ms;
        
        // Check latency compliance
        let latency_compliant = total_latency_ms <= self.target_latency_ms;
        
        PerformanceMetrics {
            processing_rate_hz,
            total_latency_ms,
            buffer_latency_ms,
            processing_latency_ms,
            latency_compliant,
            target_latency_ms: self.target_latency_ms,
        }
    }
    
    /// Validate audio pipeline status
    /// Replaces JavaScript connection validation logic
    pub fn validate_audio_pipeline(&self) -> PipelineStatus {
        // Overall pipeline health check
        let pipeline_healthy = self.is_wasm_connected && 
                              self.is_configuration_valid() &&
                              self.get_performance_metrics().latency_compliant;
        
        PipelineStatus {
            pipeline_healthy,
            wasm_connected: self.is_wasm_connected,
            audio_flowing: self.is_audio_flowing,
            configuration_valid: self.is_configuration_valid(),
            sample_rate: self.sample_rate,
            buffer_size: self.buffer_size,
        }
    }
    
    /// Check if performance reporting is due
    pub fn should_report_performance(&mut self) -> bool {
        let current_time = js_sys::Date::now();
        if current_time - self.last_performance_report >= self.performance_report_interval {
            self.last_performance_report = current_time;
            true
        } else {
            false
        }
    }
    
    /// Calculate processing rate in Hz
    fn calculate_processing_rate(&self) -> f32 {
        // Estimate based on sample rate and buffer size
        self.sample_rate / self.buffer_size as f32
    }
    
    /// Calculate average processing latency
    fn calculate_average_processing_latency(&self) -> f32 {
        if self.processing_times.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = self.processing_times.iter().sum();
        sum / self.processing_times.len() as f32
    }
    
    /// Check configuration validity
    fn is_configuration_valid(&self) -> bool {
        // Validate sample rate and buffer size are reasonable
        self.sample_rate >= 8000.0 && self.sample_rate <= 96000.0 &&
        self.buffer_size >= 128 && self.buffer_size <= 8192 &&
        (self.buffer_size & (self.buffer_size - 1)) == 0 // Power of 2
    }
    
    /// Get processing statistics for debugging
    pub fn get_debug_stats(&self) -> (u64, f32, f32, usize) {
        let avg_processing_time = self.calculate_average_processing_latency();
        let min_processing_time = self.processing_times.iter().cloned().fold(f32::INFINITY, f32::min);
        let _max_processing_time = self.processing_times.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        
        (
            self.process_count,
            avg_processing_time,
            if min_processing_time.is_finite() { min_processing_time } else { 0.0 },
            self.processing_times.len()
        )
    }
    
    /// Reset performance counters
    pub fn reset_counters(&mut self) {
        self.process_count = 0;
        self.processing_times.clear();
        self.last_performance_report = js_sys::Date::now();
    }
} 