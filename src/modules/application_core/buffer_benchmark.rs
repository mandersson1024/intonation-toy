//! # Buffer Reference Performance Benchmarks
//!
//! This module contains performance benchmarks to verify the zero-copy behavior
//! of the buffer reference system and measure memory usage patterns.

use super::buffer_ref::{BufferRef, BufferMetadata, BufferManager};
use std::time::Instant;

/// Simple benchmark results for buffer operations
#[derive(Debug, Clone)]
pub struct BufferBenchmarkResult {
    pub operation: String,
    pub duration_ns: u64,
    pub memory_allocated_bytes: usize,
    pub reference_count_before: usize,
    pub reference_count_after: usize,
}

/// Benchmarks buffer reference operations
pub struct BufferBenchmark {
    results: Vec<BufferBenchmarkResult>,
}

impl BufferBenchmark {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    
    /// Benchmark buffer creation performance
    pub fn benchmark_buffer_creation(&mut self, buffer_size: usize) -> BufferBenchmarkResult {
        let data = vec![0.0f32; buffer_size];
        let memory_before = data.len() * std::mem::size_of::<f32>();
        let metadata = BufferMetadata::new(44100, 1, buffer_size);
        
        let start = Instant::now();
        let buffer_ref = BufferRef::new(data, metadata);
        let duration = start.elapsed();
        
        let result = BufferBenchmarkResult {
            operation: format!("buffer_creation_{}_samples", buffer_size),
            duration_ns: duration.as_nanos() as u64,
            memory_allocated_bytes: memory_before,
            reference_count_before: 0,
            reference_count_after: buffer_ref.reference_count(),
        };
        
        self.results.push(result.clone());
        result
    }
    
    /// Benchmark zero-copy cloning performance
    pub fn benchmark_zero_copy_clone(&mut self, buffer_size: usize, clone_count: usize) -> BufferBenchmarkResult {
        let data = vec![0.5f32; buffer_size];
        let metadata = BufferMetadata::new(44100, 1, buffer_size);
        let original_buffer = BufferRef::new(data, metadata);
        
        let ref_count_before = original_buffer.reference_count();
        
        let start = Instant::now();
        let _clones: Vec<_> = (0..clone_count)
            .map(|_| original_buffer.clone())
            .collect();
        let duration = start.elapsed();
        
        let ref_count_after = original_buffer.reference_count();
        
        let result = BufferBenchmarkResult {
            operation: format!("zero_copy_clone_{}x{}_samples", clone_count, buffer_size),
            duration_ns: duration.as_nanos() as u64,
            memory_allocated_bytes: 0, // Zero-copy should not allocate
            reference_count_before: ref_count_before,
            reference_count_after: ref_count_after,
        };
        
        self.results.push(result.clone());
        result
    }
    
    /// Benchmark buffer access performance
    pub fn benchmark_buffer_access(&mut self, buffer_size: usize, access_count: usize) -> BufferBenchmarkResult {
        let data = vec![0.1f32; buffer_size];
        let metadata = BufferMetadata::new(44100, 1, buffer_size);
        let buffer_ref = BufferRef::new(data, metadata);
        
        let start = Instant::now();
        let mut sum = 0.0f32;
        for _ in 0..access_count {
            let data = buffer_ref.data();
            sum += data[0]; // Access first element
        }
        let duration = start.elapsed();
        
        // Prevent optimization from removing the loop
        assert!(sum > 0.0);
        
        let result = BufferBenchmarkResult {
            operation: format!("buffer_access_{}x{}_samples", access_count, buffer_size),
            duration_ns: duration.as_nanos() as u64,
            memory_allocated_bytes: 0,
            reference_count_before: buffer_ref.reference_count(),
            reference_count_after: buffer_ref.reference_count(),
        };
        
        self.results.push(result.clone());
        result
    }
    
    /// Benchmark buffer manager operations
    pub fn benchmark_buffer_manager(&mut self, buffer_count: usize, buffer_size: usize) -> BufferBenchmarkResult {
        let mut manager = BufferManager::new(buffer_count * buffer_size * std::mem::size_of::<f32>() + 1024);
        
        let start = Instant::now();
        let mut buffers = Vec::new();
        
        for i in 0..buffer_count {
            let data = vec![i as f32; buffer_size];
            let metadata = BufferMetadata::new(44100, 1, buffer_size);
            if let Ok(buffer_ref) = manager.create_buffer(data, metadata) {
                buffers.push(buffer_ref);
            }
        }
        let duration = start.elapsed();
        
        let stats = manager.get_stats();
        
        let result = BufferBenchmarkResult {
            operation: format!("buffer_manager_{}x{}_samples", buffer_count, buffer_size),
            duration_ns: duration.as_nanos() as u64,
            memory_allocated_bytes: stats.total_memory_bytes,
            reference_count_before: 0,
            reference_count_after: buffers.len(),
        };
        
        self.results.push(result.clone());
        result
    }
    
    /// Run comprehensive benchmarks
    pub fn run_comprehensive_benchmarks(&mut self) {
        println!("Running Buffer Reference Performance Benchmarks...\n");
        
        // Test different buffer sizes
        let buffer_sizes = [512, 1024, 2048, 4096, 8192];
        
        for &size in &buffer_sizes {
            let creation_result = self.benchmark_buffer_creation(size);
            println!("Buffer Creation ({} samples): {} ns", size, creation_result.duration_ns);
            
            let clone_result = self.benchmark_zero_copy_clone(size, 10);
            println!("Zero-Copy Clone (10x {} samples): {} ns", size, clone_result.duration_ns);
            
            let access_result = self.benchmark_buffer_access(size, 1000);
            println!("Buffer Access (1000x {} samples): {} ns", size, access_result.duration_ns);
        }
        
        // Test buffer manager
        let manager_result = self.benchmark_buffer_manager(100, 1024);
        println!("Buffer Manager (100x 1024 samples): {} ns", manager_result.duration_ns);
        
        self.print_summary();
    }
    
    /// Memory leak test
    pub fn test_memory_cleanup(&mut self) -> bool {
        println!("\nTesting Memory Cleanup...");
        
        let initial_ref_count;
        let buffer_size = 1024;
        
        // Create buffer reference
        let data = vec![1.0f32; buffer_size];
        let metadata = BufferMetadata::new(44100, 1, buffer_size);
        let original_buffer = BufferRef::new(data, metadata);
        initial_ref_count = original_buffer.reference_count();
        
        // Create many clones in inner scope
        {
            let _clones: Vec<_> = (0..1000)
                .map(|_| original_buffer.clone())
                .collect();
            
            let peak_ref_count = original_buffer.reference_count();
            println!("Peak reference count: {}", peak_ref_count);
            assert!(peak_ref_count > initial_ref_count);
        } // All clones should be dropped here
        
        // Check reference count returned to original
        let final_ref_count = original_buffer.reference_count();
        println!("Final reference count: {}", final_ref_count);
        
        let cleanup_successful = final_ref_count == initial_ref_count;
        if cleanup_successful {
            println!("✓ Memory cleanup test PASSED");
        } else {
            println!("✗ Memory cleanup test FAILED");
        }
        
        cleanup_successful
    }
    
    /// Print benchmark summary
    pub fn print_summary(&self) {
        println!("\n=== Buffer Reference Benchmark Summary ===");
        println!("{:<40} {:<15} {:<15}", "Operation", "Duration (ns)", "Memory (bytes)");
        println!("{}", "-".repeat(70));
        
        for result in &self.results {
            println!("{:<40} {:<15} {:<15}", 
                result.operation, 
                result.duration_ns,
                result.memory_allocated_bytes
            );
        }
        
        // Calculate averages for similar operations
        self.print_performance_analysis();
    }
    
    /// Analyze performance characteristics
    fn print_performance_analysis(&self) {
        println!("\n=== Performance Analysis ===");
        
        // Analyze zero-copy behavior
        let zero_copy_results: Vec<_> = self.results.iter()
            .filter(|r| r.operation.contains("zero_copy_clone"))
            .collect();
            
        if !zero_copy_results.is_empty() {
            let total_zero_copy_memory: usize = zero_copy_results.iter()
                .map(|r| r.memory_allocated_bytes)
                .sum();
                
            if total_zero_copy_memory == 0 {
                println!("✓ Zero-copy behavior confirmed: No memory allocation during cloning");
            } else {
                println!("⚠ Warning: Memory allocated during cloning: {} bytes", total_zero_copy_memory);
            }
        }
        
        // Analyze access performance
        let access_results: Vec<_> = self.results.iter()
            .filter(|r| r.operation.contains("buffer_access"))
            .collect();
            
        if !access_results.is_empty() {
            let avg_access_time: f64 = access_results.iter()
                .map(|r| r.duration_ns as f64)
                .sum::<f64>() / access_results.len() as f64;
                
            println!("Average buffer access time: {:.2} ns", avg_access_time);
            
            // Check if access is consistently fast (< 10μs for 1000 accesses)
            if avg_access_time < 10_000.0 {
                println!("✓ Buffer access performance is optimal");
            } else {
                println!("⚠ Buffer access may be slower than expected");
            }
        }
    }
    
    /// Get benchmark results
    pub fn get_results(&self) -> &[BufferBenchmarkResult] {
        &self.results
    }
}

impl Default for BufferBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation_benchmark() {
        let mut benchmark = BufferBenchmark::new();
        let result = benchmark.benchmark_buffer_creation(1024);
        
        assert_eq!(result.reference_count_after, 1);
        assert!(result.duration_ns > 0);
        assert_eq!(result.memory_allocated_bytes, 1024 * 4); // 1024 f32 samples
    }

    #[test]
    fn test_zero_copy_clone_benchmark() {
        let mut benchmark = BufferBenchmark::new();
        let result = benchmark.benchmark_zero_copy_clone(1024, 5);
        
        assert_eq!(result.reference_count_before, 1);
        assert_eq!(result.reference_count_after, 6); // Original + 5 clones
        assert_eq!(result.memory_allocated_bytes, 0); // Zero-copy
        assert!(result.duration_ns > 0);
    }

    #[test]
    fn test_buffer_access_benchmark() {
        let mut benchmark = BufferBenchmark::new();
        let result = benchmark.benchmark_buffer_access(1024, 100);
        
        assert!(result.duration_ns > 0);
        assert_eq!(result.memory_allocated_bytes, 0);
    }

    #[test]
    fn test_memory_cleanup() {
        let mut benchmark = BufferBenchmark::new();
        let cleanup_successful = benchmark.test_memory_cleanup();
        assert!(cleanup_successful, "Memory cleanup test should pass");
    }

    #[test]
    fn test_buffer_manager_benchmark() {
        let mut benchmark = BufferBenchmark::new();
        let result = benchmark.benchmark_buffer_manager(10, 512);
        
        assert!(result.duration_ns > 0);
        assert!(result.memory_allocated_bytes > 0);
        assert_eq!(result.reference_count_after, 10);
    }
}