//! # Platform Abstraction Module
//!
//! This module provides comprehensive platform abstraction capabilities including
//! cross-browser compatibility, device capability detection, WebAssembly bridge utilities,
//! and platform-specific optimizations for the pitch-toy application.

pub mod platform_abstraction_module;
pub mod browser_compat;
pub mod device_capabilities;
pub mod wasm_bridge;
pub mod optimization_engine;
pub mod error_recovery;
pub mod user_guidance_system;

#[cfg(test)]
mod tests;

// Re-exports for clean API surface
pub use platform_abstraction_module::*;
pub use browser_compat::*;
pub use device_capabilities::*;
pub use wasm_bridge::*;
pub use optimization_engine::*;
pub use error_recovery::*;
pub use user_guidance_system::*;

/// Core traits for platform abstraction
pub mod traits {
    use super::*;
    
    /// Browser compatibility detection and validation trait
    pub trait BrowserCompatibility: Send + Sync {
        /// Detect current browser capabilities and compatibility
        fn detect_browser(&self) -> Result<BrowserInfo, PlatformError>;
        
        /// Get cached browser information (for <5ms requirement)
        fn get_cached_browser_info(&self) -> Option<&BrowserInfo>;
        
        /// Check if specific feature is supported
        fn is_feature_supported(&self, feature: &str) -> bool;
        
        /// Get browser performance characteristics
        fn get_performance_profile(&self) -> BrowserPerformanceProfile;
    }
    
    /// Device capability detection trait
    pub trait DeviceCapabilityDetector: Send + Sync {
        /// Detect all device capabilities
        fn detect_all(&self) -> Result<DeviceCapabilities, PlatformError>;
        
        /// Check for hardware acceleration support
        fn has_hardware_acceleration(&self) -> bool;
        
        /// Get performance capability assessment
        fn assess_performance_capability(&self) -> PerformanceCapability;
        
        /// Monitor device capability changes
        fn start_capability_monitoring(&self) -> Result<(), PlatformError>;
    }
    
    /// WebAssembly bridge utilities trait
    pub trait WasmBridge: Send + Sync {
        /// Handle async operations between JS and WASM
        fn handle_async_operation(&self, operation: AsyncOperation) -> Result<AsyncResult, PlatformError>;
        
        /// Optimize frequent interop calls
        fn optimize_interop_call(&self, call: InteropCall) -> Result<OptimizedCall, PlatformError>;
        
        /// Create type-safe wrapper for JavaScript function (concrete implementation)
        fn create_js_wrapper_concrete(&self, function_name: &str) -> Result<JsWrapper, PlatformError>;
    }
    
    /// Platform optimization engine trait
    pub trait PlatformOptimizationEngine: Send + Sync {
        /// Apply browser-specific optimizations
        fn apply_optimizations(&self, browser_info: &BrowserInfo, capabilities: &DeviceCapabilities) -> Result<(), PlatformError>;
        
        /// Get optimization profile for current platform
        fn get_optimization_profile(&self) -> OptimizationProfile;
        
        /// Monitor optimization effectiveness
        fn monitor_optimization_effectiveness(&self) -> PerformanceMetrics;
    }
}

/// Core types for platform abstraction
pub mod types {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    
    /// Platform-specific error types
    #[derive(Debug, Clone)]
    pub enum PlatformError {
        BrowserDetectionFailed(String),
        UnsupportedFeature(String),
        DeviceCapabilityError(String),
        WasmBridgeError(String),
        OptimizationError(String),
        PermissionDenied(String),
    }
    
    impl std::fmt::Display for PlatformError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                PlatformError::BrowserDetectionFailed(msg) => write!(f, "Browser detection failed: {}", msg),
                PlatformError::UnsupportedFeature(feature) => write!(f, "Unsupported feature: {}", feature),
                PlatformError::DeviceCapabilityError(msg) => write!(f, "Device capability error: {}", msg),
                PlatformError::WasmBridgeError(msg) => write!(f, "WASM bridge error: {}", msg),
                PlatformError::OptimizationError(msg) => write!(f, "Optimization error: {}", msg),
                PlatformError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            }
        }
    }
    
    impl std::error::Error for PlatformError {}
    
    /// Browser information structure
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BrowserInfo {
        pub browser_name: String,
        pub version: BrowserVersion,
        pub user_agent: String,
        pub capabilities: BrowserCapabilities,
        pub compatibility_level: CompatibilityLevel,
    }
    
    /// Browser version information
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BrowserVersion {
        pub major: u32,
        pub minor: u32,
        pub patch: u32,
    }
    
    /// Browser capabilities structure
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BrowserCapabilities {
        pub supports_wasm: bool,
        pub supports_wasm_streaming: bool,
        pub supports_wasm_simd: bool,
        pub supports_audio_context: bool,
        pub supports_audio_worklet: bool,
        pub supports_media_devices: bool,
        pub supports_shared_array_buffer: bool,
        pub performance_api: bool,
    }
    
    /// Compatibility level enum
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum CompatibilityLevel {
        FullySupported,
        MostlySupported,
        PartiallySupported,
        Unsupported,
    }
    
    /// Browser performance profile
    #[derive(Debug, Clone, PartialEq)]
    pub struct BrowserPerformanceProfile {
        pub audio_latency_profile: AudioLatencyProfile,
        pub memory_characteristics: MemoryCharacteristics,
        pub cpu_performance_tier: CpuPerformanceTier,
    }
    
    /// Audio latency characteristics
    #[derive(Debug, Clone, PartialEq)]
    pub enum AudioLatencyProfile {
        UltraLow,   // <5ms
        Low,        // 5-10ms
        Medium,     // 10-50ms
        High,       // >50ms
    }
    
    /// Memory characteristics
    #[derive(Debug, Clone, PartialEq)]
    pub struct MemoryCharacteristics {
        pub available_heap: u64,
        pub supports_shared_memory: bool,
        pub garbage_collection_impact: GcImpact,
    }
    
    /// Garbage collection impact level
    #[derive(Debug, Clone, PartialEq)]
    pub enum GcImpact {
        Minimal,
        Moderate,
        Significant,
    }
    
    /// CPU performance tier
    #[derive(Debug, Clone, PartialEq)]
    pub enum CpuPerformanceTier {
        High,
        Medium,
        Low,
    }
    
    /// Device capabilities structure
    #[derive(Debug, Clone, PartialEq)]
    pub struct DeviceCapabilities {
        pub hardware_acceleration: bool,
        pub max_sample_rate: f32,
        pub min_buffer_size: usize,
        pub max_buffer_size: usize,
        pub audio_input_devices: Vec<AudioDevice>,
        pub performance_capability: PerformanceCapability,
    }
    
    /// Audio device information
    #[derive(Debug, Clone, PartialEq)]
    pub struct AudioDevice {
        pub id: String,
        pub name: String,
        pub is_default: bool,
        pub max_channels: u32,
    }
    
    /// Performance capability assessment
    #[derive(Debug, Clone, PartialEq)]
    pub enum PerformanceCapability {
        Excellent,
        Good,
        Fair,
        Poor,
    }
    
    /// WebAssembly bridge types
    #[derive(Debug)]
    pub struct JsWrapper {
        pub id: String,
        pub function_name: String,
    }
    
    /// Async operation wrapper
    #[derive(Debug)]
    pub struct AsyncOperation {
        pub id: String,
        pub operation_type: String,
    }
    
    /// Async operation result
    #[derive(Debug)]
    pub struct AsyncResult {
        pub id: String,
        pub success: bool,
        pub data: Option<String>,
    }
    
    /// Interop call representation
    #[derive(Debug)]
    pub struct InteropCall {
        pub function_name: String,
        pub args: Vec<String>,
    }
    
    /// Optimized interop call
    #[derive(Debug)]
    pub struct OptimizedCall {
        pub call: InteropCall,
        pub optimization_applied: String,
    }
    
    /// Platform optimization profile
    #[derive(Debug, Clone, PartialEq)]
    pub struct OptimizationProfile {
        pub browser_optimizations: HashMap<String, String>,
        pub memory_optimization_level: u8,
        pub audio_buffer_optimization: bool,
        pub wasm_optimization_flags: Vec<String>,
    }
    
    /// Performance metrics for optimization monitoring
    #[derive(Debug, Clone, PartialEq)]
    pub struct PerformanceMetrics {
        pub cpu_usage: f32,
        pub memory_usage: u64,
        pub audio_latency: f32,
        pub optimization_effectiveness: f32,
    }
}

// Re-export types for convenience
pub use types::*;
pub use traits::*;

/// Platform-specific events for TypedEventBus integration
pub mod events {
    use super::types::*;
    use super::error_recovery::*;
    use crate::modules::application_core::{Event, ModuleId, EventPriority};
    use std::any::Any;
    use std::time::Instant;
    
    /// Platform ready event - published when platform detection is complete
    #[derive(Debug, Clone)]
    pub struct PlatformReadyEvent {
        pub timestamp: Instant,
        pub browser_info: BrowserInfo,
        pub capabilities: DeviceCapabilities,
        pub module_id: ModuleId,
    }
    
    impl Event for PlatformReadyEvent {
        fn event_type(&self) -> &'static str {
            "platform_ready"
        }
        
        fn timestamp(&self) -> u64 {
            self.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
        }
        
        fn priority(&self) -> EventPriority {
            EventPriority::Critical
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    /// Platform capability change event
    #[derive(Debug, Clone)]
    pub struct PlatformCapabilityChangeEvent {
        pub timestamp: Instant,
        pub old_capabilities: DeviceCapabilities,
        pub new_capabilities: DeviceCapabilities,
        pub module_id: ModuleId,
    }
    
    impl Event for PlatformCapabilityChangeEvent {
        fn event_type(&self) -> &'static str {
            "platform_capability_change"
        }
        
        fn timestamp(&self) -> u64 {
            self.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
        }
        
        fn priority(&self) -> EventPriority {
            EventPriority::High
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    /// Platform optimization update event
    #[derive(Debug, Clone)]
    pub struct PlatformOptimizationUpdateEvent {
        pub timestamp: Instant,
        pub optimization_profile: OptimizationProfile,
        pub performance_metrics: PerformanceMetrics,
        pub module_id: ModuleId,
    }
    
    impl Event for PlatformOptimizationUpdateEvent {
        fn event_type(&self) -> &'static str {
            "platform_optimization_update"
        }
        
        fn timestamp(&self) -> u64 {
            self.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
        }
        
        fn priority(&self) -> EventPriority {
            EventPriority::Normal
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    /// Platform error event with recovery information
    #[derive(Debug, Clone)]
    pub struct PlatformErrorEvent {
        pub timestamp: Instant,
        pub error: PlatformError,
        pub recovery_result: Option<RecoveryResult>,
        pub module_id: ModuleId,
    }
    
    impl Event for PlatformErrorEvent {
        fn event_type(&self) -> &'static str {
            "platform_error"
        }
        
        fn timestamp(&self) -> u64 {
            self.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
        }
        
        fn priority(&self) -> EventPriority {
            EventPriority::Critical
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}

// Re-export events for convenience
pub use events::*; 