// Buffer Pool Performance Comparison Tests
// 
// This test module compares the performance of the ping-pong buffer pool pattern
// vs direct allocation for audio buffer management.

use wasm_bindgen_test::*;
use js_sys::ArrayBuffer;
use wasm_bindgen::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Performance metrics for buffer allocation comparison
#[derive(Default)]
struct PerformanceMetrics {
    total_allocations: u32,
    total_time_ms: f64,
    min_time_ms: f64,
    max_time_ms: f64,
    avg_time_ms: f64,
    gc_pauses_detected: u32,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            min_time_ms: f64::INFINITY,
            ..Default::default()
        }
    }
    
    fn record_allocation(&mut self, time_ms: f64) {
        self.total_allocations += 1;
        self.total_time_ms += time_ms;
        self.min_time_ms = self.min_time_ms.min(time_ms);
        self.max_time_ms = self.max_time_ms.max(time_ms);
        self.avg_time_ms = self.total_time_ms / self.total_allocations as f64;
    }
    
    fn print_summary(&self, name: &str) {
        console_log!("\n=== {} Performance Summary ===", name);
        console_log!("Total allocations: {}", self.total_allocations);
        console_log!("Total time: {:.2}ms", self.total_time_ms);
        console_log!("Average time: {:.3}ms", self.avg_time_ms);
        console_log!("Min time: {:.3}ms", self.min_time_ms);
        console_log!("Max time: {:.3}ms", self.max_time_ms);
        console_log!("GC pauses detected: {}", self.gc_pauses_detected);
    }
}

/// Test direct allocation performance
fn benchmark_direct_allocation(iterations: u32, buffer_size: usize) -> PerformanceMetrics {
    let mut metrics = PerformanceMetrics::new();
    let mut last_time = now();
    
    console_log!("Starting direct allocation benchmark...");
    
    for i in 0..iterations {
        let start = now();
        
        // Detect potential GC pause
        if start - last_time > 50.0 {
            metrics.gc_pauses_detected += 1;
            console_log!("GC pause detected at iteration {}: {:.2}ms", i, start - last_time);
        }
        last_time = start;
        
        // Direct allocation
        let _buffer = ArrayBuffer::new(buffer_size as u32);
        
        let elapsed = now() - start;
        metrics.record_allocation(elapsed);
        
        // Log progress every 1000 iterations
        if i % 1000 == 0 {
            console_log!("Direct allocation progress: {}/{}", i, iterations);
        }
    }
    
    metrics
}

/// Test buffer pool performance
fn benchmark_buffer_pool(iterations: u32, buffer_size: usize, pool_size: usize) -> PerformanceMetrics {
    let mut metrics = PerformanceMetrics::new();
    let mut last_time = now();
    
    console_log!("Starting buffer pool benchmark...");
    
    // Create a simple buffer pool simulation
    let mut pool: Vec<ArrayBuffer> = Vec::with_capacity(pool_size);
    let mut in_use: Vec<ArrayBuffer> = Vec::new();
    
    // Pre-allocate pool
    for _ in 0..pool_size {
        pool.push(ArrayBuffer::new(buffer_size as u32));
    }
    
    for i in 0..iterations {
        let start = now();
        
        // Detect potential GC pause
        if start - last_time > 50.0 {
            metrics.gc_pauses_detected += 1;
            console_log!("GC pause detected at iteration {}: {:.2}ms", i, start - last_time);
        }
        last_time = start;
        
        // Acquire from pool or allocate
        let buffer = if let Some(buf) = pool.pop() {
            buf
        } else {
            // Pool exhausted, need to allocate
            metrics.total_allocations += 1;
            ArrayBuffer::new(buffer_size as u32)
        };
        
        in_use.push(buffer);
        
        // Simulate buffer return (every 4 acquisitions)
        if i % 4 == 3 && in_use.len() > 0 {
            // Return oldest buffers to pool
            while in_use.len() > pool_size / 2 && pool.len() < pool_size {
                let returned = in_use.remove(0);
                pool.push(returned);
            }
        }
        
        let elapsed = now() - start;
        metrics.record_allocation(elapsed);
        
        // Log progress every 1000 iterations
        if i % 1000 == 0 {
            console_log!("Buffer pool progress: {}/{} (pool: {}, in use: {})", 
                        i, iterations, pool.len(), in_use.len());
        }
    }
    
    // Count actual allocations (initial pool + any extras)
    metrics.total_allocations = pool_size as u32 + metrics.total_allocations;
    
    metrics
}

#[wasm_bindgen_test]
async fn test_buffer_allocation_performance_comparison() {
    console_log!("\n🔬 Buffer Allocation Performance Comparison Test");
    
    const ITERATIONS: u32 = 10000;
    const BUFFER_SIZE: usize = 4096; // 1024 samples * 4 bytes
    const POOL_SIZE: usize = 16;
    
    // Warm up
    console_log!("\nWarming up...");
    for _ in 0..100 {
        let _ = ArrayBuffer::new(BUFFER_SIZE as u32);
    }
    
    // Run benchmarks
    console_log!("\nRunning benchmarks with {} iterations, buffer size: {} bytes", 
                 ITERATIONS, BUFFER_SIZE);
    
    // Direct allocation benchmark
    let direct_metrics = benchmark_direct_allocation(ITERATIONS, BUFFER_SIZE);
    
    // Small delay between tests
    let delay = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 1000)
            .unwrap();
    });
    wasm_bindgen_futures::JsFuture::from(delay).await.unwrap();
    
    // Buffer pool benchmark
    let pool_metrics = benchmark_buffer_pool(ITERATIONS, BUFFER_SIZE, POOL_SIZE);
    
    // Print results
    direct_metrics.print_summary("Direct Allocation");
    pool_metrics.print_summary("Buffer Pool");
    
    // Calculate improvements
    console_log!("\n=== Performance Comparison ===");
    let allocation_reduction = ((direct_metrics.total_allocations as f64 - pool_metrics.total_allocations as f64) 
                                / direct_metrics.total_allocations as f64) * 100.0;
    console_log!("Allocation reduction: {:.1}%", allocation_reduction);
    
    let gc_pause_reduction = if direct_metrics.gc_pauses_detected > 0 {
        ((direct_metrics.gc_pauses_detected as f64 - pool_metrics.gc_pauses_detected as f64) 
         / direct_metrics.gc_pauses_detected as f64) * 100.0
    } else {
        0.0
    };
    console_log!("GC pause reduction: {:.1}%", gc_pause_reduction);
    
    // Assertions
    assert!(pool_metrics.total_allocations < direct_metrics.total_allocations,
            "Buffer pool should allocate fewer buffers");
    assert!(allocation_reduction > 90.0,
            "Buffer pool should reduce allocations by at least 90%");
}

#[wasm_bindgen_test]
async fn test_pool_performance_under_load() {
    console_log!("\n🔬 Buffer Pool Performance Under Load Test");
    
    const BUFFER_SIZE: usize = 4096;
    const POOL_SIZES: &[usize] = &[4, 8, 16, 32];
    const ITERATIONS: u32 = 5000;
    
    for &pool_size in POOL_SIZES {
        console_log!("\nTesting pool size: {}", pool_size);
        
        let metrics = benchmark_buffer_pool(ITERATIONS, BUFFER_SIZE, pool_size);
        metrics.print_summary(&format!("Pool Size {}", pool_size));
        
        // Calculate pool efficiency
        let expected_allocations = pool_size as u32;
        let extra_allocations = metrics.total_allocations - expected_allocations;
        let efficiency = (1.0 - (extra_allocations as f64 / ITERATIONS as f64)) * 100.0;
        
        console_log!("Pool efficiency: {:.1}%", efficiency);
        console_log!("Extra allocations: {} ({:.1}% of iterations)", 
                     extra_allocations, 
                     (extra_allocations as f64 / ITERATIONS as f64) * 100.0);
    }
}

#[wasm_bindgen_test]
async fn test_gc_pause_detection() {
    console_log!("\n🔬 GC Pause Detection Test");
    
    const ITERATIONS: u32 = 5000;
    const BUFFER_SIZE: usize = 8192; // Larger buffers to increase GC pressure
    
    let mut metrics = PerformanceMetrics::new();
    let mut last_time = now();
    let mut buffers = Vec::new();
    
    console_log!("Creating GC pressure with large allocations...");
    
    for i in 0..ITERATIONS {
        let start = now();
        
        // Detect GC pause
        let time_since_last = start - last_time;
        if time_since_last > 10.0 { // Lower threshold for detection
            metrics.gc_pauses_detected += 1;
            console_log!("GC pause at iteration {}: {:.2}ms", i, time_since_last);
        }
        last_time = start;
        
        // Allocate and hold some buffers to increase memory pressure
        let buffer = ArrayBuffer::new(BUFFER_SIZE as u32);
        buffers.push(buffer);
        
        // Periodically clear to trigger GC
        if i % 500 == 0 && i > 0 {
            console_log!("Clearing buffers at iteration {}", i);
            buffers.clear();
        }
        
        let elapsed = now() - start;
        metrics.record_allocation(elapsed);
    }
    
    metrics.print_summary("GC Pause Detection");
    console_log!("\nNote: GC pause detection is probabilistic and may vary between runs");
}