//! # Buffer Recycling Pool Implementation
//!
//! This module provides a buffer recycling pool system for memory efficiency with
//! size-based allocation strategies and <1ms allocation overhead. The pool minimizes
//! memory allocation pressure through intelligent buffer reuse and automatic sizing.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};

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
    /// Pool configuration
    config: PoolConfig,
    /// Pool statistics and metrics
    stats: Arc<RwLock<PoolStats>>,
    /// Maximum total capacity
    max_capacity: usize,
    /// Current total buffers in pool
    current_count: Arc<Mutex<usize>>,
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
}

impl BufferRecyclingPool {
    /// Create a new buffer recycling pool
    pub fn new(max_capacity: usize) -> Self {
        Self {
            size_pools: Arc::new(RwLock::new(HashMap::new())),
            config: PoolConfig::default(),
            stats: Arc::new(RwLock::new(PoolStats::default())),
            max_capacity,
            current_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a buffer pool with custom configuration
    pub fn with_config(max_capacity: usize, config: PoolConfig) -> Self {
        Self {
            size_pools: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(PoolStats::default())),
            max_capacity,
            current_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get buffer from pool or create new one with <1ms overhead
    pub fn get_or_create(&self, size: usize) -> Result<Vec<f32>, PoolError> {
        let start_time = Instant::now();
        
        // Validate size
        if size < self.config.min_buffer_size || size > self.config.max_buffer_size {
            return Err(PoolError::InvalidSize(format!("Size {} out of range [{}, {}]", 
                size, self.config.min_buffer_size, self.config.max_buffer_size)));
        }

        // Try to get from pool first
        let buffer = if let Ok(mut pools) = self.size_pools.write() {
            if let Some(queue) = pools.get_mut(&size) {
                queue.pop_front()
            } else {
                None
            }
        } else {
            None
        };

        let (result_buffer, from_pool) = if let Some(mut pooled_buffer) = buffer {
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
        self.update_stats(from_pool, allocation_time.as_nanos() as u64, size);

        Ok(result_buffer)
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

    /// Get pool efficiency metrics
    pub fn get_efficiency_metrics(&self) -> PoolEfficiencyReport {
        if let Ok(stats) = self.stats.read() {
            let memory_efficiency = if stats.total_memory_bytes > 0 {
                1.0 - (stats.total_memory_bytes as f64 / (self.max_capacity * 4096) as f64)
            } else {
                1.0
            };

            let allocation_efficiency = if stats.avg_allocation_time_ns > 0 {
                1.0 - (stats.avg_allocation_time_ns as f64 / 1_000_000.0) // 1ms target
            } else {
                1.0
            };

            PoolEfficiencyReport {
                hit_rate_percent: stats.hit_rate_percent,
                memory_efficiency_percent: memory_efficiency * 100.0,
                allocation_efficiency_percent: allocation_efficiency.max(0.0) * 100.0,
                fragmentation_level: self.calculate_fragmentation(),
                avg_allocation_time_ns: stats.avg_allocation_time_ns,
                total_buffers: stats.current_buffer_count,
                size_categories: stats.size_categories,
                memory_utilization_bytes: stats.total_memory_bytes,
            }
        } else {
            PoolEfficiencyReport::default()
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
        // Implementation would analyze usage patterns and expand specific size pools
        // For now, this is a placeholder
        Ok(())
    }

    /// Shrink pools that are overprovisioned
    fn shrink_overprovisioned_pools(&self) -> Result<(), PoolError> {
        // Implementation would identify rarely used pools and shrink them
        // For now, this is a placeholder
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

/// Pool efficiency report
#[derive(Debug, Clone)]
pub struct PoolEfficiencyReport {
    pub hit_rate_percent: f64,
    pub memory_efficiency_percent: f64,
    pub allocation_efficiency_percent: f64,
    pub fragmentation_level: f64,
    pub avg_allocation_time_ns: u64,
    pub total_buffers: usize,
    pub size_categories: usize,
    pub memory_utilization_bytes: usize,
}

impl Default for PoolEfficiencyReport {
    fn default() -> Self {
        Self {
            hit_rate_percent: 0.0,
            memory_efficiency_percent: 100.0,
            allocation_efficiency_percent: 100.0,
            fragmentation_level: 0.0,
            avg_allocation_time_ns: 0,
            total_buffers: 0,
            size_categories: 0,
            memory_utilization_bytes: 0,
        }
    }
} 