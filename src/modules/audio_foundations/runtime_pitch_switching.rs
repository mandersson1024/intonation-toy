// Runtime Algorithm Switching - STORY-015
// Provides seamless algorithm switching without audio interruption

use std::sync::{Arc, RwLock};
use std::time::Instant;
use crate::modules::audio_foundations::{
    multi_algorithm_pitch_detector::{
        MultiAlgorithmPitchDetector, PitchDetector, PitchAlgorithm, 
        PitchDetectionConfig, PitchResult, PitchError, PerformanceComparison
    },
    audio_events::{AlgorithmSwitchEvent, AlgorithmSwitchReason, get_timestamp_ns}
};
use crate::modules::application_core::event_bus::{EventBus, Event};

/// Runtime algorithm switching manager
pub struct RuntimePitchSwitcher {
    /// Primary detector instance
    detector: Arc<RwLock<MultiAlgorithmPitchDetector>>,
    /// Event bus for notifications
    event_bus: Option<Arc<dyn EventBus>>,
    /// Switch history for tracking performance
    switch_history: Vec<AlgorithmSwitchRecord>,
    /// Configuration for automatic switching behavior
    auto_switch_config: AutoSwitchConfig,
    /// Performance tracking for switch decisions
    performance_tracker: PerformanceTracker,
}

/// Configuration for automatic algorithm switching
#[derive(Debug, Clone)]
pub struct AutoSwitchConfig {
    /// Enable automatic switching based on signal characteristics
    pub enable_auto_switch: bool,
    /// Minimum time between switches (to prevent oscillation)
    pub min_switch_interval_ms: u64,
    /// Performance difference threshold for switching
    pub performance_threshold: f32,
    /// Signal quality threshold for switching decisions
    pub signal_quality_threshold: f32,
    /// Number of consecutive poor results before switching
    pub poor_results_threshold: u32,
}

impl Default for AutoSwitchConfig {
    fn default() -> Self {
        Self {
            enable_auto_switch: true,
            min_switch_interval_ms: 2000, // 2 seconds minimum between switches
            performance_threshold: 0.1, // 10% performance difference
            signal_quality_threshold: 0.5, // 50% signal quality
            poor_results_threshold: 5, // 5 consecutive poor results
        }
    }
}

/// Track algorithm switch events
#[derive(Debug, Clone)]
struct AlgorithmSwitchRecord {
    timestamp: Instant,
    old_algorithm: PitchAlgorithm,
    new_algorithm: PitchAlgorithm,
    reason: AlgorithmSwitchReason,
    performance_before: f32,
    performance_after: Option<f32>, // Set after some processing time
}

/// Performance tracking for switch decisions
#[derive(Debug, Default)]
struct PerformanceTracker {
    consecutive_poor_results: u32,
    last_switch_time: Option<Instant>,
    recent_confidences: Vec<f32>,
    recent_processing_times: Vec<u64>,
}

impl PerformanceTracker {
    fn update(&mut self, confidence: f32, processing_time_ns: u64, quality_threshold: f32) {
        // Track recent performance
        self.recent_confidences.push(confidence);
        self.recent_processing_times.push(processing_time_ns);
        
        // Keep only recent history (last 20 results)
        if self.recent_confidences.len() > 20 {
            self.recent_confidences.remove(0);
            self.recent_processing_times.remove(0);
        }
        
        // Update consecutive poor results counter
        if confidence < quality_threshold {
            self.consecutive_poor_results += 1;
        } else {
            self.consecutive_poor_results = 0;
        }
    }
    
    fn average_confidence(&self) -> f32 {
        if self.recent_confidences.is_empty() {
            return 0.5;
        }
        self.recent_confidences.iter().sum::<f32>() / self.recent_confidences.len() as f32
    }
    
    fn average_processing_time(&self) -> u64 {
        if self.recent_processing_times.is_empty() {
            return 1_000_000; // 1ms default
        }
        self.recent_processing_times.iter().sum::<u64>() / self.recent_processing_times.len() as u64
    }
    
    fn should_consider_switch(&self, config: &AutoSwitchConfig) -> bool {
        // Check if enough time has passed since last switch
        if let Some(last_switch) = self.last_switch_time {
            if last_switch.elapsed().as_millis() < config.min_switch_interval_ms as u128 {
                return false;
            }
        }
        
        // Check if we have consecutive poor results
        self.consecutive_poor_results >= config.poor_results_threshold
    }
    
    fn record_switch(&mut self) {
        self.last_switch_time = Some(Instant::now());
        self.consecutive_poor_results = 0; // Reset counter after switch
    }
}

impl RuntimePitchSwitcher {
    /// Create new runtime pitch switcher
    pub fn new(
        detector: MultiAlgorithmPitchDetector, 
        event_bus: Option<Arc<dyn EventBus>>
    ) -> Self {
        Self {
            detector: Arc::new(RwLock::new(detector)),
            event_bus,
            switch_history: Vec::new(),
            auto_switch_config: AutoSwitchConfig::default(),
            performance_tracker: PerformanceTracker::default(),
        }
    }
    
    /// Configure automatic switching behavior
    pub fn configure_auto_switch(&mut self, config: AutoSwitchConfig) {
        self.auto_switch_config = config;
    }
    
    /// Manually switch to a specific algorithm
    pub fn switch_algorithm(&mut self, new_algorithm: PitchAlgorithm) -> Result<(), PitchError> {
        let mut detector = self.detector.write().unwrap();
        let old_algorithm = detector.get_algorithm_info().name;
        
        if old_algorithm == new_algorithm {
            return Ok(());
        }
        
        // Record switch event
        let switch_record = AlgorithmSwitchRecord {
            timestamp: Instant::now(),
            old_algorithm,
            new_algorithm,
            reason: AlgorithmSwitchReason::UserSelection,
            performance_before: self.performance_tracker.average_confidence(),
            performance_after: None,
        };
        
        // Perform the switch
        detector.set_algorithm(new_algorithm)?;
        
        // Update tracking
        self.switch_history.push(switch_record);
        self.performance_tracker.record_switch();
        
        // Publish event
        self.publish_switch_event(old_algorithm, new_algorithm, AlgorithmSwitchReason::UserSelection);
        
        Ok(())
    }
    
    /// Process audio with automatic switching consideration
    pub fn detect_pitch_with_auto_switch(&mut self, buffer: &[f32]) -> Result<PitchResult, PitchError> {
        // Get current detection result
        let result = {
            let mut detector = self.detector.write().unwrap();
            detector.detect_pitch(buffer)?
        };
        
        // Update performance tracking
        self.performance_tracker.update(
            result.confidence,
            result.processing_time_ns,
            self.auto_switch_config.signal_quality_threshold,
        );
        
        // Consider automatic switching if enabled
        if self.auto_switch_config.enable_auto_switch {
            self.consider_automatic_switch(&result)?;
        }
        
        Ok(result)
    }
    
    /// Consider automatic algorithm switching based on current performance
    fn consider_automatic_switch(&mut self, current_result: &PitchResult) -> Result<(), PitchError> {
        // Check if we should consider switching
        if !self.performance_tracker.should_consider_switch(&self.auto_switch_config) {
            return Ok(());
        }
        
        // Get performance comparison
        let comparison = {
            let detector = self.detector.read().unwrap();
            detector.get_performance_comparison()
        };
        
        let current_algorithm = current_result.algorithm_used;
        let recommended_algorithm = comparison.recommended_algorithm;
        
        // Only switch if recommendation is different and confident
        if current_algorithm != recommended_algorithm && 
           comparison.recommendation_confidence > 0.7 {
            
            // Check if the recommended algorithm would significantly improve performance
            let current_perf = self.performance_tracker.average_confidence();
            let potential_improvement = self.estimate_performance_improvement(&comparison, current_algorithm);
            
            if potential_improvement > self.auto_switch_config.performance_threshold {
                self.perform_automatic_switch(current_algorithm, recommended_algorithm)?;
            }
        }
        
        Ok(())
    }
    
    /// Estimate potential performance improvement from switching algorithms
    fn estimate_performance_improvement(&self, comparison: &PerformanceComparison, current: PitchAlgorithm) -> f32 {
        let current_score = match current {
            PitchAlgorithm::YIN => comparison.yin_performance.accuracy_score,
            PitchAlgorithm::McLeod => comparison.mcleod_performance.accuracy_score,
            PitchAlgorithm::Auto => 0.5, // Neutral baseline
        };
        
        let recommended_score = match comparison.recommended_algorithm {
            PitchAlgorithm::YIN => comparison.yin_performance.accuracy_score,
            PitchAlgorithm::McLeod => comparison.mcleod_performance.accuracy_score,
            PitchAlgorithm::Auto => 0.5, // Neutral baseline
        };
        
        (recommended_score - current_score).max(0.0)
    }
    
    /// Perform automatic algorithm switch
    fn perform_automatic_switch(
        &mut self, 
        old_algorithm: PitchAlgorithm, 
        new_algorithm: PitchAlgorithm
    ) -> Result<(), PitchError> {
        
        // Record switch event
        let switch_record = AlgorithmSwitchRecord {
            timestamp: Instant::now(),
            old_algorithm,
            new_algorithm,
            reason: AlgorithmSwitchReason::AutomaticOptimization,
            performance_before: self.performance_tracker.average_confidence(),
            performance_after: None,
        };
        
        // Perform the switch
        {
            let mut detector = self.detector.write().unwrap();
            detector.set_algorithm(new_algorithm)?;
        }
        
        // Update tracking
        self.switch_history.push(switch_record);
        self.performance_tracker.record_switch();
        
        // Publish event
        self.publish_switch_event(old_algorithm, new_algorithm, AlgorithmSwitchReason::AutomaticOptimization);
        
        Ok(())
    }
    
    /// Publish algorithm switch event
    fn publish_switch_event(&self, old: PitchAlgorithm, new: PitchAlgorithm, reason: AlgorithmSwitchReason) {
        if let Some(ref event_bus) = self.event_bus {
            let event = AlgorithmSwitchEvent {
                old_algorithm: old,
                new_algorithm: new,
                reason,
                timestamp_ns: get_timestamp_ns(),
            };
            
            let _ = event_bus.publish_normal(Box::new(event));
        }
    }
    
    /// Get current algorithm
    pub fn current_algorithm(&self) -> PitchAlgorithm {
        let detector = self.detector.read().unwrap();
        detector.get_algorithm_info().name
    }
    
    /// Get performance comparison for UI display
    pub fn get_performance_comparison(&self) -> PerformanceComparison {
        let detector = self.detector.read().unwrap();
        detector.get_performance_comparison()
    }
    
    /// Get switch history for analysis
    pub fn get_switch_history(&self) -> &[AlgorithmSwitchRecord] {
        &self.switch_history
    }
    
    /// Configure pitch detection parameters
    pub fn configure(&mut self, config: PitchDetectionConfig) -> Result<(), PitchError> {
        let mut detector = self.detector.write().unwrap();
        detector.configure(config)
    }
    
    /// Enable/disable event publishing
    pub fn set_event_publishing(&mut self, enabled: bool) {
        let mut detector = self.detector.write().unwrap();
        detector.set_event_publishing(enabled);
    }
    
    /// Get current performance statistics
    pub fn get_performance_stats(&self) -> RuntimePerformanceStats {
        RuntimePerformanceStats {
            total_switches: self.switch_history.len(),
            automatic_switches: self.switch_history.iter()
                .filter(|s| matches!(s.reason, AlgorithmSwitchReason::AutomaticOptimization))
                .count(),
            manual_switches: self.switch_history.iter()
                .filter(|s| matches!(s.reason, AlgorithmSwitchReason::UserSelection))
                .count(),
            average_confidence: self.performance_tracker.average_confidence(),
            average_processing_time_ns: self.performance_tracker.average_processing_time(),
            consecutive_poor_results: self.performance_tracker.consecutive_poor_results,
            auto_switch_enabled: self.auto_switch_config.enable_auto_switch,
        }
    }
}

/// Runtime performance statistics
#[derive(Debug, Clone)]
pub struct RuntimePerformanceStats {
    pub total_switches: usize,
    pub automatic_switches: usize,
    pub manual_switches: usize,
    pub average_confidence: f32,
    pub average_processing_time_ns: u64,
    pub consecutive_poor_results: u32,
    pub auto_switch_enabled: bool,
}

// Thread-safe interface for concurrent access
impl Clone for RuntimePitchSwitcher {
    fn clone(&self) -> Self {
        Self {
            detector: Arc::clone(&self.detector),
            event_bus: self.event_bus.clone(),
            switch_history: self.switch_history.clone(),
            auto_switch_config: self.auto_switch_config.clone(),
            performance_tracker: PerformanceTracker::default(), // Reset for clone
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::audio_foundations::multi_algorithm_pitch_detector::PitchDetectionConfig;
    
    fn create_test_switcher() -> RuntimePitchSwitcher {
        let config = PitchDetectionConfig::default();
        let detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        RuntimePitchSwitcher::new(detector, None)
    }
    
    fn generate_sine_wave(frequency: f32, sample_rate: f32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                0.5 * (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect()
    }
    
    #[test]
    fn test_switcher_creation() {
        let switcher = create_test_switcher();
        assert_eq!(switcher.current_algorithm(), PitchAlgorithm::YIN); // Default
    }
    
    #[test]
    fn test_manual_algorithm_switch() {
        let mut switcher = create_test_switcher();
        
        // Switch to McLeod
        assert!(switcher.switch_algorithm(PitchAlgorithm::McLeod).is_ok());
        assert_eq!(switcher.current_algorithm(), PitchAlgorithm::McLeod);
        
        // Switch to Auto
        assert!(switcher.switch_algorithm(PitchAlgorithm::Auto).is_ok());
        assert_eq!(switcher.current_algorithm(), PitchAlgorithm::Auto);
        
        // Check switch history
        let history = switcher.get_switch_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].new_algorithm, PitchAlgorithm::McLeod);
        assert_eq!(history[1].new_algorithm, PitchAlgorithm::Auto);
    }
    
    #[test]
    fn test_auto_switch_configuration() {
        let mut switcher = create_test_switcher();
        
        let mut config = AutoSwitchConfig::default();
        config.enable_auto_switch = false;
        config.min_switch_interval_ms = 5000;
        
        switcher.configure_auto_switch(config.clone());
        assert_eq!(switcher.auto_switch_config.enable_auto_switch, false);
        assert_eq!(switcher.auto_switch_config.min_switch_interval_ms, 5000);
    }
    
    #[test]
    fn test_pitch_detection_with_auto_switch() {
        let mut switcher = create_test_switcher();
        
        // Enable auto switch
        let mut config = AutoSwitchConfig::default();
        config.enable_auto_switch = true;
        config.poor_results_threshold = 2; // Low threshold for testing
        switcher.configure_auto_switch(config);
        
        let buffer = generate_sine_wave(440.0, 44100.0, 2048);
        
        // Process several times
        for _ in 0..5 {
            let result = switcher.detect_pitch_with_auto_switch(&buffer);
            assert!(result.is_ok());
        }
        
        // Should have some performance data
        let stats = switcher.get_performance_stats();
        assert!(stats.average_confidence >= 0.0);
        assert!(stats.average_processing_time_ns > 0);
    }
    
    #[test]
    fn test_performance_tracking() {
        let mut switcher = create_test_switcher();
        let buffer = generate_sine_wave(440.0, 44100.0, 2048);
        
        // Process some audio to build performance history
        for _ in 0..10 {
            let _ = switcher.detect_pitch_with_auto_switch(&buffer);
        }
        
        let stats = switcher.get_performance_stats();
        assert!(stats.average_confidence > 0.0);
        assert!(stats.average_processing_time_ns > 0);
    }
    
    #[test]
    fn test_redundant_switch_ignored() {
        let mut switcher = create_test_switcher();
        
        // Switch to same algorithm should be ignored
        assert!(switcher.switch_algorithm(PitchAlgorithm::YIN).is_ok());
        assert_eq!(switcher.get_switch_history().len(), 0); // No switch recorded
    }
    
    #[test]
    fn test_switch_interval_enforcement() {
        let mut switcher = create_test_switcher();
        
        // Configure very long minimum interval
        let mut config = AutoSwitchConfig::default();
        config.min_switch_interval_ms = 10_000; // 10 seconds
        switcher.configure_auto_switch(config);
        
        // Manual switch should still work
        assert!(switcher.switch_algorithm(PitchAlgorithm::McLeod).is_ok());
        
        // Automatic switch consideration should be throttled
        // (This is tested indirectly through the should_consider_switch logic)
        assert!(switcher.performance_tracker.should_consider_switch(&switcher.auto_switch_config) == false);
    }
    
    #[test]
    fn test_performance_comparison_retrieval() {
        let switcher = create_test_switcher();
        let comparison = switcher.get_performance_comparison();
        
        // Should have basic structure
        assert!(matches!(comparison.yin_performance.name, PitchAlgorithm::YIN));
        assert!(matches!(comparison.mcleod_performance.name, PitchAlgorithm::McLeod));
        assert!(comparison.recommendation_confidence >= 0.0);
        assert!(comparison.recommendation_confidence <= 1.0);
    }
    
    #[test]
    fn test_event_publishing_configuration() {
        let mut switcher = create_test_switcher();
        
        // Should not panic when configuring event publishing
        switcher.set_event_publishing(true);
        switcher.set_event_publishing(false);
    }
    
    #[test]
    fn test_clone_functionality() {
        let switcher = create_test_switcher();
        let cloned = switcher.clone();
        
        // Should be able to use both instances
        assert_eq!(switcher.current_algorithm(), cloned.current_algorithm());
    }
}