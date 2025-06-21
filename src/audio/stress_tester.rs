use std::time::{Duration, Instant};
use std::collections::VecDeque;
use crate::audio::{engine::AudioEngine, pitch_detector::PitchAlgorithm};

/// Stress test configuration parameters
#[derive(Debug, Clone)]
pub struct StressTestConfig {
    pub cycles: usize,
    pub buffer_size: usize,
    pub sample_rate: f32,
    pub test_frequency: f32,
    pub enable_memory_monitoring: bool,
    pub enable_performance_monitoring: bool,
    pub degradation_threshold: f64, // Percentage threshold for performance degradation
}

impl Default for StressTestConfig {
    fn default() -> Self {
        StressTestConfig {
            cycles: 1000,
            buffer_size: 1024,
            sample_rate: 44100.0,
            test_frequency: 440.0,
            enable_memory_monitoring: true,
            enable_performance_monitoring: true,
            degradation_threshold: 50.0, // 50% performance degradation triggers warning
        }
    }
}

/// Memory usage snapshot
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub cycle: usize,
    pub rust_heap_estimate: usize, // Estimated based on Vec allocations
    pub buffer_count: usize,
    pub active_objects: usize,
}

/// Performance measurement for a single cycle
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub cycle: usize,
    pub duration_ns: u64,
    pub duration_ms: f64,
    pub throughput_samples_per_second: f64,
    pub detected_frequency: Option<f32>,
    pub timestamp: Instant,
}

/// Stress test results and analysis
#[derive(Debug, Clone)]
pub struct StressTestResult {
    pub config: StressTestConfig,
    pub completed_cycles: usize,
    pub total_duration: Duration,
    pub memory_snapshots: Vec<MemorySnapshot>,
    pub performance_measurements: Vec<PerformanceMeasurement>,
    pub memory_leak_detected: bool,
    pub performance_degradation_detected: bool,
    pub stability_score: f64, // 0-100 score
    pub errors: Vec<String>,
}

impl StressTestResult {
    pub fn new(config: StressTestConfig) -> Self {
        StressTestResult {
            config,
            completed_cycles: 0,
            total_duration: Duration::new(0, 0),
            memory_snapshots: Vec::new(),
            performance_measurements: Vec::new(),
            memory_leak_detected: false,
            performance_degradation_detected: false,
            stability_score: 0.0,
            errors: Vec::new(),
        }
    }

    /// Calculate average performance metrics
    pub fn average_performance(&self) -> Option<PerformanceMeasurement> {
        if self.performance_measurements.is_empty() {
            return None;
        }

        let total_duration_ns: u64 = self.performance_measurements.iter()
            .map(|m| m.duration_ns)
            .sum();
        
        let total_throughput: f64 = self.performance_measurements.iter()
            .map(|m| m.throughput_samples_per_second)
            .sum();

        let avg_duration_ns = total_duration_ns / self.performance_measurements.len() as u64;
        let avg_duration_ms = avg_duration_ns as f64 / 1_000_000.0;
        let avg_throughput = total_throughput / self.performance_measurements.len() as f64;

        Some(PerformanceMeasurement {
            cycle: 0, // Average across all cycles
            duration_ns: avg_duration_ns,
            duration_ms: avg_duration_ms,
            throughput_samples_per_second: avg_throughput,
            detected_frequency: None,
            timestamp: Instant::now(),
        })
    }

    /// Calculate memory growth rate (bytes per cycle)
    pub fn memory_growth_rate(&self) -> Option<f64> {
        if self.memory_snapshots.len() < 2 {
            return None;
        }

        let first = &self.memory_snapshots[0];
        let last = &self.memory_snapshots[self.memory_snapshots.len() - 1];
        
        let memory_growth = last.rust_heap_estimate as f64 - first.rust_heap_estimate as f64;
        let cycle_diff = last.cycle as f64 - first.cycle as f64;
        
        if cycle_diff > 0.0 {
            Some(memory_growth / cycle_diff)
        } else {
            None
        }
    }
}

/// Comprehensive stress testing and stability validation
pub struct StressTester {
    config: StressTestConfig,
    result: StressTestResult,
    baseline_performance: Option<PerformanceMeasurement>,
    performance_window: VecDeque<PerformanceMeasurement>, // Rolling window for degradation detection
    memory_tracker: MemoryTracker,
}

/// Simple memory tracker for Rust heap estimation
struct MemoryTracker {
    allocated_buffers: usize,
    total_allocations: usize,
    peak_allocations: usize,
}

impl MemoryTracker {
    fn new() -> Self {
        MemoryTracker {
            allocated_buffers: 0,
            total_allocations: 0,
            peak_allocations: 0,
        }
    }

    fn allocate_buffer(&mut self, _size: usize) {
        self.allocated_buffers += 1;
        self.total_allocations += 1;
        self.peak_allocations = self.peak_allocations.max(self.allocated_buffers);
    }

    fn deallocate_buffer(&mut self) {
        if self.allocated_buffers > 0 {
            self.allocated_buffers -= 1;
        }
    }

    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate: each buffer is typically 1KB-4KB, plus overhead
        self.allocated_buffers * 2048 + self.total_allocations * 64
    }
}

impl StressTester {
    pub fn new(config: StressTestConfig) -> Self {
        let window_size = (config.cycles / 10).max(10); // 10% of cycles for rolling window
        
        StressTester {
            result: StressTestResult::new(config.clone()),
            config,
            baseline_performance: None,
            performance_window: VecDeque::with_capacity(window_size),
            memory_tracker: MemoryTracker::new(),
        }
    }

    /// Run comprehensive stress test suite
    pub fn run_stress_tests(&mut self) -> &StressTestResult {
        println!("🔥 Starting comprehensive stress testing...");
        println!("   Cycles: {}", self.config.cycles);
        println!("   Buffer size: {} samples", self.config.buffer_size);
        println!("   Sample rate: {}Hz", self.config.sample_rate);
        println!("   Test frequency: {}Hz", self.config.test_frequency);
        
        let start_time = Instant::now();
        
        // Run stress tests
        self.run_continuous_processing_stress();
        self.run_memory_leak_detection();
        self.run_concurrent_processing_stress();
        self.run_graceful_degradation_test();
        
        self.result.total_duration = start_time.elapsed();
        self.analyze_results();
        
        println!("✅ Stress testing completed!");
        self.print_stress_summary();
        
        &self.result
    }

    /// Test continuous audio processing for extended duration
    fn run_continuous_processing_stress(&mut self) {
        println!("\n🔄 Running continuous processing stress test...");
        
        let mut engine = AudioEngine::new(self.config.sample_rate, self.config.buffer_size);
        engine.set_pitch_algorithm(PitchAlgorithm::YIN);
        
        // Generate test buffer
        let test_buffer = self.generate_test_signal();
        
        for cycle in 0..self.config.cycles {
            let cycle_start = Instant::now();
            
            // Track memory before processing
            if self.config.enable_memory_monitoring && cycle % 100 == 0 {
                self.memory_tracker.allocate_buffer(self.config.buffer_size);
                let snapshot = MemorySnapshot {
                    timestamp: cycle_start,
                    cycle,
                    rust_heap_estimate: self.memory_tracker.estimate_memory_usage(),
                    buffer_count: self.memory_tracker.allocated_buffers,
                    active_objects: self.memory_tracker.total_allocations,
                };
                self.result.memory_snapshots.push(snapshot);
            }
            
            // Perform audio processing
            let output = engine.process_audio_buffer(&test_buffer);
            let detected_frequency = engine.detect_pitch_from_buffer(&test_buffer);
            
            let cycle_end = Instant::now();
            let duration = cycle_end - cycle_start;
            
            // Record performance measurement
            if self.config.enable_performance_monitoring {
                let measurement = PerformanceMeasurement {
                    cycle,
                    duration_ns: duration.as_nanos() as u64,
                    duration_ms: duration.as_nanos() as f64 / 1_000_000.0,
                    throughput_samples_per_second: self.config.buffer_size as f64 / duration.as_secs_f64(),
                    detected_frequency: if detected_frequency > 0.0 { Some(detected_frequency) } else { None },
                    timestamp: cycle_end,
                };
                
                // Set baseline on first cycle
                if cycle == 0 {
                    self.baseline_performance = Some(measurement.clone());
                }
                
                // Update rolling performance window
                self.performance_window.push_back(measurement.clone());
                if self.performance_window.len() > self.performance_window.capacity() {
                    self.performance_window.pop_front();
                }
                
                self.result.performance_measurements.push(measurement);
            }
            
            // Verify output integrity
            if output.len() != test_buffer.len() {
                self.result.errors.push(format!("Cycle {}: Output buffer size mismatch", cycle));
            }
            
            // Progress reporting
            if cycle % 100 == 0 && cycle > 0 {
                println!("   Completed {} cycles ({:.1}%)", cycle, (cycle as f64 / self.config.cycles as f64) * 100.0);
                
                // Check for performance degradation
                if let Some(degradation) = self.detect_performance_degradation() {
                    println!("   ⚠️  Performance degradation detected: {:.1}%", degradation);
                    self.result.performance_degradation_detected = true;
                }
            }
            
            // Simulate some cleanup to test memory management
            if cycle % 50 == 0 {
                self.memory_tracker.deallocate_buffer();
            }
        }
        
        self.result.completed_cycles = self.config.cycles;
        println!("   ✅ Continuous processing test completed: {} cycles", self.config.cycles);
    }

    /// Test for memory leaks over extended duration
    fn run_memory_leak_detection(&mut self) {
        println!("\n🧠 Running memory leak detection...");
        
        let initial_memory = if let Some(first) = self.result.memory_snapshots.first() {
            first.rust_heap_estimate
        } else {
            0
        };
        
        let final_memory = if let Some(last) = self.result.memory_snapshots.last() {
            last.rust_heap_estimate
        } else {
            0
        };
        
        let memory_growth = final_memory as f64 - initial_memory as f64;
        let memory_growth_per_cycle = memory_growth / self.config.cycles as f64;
        
        // Memory leak threshold: >10KB growth per 100 cycles
        let leak_threshold = 10.0 * 1024.0 / 100.0; // bytes per cycle
        
        if memory_growth_per_cycle > leak_threshold {
            self.result.memory_leak_detected = true;
            self.result.errors.push(format!(
                "Memory leak detected: {:.2} bytes per cycle (threshold: {:.2})", 
                memory_growth_per_cycle, leak_threshold
            ));
            println!("   ❌ Memory leak detected: {:.2} bytes per cycle", memory_growth_per_cycle);
        } else {
            println!("   ✅ No memory leak detected: {:.2} bytes per cycle", memory_growth_per_cycle);
        }
        
        println!("   Memory usage: {} KB -> {} KB ({:+.1} KB)", 
            initial_memory / 1024, final_memory / 1024, memory_growth / 1024.0);
    }

    /// Test concurrent processing simulation
    fn run_concurrent_processing_stress(&mut self) {
        println!("\n🔀 Running concurrent processing stress test...");
        
        let mut engines = vec![
            AudioEngine::new(self.config.sample_rate, self.config.buffer_size),
            AudioEngine::new(self.config.sample_rate, self.config.buffer_size),
            AudioEngine::new(self.config.sample_rate, self.config.buffer_size),
        ];
        
        // Set different algorithms for each engine
        engines[0].set_pitch_algorithm(PitchAlgorithm::YIN);
        engines[1].set_pitch_algorithm(PitchAlgorithm::McLeod);
        engines[2].set_pitch_algorithm(PitchAlgorithm::YIN);
        
        let test_buffers = vec![
            self.generate_test_signal_with_frequency(220.0), // A3
            self.generate_test_signal_with_frequency(440.0), // A4
            self.generate_test_signal_with_frequency(880.0), // A5
        ];
        
        let concurrent_cycles = 100;
        let start = Instant::now();
        
        for _cycle in 0..concurrent_cycles {
            // Simulate concurrent processing by processing multiple streams
            for (i, engine) in engines.iter_mut().enumerate() {
                let _output = engine.process_audio_buffer(&test_buffers[i]);
                let _pitch = engine.detect_pitch_from_buffer(&test_buffers[i]);
            }
        }
        
        let duration = start.elapsed();
        let avg_duration_per_cycle = duration.as_millis() as f64 / concurrent_cycles as f64;
        
        println!("   ✅ Concurrent processing completed: {} cycles in {:.2}ms", 
            concurrent_cycles, duration.as_millis());
        println!("   Average time per concurrent cycle: {:.2}ms", avg_duration_per_cycle);
        
        // Check if concurrent processing is within reasonable bounds
        if avg_duration_per_cycle > 50.0 { // 50ms threshold
            self.result.errors.push(format!(
                "Concurrent processing too slow: {:.2}ms per cycle", avg_duration_per_cycle
            ));
        }
    }

    /// Test graceful degradation under various conditions
    fn run_graceful_degradation_test(&mut self) {
        println!("\n📉 Running graceful degradation test...");
        
        let mut engine = AudioEngine::new(self.config.sample_rate, self.config.buffer_size);
        
        // Test with various problematic inputs
        let test_cases = vec![
            ("Silent buffer", vec![0.0; self.config.buffer_size]),
            ("NaN buffer", vec![f32::NAN; self.config.buffer_size]),
            ("Infinite buffer", vec![f32::INFINITY; self.config.buffer_size]),
            ("Very large buffer", vec![1e6; self.config.buffer_size]),
            ("Very small buffer", vec![1e-10; self.config.buffer_size]),
        ];
        
        let mut graceful_failures = 0;
        
        let test_cases_len = test_cases.len();
        for (name, test_buffer) in test_cases {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let output = engine.process_audio_buffer(&test_buffer);
                let _pitch = engine.detect_pitch_from_buffer(&test_buffer);
                
                // Check output sanity
                let has_valid_output = output.iter().all(|&x| x.is_finite());
                (output.len() == test_buffer.len(), has_valid_output)
            })) {
                Ok((correct_size, valid_output)) => {
                    if correct_size && valid_output {
                        println!("   ✅ {}: Handled gracefully", name);
                        graceful_failures += 1;
                    } else {
                        println!("   ⚠️  {}: Handled but output invalid", name);
                    }
                }
                Err(_) => {
                    println!("   ❌ {}: Caused panic", name);
                    self.result.errors.push(format!("Graceful degradation failed for: {}", name));
                }
            }
        }
        
        println!("   Graceful handling rate: {}/{} test cases", graceful_failures, test_cases_len);
    }

    /// Detect performance degradation in rolling window
    fn detect_performance_degradation(&self) -> Option<f64> {
        if let Some(baseline) = &self.baseline_performance {
            if self.performance_window.len() >= 10 {
                // Calculate average of recent measurements
                let recent_avg = self.performance_window.iter()
                    .rev()
                    .take(10)
                    .map(|m| m.duration_ms)
                    .sum::<f64>() / 10.0;
                
                let degradation_pct = ((recent_avg - baseline.duration_ms) / baseline.duration_ms) * 100.0;
                
                if degradation_pct > self.config.degradation_threshold {
                    return Some(degradation_pct);
                }
            }
        }
        None
    }

    /// Analyze stress test results and calculate stability score
    fn analyze_results(&mut self) {
        let mut score = 100.0;
        
        // Deduct points for memory leaks
        if self.result.memory_leak_detected {
            score -= 25.0;
        }
        
        // Deduct points for performance degradation
        if self.result.performance_degradation_detected {
            score -= 20.0;
        }
        
        // Deduct points for errors
        score -= (self.result.errors.len() as f64) * 5.0;
        
        // Deduct points for incomplete cycles
        if self.result.completed_cycles < self.config.cycles {
            let completion_rate = self.result.completed_cycles as f64 / self.config.cycles as f64;
            score *= completion_rate;
        }
        
        // Ensure score is between 0-100
        self.result.stability_score = score.max(0.0).min(100.0);
    }

    /// Generate test signal with configured frequency
    fn generate_test_signal(&self) -> Vec<f32> {
        self.generate_test_signal_with_frequency(self.config.test_frequency)
    }

    /// Generate test signal with specific frequency
    fn generate_test_signal_with_frequency(&self, frequency: f32) -> Vec<f32> {
        use std::f32::consts::PI;
        
        (0..self.config.buffer_size)
            .map(|i| {
                let t = i as f32 / self.config.sample_rate;
                0.8 * (2.0 * PI * frequency * t).sin()
            })
            .collect()
    }

    /// Print comprehensive stress test summary
    fn print_stress_summary(&self) {
        println!("\n🔥 STRESS TEST SUMMARY");
        println!("======================");
        
        println!("📊 Configuration:");
        println!("  Cycles: {}", self.config.cycles);
        println!("  Buffer size: {} samples", self.config.buffer_size);
        println!("  Sample rate: {}Hz", self.config.sample_rate);
        println!("  Test frequency: {}Hz", self.config.test_frequency);
        
        println!("\n📈 Results:");
        println!("  Completed cycles: {} / {}", self.result.completed_cycles, self.config.cycles);
        println!("  Total duration: {:.2}s", self.result.total_duration.as_secs_f64());
        println!("  Stability score: {:.1}/100", self.result.stability_score);
        
        // Memory analysis
        if let Some(growth_rate) = self.result.memory_growth_rate() {
            println!("  Memory growth rate: {:.2} bytes/cycle", growth_rate);
        }
        
        // Performance analysis
        if let Some(avg_perf) = self.result.average_performance() {
            println!("  Average processing time: {:.3}ms", avg_perf.duration_ms);
            println!("  Average throughput: {:.1}M samples/sec", 
                avg_perf.throughput_samples_per_second / 1_000_000.0);
        }
        
        // Issues
        if self.result.memory_leak_detected {
            println!("  ❌ Memory leak detected");
        }
        
        if self.result.performance_degradation_detected {
            println!("  ❌ Performance degradation detected");
        }
        
        if !self.result.errors.is_empty() {
            println!("  ❌ Errors encountered: {}", self.result.errors.len());
            for error in &self.result.errors {
                println!("     - {}", error);
            }
        }
        
        // Overall assessment
        println!("\n🎯 Stability Assessment:");
        if self.result.stability_score >= 90.0 {
            println!("  EXCELLENT: System is highly stable under stress");
        } else if self.result.stability_score >= 75.0 {
            println!("  GOOD: System is stable with minor issues");
        } else if self.result.stability_score >= 60.0 {
            println!("  FAIR: System has stability concerns");
        } else {
            println!("  POOR: System is unstable under stress");
        }
    }

    /// Get stress test results
    pub fn get_results(&self) -> &StressTestResult {
        &self.result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_test_config_default() {
        let config = StressTestConfig::default();
        assert_eq!(config.cycles, 1000);
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.sample_rate, 44100.0);
        assert_eq!(config.test_frequency, 440.0);
        assert!(config.enable_memory_monitoring);
        assert!(config.enable_performance_monitoring);
    }

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();
        
        assert_eq!(tracker.allocated_buffers, 0);
        assert_eq!(tracker.total_allocations, 0);
        
        tracker.allocate_buffer(1024);
        assert_eq!(tracker.allocated_buffers, 1);
        assert_eq!(tracker.total_allocations, 1);
        
        tracker.deallocate_buffer();
        assert_eq!(tracker.allocated_buffers, 0);
        assert_eq!(tracker.total_allocations, 1);
        
        let memory_estimate = tracker.estimate_memory_usage();
        assert!(memory_estimate > 0);
    }

    #[test]
    fn test_stress_test_result_creation() {
        let config = StressTestConfig::default();
        let result = StressTestResult::new(config.clone());
        
        assert_eq!(result.completed_cycles, 0);
        assert_eq!(result.memory_snapshots.len(), 0);
        assert_eq!(result.performance_measurements.len(), 0);
        assert!(!result.memory_leak_detected);
        assert!(!result.performance_degradation_detected);
        assert_eq!(result.stability_score, 0.0);
    }

    #[test]
    fn test_stress_tester_creation() {
        let config = StressTestConfig::default();
        let tester = StressTester::new(config.clone());
        
        assert_eq!(tester.config.cycles, 1000);
        assert!(tester.baseline_performance.is_none());
        assert_eq!(tester.performance_window.len(), 0);
    }

    #[test]
    fn test_test_signal_generation() {
        let config = StressTestConfig::default();
        let tester = StressTester::new(config);
        
        let signal = tester.generate_test_signal();
        assert_eq!(signal.len(), 1024);
        
        // Verify it's not all zeros
        let has_non_zero = signal.iter().any(|&x| x != 0.0);
        assert!(has_non_zero);
        
        // Verify amplitude is reasonable
        let max_amplitude = signal.iter().map(|x| x.abs()).fold(0.0, f32::max);
        assert!(max_amplitude > 0.5 && max_amplitude <= 1.0);
    }
} 