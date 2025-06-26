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
//!
//! ## Performance Targets
//!
//! - Buffer operations: <1ms allocation/deallocation overhead
//! - Memory efficiency: <5% overhead for buffer management tracking
//! - Throughput: Support for 1000+ buffers/second without performance degradation

pub mod data_management_module;
pub mod audio_buffer_manager;
pub mod buffer_recycling_pool;
pub mod data_flow_coordinator;
pub mod buffer_utilization_monitor;

#[cfg(test)]
mod integration_tests;

// Re-exports for clean API
pub use data_management_module::*;
pub use audio_buffer_manager::*;
pub use buffer_recycling_pool::*;
pub use data_flow_coordinator::*;
pub use buffer_utilization_monitor::*; 