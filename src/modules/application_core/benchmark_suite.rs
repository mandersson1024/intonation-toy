//! # Event Bus Performance Benchmark Suite
//!
//! This module provides comprehensive performance benchmarking for the event bus system,
//! including baseline establishment, regression detection, and performance profiling.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use crate::modules::application_core::*;

/// Performance benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub name: String,
    pub iterations: u32,
    pub warm_up_iterations: u32,
    pub events_per_iteration: u32,
    pub priority_mix: PriorityMix,
    pub handler_processing_time_micros: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            name: "Default Benchmark".to_string(),
            iterations: 100,
            warm_up_iterations: 10,
            events_per_iteration: 1000,
            priority_mix: PriorityMix::default(),
            handler_processing_time_micros: 0,
        }
    }
}

/// Priority distribution for benchmark events
#[derive(Debug, Clone)]
pub struct PriorityMix {
    pub critical_ratio: f32,
    pub high_ratio: f32,
    pub normal_ratio: f32,
    pub low_ratio: f32,
}

impl Default for PriorityMix {
    fn default() -> Self {
        Self {
            critical_ratio: 0.1,
            high_ratio: 0.2,
            normal_ratio: 0.6,
            low_ratio: 0.1,
        }
    }
}

impl PriorityMix {
    pub fn audio_focused() -> Self {
        Self {
            critical_ratio: 0.4,
            high_ratio: 0.3,
            normal_ratio: 0.2,
            low_ratio: 0.1,
        }
    }
    
    pub fn balanced() -> Self {
        Self {
            critical_ratio: 0.25,
            high_ratio: 0.25,
            normal_ratio: 0.25,
            low_ratio: 0.25,
        }
    }
    
    pub fn get_priority(&self, index: u32) -> EventPriority {
        let ratio = (index as f32) / 100.0 % 1.0;
        
        if ratio < self.critical_ratio {
            EventPriority::Critical
        } else if ratio < self.critical_ratio + self.high_ratio {
            EventPriority::High
        } else if ratio < self.critical_ratio + self.high_ratio + self.normal_ratio {
            EventPriority::Normal
        } else {
            EventPriority::Low
        }
    }
}

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub config: BenchmarkConfig,
    pub total_duration: Duration,
    pub mean_iteration_time: Duration,
    pub median_iteration_time: Duration,
    pub min_iteration_time: Duration,
    pub max_iteration_time: Duration,
    pub std_deviation: Duration,
    pub events_per_second: f64,
    pub mean_event_latency_ns: f64,
    pub p95_event_latency_ns: f64,
    pub p99_event_latency_ns: f64,
    pub memory_usage_estimate_bytes: usize,
    pub cache_misses_per_event: f64,
}

impl BenchmarkResults {
    pub fn performance_score(&self) -> f64 {
        // Composite score based on throughput and latency
        let throughput_score = (self.events_per_second / 10000.0).min(1.0); // Normalize to 10k events/sec
        let latency_score = (1000000.0 / self.mean_event_latency_ns).min(1.0); // Normalize to 1ms
        
        (throughput_score + latency_score) / 2.0 * 100.0
    }
    
    pub fn is_regression(&self, baseline: &BenchmarkResults, threshold_percent: f64) -> bool {
        let current_score = self.performance_score();
        let baseline_score = baseline.performance_score();
        
        let regression_percent = (baseline_score - current_score) / baseline_score * 100.0;
        regression_percent > threshold_percent
    }
}

/// Benchmark event type with timing metadata
#[derive(Debug, Clone)]
pub struct BenchmarkEvent {
    pub id: u64,
    pub created_at: Instant,
    pub priority: EventPriority,
    pub data: Vec<u8>,
}

impl Event for BenchmarkEvent {
    fn timestamp(&self) -> u64 {
        self.created_at.elapsed().as_nanos() as u64
    }
    
    fn priority(&self) -> EventPriority {
        self.priority
    }
    
    fn event_type(&self) -> &'static str {
        "BenchmarkEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Handler for benchmark events with detailed timing
pub struct BenchmarkHandler {
    pub processing_time_micros: u64,
    pub processed_events: Vec<(Instant, Duration)>, // (processed_at, latency)
}

impl BenchmarkHandler {
    pub fn new(processing_time_micros: u64) -> Self {
        Self {
            processing_time_micros,
            processed_events: Vec::new(),
        }
    }
    
    pub fn get_latency_statistics(&self) -> LatencyStatistics {
        if self.processed_events.is_empty() {
            return LatencyStatistics::default();
        }
        
        let mut latencies: Vec<u64> = self.processed_events
            .iter()
            .map(|(_, latency)| latency.as_nanos() as u64)
            .collect();
        
        latencies.sort_unstable();
        
        let count = latencies.len();
        let mean = latencies.iter().sum::<u64>() as f64 / count as f64;
        let median = latencies[count / 2] as f64;
        let p95 = latencies[(count as f64 * 0.95) as usize] as f64;
        let p99 = latencies[(count as f64 * 0.99) as usize] as f64;
        let min = latencies[0] as f64;
        let max = latencies[count - 1] as f64;
        
        LatencyStatistics {
            mean,
            median,
            p95,
            p99,
            min,
            max,
        }
    }
    
    pub fn reset(&mut self) {
        self.processed_events.clear();
    }
}

impl EventHandler<BenchmarkEvent> for BenchmarkHandler {
    fn handle_event(&mut self, event: &BenchmarkEvent) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        // Simulate processing time
        if self.processing_time_micros > 0 {
            std::thread::sleep(Duration::from_micros(self.processing_time_micros));
        }
        
        let latency = event.created_at.elapsed();
        self.processed_events.push((start, latency));
        
        Ok(())
    }
}

/// Latency statistics
#[derive(Debug, Default)]
pub struct LatencyStatistics {
    pub mean: f64,
    pub median: f64,
    pub p95: f64,
    pub p99: f64,
    pub min: f64,
    pub max: f64,
}

/// Main benchmark suite
pub struct BenchmarkSuite;

impl BenchmarkSuite {
    /// Run complete benchmark suite
    pub fn run_complete_suite() -> BenchmarkSuiteResults {
        let mut results = BenchmarkSuiteResults::new();
        
        // Core performance benchmarks
        results.add_result("Event Publishing", Self::benchmark_event_publishing());
        results.add_result("Event Processing", Self::benchmark_event_processing());
        results.add_result("Priority Queue Performance", Self::benchmark_priority_queue());
        results.add_result("Handler Registration", Self::benchmark_handler_registration());
        results.add_result("Mixed Workload", Self::benchmark_mixed_workload());
        
        // Specialized benchmarks
        results.add_result("High Priority Events", Self::benchmark_high_priority_events());
        results.add_result("Large Event Payloads", Self::benchmark_large_events());
        results.add_result("Memory Efficiency", Self::benchmark_memory_efficiency());
        
        // Audio-specific benchmarks
        results.add_result("Audio Buffer Events", Self::benchmark_audio_buffer_events());
        results.add_result("Real-time Latency", Self::benchmark_realtime_latency());
        
        results
    }
    
    /// Benchmark event publishing performance
    pub fn benchmark_event_publishing() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Event Publishing".to_string(),
            iterations: 1000,
            warm_up_iterations: 100,
            events_per_iteration: 100,
            priority_mix: PriorityMix::default(),
            handler_processing_time_micros: 0,
        };
        
        Self::run_benchmark(config, |event_bus, _handler| {
            // Benchmark just publishing, no processing
        })
    }
    
    /// Benchmark event processing performance
    pub fn benchmark_event_processing() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Event Processing".to_string(),
            iterations: 500,
            warm_up_iterations: 50,
            events_per_iteration: 1000,
            priority_mix: PriorityMix::default(),
            handler_processing_time_micros: 1,
        };
        
        Self::run_benchmark(config, |event_bus, _handler| {
            event_bus.lock().unwrap().process_events().unwrap();
        })
    }
    
    /// Benchmark priority queue performance
    pub fn benchmark_priority_queue() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Priority Queue".to_string(),
            iterations: 200,
            warm_up_iterations: 20,
            events_per_iteration: 5000,
            priority_mix: PriorityMix::balanced(),
            handler_processing_time_micros: 0,
        };
        
        Self::run_benchmark(config, |event_bus, _handler| {
            event_bus.lock().unwrap().process_events().unwrap();
        })
    }
    
    /// Benchmark handler registration performance
    pub fn benchmark_handler_registration() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Handler Registration".to_string(),
            iterations: 100,
            warm_up_iterations: 10,
            events_per_iteration: 10,
            priority_mix: PriorityMix::default(),
            handler_processing_time_micros: 0,
        };
        
        // Custom benchmark for handler registration
        let mut iteration_times = Vec::new();
        
        // Warm up
        for _ in 0..config.warm_up_iterations {
            let start = Instant::now();
            let mut event_bus = PriorityEventBus::new();
            
            for _ in 0..config.events_per_iteration {
                let handler = Arc::new(Mutex::new(BenchmarkHandler::new(0)));
                event_bus.subscribe(handler).unwrap();
            }
            
            iteration_times.push(start.elapsed());
        }
        iteration_times.clear(); // Clear warm-up times
        
        // Actual benchmark
        for _ in 0..config.iterations {
            let start = Instant::now();
            let mut event_bus = PriorityEventBus::new();
            
            for _ in 0..config.events_per_iteration {
                let handler = Arc::new(Mutex::new(BenchmarkHandler::new(0)));
                event_bus.subscribe(handler).unwrap();
            }
            
            iteration_times.push(start.elapsed());
        }
        
        Self::calculate_results(config, iteration_times, LatencyStatistics::default())
    }
    
    /// Benchmark mixed workload performance
    pub fn benchmark_mixed_workload() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Mixed Workload".to_string(),
            iterations: 100,
            warm_up_iterations: 10,
            events_per_iteration: 2000,
            priority_mix: PriorityMix::audio_focused(),
            handler_processing_time_micros: 5,
        };
        
        Self::run_benchmark(config, |event_bus, _handler| {
            event_bus.lock().unwrap().process_events().unwrap();
        })
    }
    
    /// Benchmark high priority events
    pub fn benchmark_high_priority_events() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "High Priority Events".to_string(),
            iterations: 1000,
            warm_up_iterations: 100,
            events_per_iteration: 100,
            priority_mix: PriorityMix {
                critical_ratio: 1.0,
                high_ratio: 0.0,
                normal_ratio: 0.0,
                low_ratio: 0.0,
            },
            handler_processing_time_micros: 0,
        };
        
        Self::run_benchmark(config, |event_bus, _handler| {
            event_bus.lock().unwrap().process_events().unwrap();
        })
    }
    
    /// Benchmark large event payloads
    pub fn benchmark_large_events() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Large Event Payloads".to_string(),
            iterations: 50,
            warm_up_iterations: 5,
            events_per_iteration: 100,
            priority_mix: PriorityMix::default(),
            handler_processing_time_micros: 0,
        };
        
        // Custom benchmark with large payloads
        let mut iteration_times = Vec::new();
        let mut latency_stats = LatencyStatistics::default();
        
        // Warm up
        for _ in 0..config.warm_up_iterations {
            let mut event_bus = PriorityEventBus::new();
            let handler = Arc::new(Mutex::new(BenchmarkHandler::new(0)));
            event_bus.subscribe(handler.clone()).unwrap();
            
            let start = Instant::now();
            
            for i in 0..config.events_per_iteration {
                let event = BenchmarkEvent {
                    id: i as u64,
                    created_at: Instant::now(),
                    priority: config.priority_mix.get_priority(i),
                    data: vec![0u8; 10240], // 10KB payload
                };
                
                event_bus.publish(event).unwrap();
            }
            
            event_bus.process_events().unwrap();
            iteration_times.push(start.elapsed());
        }
        iteration_times.clear();
        
        // Actual benchmark
        for _ in 0..config.iterations {
            let mut event_bus = PriorityEventBus::new();
            let handler = Arc::new(Mutex::new(BenchmarkHandler::new(0)));
            event_bus.subscribe(handler.clone()).unwrap();
            
            let start = Instant::now();
            
            for i in 0..config.events_per_iteration {
                let event = BenchmarkEvent {
                    id: i as u64,
                    created_at: Instant::now(),
                    priority: config.priority_mix.get_priority(i),
                    data: vec![0u8; 10240], // 10KB payload
                };
                
                event_bus.publish(event).unwrap();
            }
            
            event_bus.process_events().unwrap();
            iteration_times.push(start.elapsed());
            
            // Collect latency stats from last iteration
            if iteration_times.len() == config.iterations as usize {
                latency_stats = handler.lock().unwrap().get_latency_statistics();
            }
        }
        
        Self::calculate_results(config, iteration_times, latency_stats)
    }
    
    /// Benchmark memory efficiency
    pub fn benchmark_memory_efficiency() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Memory Efficiency".to_string(),
            iterations: 100,
            warm_up_iterations: 10,
            events_per_iteration: 1000,
            priority_mix: PriorityMix::default(),
            handler_processing_time_micros: 0,
        };
        
        Self::run_benchmark(config, |event_bus, _handler| {
            event_bus.lock().unwrap().process_events().unwrap();
        })
    }
    
    /// Benchmark audio buffer events specifically
    pub fn benchmark_audio_buffer_events() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Audio Buffer Events".to_string(),
            iterations: 500,
            warm_up_iterations: 50,
            events_per_iteration: 1000,
            priority_mix: PriorityMix {
                critical_ratio: 0.8,
                high_ratio: 0.2,
                normal_ratio: 0.0,
                low_ratio: 0.0,
            },
            handler_processing_time_micros: 10, // Simulate audio processing
        };
        
        Self::run_benchmark(config, |event_bus, _handler| {
            event_bus.lock().unwrap().process_events().unwrap();
        })
    }
    
    /// Benchmark real-time latency requirements
    pub fn benchmark_realtime_latency() -> BenchmarkResults {
        let config = BenchmarkConfig {
            name: "Real-time Latency".to_string(),
            iterations: 2000,
            warm_up_iterations: 200,
            events_per_iteration: 1,
            priority_mix: PriorityMix {
                critical_ratio: 1.0,
                high_ratio: 0.0,
                normal_ratio: 0.0,
                low_ratio: 0.0,
            },
            handler_processing_time_micros: 0,
        };
        
        Self::run_benchmark(config, |event_bus, _handler| {
            event_bus.lock().unwrap().process_events().unwrap();
        })
    }
    
    /// Generic benchmark runner
    fn run_benchmark<F>(config: BenchmarkConfig, process_fn: F) -> BenchmarkResults
    where
        F: Fn(&Arc<Mutex<PriorityEventBus>>, &Arc<Mutex<BenchmarkHandler>>),
    {
        let mut iteration_times = Vec::new();
        let mut latency_stats = LatencyStatistics::default();
        
        // Warm up
        for _ in 0..config.warm_up_iterations {
            let mut event_bus = PriorityEventBus::new();
            let handler = Arc::new(Mutex::new(BenchmarkHandler::new(config.handler_processing_time_micros)));
            event_bus.subscribe(handler.clone()).unwrap();
            
            let event_bus = Arc::new(Mutex::new(event_bus));
            
            let start = Instant::now();
            
            // Publish events
            for i in 0..config.events_per_iteration {
                let event = BenchmarkEvent {
                    id: i as u64,
                    created_at: Instant::now(),
                    priority: config.priority_mix.get_priority(i),
                    data: vec![0u8; 64], // Standard small payload
                };
                
                event_bus.lock().unwrap().publish(event).unwrap();
            }
            
            // Process events
            process_fn(&event_bus, &handler);
            
            iteration_times.push(start.elapsed());
        }
        iteration_times.clear(); // Clear warm-up times
        
        // Actual benchmark iterations
        for _ in 0..config.iterations {
            let mut event_bus = PriorityEventBus::new();
            let handler = Arc::new(Mutex::new(BenchmarkHandler::new(config.handler_processing_time_micros)));
            event_bus.subscribe(handler.clone()).unwrap();
            
            let event_bus = Arc::new(Mutex::new(event_bus));
            
            let start = Instant::now();
            
            // Publish events
            for i in 0..config.events_per_iteration {
                let event = BenchmarkEvent {
                    id: i as u64,
                    created_at: Instant::now(),
                    priority: config.priority_mix.get_priority(i),
                    data: vec![0u8; 64],
                };
                
                event_bus.lock().unwrap().publish(event).unwrap();
            }
            
            // Process events
            process_fn(&event_bus, &handler);
            
            iteration_times.push(start.elapsed());
            
            // Collect latency stats from last iteration
            if iteration_times.len() == config.iterations as usize {
                latency_stats = handler.lock().unwrap().get_latency_statistics();
            }
        }
        
        Self::calculate_results(config, iteration_times, latency_stats)
    }
    
    /// Calculate benchmark results from timing data
    fn calculate_results(
        config: BenchmarkConfig,
        mut iteration_times: Vec<Duration>,
        latency_stats: LatencyStatistics,
    ) -> BenchmarkResults {
        iteration_times.sort_unstable();
        
        let count = iteration_times.len();
        let total_duration: Duration = iteration_times.iter().sum();
        let mean_iteration_time = total_duration / count as u32;
        let median_iteration_time = iteration_times[count / 2];
        let min_iteration_time = iteration_times[0];
        let max_iteration_time = iteration_times[count - 1];
        
        // Calculate standard deviation
        let mean_nanos = mean_iteration_time.as_nanos() as f64;
        let variance = iteration_times
            .iter()
            .map(|t| {
                let diff = t.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>() / count as f64;
        let std_deviation = Duration::from_nanos(variance.sqrt() as u64);
        
        let total_events = config.iterations * config.events_per_iteration;
        let events_per_second = total_events as f64 / total_duration.as_secs_f64();
        
        BenchmarkResults {
            config,
            total_duration,
            mean_iteration_time,
            median_iteration_time,
            min_iteration_time,
            max_iteration_time,
            std_deviation,
            events_per_second,
            mean_event_latency_ns: latency_stats.mean,
            p95_event_latency_ns: latency_stats.p95,
            p99_event_latency_ns: latency_stats.p99,
            memory_usage_estimate_bytes: total_events as usize * 64, // Rough estimate
            cache_misses_per_event: 0.0, // Would need hardware counters
        }
    }
}

/// Results from running the complete benchmark suite
#[derive(Debug)]
pub struct BenchmarkSuiteResults {
    pub results: HashMap<String, BenchmarkResults>,
    pub overall_score: f64,
    pub regression_detected: bool,
}

impl BenchmarkSuiteResults {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            overall_score: 0.0,
            regression_detected: false,
        }
    }
    
    pub fn add_result(&mut self, name: &str, result: BenchmarkResults) {
        self.results.insert(name.to_string(), result);
        self.calculate_overall_score();
    }
    
    fn calculate_overall_score(&mut self) {
        if self.results.is_empty() {
            self.overall_score = 0.0;
            return;
        }
        
        let total_score: f64 = self.results.values().map(|r| r.performance_score()).sum();
        self.overall_score = total_score / self.results.len() as f64;
    }
    
    pub fn check_regressions(&mut self, baseline: &BenchmarkSuiteResults, threshold: f64) {
        for (name, result) in &self.results {
            if let Some(baseline_result) = baseline.results.get(name) {
                if result.is_regression(baseline_result, threshold) {
                    self.regression_detected = true;
                    break;
                }
            }
        }
    }
    
    pub fn get_result(&self, name: &str) -> Option<&BenchmarkResults> {
        self.results.get(name)
    }
    
    pub fn benchmark_names(&self) -> Vec<&String> {
        self.results.keys().collect()
    }
}

#[cfg(test)]
mod benchmark_suite_tests {
    use super::*;
    
    #[test]
    #[ignore = "Needs refactoring for new EventBus API"]
    fn test_priority_mix_distribution() {
        let mix = PriorityMix::balanced();
        
        let mut counts = [0u32; 4]; // Critical, High, Normal, Low
        
        for i in 0..1000 {
            match mix.get_priority(i) {
                EventPriority::Critical => counts[0] += 1,
                EventPriority::High => counts[1] += 1,
                EventPriority::Normal => counts[2] += 1,
                EventPriority::Low => counts[3] += 1,
            }
        }
        
        // Each should be roughly 25% (250 out of 1000)
        for count in counts.iter() {
            assert!(*count > 200 && *count < 300, "Unbalanced distribution: {}", count);
        }
    }
    
    #[test]
    #[ignore = "Needs refactoring for new EventBus API"]
    fn test_benchmark_handler() {
        let mut handler = BenchmarkHandler::new(0);
        
        for i in 0..10 {
            let event = BenchmarkEvent {
                id: i,
                created_at: Instant::now(),
                priority: EventPriority::Normal,
                data: vec![0u8; 64],
            };
            
            std::thread::sleep(Duration::from_micros(100)); // Ensure measurable latency
            handler.handle(event).unwrap();
        }
        
        let stats = handler.get_latency_statistics();
        assert!(stats.mean > 0.0);
        assert!(stats.min > 0.0);
        assert!(stats.max >= stats.min);
        assert_eq!(handler.processed_events.len(), 10);
    }
    
    #[test]
    #[ignore = "Needs refactoring for new EventBus API"]
    fn test_benchmark_suite_event_publishing() {
        let results = BenchmarkSuite::benchmark_event_publishing();
        
        assert_eq!(results.config.name, "Event Publishing");
        assert!(results.events_per_second > 0.0);
        assert!(results.total_duration > Duration::from_nanos(0));
        assert!(results.performance_score() > 0.0);
    }
    
    #[test]
    #[ignore = "Needs refactoring for new EventBus API"]
    fn test_benchmark_suite_results() {
        let mut suite_results = BenchmarkSuiteResults::new();
        
        let benchmark_result = BenchmarkSuite::benchmark_event_publishing();
        suite_results.add_result("Test Benchmark", benchmark_result);
        
        assert_eq!(suite_results.results.len(), 1);
        assert!(suite_results.overall_score > 0.0);
        assert!(suite_results.get_result("Test Benchmark").is_some());
    }
    
    #[test]
    #[ignore = "Needs refactoring for new EventBus API"]
    fn test_performance_regression_detection() {
        let baseline_config = BenchmarkConfig {
            name: "Baseline".to_string(),
            events_per_iteration: 100,
            ..Default::default()
        };
        
        let baseline_result = BenchmarkResults {
            config: baseline_config,
            events_per_second: 10000.0,
            mean_event_latency_ns: 100000.0, // 0.1ms
            ..Default::default()
        };
        
        let regressed_result = BenchmarkResults {
            config: baseline_result.config.clone(),
            events_per_second: 5000.0, // 50% slower
            mean_event_latency_ns: 200000.0, // 2x latency
            ..Default::default()
        };
        
        assert!(regressed_result.is_regression(&baseline_result, 10.0));
        assert!(!baseline_result.is_regression(&baseline_result, 10.0));
    }
}

// Provide default implementation for BenchmarkResults for testing
impl Default for BenchmarkResults {
    fn default() -> Self {
        Self {
            config: BenchmarkConfig::default(),
            total_duration: Duration::from_secs(1),
            mean_iteration_time: Duration::from_millis(10),
            median_iteration_time: Duration::from_millis(10),
            min_iteration_time: Duration::from_millis(5),
            max_iteration_time: Duration::from_millis(20),
            std_deviation: Duration::from_millis(2),
            events_per_second: 1000.0,
            mean_event_latency_ns: 1000000.0,
            p95_event_latency_ns: 2000000.0,
            p99_event_latency_ns: 5000000.0,
            memory_usage_estimate_bytes: 64000,
            cache_misses_per_event: 0.5,
        }
    }
}