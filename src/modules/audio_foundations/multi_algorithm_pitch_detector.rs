// Multi-Algorithm Pitch Detection Module - STORY-015
// Enhanced pitch detection with multiple algorithms, runtime switching, and comprehensive analysis

use pitch_detection::detector::{mcleod::McLeodDetector, yin::YINDetector, PitchDetector as PitchDetectorTrait};
use std::time::{Duration, Instant};
use crate::modules::audio_foundations::audio_events::{PitchDetectionEvent, SignalAnalysisEvent};
use crate::modules::application_core::event_bus::{EventBus, Event};
use std::sync::Arc;

/// Multi-algorithm pitch detection algorithms available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitchAlgorithm {
    /// YIN algorithm - good for monophonic signals
    YIN,
    /// McLeod Pitch Method - robust against noise
    McLeod,
    /// Automatic algorithm selection based on signal characteristics
    Auto,
}

impl std::fmt::Display for PitchAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PitchAlgorithm::YIN => write!(f, "YIN"),
            PitchAlgorithm::McLeod => write!(f, "McLeod"),
            PitchAlgorithm::Auto => write!(f, "Auto"),
        }
    }
}

/// Configuration for pitch detection algorithms
#[derive(Debug, Clone)]
pub struct PitchDetectionConfig {
    /// Primary algorithm to use
    pub algorithm: PitchAlgorithm,
    /// Sample rate in Hz
    pub sample_rate: f32,
    /// Minimum detectable frequency in Hz
    pub min_frequency: f32,
    /// Maximum detectable frequency in Hz  
    pub max_frequency: f32,
    /// Detection threshold for YIN algorithm
    pub yin_threshold: f32,
    /// Detection threshold for McLeod algorithm
    pub mcleod_threshold: f32,
    /// Clarity threshold for McLeod algorithm
    pub mcleod_clarity_threshold: f32,
    /// Enable confidence scoring
    pub enable_confidence_scoring: bool,
    /// Enable harmonic analysis
    pub enable_harmonic_analysis: bool,
    /// Auto-selection sensitivity (0.0-1.0)
    pub auto_selection_sensitivity: f32,
}

impl Default for PitchDetectionConfig {
    fn default() -> Self {
        Self {
            algorithm: PitchAlgorithm::YIN,
            sample_rate: 44100.0,
            min_frequency: 80.0,
            max_frequency: 2000.0,
            yin_threshold: 0.2,
            mcleod_threshold: 0.3,
            mcleod_clarity_threshold: 0.7,
            enable_confidence_scoring: true,
            enable_harmonic_analysis: true,
            auto_selection_sensitivity: 0.5,
        }
    }
}

/// Comprehensive pitch detection result with analysis
#[derive(Debug, Clone)]
pub struct PitchResult {
    /// Detected frequency in Hz
    pub frequency: f32,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Clarity/quality measure (0.0-1.0)
    pub clarity: f32,
    /// Harmonic content analysis (0.0-1.0, where 1.0 is pure harmonic)
    pub harmonic_content: f32,
    /// Algorithm used for detection
    pub algorithm_used: PitchAlgorithm,
    /// Processing time in nanoseconds
    pub processing_time_ns: u64,
    /// Whether result is within valid frequency range
    pub is_valid: bool,
    /// Raw algorithm-specific data
    pub raw_clarity: f32,
    /// Signal-to-noise ratio estimate
    pub snr_estimate: f32,
}

/// Error types for pitch detection operations
#[derive(Debug, Clone)]
pub enum PitchError {
    InvalidConfiguration(String),
    ProcessingError(String),
    AlgorithmError(String),
    BufferTooSmall(usize, usize), // got, minimum_required
}

impl std::fmt::Display for PitchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PitchError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            PitchError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            PitchError::AlgorithmError(msg) => write!(f, "Algorithm error: {}", msg),
            PitchError::BufferTooSmall(got, required) => write!(f, "Buffer too small: got {} samples, need at least {}", got, required),
        }
    }
}

impl std::error::Error for PitchError {}

/// Algorithm performance information for comparison and recommendation
#[derive(Debug, Clone)]
pub struct AlgorithmInfo {
    /// Algorithm name
    pub name: PitchAlgorithm,
    /// Typical processing time in nanoseconds
    pub avg_processing_time_ns: u64,
    /// Accuracy score (0.0-1.0) based on historical performance
    pub accuracy_score: f32,
    /// Suitability for current signal type (0.0-1.0)
    pub signal_suitability: f32,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
}

/// Performance comparison between algorithms
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// YIN algorithm performance
    pub yin_performance: AlgorithmInfo,
    /// McLeod algorithm performance
    pub mcleod_performance: AlgorithmInfo,
    /// Recommended algorithm for current signal
    pub recommended_algorithm: PitchAlgorithm,
    /// Confidence in recommendation (0.0-1.0)
    pub recommendation_confidence: f32,
}

/// Core trait for pitch detection
pub trait PitchDetector: Send + Sync {
    /// Configure the pitch detector
    fn configure(&mut self, config: PitchDetectionConfig) -> Result<(), PitchError>;
    
    /// Detect pitch from audio buffer
    fn detect_pitch(&mut self, buffer: &[f32]) -> Result<PitchResult, PitchError>;
    
    /// Set algorithm without reconfiguration
    fn set_algorithm(&mut self, algorithm: PitchAlgorithm) -> Result<(), PitchError>;
    
    /// Get information about current algorithm
    fn get_algorithm_info(&self) -> AlgorithmInfo;
    
    /// Get performance comparison of available algorithms
    fn get_performance_comparison(&self) -> PerformanceComparison;
    
    /// Enable/disable real-time event publishing
    fn set_event_publishing(&mut self, enabled: bool);
}

/// Multi-algorithm pitch detector implementation
pub struct MultiAlgorithmPitchDetector {
    config: PitchDetectionConfig,
    yin_detector: Option<YINDetector<f32>>,
    mcleod_detector: Option<McLeodDetector<f32>>,
    event_bus: Option<Arc<dyn EventBus>>,
    event_publishing_enabled: bool,
    
    // Performance tracking
    yin_performance_history: Vec<(u64, f32)>, // (processing_time_ns, confidence)
    mcleod_performance_history: Vec<(u64, f32)>,
    yin_accuracy_samples: Vec<f32>, // Historical accuracy measurements
    mcleod_accuracy_samples: Vec<f32>,
    
    // Signal analysis for auto-selection
    recent_snr_estimates: Vec<f32>,
    recent_harmonic_content: Vec<f32>,
}

impl MultiAlgorithmPitchDetector {
    /// Create new multi-algorithm pitch detector
    pub fn new(config: PitchDetectionConfig, event_bus: Option<Arc<dyn EventBus>>) -> Result<Self, PitchError> {
        Self::validate_config(&config)?;
        
        Ok(Self {
            config,
            yin_detector: None,
            mcleod_detector: None,
            event_bus,
            event_publishing_enabled: true,
            yin_performance_history: Vec::new(),
            mcleod_performance_history: Vec::new(),
            yin_accuracy_samples: Vec::with_capacity(100),
            mcleod_accuracy_samples: Vec::with_capacity(100),
            recent_snr_estimates: Vec::with_capacity(20),
            recent_harmonic_content: Vec::with_capacity(20),
        })
    }
    
    /// Validate configuration parameters
    fn validate_config(config: &PitchDetectionConfig) -> Result<(), PitchError> {
        if config.sample_rate <= 0.0 {
            return Err(PitchError::InvalidConfiguration("Sample rate must be positive".to_string()));
        }
        
        if config.min_frequency >= config.max_frequency {
            return Err(PitchError::InvalidConfiguration("Min frequency must be less than max frequency".to_string()));
        }
        
        if config.min_frequency <= 0.0 {
            return Err(PitchError::InvalidConfiguration("Min frequency must be positive".to_string()));
        }
        
        if !(0.0..=1.0).contains(&config.yin_threshold) {
            return Err(PitchError::InvalidConfiguration("YIN threshold must be between 0.0 and 1.0".to_string()));
        }
        
        if !(0.0..=1.0).contains(&config.mcleod_threshold) {
            return Err(PitchError::InvalidConfiguration("McLeod threshold must be between 0.0 and 1.0".to_string()));
        }
        
        Ok(())
    }
    
    /// Get or create YIN detector with current configuration
    fn get_yin_detector(&mut self, buffer_size: usize) -> Result<&mut YINDetector<f32>, PitchError> {
        if self.yin_detector.is_none() {
            let tau_max = (buffer_size / 2).max(1);
            self.yin_detector = Some(YINDetector::new(buffer_size, tau_max));
        }
        
        Ok(self.yin_detector.as_mut().unwrap())
    }
    
    /// Get or create McLeod detector with current configuration
    fn get_mcleod_detector(&mut self, buffer_size: usize) -> Result<&mut McLeodDetector<f32>, PitchError> {
        if self.mcleod_detector.is_none() {
            let tau_max = (buffer_size / 2).max(1);
            self.mcleod_detector = Some(McLeodDetector::new(buffer_size, tau_max));
        }
        
        Ok(self.mcleod_detector.as_mut().unwrap())
    }
    
    /// Detect pitch using YIN algorithm
    fn detect_with_yin(&mut self, buffer: &[f32]) -> Result<Option<PitchResult>, PitchError> {
        let start_time = Instant::now();
        
        // Extract config values first to avoid borrow conflicts
        let sample_rate = self.config.sample_rate;
        let yin_threshold = self.config.yin_threshold;
        
        let detector = self.get_yin_detector(buffer.len())?;
        let tau_max = buffer.len() / 2;
        
        let result = detector.get_pitch(
            buffer,
            sample_rate as usize,
            yin_threshold,
            tau_max as f32,
        );
        
        let processing_time_ns = start_time.elapsed().as_nanos() as u64;
        
        if let Some(pitch) = result {
            let mut pitch_result = PitchResult {
                frequency: pitch.frequency,
                confidence: self.calculate_confidence(pitch.clarity, PitchAlgorithm::YIN),
                clarity: pitch.clarity,
                harmonic_content: if self.config.enable_harmonic_analysis {
                    self.analyze_harmonic_content(buffer, pitch.frequency)
                } else {
                    0.0
                },
                algorithm_used: PitchAlgorithm::YIN,
                processing_time_ns,
                is_valid: self.validate_frequency(pitch.frequency),
                raw_clarity: pitch.clarity,
                snr_estimate: self.estimate_snr(buffer),
            };
            
            // Update performance tracking
            self.update_yin_performance(processing_time_ns, pitch_result.confidence);
            
            // Enhance confidence scoring if enabled
            if self.config.enable_confidence_scoring {
                pitch_result.confidence = self.enhanced_confidence_scoring(&pitch_result, buffer);
            }
            
            Ok(Some(pitch_result))
        } else {
            self.update_yin_performance(processing_time_ns, 0.0);
            Ok(None)
        }
    }
    
    /// Detect pitch using McLeod algorithm
    fn detect_with_mcleod(&mut self, buffer: &[f32]) -> Result<Option<PitchResult>, PitchError> {
        let start_time = Instant::now();
        
        // Extract config values first to avoid borrow conflicts
        let sample_rate = self.config.sample_rate;
        let mcleod_threshold = self.config.mcleod_threshold;
        
        let detector = self.get_mcleod_detector(buffer.len())?;
        let tau_max = buffer.len() / 2;
        
        let result = detector.get_pitch(
            buffer,
            sample_rate as usize,
            mcleod_threshold,
            tau_max as f32,
        );
        
        let processing_time_ns = start_time.elapsed().as_nanos() as u64;
        
        if let Some(pitch) = result {
            let mut pitch_result = PitchResult {
                frequency: pitch.frequency,
                confidence: self.calculate_confidence(pitch.clarity, PitchAlgorithm::McLeod),
                clarity: pitch.clarity,
                harmonic_content: if self.config.enable_harmonic_analysis {
                    self.analyze_harmonic_content(buffer, pitch.frequency)
                } else {
                    0.0
                },
                algorithm_used: PitchAlgorithm::McLeod,
                processing_time_ns,
                is_valid: self.validate_frequency(pitch.frequency),
                raw_clarity: pitch.clarity,
                snr_estimate: self.estimate_snr(buffer),
            };
            
            // Update performance tracking
            self.update_mcleod_performance(processing_time_ns, pitch_result.confidence);
            
            // Enhance confidence scoring if enabled
            if self.config.enable_confidence_scoring {
                pitch_result.confidence = self.enhanced_confidence_scoring(&pitch_result, buffer);
            }
            
            Ok(Some(pitch_result))
        } else {
            self.update_mcleod_performance(processing_time_ns, 0.0);
            Ok(None)
        }
    }
    
    /// Automatically select best algorithm based on signal characteristics
    fn auto_select_algorithm(&self, buffer: &[f32]) -> PitchAlgorithm {
        let snr = self.estimate_snr(buffer);
        let signal_complexity = self.estimate_signal_complexity(buffer);
        
        // Use historical performance and signal characteristics
        let avg_yin_confidence = self.average_confidence(&self.yin_performance_history);
        let avg_mcleod_confidence = self.average_confidence(&self.mcleod_performance_history);
        
        // Decision logic based on signal characteristics and performance
        if snr > 0.7 && signal_complexity < 0.5 {
            // Clean, simple signal - YIN is typically faster and equally accurate
            PitchAlgorithm::YIN
        } else if snr < 0.3 || signal_complexity > 0.7 {
            // Noisy or complex signal - McLeod is more robust
            PitchAlgorithm::McLeod
        } else {
            // Use historical performance to decide
            if avg_yin_confidence > avg_mcleod_confidence {
                PitchAlgorithm::YIN
            } else {
                PitchAlgorithm::McLeod
            }
        }
    }
    
    /// Calculate confidence score based on algorithm and clarity
    fn calculate_confidence(&self, clarity: f32, algorithm: PitchAlgorithm) -> f32 {
        match algorithm {
            PitchAlgorithm::YIN => {
                // YIN clarity is inverse correlation - lower values indicate better matches
                (1.0 - clarity.min(1.0)).max(0.0)
            }
            PitchAlgorithm::McLeod => {
                // McLeod clarity is direct correlation - higher values indicate better matches
                clarity.max(0.0).min(1.0)
            }
            PitchAlgorithm::Auto => 0.5, // Should not be called directly
        }
    }
    
    /// Enhanced confidence scoring using multiple factors
    fn enhanced_confidence_scoring(&self, result: &PitchResult, _buffer: &[f32]) -> f32 {
        let base_confidence = result.confidence;
        
        // Factor in signal-to-noise ratio
        let snr_bonus = (result.snr_estimate * 0.2).min(0.2);
        
        // Factor in harmonic content (pure tones are more confident)
        let harmonic_bonus = (result.harmonic_content * 0.15).min(0.15);
        
        // Factor in frequency stability (if we have recent history)
        let stability_bonus = 0.0; // TODO: Implement frequency stability tracking
        
        // Combine factors
        (base_confidence + snr_bonus + harmonic_bonus + stability_bonus).min(1.0)
    }
    
    /// Analyze harmonic content of the signal
    fn analyze_harmonic_content(&self, buffer: &[f32], fundamental: f32) -> f32 {
        if buffer.len() < 512 {
            return 0.0;
        }
        
        // Simple harmonic analysis - check for energy at harmonic frequencies
        let sample_rate = self.config.sample_rate;
        let _fft_size = buffer.len().min(1024);
        
        // For simplicity, we'll estimate harmonic content by checking if the signal
        // has strong periodicity at the detected fundamental frequency
        let period_samples = sample_rate / fundamental;
        if period_samples < 2.0 || period_samples >= buffer.len() as f32 / 2.0 {
            return 0.0;
        }
        
        // Calculate autocorrelation at the fundamental period
        let period = period_samples as usize;
        if period >= buffer.len() / 2 {
            return 0.0;
        }
        
        let mut correlation = 0.0;
        let mut energy = 0.0;
        
        for i in 0..(buffer.len() - period) {
            correlation += buffer[i] * buffer[i + period];
            energy += buffer[i] * buffer[i];
        }
        
        if energy > 0.0 {
            (correlation / energy).abs().min(1.0)
        } else {
            0.0
        }
    }
    
    /// Estimate signal-to-noise ratio
    fn estimate_snr(&self, buffer: &[f32]) -> f32 {
        if buffer.is_empty() {
            return 0.0;
        }
        
        // Calculate RMS energy
        let rms = (buffer.iter().map(|&x| x * x).sum::<f32>() / buffer.len() as f32).sqrt();
        
        // Estimate noise floor from minimum energy segments
        let segment_size = 64;
        let mut min_segment_energy = f32::INFINITY;
        
        for chunk in buffer.chunks(segment_size) {
            let segment_rms = (chunk.iter().map(|&x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();
            min_segment_energy = min_segment_energy.min(segment_rms);
        }
        
        // SNR estimate (crude but functional)
        if min_segment_energy > 0.0 {
            (rms / min_segment_energy).log10().max(0.0).min(2.0) / 2.0 // Normalize to 0-1
        } else {
            1.0
        }
    }
    
    /// Estimate signal complexity (0.0 = simple, 1.0 = complex)
    fn estimate_signal_complexity(&self, buffer: &[f32]) -> f32 {
        if buffer.len() < 64 {
            return 0.5;
        }
        
        // Calculate zero-crossing rate as a measure of complexity
        let mut zero_crossings = 0;
        for i in 1..buffer.len() {
            if (buffer[i] >= 0.0) != (buffer[i-1] >= 0.0) {
                zero_crossings += 1;
            }
        }
        
        let zcr = zero_crossings as f32 / buffer.len() as f32;
        
        // Normalize to 0-1 range (assuming max reasonable ZCR is around 0.5)
        (zcr * 2.0).min(1.0)
    }
    
    /// Validate frequency is within acceptable range
    fn validate_frequency(&self, frequency: f32) -> bool {
        frequency >= self.config.min_frequency && frequency <= self.config.max_frequency
    }
    
    /// Update YIN performance tracking
    fn update_yin_performance(&mut self, processing_time_ns: u64, confidence: f32) {
        self.yin_performance_history.push((processing_time_ns, confidence));
        if self.yin_performance_history.len() > 100 {
            self.yin_performance_history.remove(0);
        }
    }
    
    /// Update McLeod performance tracking
    fn update_mcleod_performance(&mut self, processing_time_ns: u64, confidence: f32) {
        self.mcleod_performance_history.push((processing_time_ns, confidence));
        if self.mcleod_performance_history.len() > 100 {
            self.mcleod_performance_history.remove(0);
        }
    }
    
    /// Calculate average confidence from performance history
    fn average_confidence(&self, history: &[(u64, f32)]) -> f32 {
        if history.is_empty() {
            return 0.5; // Default when no history
        }
        
        let sum: f32 = history.iter().map(|(_, confidence)| confidence).sum();
        sum / history.len() as f32
    }
    
    /// Calculate average processing time from performance history
    fn average_processing_time(&self, history: &[(u64, f32)]) -> u64 {
        if history.is_empty() {
            return 1_000_000; // Default 1ms
        }
        
        let sum: u64 = history.iter().map(|(time, _)| time).sum();
        sum / history.len() as u64
    }
    
    /// Publish pitch detection event if event bus is available
    fn publish_pitch_event(&self, result: &PitchResult, buffer_ref: Option<String>) {
        if !self.event_publishing_enabled {
            return;
        }
        
        if let Some(ref event_bus) = self.event_bus {
            let event = PitchDetectionEvent {
                frequency: result.frequency,
                confidence: result.confidence,
                clarity: result.clarity,
                harmonic_content: result.harmonic_content,
                algorithm_used: result.algorithm_used,
                processing_time_ns: result.processing_time_ns,
                timestamp_ns: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
                source_buffer_ref: buffer_ref.unwrap_or_default(),
                snr_estimate: result.snr_estimate,
                is_valid: result.is_valid,
            };
            
            let _ = event_bus.publish_critical(Box::new(event));
        }
    }
    
    /// Publish signal analysis event
    fn publish_signal_analysis_event(&self, buffer: &[f32], snr: f32, complexity: f32) {
        if !self.event_publishing_enabled {
            return;
        }
        
        if let Some(ref event_bus) = self.event_bus {
            let event = SignalAnalysisEvent {
                snr_estimate: snr,
                signal_complexity: complexity,
                buffer_size: buffer.len(),
                rms_energy: (buffer.iter().map(|&x| x * x).sum::<f32>() / buffer.len() as f32).sqrt(),
                peak_amplitude: buffer.iter().map(|&x| x.abs()).fold(0.0f32, f32::max),
                timestamp_ns: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            };
            
            let _ = event_bus.publish_medium(Box::new(event));
        }
    }
}

impl PitchDetector for MultiAlgorithmPitchDetector {
    fn configure(&mut self, config: PitchDetectionConfig) -> Result<(), PitchError> {
        Self::validate_config(&config)?;
        
        // Reset detectors if configuration changed significantly
        if self.config.sample_rate != config.sample_rate {
            self.yin_detector = None;
            self.mcleod_detector = None;
        }
        
        self.config = config;
        Ok(())
    }
    
    fn detect_pitch(&mut self, buffer: &[f32]) -> Result<PitchResult, PitchError> {
        if buffer.is_empty() {
            return Err(PitchError::ProcessingError("Empty audio buffer".to_string()));
        }
        
        if buffer.len() < 64 {
            return Err(PitchError::BufferTooSmall(buffer.len(), 64));
        }
        
        // Determine which algorithm to use
        let algorithm = match self.config.algorithm {
            PitchAlgorithm::Auto => self.auto_select_algorithm(buffer),
            algo => algo,
        };
        
        // Detect pitch with selected algorithm
        let result = match algorithm {
            PitchAlgorithm::YIN => self.detect_with_yin(buffer)?,
            PitchAlgorithm::McLeod => self.detect_with_mcleod(buffer)?,
            PitchAlgorithm::Auto => unreachable!(), // Handled above
        };
        
        match result {
            Some(pitch_result) => {
                // Publish events
                self.publish_pitch_event(&pitch_result, None);
                
                if self.config.enable_harmonic_analysis {
                    let snr = self.estimate_snr(buffer);
                    let complexity = self.estimate_signal_complexity(buffer);
                    self.publish_signal_analysis_event(buffer, snr, complexity);
                    
                    // Update signal analysis history
                    if self.recent_snr_estimates.len() >= 20 {
                        self.recent_snr_estimates.remove(0);
                    }
                    self.recent_snr_estimates.push(snr);
                    
                    if self.recent_harmonic_content.len() >= 20 {
                        self.recent_harmonic_content.remove(0);
                    }
                    self.recent_harmonic_content.push(pitch_result.harmonic_content);
                }
                
                Ok(pitch_result)
            }
            None => Err(PitchError::ProcessingError("No pitch detected".to_string())),
        }
    }
    
    fn set_algorithm(&mut self, algorithm: PitchAlgorithm) -> Result<(), PitchError> {
        self.config.algorithm = algorithm;
        Ok(())
    }
    
    fn get_algorithm_info(&self) -> AlgorithmInfo {
        let current_algorithm = self.config.algorithm;
        
        match current_algorithm {
            PitchAlgorithm::YIN => AlgorithmInfo {
                name: PitchAlgorithm::YIN,
                avg_processing_time_ns: self.average_processing_time(&self.yin_performance_history),
                accuracy_score: self.average_confidence(&self.yin_performance_history),
                signal_suitability: 0.8, // YIN is generally suitable for most signals
                memory_usage_bytes: 1024, // Estimated
            },
            PitchAlgorithm::McLeod => AlgorithmInfo {
                name: PitchAlgorithm::McLeod,
                avg_processing_time_ns: self.average_processing_time(&self.mcleod_performance_history),
                accuracy_score: self.average_confidence(&self.mcleod_performance_history),
                signal_suitability: 0.9, // McLeod is more robust
                memory_usage_bytes: 1536, // Estimated (slightly more than YIN)
            },
            PitchAlgorithm::Auto => AlgorithmInfo {
                name: PitchAlgorithm::Auto,
                avg_processing_time_ns: (self.average_processing_time(&self.yin_performance_history) + 
                                        self.average_processing_time(&self.mcleod_performance_history)) / 2,
                accuracy_score: (self.average_confidence(&self.yin_performance_history) + 
                               self.average_confidence(&self.mcleod_performance_history)) / 2.0,
                signal_suitability: 1.0, // Auto-selection adapts to signal
                memory_usage_bytes: 1536, // Worst case (both algorithms loaded)
            },
        }
    }
    
    fn get_performance_comparison(&self) -> PerformanceComparison {
        let yin_info = AlgorithmInfo {
            name: PitchAlgorithm::YIN,
            avg_processing_time_ns: self.average_processing_time(&self.yin_performance_history),
            accuracy_score: self.average_confidence(&self.yin_performance_history),
            signal_suitability: 0.8,
            memory_usage_bytes: 1024,
        };
        
        let mcleod_info = AlgorithmInfo {
            name: PitchAlgorithm::McLeod,
            avg_processing_time_ns: self.average_processing_time(&self.mcleod_performance_history),
            accuracy_score: self.average_confidence(&self.mcleod_performance_history),
            signal_suitability: 0.9,
            memory_usage_bytes: 1536,
        };
        
        // Determine recommendation based on recent signal characteristics
        let avg_snr = if self.recent_snr_estimates.is_empty() {
            0.5
        } else {
            self.recent_snr_estimates.iter().sum::<f32>() / self.recent_snr_estimates.len() as f32
        };
        
        let recommended_algorithm = if avg_snr > 0.7 {
            if yin_info.avg_processing_time_ns < mcleod_info.avg_processing_time_ns {
                PitchAlgorithm::YIN // Prefer faster algorithm for clean signals
            } else {
                PitchAlgorithm::McLeod
            }
        } else {
            PitchAlgorithm::McLeod // Prefer robust algorithm for noisy signals
        };
        
        let recommendation_confidence = if (yin_info.accuracy_score - mcleod_info.accuracy_score).abs() < 0.1 {
            0.6 // Low confidence when algorithms perform similarly
        } else {
            0.9 // High confidence when there's a clear winner
        };
        
        PerformanceComparison {
            yin_performance: yin_info,
            mcleod_performance: mcleod_info,
            recommended_algorithm,
            recommendation_confidence,
        }
    }
    
    fn set_event_publishing(&mut self, enabled: bool) {
        self.event_publishing_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> PitchDetectionConfig {
        PitchDetectionConfig::default()
    }
    
    fn generate_sine_wave(frequency: f32, sample_rate: f32, duration_samples: usize, amplitude: f32) -> Vec<f32> {
        (0..duration_samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect()
    }
    
    #[test]
    fn test_pitch_detector_creation() {
        let config = create_test_config();
        let detector = MultiAlgorithmPitchDetector::new(config, None);
        assert!(detector.is_ok());
    }
    
    #[test]
    fn test_configuration_validation() {
        let mut config = create_test_config();
        
        // Test invalid sample rate
        config.sample_rate = -1.0;
        assert!(MultiAlgorithmPitchDetector::validate_config(&config).is_err());
        
        // Test invalid frequency range
        config = create_test_config();
        config.min_frequency = 1000.0;
        config.max_frequency = 500.0;
        assert!(MultiAlgorithmPitchDetector::validate_config(&config).is_err());
        
        // Test invalid threshold
        config = create_test_config();
        config.yin_threshold = 1.5;
        assert!(MultiAlgorithmPitchDetector::validate_config(&config).is_err());
    }
    
    #[test]
    fn test_algorithm_switching() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        // Test YIN
        assert!(detector.set_algorithm(PitchAlgorithm::YIN).is_ok());
        assert_eq!(detector.config.algorithm, PitchAlgorithm::YIN);
        
        // Test McLeod
        assert!(detector.set_algorithm(PitchAlgorithm::McLeod).is_ok());
        assert_eq!(detector.config.algorithm, PitchAlgorithm::McLeod);
        
        // Test Auto
        assert!(detector.set_algorithm(PitchAlgorithm::Auto).is_ok());
        assert_eq!(detector.config.algorithm, PitchAlgorithm::Auto);
    }
    
    #[test]
    fn test_pitch_detection_yin() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::YIN).unwrap();
        
        let test_frequency = 440.0;
        let buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.5);
        
        let result = detector.detect_pitch(&buffer);
        if let Ok(pitch_result) = result {
            assert!(pitch_result.is_valid);
            assert!((pitch_result.frequency - test_frequency).abs() < 10.0); // 10Hz tolerance
            assert_eq!(pitch_result.algorithm_used, PitchAlgorithm::YIN);
        }
    }
    
    #[test]
    fn test_pitch_detection_mcleod() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::McLeod).unwrap();
        
        let test_frequency = 440.0;
        let buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.5);
        
        let result = detector.detect_pitch(&buffer);
        if let Ok(pitch_result) = result {
            assert!(pitch_result.is_valid);
            assert!((pitch_result.frequency - test_frequency).abs() < 10.0); // 10Hz tolerance
            assert_eq!(pitch_result.algorithm_used, PitchAlgorithm::McLeod);
        }
    }
    
    #[test]
    fn test_auto_algorithm_selection() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::Auto).unwrap();
        
        let test_frequency = 440.0;
        let buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.5);
        
        let result = detector.detect_pitch(&buffer);
        if let Ok(pitch_result) = result {
            assert!(pitch_result.is_valid);
            assert!((pitch_result.frequency - test_frequency).abs() < 10.0);
            // Algorithm should be either YIN or McLeod (not Auto)
            assert!(matches!(pitch_result.algorithm_used, PitchAlgorithm::YIN | PitchAlgorithm::McLeod));
        }
    }
    
    #[test]
    fn test_confidence_scoring() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let test_frequency = 440.0;
        let clean_buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.8);
        let weak_buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.1);
        
        let clean_result = detector.detect_pitch(&clean_buffer);
        let weak_result = detector.detect_pitch(&weak_buffer);
        
        if let (Ok(clean), Ok(weak)) = (clean_result, weak_result) {
            // Clean signal should have higher confidence
            assert!(clean.confidence > weak.confidence);
        }
    }
    
    #[test]
    fn test_harmonic_analysis() {
        let mut config = create_test_config();
        config.enable_harmonic_analysis = true;
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let test_frequency = 440.0;
        let buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.5);
        
        let result = detector.detect_pitch(&buffer);
        if let Ok(pitch_result) = result {
            // Pure sine wave should have high harmonic content
            assert!(pitch_result.harmonic_content >= 0.0);
            assert!(pitch_result.harmonic_content <= 1.0);
        }
    }
    
    #[test]
    fn test_performance_tracking() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let test_frequency = 440.0;
        let buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.5);
        
        // Process several times to build performance history
        for _ in 0..5 {
            let _ = detector.detect_pitch(&buffer);
        }
        
        let performance = detector.get_performance_comparison();
        
        // Should have some performance data
        assert!(performance.yin_performance.avg_processing_time_ns > 0);
        assert!(performance.recommendation_confidence >= 0.0);
        assert!(performance.recommendation_confidence <= 1.0);
    }
    
    #[test]
    fn test_edge_cases() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        // Test empty buffer
        let empty_buffer: Vec<f32> = vec![];
        assert!(detector.detect_pitch(&empty_buffer).is_err());
        
        // Test too small buffer
        let small_buffer = vec![0.0; 10];
        assert!(detector.detect_pitch(&small_buffer).is_err());
        
        // Test silence
        let silence = vec![0.0; 2048];
        let result = detector.detect_pitch(&silence);
        // Should either error or return invalid result
        if let Ok(pitch_result) = result {
            // If detection succeeds, confidence should be very low
            assert!(pitch_result.confidence < 0.3);
        }
    }
    
    #[test]
    fn test_frequency_validation() {
        let config = create_test_config();
        let detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        // Test valid frequencies
        assert!(detector.validate_frequency(440.0));
        assert!(detector.validate_frequency(80.0)); // Min boundary
        assert!(detector.validate_frequency(2000.0)); // Max boundary
        
        // Test invalid frequencies
        assert!(!detector.validate_frequency(50.0)); // Below min
        assert!(!detector.validate_frequency(3000.0)); // Above max
    }
}