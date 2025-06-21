use std::time::{Duration, Instant};
use std::collections::HashMap;
use crate::audio::{engine::AudioEngine, pitch_detector::PitchAlgorithm};

/// Performance benchmark results for a single test
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub duration_ns: u64,
    pub duration_ms: f64,
    pub buffer_size: usize,
    pub sample_rate: f32,
    pub algorithm: Option<PitchAlgorithm>,
    pub throughput_samples_per_second: f64,
    pub meets_latency_requirement: bool, // <50ms requirement
}

impl BenchmarkResult {
    pub fn new(
        test_name: String,
        duration: Duration,
        buffer_size: usize,
        sample_rate: f32,
        algorithm: Option<PitchAlgorithm>,
    ) -> Self {
        let duration_ns = duration.as_nanos() as u64;
        let duration_ms = duration_ns as f64 / 1_000_000.0;
        let throughput_samples_per_second = (buffer_size as f64) / (duration_ns as f64 / 1_000_000_000.0);
        let meets_latency_requirement = duration_ms < 50.0;

        BenchmarkResult {
            test_name,
            duration_ns,
            duration_ms,
            buffer_size,
            sample_rate,
            algorithm,
            throughput_samples_per_second,
            meets_latency_requirement,
        }
    }
}

/// Performance benchmark suite for audio processing
pub struct PerformanceBenchmark {
    results: Vec<BenchmarkResult>,
    baseline_results: HashMap<String, BenchmarkResult>,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        PerformanceBenchmark {
            results: Vec::new(),
            baseline_results: HashMap::new(),
        }
    }

    /// Run comprehensive performance benchmarks
    pub fn run_all_benchmarks(&mut self, sample_rate: f32) {
        println!("🚀 Starting comprehensive performance benchmarks...");
        
        // Latency benchmarks
        self.benchmark_audio_processing_latency(sample_rate);
        self.benchmark_pitch_detection_latency(sample_rate);
        self.benchmark_combined_processing_latency(sample_rate);
        
        // Throughput benchmarks
        self.benchmark_throughput_different_buffer_sizes(sample_rate);
        self.benchmark_algorithm_comparison(sample_rate);
        
        // Stress benchmarks
        self.benchmark_sustained_performance(sample_rate);
        
        println!("\n✅ All benchmarks completed!");
        self.print_summary();
    }

    /// Benchmark basic audio processing latency
    pub fn benchmark_audio_processing_latency(&mut self, sample_rate: f32) {
        println!("\n📊 Benchmarking audio processing latency...");
        
        let buffer_sizes = [512, 1024, 2048, 4096];
        
        for &buffer_size in &buffer_sizes {
            let mut engine = AudioEngine::new(sample_rate, buffer_size);
            let test_buffer = vec![0.5; buffer_size];
            
            // Warmup
            for _ in 0..10 {
                let _ = engine.process_audio_buffer(&test_buffer);
            }
            
            // Benchmark
            let start = Instant::now();
            for _ in 0..1000 {
                let _ = engine.process_audio_buffer(&test_buffer);
            }
            let duration = start.elapsed() / 1000; // Average per iteration
            
            let result = BenchmarkResult::new(
                format!("audio_processing_{}samples", buffer_size),
                duration,
                buffer_size,
                sample_rate,
                None,
            );
            
            println!("  Buffer size {}: {:.3}ms ({:.1}x real-time)", 
                buffer_size, result.duration_ms, 
                (buffer_size as f64 / sample_rate as f64 * 1000.0) / result.duration_ms);
            
            self.results.push(result);
        }
    }

    /// Benchmark pitch detection latency for different algorithms
    pub fn benchmark_pitch_detection_latency(&mut self, sample_rate: f32) {
        println!("\n📊 Benchmarking pitch detection latency...");
        
        let buffer_size = 2048;
        let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod];
        
        // Generate test signal (440Hz sine wave)
        let test_buffer: Vec<f32> = (0..buffer_size)
            .map(|i| 0.8 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sample_rate).sin())
            .collect();
        
        for algorithm in &algorithms {
            let mut engine = AudioEngine::new(sample_rate, buffer_size);
            engine.set_pitch_algorithm(*algorithm);
            
            // Warmup
            for _ in 0..10 {
                let _ = engine.detect_pitch_from_buffer(&test_buffer);
            }
            
            // Benchmark
            let start = Instant::now();
            for _ in 0..100 {
                let _ = engine.detect_pitch_from_buffer(&test_buffer);
            }
            let duration = start.elapsed() / 100; // Average per iteration
            
            let result = BenchmarkResult::new(
                format!("pitch_detection_{:?}", algorithm),
                duration,
                buffer_size,
                sample_rate,
                Some(*algorithm),
            );
            
            println!("  {:?} algorithm: {:.3}ms ({:.1}x real-time)", 
                algorithm, result.duration_ms,
                (buffer_size as f64 / sample_rate as f64 * 1000.0) / result.duration_ms);
            
            self.results.push(result);
        }
    }

    /// Benchmark combined audio processing and pitch detection
    pub fn benchmark_combined_processing_latency(&mut self, sample_rate: f32) {
        println!("\n📊 Benchmarking combined processing latency...");
        
        let buffer_size = 1024;
        let mut engine = AudioEngine::new(sample_rate, buffer_size);
        
        // Generate test signal
        let test_buffer: Vec<f32> = (0..buffer_size)
            .map(|i| 0.6 * (2.0 * std::f32::consts::PI * 220.0 * i as f32 / sample_rate).sin())
            .collect();
        
        // Warmup
        for _ in 0..10 {
            let _ = engine.process_audio_with_pitch(&test_buffer);
        }
        
        // Benchmark
        let start = Instant::now();
        for _ in 0..500 {
            let _ = engine.process_audio_with_pitch(&test_buffer);
        }
        let duration = start.elapsed() / 500; // Average per iteration
        
        let result = BenchmarkResult::new(
            "combined_processing".to_string(),
            duration,
            buffer_size,
            sample_rate,
            Some(PitchAlgorithm::YIN), // Default algorithm
        );
        
        println!("  Combined processing: {:.3}ms ({:.1}x real-time)", 
            result.duration_ms,
            (buffer_size as f64 / sample_rate as f64 * 1000.0) / result.duration_ms);
        
        self.results.push(result);
    }

    /// Benchmark throughput for different buffer sizes
    pub fn benchmark_throughput_different_buffer_sizes(&mut self, sample_rate: f32) {
        println!("\n📊 Benchmarking throughput for different buffer sizes...");
        
        let buffer_sizes = [256, 512, 1024, 2048, 4096, 8192];
        
        for &buffer_size in &buffer_sizes {
            let mut engine = AudioEngine::new(sample_rate, buffer_size);
            let test_buffer = vec![0.3; buffer_size];
            
            // Warmup
            for _ in 0..5 {
                let _ = engine.process_audio_buffer(&test_buffer);
            }
            
            // Benchmark throughput (samples per second)
            let start = Instant::now();
            let iterations = 1000;
            for _ in 0..iterations {
                let _ = engine.process_audio_buffer(&test_buffer);
            }
            let duration = start.elapsed();
            let total_samples = buffer_size * iterations;
            let throughput = total_samples as f64 / duration.as_secs_f64();
            
            let avg_duration = duration / iterations as u32;
            let result = BenchmarkResult::new(
                format!("throughput_{}samples", buffer_size),
                avg_duration,
                buffer_size,
                sample_rate,
                None,
            );
            
            println!("  Buffer size {}: {:.1}M samples/sec", 
                buffer_size, throughput / 1_000_000.0);
            
            self.results.push(result);
        }
    }

    /// Compare algorithm performance
    pub fn benchmark_algorithm_comparison(&mut self, sample_rate: f32) {
        println!("\n📊 Benchmarking algorithm comparison...");
        
        let buffer_size = 2048;
        let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod];
        
        // Test different frequencies
        let test_frequencies = [110.0, 220.0, 440.0, 880.0];
        
        for algorithm in &algorithms {
            let mut total_duration = Duration::new(0, 0);
            let mut engine = AudioEngine::new(sample_rate, buffer_size);
            engine.set_pitch_algorithm(*algorithm);
            
            for &freq in &test_frequencies {
                let test_buffer: Vec<f32> = (0..buffer_size)
                    .map(|i| 0.7 * (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin())
                    .collect();
                
                // Warmup
                for _ in 0..5 {
                    let _ = engine.detect_pitch_from_buffer(&test_buffer);
                }
                
                // Benchmark
                let start = Instant::now();
                for _ in 0..50 {
                    let _ = engine.detect_pitch_from_buffer(&test_buffer);
                }
                total_duration += start.elapsed();
            }
            
            let avg_duration = total_duration / (test_frequencies.len() * 50) as u32;
            let result = BenchmarkResult::new(
                format!("algorithm_comparison_{:?}", algorithm),
                avg_duration,
                buffer_size,
                sample_rate,
                Some(*algorithm),
            );
            
            println!("  {:?}: {:.3}ms average across frequencies", 
                algorithm, result.duration_ms);
            
            self.results.push(result);
        }
    }

    /// Benchmark sustained performance (simulating real-time processing)
    pub fn benchmark_sustained_performance(&mut self, sample_rate: f32) {
        println!("\n📊 Benchmarking sustained performance...");
        
        let buffer_size = 1024;
        let mut engine = AudioEngine::new(sample_rate, buffer_size);
        let test_buffer = vec![0.4; buffer_size];
        
        // Simulate 1 second of real-time processing
        let iterations = (sample_rate / buffer_size as f32) as usize; // ~43 iterations for 44.1kHz/1024
        
        let mut durations = Vec::new();
        
        for i in 0..iterations {
            let start = Instant::now();
            let _ = engine.process_audio_with_pitch(&test_buffer);
            let duration = start.elapsed();
            durations.push(duration);
            
            // Check for performance degradation
            if i > 0 && i % 10 == 0 {
                let recent_avg = durations.iter().rev().take(10).map(|d| d.as_nanos()).sum::<u128>() / 10;
                let initial_avg = durations.iter().take(10).map(|d| d.as_nanos()).sum::<u128>() / 10;
                
                if recent_avg > initial_avg * 2 {
                    println!("  ⚠️  Performance degradation detected at iteration {}", i);
                }
            }
        }
        
        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let max_duration = durations.iter().max().unwrap();
        let min_duration = durations.iter().min().unwrap();
        
        let result = BenchmarkResult::new(
            "sustained_performance".to_string(),
            avg_duration,
            buffer_size,
            sample_rate,
            Some(PitchAlgorithm::YIN),
        );
        
        println!("  Sustained performance: avg={:.3}ms, min={:.3}ms, max={:.3}ms", 
            result.duration_ms,
            min_duration.as_nanos() as f64 / 1_000_000.0,
            max_duration.as_nanos() as f64 / 1_000_000.0);
        
        self.results.push(result);
    }

    /// Print comprehensive benchmark summary
    pub fn print_summary(&self) {
        println!("\n📋 PERFORMANCE BENCHMARK SUMMARY");
        println!("================================");
        
        // Latency requirement check
        let failed_latency: Vec<_> = self.results.iter()
            .filter(|r| !r.meets_latency_requirement)
            .collect();
        
        if failed_latency.is_empty() {
            println!("✅ All tests meet <50ms latency requirement");
        } else {
            println!("❌ {} tests failed latency requirement:", failed_latency.len());
            for result in failed_latency {
                println!("   - {}: {:.3}ms", result.test_name, result.duration_ms);
            }
        }
        
        // Best and worst performers
        if let Some(fastest) = self.results.iter().min_by_key(|r| r.duration_ns) {
            println!("🏆 Fastest test: {} ({:.3}ms)", fastest.test_name, fastest.duration_ms);
        }
        
        if let Some(slowest) = self.results.iter().max_by_key(|r| r.duration_ns) {
            println!("🐌 Slowest test: {} ({:.3}ms)", slowest.test_name, slowest.duration_ms);
        }
        
        // Throughput summary
        let throughput_results: Vec<_> = self.results.iter()
            .filter(|r| r.test_name.starts_with("throughput_"))
            .collect();
        
        if !throughput_results.is_empty() {
            let max_throughput = throughput_results.iter()
                .max_by(|a, b| a.throughput_samples_per_second.partial_cmp(&b.throughput_samples_per_second).unwrap())
                .unwrap();
            println!("⚡ Peak throughput: {:.1}M samples/sec ({})", 
                max_throughput.throughput_samples_per_second / 1_000_000.0,
                max_throughput.test_name);
        }
        
        println!("\n📊 Detailed Results:");
        for result in &self.results {
            println!("  {}: {:.3}ms | {:.1}M sps | {} latency req", 
                result.test_name,
                result.duration_ms,
                result.throughput_samples_per_second / 1_000_000.0,
                if result.meets_latency_requirement { "✅" } else { "❌" });
        }
    }

    /// Save baseline results for regression detection
    pub fn save_baseline(&mut self) {
        self.baseline_results.clear();
        for result in &self.results {
            self.baseline_results.insert(result.test_name.clone(), result.clone());
        }
        println!("💾 Baseline results saved ({} tests)", self.baseline_results.len());
    }

    /// Detect performance regressions against baseline
    pub fn detect_regressions(&self) -> Vec<(String, f64)> {
        let mut regressions = Vec::new();
        let regression_threshold = 1.5; // 50% slower is considered a regression
        
        for result in &self.results {
            if let Some(baseline) = self.baseline_results.get(&result.test_name) {
                let ratio = result.duration_ms / baseline.duration_ms;
                if ratio > regression_threshold {
                    regressions.push((result.test_name.clone(), ratio));
                }
            }
        }
        
        regressions
    }

    /// Get all benchmark results
    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_creation() {
        let duration = Duration::from_millis(5);
        let result = BenchmarkResult::new(
            "test_benchmark".to_string(),
            duration,
            1024,
            44100.0,
            Some(PitchAlgorithm::YIN),
        );

        assert_eq!(result.test_name, "test_benchmark");
        assert_eq!(result.duration_ms, 5.0);
        assert_eq!(result.buffer_size, 1024);
        assert_eq!(result.sample_rate, 44100.0);
        assert!(result.meets_latency_requirement); // 5ms < 50ms
        assert!(result.throughput_samples_per_second > 0.0);
    }

    #[test]
    fn test_benchmark_suite_creation() {
        let benchmark = PerformanceBenchmark::new();
        assert_eq!(benchmark.results.len(), 0);
        assert_eq!(benchmark.baseline_results.len(), 0);
    }

    #[test]
    fn test_latency_requirement_detection() {
        let fast_duration = Duration::from_millis(10);
        let slow_duration = Duration::from_millis(60);

        let fast_result = BenchmarkResult::new(
            "fast_test".to_string(),
            fast_duration,
            1024,
            44100.0,
            None,
        );

        let slow_result = BenchmarkResult::new(
            "slow_test".to_string(),
            slow_duration,
            1024,
            44100.0,
            None,
        );

        assert!(fast_result.meets_latency_requirement);
        assert!(!slow_result.meets_latency_requirement);
    }
} 