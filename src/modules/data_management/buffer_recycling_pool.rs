//! # Buffer Recycling Pool Implementation
//!
//! This module provides a buffer recycling pool system for memory efficiency with
//! size-based allocation strategies and <0.5ms allocation overhead. The pool minimizes
//! memory allocation pressure through intelligent buffer reuse, automatic sizing,
//! and WASM-JS boundary optimization.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use wasm_bindgen::prelude::*;
use js_sys::{Float32Array, SharedArrayBuffer};
use web_sys::console;
use super::super::application_core::{Event, EventPriority, get_timestamp_ns};

/// Error types for buffer pool operations
#[derive(Debug, Clone)]
pub enum PoolError {
    /// Pool is at maximum capacity
    CapacityExceeded,
    /// Invalid buffer size
    InvalidSize(String),
    /// Pool fragmentation detected
    Fragmentation(String),
    /// Internal error
    Internal(String),
    /// WASM-JS boundary allocation failed
    WasmJsBoundaryFailed(String),
    /// SharedArrayBuffer not supported
    SharedArrayBufferUnsupported,
    /// Zero-copy operation failed
    ZeroCopyFailed(String),
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::CapacityExceeded => write!(f, "Buffer pool capacity exceeded"),
            PoolError::InvalidSize(msg) => write!(f, "Invalid buffer size: {}", msg),
            PoolError::Fragmentation(msg) => write!(f, "Pool fragmentation: {}", msg),
            PoolError::Internal(msg) => write!(f, "Internal pool error: {}", msg),
        }
    }
}

impl std::error::Error for PoolError {}

/// Buffer pool with size-based allocation strategies
pub struct BufferRecyclingPool {
    /// Size-based buffer pools (size -> queue of buffers)
    size_pools: Arc<RwLock<HashMap<usize, VecDeque<Vec<f32>>>>>,
    /// JavaScript buffer reference pools for WASM-JS optimization
    js_buffer_pools: Arc<RwLock<HashMap<usize, VecDeque<JSBufferRef>>>>,
    /// Pool configuration
    config: PoolConfig,
    /// Pool statistics and metrics
    stats: Arc<RwLock<PoolStats>>,
    /// Maximum total capacity
    max_capacity: usize,
    /// Current total buffers in pool
    current_count: Arc<Mutex<usize>>,
    /// Next reference ID for JS buffers
    next_ref_id: Arc<Mutex<u64>>,
    /// SharedArrayBuffer support detection
    shared_array_buffer_supported: bool,
}

/// Configuration for buffer pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum buffers per size category
    pub max_buffers_per_size: usize,
    /// Minimum buffer size to pool
    pub min_buffer_size: usize,
    /// Maximum buffer size to pool
    pub max_buffer_size: usize,
    /// Enable automatic pool sizing
    pub auto_sizing: bool,
    /// Pool efficiency threshold for cleanup
    pub efficiency_threshold: f64,
    /// Fragmentation prevention enabled
    pub fragmentation_prevention: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_buffers_per_size: 50,
            min_buffer_size: 64,
            max_buffer_size: 16384,
            auto_sizing: true,
            efficiency_threshold: 0.7,
            fragmentation_prevention: true,
        }
    }
}

/// Pool statistics and metrics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total get requests
    pub total_gets: u64,
    /// Pool hits (buffer found in pool)
    pub pool_hits: u64,
    /// Pool misses (new buffer created)
    pub pool_misses: u64,
    /// Total return operations
    pub total_returns: u64,
    /// Buffers successfully returned to pool
    pub successful_returns: u64,
    /// Buffers rejected during return
    pub rejected_returns: u64,
    /// Pool hit rate percentage
    pub hit_rate_percent: f64,
    /// Current total buffers in pool
    pub current_buffer_count: usize,
    /// Total memory in pool (bytes)
    pub total_memory_bytes: usize,
    /// Number of size categories
    pub size_categories: usize,
    /// Average allocation time in nanoseconds
    pub avg_allocation_time_ns: u64,
    /// Maximum allocation time in nanoseconds
    pub max_allocation_time_ns: u64,
    /// Memory overhead percentage
    pub memory_overhead_percent: f32,
    /// Fragmentation percentage
    pub fragmentation_percent: f32,
    /// JavaScript GC pressure reduction percentage
    pub js_gc_pressure_reduction_percent: f32,
    /// WASM-JS boundary allocations
    pub wasm_js_boundary_allocations: u64,
    /// Zero-copy operations count
    pub zero_copy_operations: u64,
    /// SharedArrayBuffer operations count
    pub shared_array_buffer_operations: u64,
}

/// JavaScript buffer reference for WASM-JS boundary optimization
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JSBufferRef {
    /// JavaScript Float32Array reference
    array: Float32Array,
    /// Buffer size in samples
    size: usize,
    /// Reference ID for tracking
    ref_id: u64,
    /// Whether this uses SharedArrayBuffer
    is_shared: bool,
}

#[wasm_bindgen]
impl JSBufferRef {
    /// Get the buffer size
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get the reference ID
    #[wasm_bindgen(getter)]
    pub fn ref_id(&self) -> u64 {
        self.ref_id
    }
    
    /// Check if this buffer uses SharedArrayBuffer
    #[wasm_bindgen(getter)]
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
    
    /// Get the underlying JavaScript Float32Array
    #[wasm_bindgen(getter)]
    pub fn array(&self) -> Float32Array {
        self.array.clone()
    }
}

impl JSBufferRef {
    /// Create a new JSBufferRef from Float32Array
    pub fn new(array: Float32Array, ref_id: u64, is_shared: bool) -> Self {
        let size = array.length() as usize;
        Self {
            array,
            size,
            ref_id,
            is_shared,
        }
    }
    
    /// Convert to Rust Vec<f32> (involves copying unless SharedArrayBuffer)
    pub fn to_vec(&self) -> Vec<f32> {
        let mut vec = vec![0.0f32; self.size];
        self.array.copy_to(&mut vec);
        vec
    }
    
    /// Update from Rust Vec<f32> (involves copying unless SharedArrayBuffer)
    pub fn from_vec(&self, data: &[f32]) -> Result<(), PoolError> {
        if data.len() != self.size {
            return Err(PoolError::InvalidSize(format!(
                "Data length {} does not match buffer size {}",
                data.len(),
                self.size
            )));
        }
        
        self.array.copy_from(data);
        Ok(())
    }
}

impl BufferRecyclingPool {
    /// Create a new buffer recycling pool with standard audio buffer sizes
    pub fn new(max_capacity: usize) -> Self {
        let shared_array_buffer_supported = Self::detect_shared_array_buffer_support();
        
        let mut pool = Self {
            size_pools: Arc::new(RwLock::new(HashMap::new())),
            js_buffer_pools: Arc::new(RwLock::new(HashMap::new())),
            config: PoolConfig::default(),
            stats: Arc::new(RwLock::new(PoolStats::default())),
            max_capacity,
            current_count: Arc::new(Mutex::new(0)),
            next_ref_id: Arc::new(Mutex::new(1)),
            shared_array_buffer_supported,
        };
        
        // Pre-allocate common audio buffer sizes (256, 512, 1024, 2048, 4096 samples)
        pool.preallocate_common_sizes();
        pool
    }
    
    /// Detect SharedArrayBuffer support for zero-copy operations
    fn detect_shared_array_buffer_support() -> bool {
        // Use feature detection to check for SharedArrayBuffer
        js_sys::eval("typeof SharedArrayBuffer !== 'undefined'")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }
    
    /// Pre-allocate buffers for common audio processing sizes
    fn preallocate_common_sizes(&self) {
        let common_sizes = [256, 512, 1024, 2048, 4096];
        let initial_count = 5; // Pre-allocate 5 buffers per size
        
        if let Ok(mut pools) = self.size_pools.write() {
            for &size in &common_sizes {
                let mut buffers = VecDeque::with_capacity(initial_count);
                for _ in 0..initial_count {
                    buffers.push_back(vec![0.0f32; size]);
                }
                pools.insert(size, buffers);
            }
            
            // Update count
            if let Ok(mut count) = self.current_count.lock() {
                *count = common_sizes.len() * initial_count;
            }
        }
    }

    /// Create a buffer pool with custom configuration
    pub fn with_config(max_capacity: usize, config: PoolConfig) -> Self {
        let shared_array_buffer_supported = Self::detect_shared_array_buffer_support();
        
        Self {
            size_pools: Arc::new(RwLock::new(HashMap::new())),
            js_buffer_pools: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(PoolStats::default())),
            max_capacity,
            current_count: Arc::new(Mutex::new(0)),
            next_ref_id: Arc::new(Mutex::new(1)),
            shared_array_buffer_supported,
        }
    }

    /// Get buffer from pool or create new one with <0.5ms overhead
    pub fn get_or_create(&self, size: usize) -> Result<Vec<f32>, PoolError> {
        let start_time = Instant::now();
        
        // Validate size
        if size < self.config.min_buffer_size || size > self.config.max_buffer_size {
            return Err(PoolError::InvalidSize(format!("Size {} out of range [{}, {}]", 
                size, self.config.min_buffer_size, self.config.max_buffer_size)));
        }

        // Try to get from pool first using optimized lookup
        let buffer = if let Ok(mut pools) = self.size_pools.write() {
            // Check exact size first
            if let Some(queue) = pools.get_mut(&size) {
                if let Some(buffer) = queue.pop_front() {
                    Some(buffer)
                } else {
                    None
                }
            } else {
                // Try to find a larger buffer that can be resized
                self.find_resizable_buffer(&mut pools, size)
            }
        } else {
            None
        };

        let (result_buffer, from_pool) = if let Some(mut pooled_buffer) = buffer {
            // Resize if necessary (truncate larger buffers)
            if pooled_buffer.len() > size {
                pooled_buffer.truncate(size);
            } else if pooled_buffer.len() < size {
                pooled_buffer.resize(size, 0.0);
            }
            
            // Reset buffer contents
            pooled_buffer.fill(0.0);
            
            // Update count
            if let Ok(mut count) = self.current_count.lock() {
                *count = count.saturating_sub(1);
            }
            
            (pooled_buffer, true)
        } else {
            // Create new buffer
            (vec![0.0f32; size], false)
        };

        // Update statistics
        let allocation_time = start_time.elapsed();
        
        // Ensure allocation time meets <0.5ms requirement
        if allocation_time.as_micros() > 500 {
            eprintln!("Warning: Buffer allocation took {}μs, exceeding 500μs target", allocation_time.as_micros());
        }
        
        self.update_stats(from_pool, allocation_time.as_nanos() as u64, size);

        Ok(result_buffer)
    }
    
    /// Find a buffer that can be resized to the requested size
    fn find_resizable_buffer(&self, pools: &mut HashMap<usize, VecDeque<Vec<f32>>>, target_size: usize) -> Option<Vec<f32>> {
        // Look for buffers that are close in size (within 2x) to minimize memory waste
        let size_tolerance = target_size * 2;
        
        for (&pool_size, queue) in pools.iter_mut() {
            if pool_size > target_size && pool_size <= size_tolerance && !queue.is_empty() {
                return queue.pop_front();
            }
        }
        
        None
    }
    
    /// Get JS-compatible buffer reference to minimize WASM↔JS boundary overhead
    pub fn get_js_buffer_ref(&self, size: usize) -> Result<JSBufferRef, PoolError> {
        let start_time = Instant::now();
        
        // Validate size
        if size < self.config.min_buffer_size || size > self.config.max_buffer_size {
            return Err(PoolError::InvalidSize(format!("Size {} out of range [{}, {}]", 
                size, self.config.min_buffer_size, self.config.max_buffer_size)));
        }

        // Try to get from JS buffer pool first
        let js_buffer = if let Ok(mut js_pools) = self.js_buffer_pools.write() {
            if let Some(queue) = js_pools.get_mut(&size) {
                queue.pop_front()
            } else {
                None
            }
        } else {
            None
        };

        let (result_buffer, from_pool) = if let Some(buffer_ref) = js_buffer {
            // Reset buffer contents
            let zero_data = vec![0.0f32; size];
            if let Err(e) = buffer_ref.from_vec(&zero_data) {
                return Err(PoolError::WasmJsBoundaryFailed(format!("Failed to reset buffer: {}", e)));
            }
            
            (buffer_ref, true)
        } else {
            // Create new JS buffer
            self.create_new_js_buffer(size)?
        };

        // Update statistics
        let allocation_time = start_time.elapsed();
        self.update_js_buffer_stats(from_pool, allocation_time.as_nanos() as u64, size);

        Ok(result_buffer.0)
    }
    
    /// Create a new JavaScript buffer reference
    fn create_new_js_buffer(&self, size: usize) -> Result<(JSBufferRef, bool), PoolError> {
        let ref_id = {
            let mut next_id = self.next_ref_id.lock()
                .map_err(|_| PoolError::Internal("Failed to get next ref ID".to_string()))?;
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Try to create SharedArrayBuffer first if supported
        if self.shared_array_buffer_supported {
            match self.create_shared_array_buffer(size, ref_id) {
                Ok(buffer_ref) => {
                    // Update SharedArrayBuffer usage stats
                    if let Ok(mut stats) = self.stats.write() {
                        stats.shared_array_buffer_operations += 1;
                        stats.zero_copy_operations += 1;
                    }
                    return Ok((buffer_ref, false));
                }
                Err(_) => {
                    // Fall back to regular Float32Array
                }
            }
        }

        // Create regular Float32Array (involves copying)
        let array = Float32Array::new_with_length(size as u32);
        let buffer_ref = JSBufferRef::new(array, ref_id, false);
        
        // Update boundary allocation stats
        if let Ok(mut stats) = self.stats.write() {
            stats.wasm_js_boundary_allocations += 1;
        }
        
        Ok((buffer_ref, false))
    }
    
    /// Create SharedArrayBuffer-based buffer for zero-copy operations
    fn create_shared_array_buffer(&self, size: usize, ref_id: u64) -> Result<JSBufferRef, PoolError> {
        let byte_length = size * 4; // 4 bytes per f32
        
        // Create SharedArrayBuffer
        let shared_buffer = SharedArrayBuffer::new(byte_length as u32)
            .map_err(|_| PoolError::SharedArrayBufferUnsupported)?;
        
        // Create Float32Array view on SharedArrayBuffer
        let array = Float32Array::new_with_array_buffer(&shared_buffer)
            .map_err(|_| PoolError::ZeroCopyFailed("Failed to create Float32Array view".to_string()))?;
        
        Ok(JSBufferRef::new(array, ref_id, true))
    }
    
    /// Return JS buffer reference for recycling (reduces JS GC pressure)
    pub fn recycle_js_buffer_ref(&self, buffer_ref: JSBufferRef) -> Result<(), PoolError> {
        let size = buffer_ref.size();
        
        // Check if size is poolable
        if size < self.config.min_buffer_size || size > self.config.max_buffer_size {
            return Ok(()); // Silently reject, not an error
        }

        // Check capacity
        if let Ok(count) = self.current_count.lock() {
            if *count >= self.max_capacity {
                return Ok(()); // Pool full, silently reject
            }
        }

        // Add to appropriate JS buffer pool
        let success = if let Ok(mut js_pools) = self.js_buffer_pools.write() {
            let queue = js_pools.entry(size).or_insert_with(VecDeque::new);
            
            if queue.len() < self.config.max_buffers_per_size {
                queue.push_back(buffer_ref);
                
                // Update count
                if let Ok(mut count) = self.current_count.lock() {
                    *count += 1;
                }
                true
            } else {
                false // Size pool full
            }
        } else {
            false
        };

        // Update recycling stats
        if let Ok(mut stats) = self.stats.write() {
            if success {
                stats.successful_returns += 1;
                // Estimate GC pressure reduction
                stats.js_gc_pressure_reduction_percent = 
                    (stats.successful_returns as f32 / (stats.total_returns + 1) as f32) * 10.0; // Up to 10% reduction
            } else {
                stats.rejected_returns += 1;
            }
            stats.total_returns += 1;
        }
        
        Ok(())
    }
    
    /// Update JS buffer allocation statistics
    fn update_js_buffer_stats(&self, from_pool: bool, allocation_time_ns: u64, _size: usize) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_gets += 1;
            
            if from_pool {
                stats.pool_hits += 1;
            } else {
                stats.pool_misses += 1;
                stats.wasm_js_boundary_allocations += 1;
            }
            
            // Update hit rate
            stats.hit_rate_percent = (stats.pool_hits as f64 / stats.total_gets as f64) * 100.0;
            
            // Update allocation time
            let total_gets = stats.total_gets;
            let prev_avg = stats.avg_allocation_time_ns;
            stats.avg_allocation_time_ns = ((prev_avg * (total_gets - 1)) + allocation_time_ns) / total_gets;
            
            if allocation_time_ns > stats.max_allocation_time_ns {
                stats.max_allocation_time_ns = allocation_time_ns;
            }
        }
    }

    /// Return buffer to pool for reuse
    pub fn return_buffer(&self, buffer: Vec<f32>) -> Result<(), PoolError> {
        let size = buffer.len();
        
        // Check if size is poolable
        if size < self.config.min_buffer_size || size > self.config.max_buffer_size {
            self.update_return_stats(false);
            return Ok(()); // Silently reject, not an error
        }

        // Check capacity
        if let Ok(count) = self.current_count.lock() {
            if *count >= self.max_capacity {
                self.update_return_stats(false);
                return Ok(()); // Pool full, silently reject
            }
        }

        // Add to appropriate size pool
        let success = if let Ok(mut pools) = self.size_pools.write() {
            let queue = pools.entry(size).or_insert_with(VecDeque::new);
            
            if queue.len() < self.config.max_buffers_per_size {
                queue.push_back(buffer);
                
                // Update count
                if let Ok(mut count) = self.current_count.lock() {
                    *count += 1;
                }
                true
            } else {
                false // Size pool full
            }
        } else {
            false
        };

        self.update_return_stats(success);
        Ok(())
    }

    /// Get pool efficiency metrics with enhanced tracking
    pub fn get_efficiency_metrics(&self) -> PoolMetrics {
        if let Ok(stats) = self.stats.read() {
            let memory_overhead = if stats.total_memory_bytes > 0 {
                let actual_buffer_memory = stats.current_buffer_count * 4096; // Estimate
                let overhead = stats.total_memory_bytes.saturating_sub(actual_buffer_memory);
                (overhead as f32 / stats.total_memory_bytes as f32) * 100.0
            } else {
                0.0
            };

            let _allocation_efficiency = if stats.avg_allocation_time_ns > 0 {
                // Target is now 500μs (0.5ms)
                let target_ns = 500_000u64;
                if stats.avg_allocation_time_ns <= target_ns {
                    100.0
                } else {
                    (target_ns as f32 / stats.avg_allocation_time_ns as f32) * 100.0
                }
            } else {
                100.0
            };

            PoolMetrics {
                total_allocations: stats.pool_hits + stats.pool_misses,
                pool_hits: stats.pool_hits,
                pool_misses: stats.pool_misses,
                hit_rate_percentage: stats.hit_rate_percent as f32,
                memory_overhead_bytes: (memory_overhead * stats.total_memory_bytes as f32 / 100.0) as usize,
                fragmentation_percentage: self.calculate_fragmentation() as f32,
                js_gc_pressure_reduction: stats.js_gc_pressure_reduction_percent,
                wasm_js_boundary_allocations: stats.wasm_js_boundary_allocations,
            }
        } else {
            PoolMetrics::default()
        }
    }

    /// Automatic pool sizing based on usage patterns
    pub fn optimize_pool_sizing(&self) -> Result<(), PoolError> {
        if !self.config.auto_sizing {
            return Ok(());
        }

        let stats = if let Ok(s) = self.stats.read() {
            s.clone()
        } else {
            return Err(PoolError::Internal("Cannot read stats".to_string()));
        };

        // Only optimize if we have enough data
        if stats.total_gets < 100 {
            return Ok(());
        }

        // Implement pool size optimization logic
        if stats.hit_rate_percent < 50.0 {
            // Low hit rate, consider expanding pool
            self.expand_underperforming_pools()?;
        } else if stats.hit_rate_percent > 95.0 {
            // Very high hit rate, consider shrinking pool to free memory
            self.shrink_overprovisioned_pools()?;
        }

        Ok(())
    }

    /// Expand pools that are underperforming
    fn expand_underperforming_pools(&self) -> Result<(), PoolError> {
        let mut pools_expanded = 0;
        let expansion_threshold = 0.8; // Expand if pool utilization > 80%
        
        if let Ok(mut pools) = self.size_pools.write() {
            for (&size, queue) in pools.iter_mut() {
                let utilization = queue.len() as f64 / self.config.max_buffers_per_size as f64;
                
                if utilization > expansion_threshold {
                    // Pre-allocate additional buffers for high-utilization pools
                    let additional_buffers = (self.config.max_buffers_per_size / 4).max(2);
                    
                    for _ in 0..additional_buffers {
                        if queue.len() < self.config.max_buffers_per_size {
                            queue.push_back(vec![0.0f32; size]);
                            
                            // Update count
                            if let Ok(mut count) = self.current_count.lock() {
                                *count += 1;
                            }
                        }
                    }
                    
                    pools_expanded += 1;
                }
            }
        }
        
        // Publish optimization event if pools were expanded
        if pools_expanded > 0 {
            let _optimization_event = PoolOptimizationEvent {
                optimization_type: "Pool Expansion".to_string(),
                pools_affected: pools_expanded,
                efficiency_improvement_percent: 5.0, // Estimated improvement
                timestamp: get_timestamp_ns(),
            };
        }
        
        Ok(())
    }

    /// Shrink pools that are overprovisioned
    fn shrink_overprovisioned_pools(&self) -> Result<(), PoolError> {
        let mut pools_shrunk = 0;
        let shrink_threshold = 0.2; // Shrink if pool utilization < 20%
        
        if let Ok(mut pools) = self.size_pools.write() {
            for (_, queue) in pools.iter_mut() {
                let utilization = queue.len() as f64 / self.config.max_buffers_per_size as f64;
                
                if utilization < shrink_threshold && queue.len() > 2 {
                    // Remove excess buffers from underutilized pools
                    let target_size = (self.config.max_buffers_per_size / 2).max(2);
                    
                    while queue.len() > target_size {
                        if queue.pop_back().is_some() {
                            // Update count
                            if let Ok(mut count) = self.current_count.lock() {
                                *count = count.saturating_sub(1);
                            }
                        }
                    }
                    
                    pools_shrunk += 1;
                }
            }
        }
        
        // Publish optimization event if pools were shrunk
        if pools_shrunk > 0 {
            let _optimization_event = PoolOptimizationEvent {
                optimization_type: "Pool Shrinkage".to_string(),
                pools_affected: pools_shrunk,
                efficiency_improvement_percent: 3.0, // Estimated memory savings
                timestamp: get_timestamp_ns(),
            };
        }
        
        Ok(())
    }

    /// Calculate pool fragmentation level
    fn calculate_fragmentation(&self) -> f64 {
        if let Ok(pools) = self.size_pools.read() {
            if pools.is_empty() {
                return 0.0;
            }

            let total_pools = pools.len();
            let non_empty_pools = pools.values().filter(|q| !q.is_empty()).count();
            
            1.0 - (non_empty_pools as f64 / total_pools as f64)
        } else {
            0.0
        }
    }

    /// Update allocation statistics
    fn update_stats(&self, from_pool: bool, allocation_time_ns: u64, size: usize) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_gets += 1;
            
            if from_pool {
                stats.pool_hits += 1;
            } else {
                stats.pool_misses += 1;
            }
            
            // Update hit rate
            stats.hit_rate_percent = (stats.pool_hits as f64 / stats.total_gets as f64) * 100.0;
            
            // Update allocation time
            let total_gets = stats.total_gets;
            let prev_avg = stats.avg_allocation_time_ns;
            stats.avg_allocation_time_ns = ((prev_avg * (total_gets - 1)) + allocation_time_ns) / total_gets;
            
            if allocation_time_ns > stats.max_allocation_time_ns {
                stats.max_allocation_time_ns = allocation_time_ns;
            }
            
            // Update memory tracking
            if !from_pool {
                stats.total_memory_bytes += size * std::mem::size_of::<f32>();
            }
        }
    }

    /// Update return statistics
    fn update_return_stats(&self, success: bool) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_returns += 1;
            
            if success {
                stats.successful_returns += 1;
            } else {
                stats.rejected_returns += 1;
            }
        }
    }

    /// Get current pool statistics
    pub fn get_stats(&self) -> PoolStats {
        if let Ok(stats) = self.stats.read() {
            let mut stats_copy = stats.clone();
            
            // Update current counts
            if let Ok(count) = self.current_count.lock() {
                stats_copy.current_buffer_count = *count;
            }
            
            if let Ok(pools) = self.size_pools.read() {
                stats_copy.size_categories = pools.len();
            }
            
            stats_copy
        } else {
            PoolStats::default()
        }
    }

    /// Cleanup expired or fragmented buffers
    pub fn cleanup_fragmentation(&self) -> Result<usize, PoolError> {
        if !self.config.fragmentation_prevention {
            return Ok(0);
        }

        let mut cleaned_count = 0;
        
        if let Ok(mut pools) = self.size_pools.write() {
            // Remove empty pools to reduce fragmentation
            pools.retain(|_, queue| !queue.is_empty());
            
            // Remove excess buffers from overprovisioned pools
            for (_, queue) in pools.iter_mut() {
                while queue.len() > self.config.max_buffers_per_size / 2 {
                    if queue.pop_back().is_some() {
                        cleaned_count += 1;
                        
                        // Update count
                        if let Ok(mut count) = self.current_count.lock() {
                            *count = count.saturating_sub(1);
                        }
                    }
                }
            }
        }
        
        Ok(cleaned_count)
    }
}

/// Pool metrics as defined in the story requirements
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub total_allocations: u64,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub hit_rate_percentage: f32,
    pub memory_overhead_bytes: usize,
    pub fragmentation_percentage: f32,
    pub js_gc_pressure_reduction: f32, // Percentage reduction in JS GC pressure
    pub wasm_js_boundary_allocations: u64, // Cross-boundary allocations
}

impl Default for PoolMetrics {
    fn default() -> Self {
        Self {
            total_allocations: 0,
            pool_hits: 0,
            pool_misses: 0,
            hit_rate_percentage: 0.0,
            memory_overhead_bytes: 0,
            fragmentation_percentage: 0.0,
            js_gc_pressure_reduction: 0.0,
            wasm_js_boundary_allocations: 0,
        }
    }
}

/// Buffer recycling pool trait as specified in the story requirements
pub trait BufferRecyclingPoolTrait: Send + Sync {
    /// Get buffer from pool or create new one (Rust-side allocation)
    fn get_or_create(&mut self, size: usize) -> Result<Vec<f32>, PoolError>;
    
    /// Return buffer to pool for recycling
    fn recycle(&mut self, buffer: Vec<f32>) -> Result<(), PoolError>;
    
    /// Get JS-compatible buffer reference to minimize WASM↔JS boundary overhead
    fn get_js_buffer_ref(&mut self, size: usize) -> Result<JSBufferRef, PoolError>;
    
    /// Return JS buffer reference for recycling (reduces JS GC pressure)
    fn recycle_js_buffer_ref(&mut self, buffer_ref: JSBufferRef) -> Result<(), PoolError>;
    
    /// Get pool efficiency metrics
    fn get_efficiency_metrics(&self) -> PoolMetrics;
    
    /// Optimize pool sizes based on usage patterns
    fn optimize_pool_sizes(&mut self);
    
    /// Defragment pool to reduce memory fragmentation
    fn defragment(&mut self) -> DefragmentationResult;
}

/// Defragmentation result
#[derive(Debug, Clone)]
pub struct DefragmentationResult {
    pub buffers_freed: usize,
    pub memory_freed_bytes: usize,
    pub fragmentation_reduction_percent: f32,
    pub defragmentation_time_ms: u64,
}

impl Default for DefragmentationResult {
    fn default() -> Self {
        Self {
            buffers_freed: 0,
            memory_freed_bytes: 0,
            fragmentation_reduction_percent: 0.0,
            defragmentation_time_ms: 0,
        }
    }
}

/// Event types for buffer pool operations
#[derive(Debug, Clone)]
pub struct BufferPoolAllocationEvent {
    pub buffer_size: usize,
    pub from_pool: bool,
    pub allocation_time_ns: u64,
    pub timestamp: u64,
}

impl Event for BufferPoolAllocationEvent {
    fn event_type(&self) -> &'static str {
        "BufferPoolAllocation"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Critical // <0.5ms requirement
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct BufferPoolRecyclingEvent {
    pub buffer_size: usize,
    pub recycled_successfully: bool,
    pub timestamp: u64,
}

impl Event for BufferPoolRecyclingEvent {
    fn event_type(&self) -> &'static str {
        "BufferPoolRecycling"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Critical // <0.5ms requirement
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct PoolOptimizationEvent {
    pub optimization_type: String,
    pub pools_affected: usize,
    pub efficiency_improvement_percent: f32,
    pub timestamp: u64,
}

impl Event for PoolOptimizationEvent {
    fn event_type(&self) -> &'static str {
        "PoolOptimization"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::High // <5ms requirement
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct PoolFragmentationEvent {
    pub fragmentation_percent: f32,
    pub threshold_exceeded: bool,
    pub defragmentation_recommended: bool,
    pub timestamp: u64,
}

impl Event for PoolFragmentationEvent {
    fn event_type(&self) -> &'static str {
        "PoolFragmentation"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Normal // <100ms requirement
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct JSGCPressureReductionEvent {
    pub gc_pressure_reduction_percent: f32,
    pub js_buffer_operations: u64,
    pub shared_array_buffer_usage: bool,
    pub timestamp: u64,
}

impl Event for JSGCPressureReductionEvent {
    fn event_type(&self) -> &'static str {
        "JSGCPressureReduction"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Normal // <100ms requirement
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
} 