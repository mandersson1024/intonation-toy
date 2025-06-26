//! # Enhanced Audio Buffer Manager Implementation
//!
//! This module provides the AudioBufferManagerImpl that extends the Application Core
//! BufferRef system with optimized buffer creation, lifecycle management, and pool integration.
//! Targets <1ms allocation/deallocation overhead for real-time audio processing.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};

use crate::modules::application_core::{
    BufferRef, BufferMetadata, BufferId, BufferManager, BufferManagerError,
    Event, EventPriority, get_timestamp_ns
};

use super::{BufferRecyclingPool, BufferAllocationEvent};

/// Enhanced audio buffer manager with pool integration
pub struct AudioBufferManagerImpl {
    /// Core buffer manager for basic operations
    core_manager: BufferManager,
    /// Buffer recycling pool for memory efficiency
    buffer_pool: Option<Arc<BufferRecyclingPool>>,
    /// Active buffer tracking with metadata
    active_buffers: Arc<RwLock<HashMap<BufferId, BufferTrackingInfo>>>,
    /// Performance metrics
    metrics: Arc<RwLock<BufferManagerMetrics>>,
    /// Initialization state
    initialized: bool,
}

/// Tracking information for each active buffer
#[derive(Debug, Clone)]
struct BufferTrackingInfo {
    /// Buffer metadata
    metadata: BufferMetadata,
    /// Buffer size in bytes
    size_bytes: usize,
    /// Creation timestamp
    created_at: Instant,
    /// Whether buffer came from pool
    from_pool: bool,
    /// Allocation time overhead in nanoseconds
    allocation_overhead_ns: u64,
    /// Number of shares/references
    reference_count: usize,
}

/// Performance metrics for buffer management
#[derive(Debug, Clone, Default)]
pub struct BufferManagerMetrics {
    /// Total buffers created
    pub total_buffers_created: u64,
    /// Buffers allocated from pool
    pub pool_hits: u64,
    /// Buffers allocated from heap
    pub pool_misses: u64,
    /// Average allocation time in nanoseconds
    pub avg_allocation_time_ns: u64,
    /// Maximum allocation time in nanoseconds
    pub max_allocation_time_ns: u64,
    /// Total memory allocated in bytes
    pub total_memory_bytes: usize,
    /// Current active buffer count
    pub active_buffer_count: usize,
    /// Pool hit rate percentage
    pub pool_hit_rate_percent: f64,
}

impl AudioBufferManagerImpl {
    /// Create a new enhanced audio buffer manager
    pub fn new() -> Self {
        let max_memory = 256 * 1024 * 1024; // 256MB default limit
        Self {
            core_manager: BufferManager::new(max_memory),
            buffer_pool: None,
            active_buffers: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(BufferManagerMetrics::default())),
            initialized: false,
        }
    }

    /// Initialize the buffer manager with a recycling pool
    pub fn initialize_with_pool(&mut self, pool: Arc<BufferRecyclingPool>) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer_pool = Some(pool);
        self.initialized = true;
        Ok(())
    }

    /// Create optimized audio buffer with metadata tracking
    pub fn create_buffer(&mut self, size: usize, channels: u8) -> Result<BufferId, BufferManagerError> {
        let start_time = Instant::now();
        
        // Try to get recycled data from pool first
        let (recycled_data, from_pool) = if let Some(pool) = &self.buffer_pool {
            match pool.get_or_create(size) {
                Ok(data) => (Some(data), true),
                Err(_) => (None, false)
            }
        } else {
            (None, false)
        };

        // Create buffer data (either from pool or new allocation)
        let buffer_data = if let Some(data) = recycled_data {
            data
        } else {
            vec![0.0f32; size]
        };

        // Create metadata
        let metadata = BufferMetadata::new(44100, channels, size); // Default sample rate
        
        // Create buffer reference
        let buffer_ref = BufferRef::new(buffer_data, metadata.clone());
        let buffer_id = buffer_ref.buffer_id();
        
        // Calculate allocation overhead
        let allocation_time = start_time.elapsed();
        let allocation_overhead_ns = allocation_time.as_nanos() as u64;

        // Register buffer with core manager
        self.core_manager.create_buffer(vec![0.0f32; size], metadata.clone())?;

        // Track buffer information
        let tracking_info = BufferTrackingInfo {
            metadata: metadata.clone(),
            size_bytes: size * std::mem::size_of::<f32>(),
            created_at: start_time,
            from_pool,
            allocation_overhead_ns,
            reference_count: 1,
        };

        // Update tracking
        if let Ok(mut buffers) = self.active_buffers.write() {
            buffers.insert(buffer_id, tracking_info);
        }

        // Update metrics
        self.update_metrics(allocation_overhead_ns, from_pool, size * std::mem::size_of::<f32>());

        // Publish buffer allocation event
        self.publish_buffer_allocation_event(buffer_id, size, channels, allocation_overhead_ns, from_pool);

        Ok(buffer_id)
    }

    /// Update buffer reference count
    pub fn update_reference_count(&mut self, buffer_id: BufferId, new_count: usize) {
        // Check if cleanup is needed first
        let needs_cleanup = if let Ok(buffers) = self.active_buffers.read() {
            if let Some(info) = buffers.get(&buffer_id) {
                new_count == 0
            } else {
                false
            }
        } else {
            false
        };

        // Update reference count
        if let Ok(mut buffers) = self.active_buffers.write() {
            if let Some(info) = buffers.get_mut(&buffer_id) {
                info.reference_count = new_count;
            }
        }

        // Clean up if needed (outside of lock)
        if needs_cleanup {
            self.cleanup_buffer(buffer_id);
        }
    }

    /// Clean up expired buffer
    fn cleanup_buffer(&mut self, buffer_id: BufferId) {
        if let Ok(mut buffers) = self.active_buffers.write() {
            if let Some(info) = buffers.remove(&buffer_id) {
                // Update metrics
                if let Ok(mut metrics) = self.metrics.write() {
                    metrics.active_buffer_count = metrics.active_buffer_count.saturating_sub(1);
                    metrics.total_memory_bytes = metrics.total_memory_bytes.saturating_sub(info.size_bytes);
                }
                
                // Return buffer to pool if it came from pool
                if info.from_pool {
                    if let Some(pool) = &self.buffer_pool {
                        // Note: In a real implementation, we would return the actual buffer data
                        // For now, this is a placeholder
                        let _ = pool.return_buffer(vec![0.0f32; info.size_bytes / std::mem::size_of::<f32>()]);
                    }
                }
            }
        }
    }

    /// Update performance metrics
    fn update_metrics(&self, allocation_time_ns: u64, from_pool: bool, size_bytes: usize) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.total_buffers_created += 1;
            
            if from_pool {
                metrics.pool_hits += 1;
            } else {
                metrics.pool_misses += 1;
            }
            
            // Update allocation time metrics
            let total_buffers = metrics.total_buffers_created;
            let prev_avg = metrics.avg_allocation_time_ns;
            metrics.avg_allocation_time_ns = ((prev_avg * (total_buffers - 1)) + allocation_time_ns) / total_buffers;
            
            if allocation_time_ns > metrics.max_allocation_time_ns {
                metrics.max_allocation_time_ns = allocation_time_ns;
            }
            
            // Update memory tracking
            metrics.total_memory_bytes += size_bytes;
            metrics.active_buffer_count += 1;
            
            // Calculate pool hit rate
            if metrics.total_buffers_created > 0 {
                metrics.pool_hit_rate_percent = (metrics.pool_hits as f64 / metrics.total_buffers_created as f64) * 100.0;
            }
        }
    }

    /// Publish buffer allocation event
    fn publish_buffer_allocation_event(&self, buffer_id: BufferId, size: usize, channels: u8, allocation_time_ns: u64, from_pool: bool) {
        // Note: In a real implementation, this would integrate with the TypedEventBus
        // For now, this is a placeholder that creates the event but doesn't publish it
        let _event = BufferAllocationEvent {
            buffer_id,
            size,
            channels,
            allocation_time_ns,
            from_pool,
            timestamp: get_timestamp_ns(),
        };
        
        // TODO: Publish event through TypedEventBus when available
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> BufferManagerMetrics {
        if let Ok(metrics) = self.metrics.read() {
            metrics.clone()
        } else {
            BufferManagerMetrics::default()
        }
    }

    /// Get buffer tracking information
    pub fn get_buffer_info(&self, buffer_id: BufferId) -> Option<BufferTrackingInfo> {
        if let Ok(buffers) = self.active_buffers.read() {
            buffers.get(&buffer_id).cloned()
        } else {
            None
        }
    }

    /// Check if buffer manager is meeting performance targets
    pub fn check_performance_targets(&self) -> BufferPerformanceReport {
        let metrics = self.get_metrics();
        
        let allocation_target_met = metrics.avg_allocation_time_ns <= 1_000_000; // 1ms target
        let max_allocation_target_met = metrics.max_allocation_time_ns <= 5_000_000; // 5ms max
        let pool_hit_rate_target_met = metrics.pool_hit_rate_percent >= 90.0; // 90% target
        
        BufferPerformanceReport {
            allocation_target_met,
            max_allocation_target_met,
            pool_hit_rate_target_met,
            avg_allocation_time_ns: metrics.avg_allocation_time_ns,
            max_allocation_time_ns: metrics.max_allocation_time_ns,
            pool_hit_rate_percent: metrics.pool_hit_rate_percent,
            active_buffer_count: metrics.active_buffer_count,
            total_memory_bytes: metrics.total_memory_bytes,
        }
    }
}

/// Performance report for buffer manager
#[derive(Debug, Clone)]
pub struct BufferPerformanceReport {
    pub allocation_target_met: bool,
    pub max_allocation_target_met: bool,
    pub pool_hit_rate_target_met: bool,
    pub avg_allocation_time_ns: u64,
    pub max_allocation_time_ns: u64,
    pub pool_hit_rate_percent: f64,
    pub active_buffer_count: usize,
    pub total_memory_bytes: usize,
}

impl Default for AudioBufferManagerImpl {
    fn default() -> Self {
        Self::new()
    }
} 