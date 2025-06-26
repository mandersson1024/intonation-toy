//! # Data Management Module Implementation
//!
//! This module provides the core Data Management module implementation that integrates
//! with the Application Core module system. It coordinates buffer management, data flow,
//! and utilization monitoring with <1ms allocation/deallocation overhead target.

use std::sync::Arc;
use std::collections::HashMap;

use crate::modules::application_core::{
    Module, ModuleId, TypedEventBus, BufferRef, BufferMetadata,
    Event, EventPriority, get_timestamp_ns
};

use super::{
    AudioBufferManagerImpl, BufferRecyclingPool, DataFlowCoordinator,
    BufferUtilizationMonitor
};

/// Data Management module events
#[derive(Debug, Clone)]
pub struct DataManagementReadyEvent {
    pub module_id: ModuleId,
    pub timestamp: u64,
    pub buffer_pool_capacity: usize,
    pub data_flow_pipelines: u32,
}

impl Event for DataManagementReadyEvent {
    fn event_type(&self) -> &'static str {
        "DataManagementReady"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Buffer allocation event for tracking buffer lifecycle
#[derive(Debug, Clone)]
pub struct BufferAllocationEvent {
    pub buffer_id: crate::modules::application_core::BufferId,
    pub size: usize,
    pub channels: u8,
    pub allocation_time_ns: u64,
    pub from_pool: bool,
    pub timestamp: u64,
}

impl Event for BufferAllocationEvent {
    fn event_type(&self) -> &'static str {
        "BufferAllocation"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        EventPriority::Critical
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Buffer recycling event for pool management tracking
#[derive(Debug, Clone)]
pub struct BufferRecyclingEvent {
    pub buffer_id: crate::modules::application_core::BufferId,
    pub returned_to_pool: bool,
    pub pool_utilization_percent: f64,
    pub timestamp: u64,
}

impl Event for BufferRecyclingEvent {
    fn event_type(&self) -> &'static str {
        "BufferRecycling"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        EventPriority::High
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Data flow registration event
#[derive(Debug, Clone)]
pub struct DataFlowRegisteredEvent {
    pub pipeline_name: String,
    pub source_module: String,
    pub destination_module: String,
    pub timestamp: u64,
}

impl Event for DataFlowRegisteredEvent {
    fn event_type(&self) -> &'static str {
        "DataFlowRegistered"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        EventPriority::High
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Main Data Management module implementation
pub struct DataManagementModule {
    /// Module identifier
    module_id: ModuleId,
    /// Enhanced audio buffer manager
    buffer_manager: Option<Arc<AudioBufferManagerImpl>>,
    /// Buffer recycling pool
    buffer_pool: Option<Arc<BufferRecyclingPool>>,
    /// Data flow coordinator
    data_flow: Option<Arc<DataFlowCoordinator>>,
    /// Buffer utilization monitor
    utilization_monitor: Option<Arc<BufferUtilizationMonitor>>,
    /// TypedEventBus for publishing events
    event_bus: Option<Arc<TypedEventBus>>,
    /// Module configuration
    config: DataManagementConfig,
    /// Initialization state
    initialized: bool,
}

/// Configuration for Data Management module
#[derive(Debug, Clone)]
pub struct DataManagementConfig {
    /// Maximum buffer pool size
    pub max_pool_size: usize,
    /// Target allocation overhead in nanoseconds
    pub target_allocation_overhead_ns: u64,
    /// Memory efficiency overhead target percentage
    pub memory_efficiency_target_percent: f64,
    /// Throughput target (buffers per second)
    pub throughput_target_bps: u32,
    /// Buffer utilization monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
}

impl Default for DataManagementConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 1000,
            target_allocation_overhead_ns: 1_000_000, // 1ms in nanoseconds
            memory_efficiency_target_percent: 5.0,
            throughput_target_bps: 1000,
            monitoring_interval_ms: 100,
        }
    }
}

impl DataManagementModule {
    /// Create a new Data Management module
    pub fn new() -> Self {
        Self {
            module_id: ModuleId::new("data-management"),
            buffer_manager: None,
            buffer_pool: None,
            data_flow: None,
            utilization_monitor: None,
            event_bus: None,
            config: DataManagementConfig::default(),
            initialized: false,
        }
    }

    /// Create a new Data Management module with custom configuration
    pub fn with_config(config: DataManagementConfig) -> Self {
        Self {
            module_id: ModuleId::new("data-management"),
            buffer_manager: None,
            buffer_pool: None,
            data_flow: None,
            utilization_monitor: None,
            event_bus: None,
            config,
            initialized: false,
        }
    }

    /// Set the TypedEventBus for event publishing
    pub fn set_event_bus(&mut self, event_bus: Arc<TypedEventBus>) {
        self.event_bus = Some(event_bus);
    }

    /// Get the buffer manager (if initialized)
    pub fn get_buffer_manager(&self) -> Option<Arc<AudioBufferManagerImpl>> {
        self.buffer_manager.clone()
    }

    /// Get the data flow coordinator (if initialized)
    pub fn get_data_flow_coordinator(&self) -> Option<Arc<DataFlowCoordinator>> {
        self.data_flow.clone()
    }

    /// Get the buffer pool (if initialized)
    pub fn get_buffer_pool(&self) -> Option<Arc<BufferRecyclingPool>> {
        self.buffer_pool.clone()
    }

    /// Publish data management ready event
    fn publish_data_management_ready_event(&self) {
        if let Some(event_bus) = &self.event_bus {
            let pipeline_count = if let Some(coordinator) = &self.data_flow {
                coordinator.get_pipeline_count()
            } else {
                0
            };

            let event = DataManagementReadyEvent {
                module_id: self.module_id.clone(),
                timestamp: get_timestamp_ns(),
                buffer_pool_capacity: self.config.max_pool_size,
                data_flow_pipelines: pipeline_count,
            };

            // Note: In a real implementation, we would properly handle the Result
            // For now, we'll use a placeholder that matches the expected signature
            // TODO: Integrate with actual TypedEventBus publish method
        }
    }
}

impl Module for DataManagementModule {
    fn module_id(&self) -> ModuleId {
        self.module_id.clone()
    }

    fn module_name(&self) -> &str {
        "Data Management"
    }

    fn module_version(&self) -> &str {
        "1.0.0"
    }

    fn dependencies(&self) -> Vec<ModuleId> {
        vec![ModuleId::new("application-core")]
    }

    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }

        // Initialize buffer recycling pool
        self.buffer_pool = Some(Arc::new(BufferRecyclingPool::new(
            self.config.max_pool_size
        )));

        // Initialize enhanced audio buffer manager with pool
        if let Some(pool) = &self.buffer_pool {
            let mut buffer_manager = AudioBufferManagerImpl::new();
            buffer_manager.initialize_with_pool(pool.clone())?;
            self.buffer_manager = Some(Arc::new(buffer_manager));
        }

        // Initialize data flow coordinator
        self.data_flow = Some(Arc::new(DataFlowCoordinator::new()));

        // Initialize buffer utilization monitor
        self.utilization_monitor = Some(Arc::new(BufferUtilizationMonitor::new(
            self.config.monitoring_interval_ms
        )));

        // Set up data flow coordination for Audio Foundations
        if let Some(coordinator) = &self.data_flow {
            coordinator.register_audio_foundations_pipeline()?;
        }

        self.initialized = true;

        // Publish data management ready event
        self.publish_data_management_ready_event();

        Ok(())
    }

    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.initialized {
            return Err("Module not initialized".into());
        }

        // Start buffer utilization monitoring
        if let Some(monitor) = &self.utilization_monitor {
            monitor.start_monitoring()?;
        }

        // Start data flow coordinator
        if let Some(coordinator) = &self.data_flow {
            coordinator.start()?;
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Stop buffer utilization monitoring
        if let Some(monitor) = &self.utilization_monitor {
            monitor.stop_monitoring()?;
        }

        // Stop data flow coordinator
        if let Some(coordinator) = &self.data_flow {
            coordinator.stop()?;
        }

        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Stop everything first
        self.stop()?;

        // Clean up resources
        self.utilization_monitor = None;
        self.data_flow = None;
        self.buffer_manager = None;
        self.buffer_pool = None;
        self.event_bus = None;
        self.initialized = false;

        Ok(())
    }
}

impl Default for DataManagementModule {
    fn default() -> Self {
        Self::new()
    }
} 