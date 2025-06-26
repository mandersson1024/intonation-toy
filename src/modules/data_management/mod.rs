//! # Data Management Module
//!
//! The Data Management module provides efficient data management for audio buffers
//! without performance bottlenecks. It builds on Application Core BufferRef system
//! and provides buffer pool management, recycling, data flow coordination,
//! and zero-copy operations.
//!
//! ## Key Components
//!
//! - [`DataManagementModule`]: Main module implementation with Module trait
//! - [`AudioBufferManagerImpl`]: Enhanced audio buffer manager extending BufferRef
//! - [`BufferRecyclingPool`]: Buffer pool and recycling system for memory efficiency
//! - [`DataFlowCoordinator`]: Data flow coordination between modules
//! - [`BufferUtilizationMonitor`]: Buffer utilization monitoring and optimization
//! - [`PoolMetricsMonitor`]: Real-time pool efficiency monitoring
//! - [`PoolOptimizationEngine`]: Automatic pool sizing optimization
//!
//! ## Performance Targets
//!
//! - Buffer operations: <0.5ms allocation/deallocation overhead
//! - Memory efficiency: <3% overhead for buffer management tracking
//! - Pool hit rate: >90% allocation success from pool
//! - Throughput: Support for 1000+ buffers/second without performance degradation

pub mod data_management_module;
pub mod audio_buffer_manager;
pub mod buffer_recycling_pool;
pub mod data_flow_coordinator;
pub mod audio_data_pipeline;
pub mod data_transformer;
pub mod backpressure_controller;
pub mod flow_metrics_collector;
pub mod flow_recovery_manager;
pub mod buffer_utilization_monitor;
pub mod pool_metrics;
pub mod pool_optimization;
pub mod wasm_js_bridge;

#[cfg(test)]
mod integration_tests;

// Re-exports for clean API
pub use data_management_module::*;
pub use audio_buffer_manager::*;
pub use buffer_recycling_pool::*;
pub use data_flow_coordinator::*;
pub use audio_data_pipeline::*;
pub use data_transformer::*;
pub use backpressure_controller::*;
pub use flow_metrics_collector::*;
pub use flow_recovery_manager::*;
pub use buffer_utilization_monitor::*;
pub use pool_metrics::*;
pub use pool_optimization::*;
pub use wasm_js_bridge::*;