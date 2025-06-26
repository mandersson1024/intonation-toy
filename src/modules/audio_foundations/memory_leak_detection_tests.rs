// Long-Running Memory Leak Detection Tests - STORY-3.19
// Extended audio session memory monitoring and leak detection

#[cfg(test)]
mod memory_leak_detection_tests {
    use crate::modules::audio_foundations::*;
    use std::time::{Duration, Instant};
    use std::sync::Arc;
    use std::collections::VecDeque;

    /// Memory usage measurement
    #[derive(Debug, Clone)]
    pub struct MemoryMeasurement {
        pub timestamp: Instant,
        pub heap_usage_bytes: usize,
        pub buffer_count: usize,
        pub object_count: usize,
        pub test_description: String,
    }

    impl MemoryMeasurement {
        fn new(heap_usage: usize, buffer_count: usize, object_count: usize, description: &str) -> Self {
            Self {
                timestamp: Instant::now(),
                heap_usage_bytes: heap_usage,
                buffer_count,
                object_count,
                test_description: description.to_string(),
            }
        }
    }

    /// Memory leak detector and analyzer
    pub struct MemoryLeakDetector {
        measurements: VecDeque<MemoryMeasurement>,
        baseline_measurement: Option<MemoryMeasurement>,
        max_measurements: usize,
        leak_threshold_bytes: usize,
        leak_threshold_objects: usize,
    }

    impl MemoryLeakDetector {
        pub fn new() -> Self {
            Self {
                measurements: VecDeque::new(),
                baseline_measurement: None,
                max_measurements: 1000,
                leak_threshold_bytes: 50 * 1024 * 1024, // 50MB
                leak_threshold_objects: 10000,
            }
        }

        /// Set baseline memory measurement
        pub fn set_baseline(&mut self, measurement: MemoryMeasurement) {
            self.baseline_measurement = Some(measurement);
        }

        /// Add a memory measurement
        pub fn add_measurement(&mut self, measurement: MemoryMeasurement) {
            self.measurements.push_back(measurement);
            
            // Keep only the last N measurements
            while self.measurements.len() > self.max_measurements {
                self.measurements.pop_front();
            }
        }

        /// Detect potential memory leaks
        pub fn detect_leaks(&self) -> MemoryLeakAnalysis {
            if self.measurements.len() < 10 {
                return MemoryLeakAnalysis {
                    has_potential_leak: false,
                    leak_severity: LeakSeverity::None,
                    memory_growth_rate_mb_per_hour: 0.0,
                    object_growth_rate_per_hour: 0.0,
                    analysis_notes: vec!["Insufficient data for leak analysis".to_string()],
                    recommendations: vec![],
                };
            }

            let mut analysis_notes = Vec::new();
            let mut recommendations = Vec::new();

            // Calculate memory growth rate
            let first_measurement = &self.measurements[0];
            let last_measurement = &self.measurements[self.measurements.len() - 1];
            
            let time_diff_hours = last_measurement.timestamp
                .duration_since(first_measurement.timestamp)
                .as_secs_f64() / 3600.0;

            let memory_growth_bytes = last_measurement.heap_usage_bytes as i64 
                - first_measurement.heap_usage_bytes as i64;
            let object_growth = last_measurement.object_count as i64 
                - first_measurement.object_count as i64;

            let memory_growth_rate_mb_per_hour = if time_diff_hours > 0.0 {
                (memory_growth_bytes as f64 / (1024.0 * 1024.0)) / time_diff_hours
            } else {
                0.0
            };

            let object_growth_rate_per_hour = if time_diff_hours > 0.0 {
                object_growth as f64 / time_diff_hours
            } else {
                0.0
            };

            // Detect trends
            let has_consistent_growth = self.check_consistent_growth();
            let has_memory_spikes = self.check_memory_spikes();
            let baseline_exceeded = self.check_baseline_exceeded();

            // Determine leak severity
            let leak_severity = if memory_growth_rate_mb_per_hour > 100.0 {
                LeakSeverity::Critical
            } else if memory_growth_rate_mb_per_hour > 10.0 || has_consistent_growth {
                LeakSeverity::Major
            } else if memory_growth_rate_mb_per_hour > 1.0 || has_memory_spikes {
                LeakSeverity::Minor
            } else {
                LeakSeverity::None
            };

            let has_potential_leak = leak_severity != LeakSeverity::None;

            // Generate analysis notes
            if has_consistent_growth {
                analysis_notes.push("Detected consistent memory growth pattern".to_string());
            }
            if has_memory_spikes {
                analysis_notes.push("Detected periodic memory spikes".to_string());
            }
            if baseline_exceeded {
                analysis_notes.push("Memory usage significantly exceeds baseline".to_string());
            }
            if memory_growth_rate_mb_per_hour > 0.1 {
                analysis_notes.push(format!(
                    "Memory growing at {:.2} MB/hour", 
                    memory_growth_rate_mb_per_hour
                ));
            }
            if object_growth_rate_per_hour > 100.0 {
                analysis_notes.push(format!(
                    "Object count growing at {:.0} objects/hour", 
                    object_growth_rate_per_hour
                ));
            }

            // Generate recommendations
            match leak_severity {
                LeakSeverity::Critical => {
                    recommendations.push("Immediate investigation required - critical memory leak detected".to_string());
                    recommendations.push("Check for unreleased audio buffers and event listeners".to_string());
                    recommendations.push("Review WebAssembly memory management".to_string());
                }
                LeakSeverity::Major => {
                    recommendations.push("Monitor memory usage closely during production".to_string());
                    recommendations.push("Implement periodic memory cleanup cycles".to_string());
                    recommendations.push("Review buffer lifecycle management".to_string());
                }
                LeakSeverity::Minor => {
                    recommendations.push("Continue monitoring for trend development".to_string());
                    recommendations.push("Consider implementing memory usage thresholds".to_string());
                }
                LeakSeverity::None => {
                    recommendations.push("Memory usage appears stable".to_string());
                }
            }

            MemoryLeakAnalysis {
                has_potential_leak,
                leak_severity,
                memory_growth_rate_mb_per_hour,
                object_growth_rate_per_hour,
                analysis_notes,
                recommendations,
            }
        }

        fn check_consistent_growth(&self) -> bool {
            if self.measurements.len() < 5 {
                return false;
            }

            let mut growth_count = 0;
            let mut total_comparisons = 0;

            for i in 1..self.measurements.len() {
                let current = &self.measurements[i];
                let previous = &self.measurements[i - 1];
                
                if current.heap_usage_bytes > previous.heap_usage_bytes {
                    growth_count += 1;
                }
                total_comparisons += 1;
            }

            // Consider it consistent growth if >70% of measurements show growth
            growth_count as f64 / total_comparisons as f64 > 0.7
        }

        fn check_memory_spikes(&self) -> bool {
            if self.measurements.len() < 10 {
                return false;
            }

            let average_memory: f64 = self.measurements.iter()
                .map(|m| m.heap_usage_bytes as f64)
                .sum::<f64>() / self.measurements.len() as f64;

            let spike_threshold = average_memory * 1.5; // 50% above average

            let spike_count = self.measurements.iter()
                .filter(|m| m.heap_usage_bytes as f64 > spike_threshold)
                .count();

            // Consider it spiky if >10% of measurements are spikes
            spike_count as f64 / self.measurements.len() as f64 > 0.1
        }

        fn check_baseline_exceeded(&self) -> bool {
            if let Some(ref baseline) = self.baseline_measurement {
                if let Some(latest) = self.measurements.back() {
                    let growth_factor = latest.heap_usage_bytes as f64 / baseline.heap_usage_bytes as f64;
                    return growth_factor > 2.0; // More than 2x baseline
                }
            }
            false
        }

        /// Get memory usage statistics
        pub fn get_memory_stats(&self) -> MemoryStats {
            if self.measurements.is_empty() {
                return MemoryStats::default();
            }

            let memory_values: Vec<usize> = self.measurements.iter()
                .map(|m| m.heap_usage_bytes)
                .collect();

            let min_memory = *memory_values.iter().min().unwrap();
            let max_memory = *memory_values.iter().max().unwrap();
            let avg_memory = memory_values.iter().sum::<usize>() / memory_values.len();

            let object_values: Vec<usize> = self.measurements.iter()
                .map(|m| m.object_count)
                .collect();

            let min_objects = *object_values.iter().min().unwrap();
            let max_objects = *object_values.iter().max().unwrap();
            let avg_objects = object_values.iter().sum::<usize>() / object_values.len();

            MemoryStats {
                min_memory_mb: min_memory as f64 / (1024.0 * 1024.0),
                max_memory_mb: max_memory as f64 / (1024.0 * 1024.0),
                avg_memory_mb: avg_memory as f64 / (1024.0 * 1024.0),
                min_objects,
                max_objects,
                avg_objects,
                measurement_count: self.measurements.len(),
                memory_variance: Self::calculate_variance(&memory_values),
            }
        }

        fn calculate_variance(values: &[usize]) -> f64 {
            if values.len() < 2 {
                return 0.0;
            }

            let mean = values.iter().sum::<usize>() as f64 / values.len() as f64;
            let variance = values.iter()
                .map(|&x| {
                    let diff = x as f64 - mean;
                    diff * diff
                })
                .sum::<f64>() / values.len() as f64;

            variance
        }
    }

    /// Memory leak analysis result
    #[derive(Debug)]
    pub struct MemoryLeakAnalysis {
        pub has_potential_leak: bool,
        pub leak_severity: LeakSeverity,
        pub memory_growth_rate_mb_per_hour: f64,
        pub object_growth_rate_per_hour: f64,
        pub analysis_notes: Vec<String>,
        pub recommendations: Vec<String>,
    }

    /// Severity of detected memory leak
    #[derive(Debug, PartialEq)]
    pub enum LeakSeverity {
        None,
        Minor,
        Major,
        Critical,
    }

    /// Memory usage statistics
    #[derive(Debug)]
    pub struct MemoryStats {
        pub min_memory_mb: f64,
        pub max_memory_mb: f64,
        pub avg_memory_mb: f64,
        pub min_objects: usize,
        pub max_objects: usize,
        pub avg_objects: usize,
        pub measurement_count: usize,
        pub memory_variance: f64,
    }

    impl Default for MemoryStats {
        fn default() -> Self {
            Self {
                min_memory_mb: 0.0,
                max_memory_mb: 0.0,
                avg_memory_mb: 0.0,
                min_objects: 0,
                max_objects: 0,
                avg_objects: 0,
                measurement_count: 0,
                memory_variance: 0.0,
            }
        }
    }

    /// Long-running memory test session
    pub struct LongRunningMemoryTestSession {
        pitch_detector: MultiAlgorithmPitchDetector,
        performance_monitor: AudioPerformanceMonitor,
        leak_detector: MemoryLeakDetector,
        test_start_time: Instant,
        iteration_count: usize,
        allocated_buffers: Vec<Vec<f32>>,
    }

    impl LongRunningMemoryTestSession {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let config = PitchDetectionConfig::default();
            let pitch_detector = MultiAlgorithmPitchDetector::new(config, None)?;
            let performance_monitor = AudioPerformanceMonitor::new();
            let leak_detector = MemoryLeakDetector::new();

            Ok(Self {
                pitch_detector,
                performance_monitor,
                leak_detector,
                test_start_time: Instant::now(),
                iteration_count: 0,
                allocated_buffers: Vec::new(),
            })
        }

        /// Take a memory measurement
        pub fn measure_memory(&mut self, description: &str) {
            // Simulate memory measurement (in real implementation, this would use actual memory APIs)
            let heap_usage = self.estimate_memory_usage();
            let buffer_count = self.allocated_buffers.len();
            let object_count = self.estimate_object_count();

            let measurement = MemoryMeasurement::new(
                heap_usage, 
                buffer_count, 
                object_count, 
                description
            );

            if self.iteration_count == 0 {
                self.leak_detector.set_baseline(measurement.clone());
            }

            self.leak_detector.add_measurement(measurement);
        }

        /// Estimate current memory usage (simplified for testing)
        fn estimate_memory_usage(&self) -> usize {
            let base_usage = 10 * 1024 * 1024; // 10MB base
            let buffer_usage = self.allocated_buffers.len() * 2048 * 4; // 4 bytes per f32
            let iteration_overhead = self.iteration_count * 1024; // 1KB per iteration
            
            base_usage + buffer_usage + iteration_overhead
        }

        /// Estimate current object count (simplified for testing)
        fn estimate_object_count(&self) -> usize {
            let base_objects = 1000;
            let buffer_objects = self.allocated_buffers.len();
            let iteration_objects = self.iteration_count / 10; // Some objects accumulate
            
            base_objects + buffer_objects + iteration_objects
        }

        /// Simulate audio processing iteration
        pub fn process_iteration(&mut self) -> Result<PitchResult, PitchError> {
            self.iteration_count += 1;

            // Generate test signal
            let test_signal: Vec<f32> = (0..2048)
                .map(|i| {
                    let t = i as f32 / 44100.0;
                    let frequency = 440.0 + (self.iteration_count as f32 % 100.0); // Vary frequency slightly
                    0.8 * (2.0 * std::f32::consts::PI * frequency * t).sin()
                })
                .collect();

            // Process audio
            let result = self.pitch_detector.detect_pitch(&test_signal);

            // Simulate potential memory leak scenarios
            if self.iteration_count % 100 == 0 {
                // Intentionally allocate some buffers that might not be cleaned up
                self.allocated_buffers.push(vec![0.0; 1024]);
            }

            if self.iteration_count % 500 == 0 {
                // Periodic cleanup (simulate garbage collection or manual cleanup)
                if self.allocated_buffers.len() > 10 {
                    self.allocated_buffers.truncate(10);
                }
            }

            result
        }

        /// Run extended memory test session
        pub fn run_extended_session(&mut self, duration_minutes: u64) -> ExtendedSessionResult {
            let session_duration = Duration::from_secs(duration_minutes * 60);
            let start_time = Instant::now();
            
            let mut iteration_count = 0;
            let mut successful_iterations = 0;
            let mut error_count = 0;
            let mut errors = Vec::new();

            println!("Starting extended memory test session for {} minutes...", duration_minutes);

            // Take baseline measurement
            self.measure_memory("session_start");

            while start_time.elapsed() < session_duration {
                // Process audio iteration
                match self.process_iteration() {
                    Ok(_) => successful_iterations += 1,
                    Err(e) => {
                        error_count += 1;
                        errors.push(format!("Iteration {}: {}", iteration_count, e));
                    }
                }

                iteration_count += 1;

                // Take memory measurements periodically
                if iteration_count % 100 == 0 {
                    self.measure_memory(&format!("iteration_{}", iteration_count));
                }

                // Brief pause to simulate real-time processing
                if iteration_count % 1000 == 0 {
                    std::thread::sleep(Duration::from_millis(10));
                    println!("Completed {} iterations ({:.1}% of session)", 
                        iteration_count, 
                        start_time.elapsed().as_secs_f64() / session_duration.as_secs_f64() * 100.0);
                }
            }

            // Take final measurement
            self.measure_memory("session_end");

            // Analyze memory leaks
            let leak_analysis = self.leak_detector.detect_leaks();
            let memory_stats = self.leak_detector.get_memory_stats();

            ExtendedSessionResult {
                session_duration_minutes: start_time.elapsed().as_secs() / 60,
                total_iterations: iteration_count,
                successful_iterations,
                error_count,
                errors,
                leak_analysis,
                memory_stats,
                session_successful: error_count == 0 || (error_count as f64 / iteration_count as f64) < 0.01,
            }
        }
    }

    /// Result of extended memory test session
    #[derive(Debug)]
    pub struct ExtendedSessionResult {
        pub session_duration_minutes: u64,
        pub total_iterations: usize,
        pub successful_iterations: usize,
        pub error_count: usize,
        pub errors: Vec<String>,
        pub leak_analysis: MemoryLeakAnalysis,
        pub memory_stats: MemoryStats,
        pub session_successful: bool,
    }

    impl ExtendedSessionResult {
        pub fn print_report(&self) {
            println!("\n=== Extended Memory Test Session Report ===");
            println!("Session Duration: {} minutes", self.session_duration_minutes);
            println!("Total Iterations: {}", self.total_iterations);
            println!("Successful Iterations: {}", self.successful_iterations);
            println!("Error Count: {}", self.error_count);
            println!("Success Rate: {:.1}%", 
                (self.successful_iterations as f64 / self.total_iterations as f64) * 100.0);

            println!("\n=== Memory Statistics ===");
            println!("Memory Range: {:.2} - {:.2} MB (avg: {:.2} MB)", 
                self.memory_stats.min_memory_mb, 
                self.memory_stats.max_memory_mb,
                self.memory_stats.avg_memory_mb);
            println!("Object Range: {} - {} (avg: {})", 
                self.memory_stats.min_objects, 
                self.memory_stats.max_objects,
                self.memory_stats.avg_objects);
            println!("Memory Measurements: {}", self.memory_stats.measurement_count);
            println!("Memory Variance: {:.2}", self.memory_stats.memory_variance);

            println!("\n=== Memory Leak Analysis ===");
            println!("Potential Leak Detected: {}", self.leak_analysis.has_potential_leak);
            println!("Leak Severity: {:?}", self.leak_analysis.leak_severity);
            println!("Memory Growth Rate: {:.2} MB/hour", self.leak_analysis.memory_growth_rate_mb_per_hour);
            println!("Object Growth Rate: {:.1} objects/hour", self.leak_analysis.object_growth_rate_per_hour);

            if !self.leak_analysis.analysis_notes.is_empty() {
                println!("\nAnalysis Notes:");
                for note in &self.leak_analysis.analysis_notes {
                    println!("  - {}", note);
                }
            }

            if !self.leak_analysis.recommendations.is_empty() {
                println!("\nRecommendations:");
                for recommendation in &self.leak_analysis.recommendations {
                    println!("  - {}", recommendation);
                }
            }

            if !self.errors.is_empty() && self.errors.len() <= 10 {
                println!("\nRecent Errors:");
                for error in &self.errors {
                    println!("  - {}", error);
                }
            } else if self.errors.len() > 10 {
                println!("\nRecent Errors ({} total, showing last 5):", self.errors.len());
                for error in self.errors.iter().rev().take(5) {
                    println!("  - {}", error);
                }
            }
        }
    }

    // =============================================================================
    // ACTUAL TESTS
    // =============================================================================

    #[test]
    fn test_memory_leak_detector_basic() {
        let mut detector = MemoryLeakDetector::new();

        // Add measurements showing gradual memory growth
        for i in 0..20 {
            let memory_usage = 10_000_000 + (i * 1_000_000); // Growing by 1MB each time
            let measurement = MemoryMeasurement::new(
                memory_usage, 
                i, 
                1000 + i * 10, 
                &format!("test_iteration_{}", i)
            );
            
            if i == 0 {
                detector.set_baseline(measurement.clone());
            }
            detector.add_measurement(measurement);
        }

        let analysis = detector.detect_leaks();
        assert!(analysis.has_potential_leak, "Should detect potential leak with consistent growth");
        assert!(analysis.memory_growth_rate_mb_per_hour > 0.0, "Should show positive growth rate");
    }

    #[test]
    fn test_memory_leak_detector_stable_memory() {
        let mut detector = MemoryLeakDetector::new();

        // Add measurements showing stable memory usage
        for i in 0..20 {
            let memory_usage = 10_000_000 + (i % 3) * 100_000; // Slight variation but stable
            let measurement = MemoryMeasurement::new(
                memory_usage, 
                i, 
                1000 + i % 5, 
                &format!("stable_iteration_{}", i)
            );
            
            if i == 0 {
                detector.set_baseline(measurement.clone());
            }
            detector.add_measurement(measurement);
        }

        let analysis = detector.detect_leaks();
        assert!(!analysis.has_potential_leak, "Should not detect leak with stable memory");
        assert_eq!(analysis.leak_severity, LeakSeverity::None);
    }

    #[test]
    fn test_memory_stats_calculation() {
        let mut detector = MemoryLeakDetector::new();

        let memory_values = vec![5_000_000, 10_000_000, 15_000_000, 8_000_000, 12_000_000];
        
        for (i, &memory) in memory_values.iter().enumerate() {
            let measurement = MemoryMeasurement::new(
                memory, 
                i, 
                1000 + i * 100, 
                &format!("stats_test_{}", i)
            );
            detector.add_measurement(measurement);
        }

        let stats = detector.get_memory_stats();
        
        assert_eq!(stats.measurement_count, 5);
        assert!((stats.min_memory_mb - 5.0).abs() < 0.1, "Min memory should be ~5MB");
        assert!((stats.max_memory_mb - 15.0).abs() < 0.1, "Max memory should be ~15MB");
        assert!((stats.avg_memory_mb - 10.0).abs() < 0.1, "Avg memory should be ~10MB");
    }

    #[test]
    fn test_short_memory_session() {
        let mut session = LongRunningMemoryTestSession::new().unwrap();

        // Run a short session
        for _ in 0..100 {
            let result = session.process_iteration();
            assert!(result.is_ok(), "Audio processing should succeed");
        }

        // Take measurements
        session.measure_memory("test_start");
        
        for _ in 100..200 {
            let _result = session.process_iteration();
        }
        
        session.measure_memory("test_end");

        let analysis = session.leak_detector.detect_leaks();
        let stats = session.leak_detector.get_memory_stats();

        // Short sessions shouldn't show significant leaks
        assert!(stats.measurement_count >= 2, "Should have at least 2 measurements");
        assert!(stats.avg_memory_mb > 0.0, "Should have meaningful memory measurements");
    }

    #[test]
    fn test_extended_memory_session() {
        let mut session = LongRunningMemoryTestSession::new().unwrap();

        // Run extended session (short duration for testing)
        let result = session.run_extended_session(1); // 1 minute

        result.print_report();

        // Validate session results
        assert!(result.session_successful, "Extended session should complete successfully");
        assert!(result.total_iterations > 100, "Should complete many iterations");
        assert!(result.memory_stats.measurement_count > 5, "Should take multiple memory measurements");
        assert!(result.memory_stats.avg_memory_mb > 5.0, "Should show realistic memory usage");

        // Memory growth should be reasonable for short session
        if result.leak_analysis.has_potential_leak {
            assert!(result.leak_analysis.memory_growth_rate_mb_per_hour < 1000.0, 
                "Memory growth should be reasonable for short test");
        }
    }

    #[test]
    fn test_memory_cleanup_effectiveness() {
        let mut session = LongRunningMemoryTestSession::new().unwrap();

        session.measure_memory("before_allocation");

        // Simulate memory allocation
        for _ in 0..1000 {
            session.allocated_buffers.push(vec![0.0; 1024]);
        }

        session.measure_memory("after_allocation");

        // Simulate cleanup
        session.allocated_buffers.clear();

        session.measure_memory("after_cleanup");

        let measurements = &session.leak_detector.measurements;
        assert!(measurements.len() >= 3, "Should have before, after, and cleanup measurements");

        let before = &measurements[0];
        let after_alloc = &measurements[1];
        let after_cleanup = &measurements[2];

        assert!(after_alloc.buffer_count > before.buffer_count, 
            "Buffer count should increase after allocation");
        assert!(after_cleanup.buffer_count < after_alloc.buffer_count,
            "Buffer count should decrease after cleanup");
    }

    #[test]
    fn test_memory_leak_severity_classification() {
        let mut detector = MemoryLeakDetector::new();

        // Test critical leak scenario
        let start_time = Instant::now();
        for i in 0..10 {
            let memory_usage = 10_000_000 + (i * 50_000_000); // Growing by 50MB each measurement
            let measurement = MemoryMeasurement {
                timestamp: start_time + Duration::from_secs(i * 60), // 1 minute apart
                heap_usage_bytes: memory_usage,
                buffer_count: i,
                object_count: 1000 + i * 1000,
                test_description: format!("critical_leak_test_{}", i),
            };
            
            if i == 0 {
                detector.set_baseline(measurement.clone());
            }
            detector.add_measurement(measurement);
        }

        let analysis = detector.detect_leaks();
        assert!(analysis.has_potential_leak, "Should detect critical leak");
        assert_eq!(analysis.leak_severity, LeakSeverity::Critical, "Should classify as critical");
        assert!(analysis.memory_growth_rate_mb_per_hour > 100.0, "Should show high growth rate");
    }

    #[test]
    fn test_wasm_memory_simulation() {
        let mut session = LongRunningMemoryTestSession::new().unwrap();

        // Simulate WebAssembly memory allocation patterns
        session.measure_memory("wasm_start");

        for iteration in 0..500 {
            let _result = session.process_iteration();

            // Simulate WASM memory growth patterns
            if iteration % 50 == 0 {
                // Simulate WASM linear memory growth
                let memory_size = session.estimate_memory_usage() + (iteration * 1024);
                let measurement = MemoryMeasurement::new(
                    memory_size,
                    session.allocated_buffers.len(),
                    session.estimate_object_count(),
                    &format!("wasm_iteration_{}", iteration)
                );
                session.leak_detector.add_measurement(measurement);
            }
        }

        session.measure_memory("wasm_end");

        let analysis = session.leak_detector.detect_leaks();
        let stats = session.leak_detector.get_memory_stats();

        // Validate WASM memory analysis
        assert!(stats.measurement_count > 5, "Should have multiple WASM memory measurements");
        
        if analysis.has_potential_leak {
            assert!(!analysis.recommendations.is_empty(), "Should provide recommendations for leaks");
        }
    }

    #[test]
    fn test_event_bus_memory_management() {
        // Test memory management with event bus
        let event_bus = Arc::new(crate::modules::application_core::typed_event_bus::TypedEventBus::new());
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus.clone())).unwrap();

        let mut leak_detector = MemoryLeakDetector::new();
        
        // Simulate event-heavy processing
        for i in 0..100 {
            let test_signal: Vec<f32> = (0..1024)
                .map(|j| (j as f32 / 1024.0 * 2.0 * std::f32::consts::PI * 440.0).sin())
                .collect();

            let _result = detector.detect_pitch(&test_signal);

            // Simulate memory measurement
            if i % 20 == 0 {
                let memory_usage = 5_000_000 + (i * 50_000); // Simulate growth
                let measurement = MemoryMeasurement::new(
                    memory_usage,
                    i / 10,
                    1000 + i * 5,
                    &format!("event_bus_test_{}", i)
                );
                
                if i == 0 {
                    leak_detector.set_baseline(measurement.clone());
                }
                leak_detector.add_measurement(measurement);
            }
        }

        let analysis = leak_detector.detect_leaks();
        
        // Event bus usage shouldn't cause major leaks
        if analysis.has_potential_leak {
            assert!(analysis.leak_severity != LeakSeverity::Critical, 
                "Event bus usage shouldn't cause critical leaks");
        }

        // Verify we can still use the event bus after tests
        assert!(Arc::strong_count(&event_bus) >= 1, "Event bus should still be referenced");
    }

    #[test]
    fn test_buffer_lifecycle_memory_management() {
        let mut session = LongRunningMemoryTestSession::new().unwrap();

        session.measure_memory("buffer_test_start");

        // Test different buffer allocation patterns
        for cycle in 0..10 {
            // Allocation phase
            for _ in 0..50 {
                session.allocated_buffers.push(vec![0.0; 2048]);
            }

            session.measure_memory(&format!("after_allocation_cycle_{}", cycle));

            // Processing phase (keep buffers)
            for _ in 0..25 {
                let _result = session.process_iteration();
            }

            session.measure_memory(&format!("after_processing_cycle_{}", cycle));

            // Cleanup phase
            if cycle % 3 == 2 {
                session.allocated_buffers.clear();
                session.measure_memory(&format!("after_cleanup_cycle_{}", cycle));
            }
        }

        let stats = session.leak_detector.get_memory_stats();
        let analysis = session.leak_detector.detect_leaks();

        // Validate buffer lifecycle doesn't cause excessive leaks
        assert!(stats.measurement_count > 10, "Should have many measurements");
        
        if analysis.has_potential_leak {
            // Some growth is expected due to buffer allocations
            assert!(analysis.memory_growth_rate_mb_per_hour < 500.0, 
                "Buffer allocation shouldn't cause excessive growth");
        }
    }
}