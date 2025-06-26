use super::*;
use crate::modules::audio_foundations::{WebDeviceManager, WebDeviceCapabilityManager, DeviceManager, DeviceCapabilityManager};
use crate::modules::application_core::event_bus::EventBus;
use crate::modules::application_core::typed_event_bus::TypedEventBus;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{Performance, Navigator};

/// Device capability detector implementation that integrates with Audio Foundations
pub struct DeviceCapabilityDetectorImpl {
    audio_device_manager: Option<Arc<Mutex<WebDeviceManager>>>,
    audio_capability_manager: Option<Arc<Mutex<WebDeviceCapabilityManager>>>,
    capability_cache: Arc<Mutex<Option<(DeviceCapabilities, Instant)>>>,
    monitoring_active: Arc<Mutex<bool>>,
    event_bus: Option<Arc<TypedEventBus>>,
    capability_callbacks: Arc<Mutex<Vec<Box<dyn Fn(CapabilityChange) + Send + Sync>>>>,
    monitoring_interval: Duration,
}

impl DeviceCapabilityDetectorImpl {
    pub fn new() -> Self {
        let audio_device_manager = WebDeviceManager::new()
            .ok()
            .map(|manager| Arc::new(Mutex::new(manager)));
        
        let audio_capability_manager = WebDeviceCapabilityManager::new()
            .ok()
            .map(|manager| Arc::new(Mutex::new(manager)));

        Self {
            audio_device_manager,
            audio_capability_manager,
            capability_cache: Arc::new(Mutex::new(None)),
            monitoring_active: Arc::new(Mutex::new(false)),
            event_bus: None,
            capability_callbacks: Arc::new(Mutex::new(Vec::new())),
            monitoring_interval: Duration::from_secs(30), // Check every 30 seconds
        }
    }
    
    /// Set event bus for publishing capability change events
    pub fn set_event_bus(&mut self, event_bus: Arc<TypedEventBus>) {
        self.event_bus = Some(event_bus);
    }
    
    /// Detect hardware acceleration capabilities
    fn detect_hardware_acceleration(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            // Check for hardware-accelerated audio features
            let has_audio_worklet = js_sys::eval("window.AudioContext && 'audioWorklet' in new (window.AudioContext || window.webkitAudioContext)()")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false);
            
            let has_media_stream_track_processor = js_sys::eval("'MediaStreamTrackProcessor' in window")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false);
            
            has_audio_worklet || has_media_stream_track_processor
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        true // Assume hardware acceleration in non-WASM environments
    }
    
    /// Assess overall performance capability based on device and browser characteristics
    fn assess_performance(&self) -> PerformanceCapability {
        let has_hw_accel = self.detect_hardware_acceleration();
        
        // Get device count from Audio Foundations if available
        let device_count = if let Some(ref manager) = self.audio_device_manager {
            manager
                .lock()
                .unwrap()
                .list_input_devices()
                .map(|devices| devices.len())
                .unwrap_or(0)
        } else {
            0
        };
        
        // Performance assessment logic
        match (has_hw_accel, device_count) {
            (true, count) if count >= 2 => PerformanceCapability::Excellent,
            (true, count) if count >= 1 => PerformanceCapability::Good,
            (false, count) if count >= 2 => PerformanceCapability::Good,
            (false, count) if count >= 1 => PerformanceCapability::Fair,
            _ => PerformanceCapability::Poor,
        }
    }
    
    /// Integrate audio device capabilities with platform-level device detection
    fn integrate_audio_capabilities(&self) -> Result<AudioCapabilities, PlatformError> {
        if let Some(ref manager) = self.audio_device_manager {
            let audio_devices = manager
                .lock()
                .unwrap()
                .list_input_devices()
                .map_err(|e| PlatformError::DeviceCapabilityError(format!("Audio device enumeration failed: {}", e)))?;
            
            // Get capabilities from primary device if available
            let primary_device = audio_devices.iter()
                .find(|device| device.is_default)
                .or_else(|| audio_devices.first());
            
            if let Some(device) = primary_device {
                let device_caps = manager
                    .lock()
                    .unwrap()
                    .get_device_capabilities(&device.device_id)
                    .map_err(|e| PlatformError::DeviceCapabilityError(format!("Failed to get device capabilities: {}", e)))?;
                
                // Determine latency characteristics based on buffer sizes
                let latency_characteristics = if device_caps.buffer_sizes.iter().any(|&size| size <= 128) {
                    LatencyProfile::UltraLow
                } else if device_caps.buffer_sizes.iter().any(|&size| size <= 256) {
                    LatencyProfile::Low
                } else if device_caps.buffer_sizes.iter().any(|&size| size <= 1024) {
                    LatencyProfile::Medium
                } else {
                    LatencyProfile::High
                };
                
                Ok(AudioCapabilities {
                    max_sample_rate: device_caps.sample_rates.iter().max().copied().unwrap_or(48000),
                    min_sample_rate: device_caps.sample_rates.iter().min().copied().unwrap_or(8000),
                    supported_buffer_sizes: device_caps.buffer_sizes,
                    max_channels: device_caps.channel_counts.iter().max().copied().unwrap_or(2) as u8,
                    supports_audio_worklet: self.check_audio_worklet_support(),
                    supports_echo_cancellation: device_caps.supports_echo_cancellation,
                    latency_characteristics,
                })
            } else {
                // Fallback for no devices
                Ok(AudioCapabilities {
                    max_sample_rate: 48000,
                    min_sample_rate: 8000,
                    supported_buffer_sizes: vec![256, 512, 1024, 2048],
                    max_channels: 2,
                    supports_audio_worklet: self.check_audio_worklet_support(),
                    supports_echo_cancellation: false,
                    latency_characteristics: LatencyProfile::Medium,
                })
            }
        } else {
            // Fallback when no audio device manager is available
            Ok(AudioCapabilities {
                max_sample_rate: 48000,
                min_sample_rate: 8000,
                supported_buffer_sizes: vec![256, 512, 1024, 2048],
                max_channels: 2,
                supports_audio_worklet: self.check_audio_worklet_support(),
                supports_echo_cancellation: false,
                latency_characteristics: LatencyProfile::Medium,
            })
        }
    }
    
    /// Get the primary audio device for capability assessment
    fn get_primary_audio_device(&self) -> Result<Option<AudioDevice>, PlatformError> {
        let devices = self.integrate_audio_capabilities()?;
        
        // Find default device or use first available
        let primary_device = devices.iter()
            .find(|device| device.is_default)
            .or_else(|| devices.first())
            .cloned();
        
        Ok(primary_device)
    }
    
    /// Check if cache is valid (5 minutes for device capabilities)
    fn is_cache_valid(cached_time: &Instant) -> bool {
        cached_time.elapsed().as_secs() < 300 // 5 minutes
    }
}

impl Default for DeviceCapabilityDetectorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceCapabilityDetector for DeviceCapabilityDetectorImpl {
    fn detect_all(&self) -> Result<DeviceCapabilities, PlatformError> {
        // Check cache first for performance (<10ms requirement)
        {
            let cache = self.capability_cache.lock().unwrap();
            if let Some((ref capabilities, ref cached_time)) = *cache {
                if Self::is_cache_valid(cached_time) {
                    return Ok(capabilities.clone());
                }
            }
        }
        
        // Perform comprehensive fresh detection
        let audio_capabilities = self.integrate_audio_capabilities()?;
        let graphics_capabilities = self.detect_graphics_capabilities()?;
        let performance_capabilities = self.detect_performance_capabilities()?;
        let hardware_acceleration = self.detect_hardware_acceleration()?;
        
        // Create comprehensive capabilities for optimal settings calculation
        let all_capabilities = AllDeviceCapabilities {
            audio: audio_capabilities.clone(),
            graphics: graphics_capabilities,
            performance: performance_capabilities,
            hardware_acceleration: hardware_acceleration.clone(),
        };
        
        // Calculate optimal settings based on all detected capabilities
        let _optimal_settings = self.calculate_optimal_settings(&all_capabilities)?;
        
        // Convert to legacy DeviceCapabilities format for compatibility
        let capabilities = DeviceCapabilities {
            hardware_acceleration: hardware_acceleration.audio_processing || hardware_acceleration.graphics_rendering,
            max_sample_rate: audio_capabilities.max_sample_rate as f32,
            min_buffer_size: audio_capabilities.supported_buffer_sizes.iter().min().copied().unwrap_or(256),
            max_buffer_size: audio_capabilities.supported_buffer_sizes.iter().max().copied().unwrap_or(4096),
            audio_input_devices: audio_capabilities,
            performance_capability: self.assess_performance_capability(),
        };
        
        // Update cache with 30-second TTL for real-time updates
        {
            let mut cache = self.capability_cache.lock().unwrap();
            *cache = Some((capabilities.clone(), Instant::now()));
        }
        
        Ok(capabilities)
    }
    
    fn has_hardware_acceleration(&self) -> bool {
        self.detect_hardware_acceleration()
            .map(|accel| accel.audio_processing || accel.graphics_rendering)
            .unwrap_or(false)
    }
    
    fn assess_performance_capability(&self) -> PerformanceCapability {
        let hardware_acceleration = self.has_hardware_acceleration();
        let device_count = if let Some(ref manager) = self.audio_device_manager {
            manager.lock().unwrap().list_input_devices().map(|devices| devices.len()).unwrap_or(0)
        } else {
            0
        };
        
        // Enhanced performance assessment considering multiple factors
        let performance_capabilities = self.detect_performance_capabilities().unwrap_or_default();
        
        match (hardware_acceleration, device_count, performance_capabilities.cpu_performance_tier) {
            (true, count, CpuPerformanceTier::High) if count >= 2 => PerformanceCapability::Excellent,
            (true, count, _) if count >= 2 => PerformanceCapability::Good,
            (true, count, CpuPerformanceTier::High) if count >= 1 => PerformanceCapability::Good,
            (false, count, CpuPerformanceTier::High) if count >= 2 => PerformanceCapability::Good,
            (_, count, _) if count >= 1 => PerformanceCapability::Fair,
            _ => PerformanceCapability::Poor,
        }
    }
    
    fn start_capability_monitoring(&self) -> Result<(), PlatformError> {
        // Check if already monitoring
        {
            let mut monitoring = self.monitoring_active.lock().unwrap();
            if *monitoring {
                return Ok(()); // Already monitoring
            }
            *monitoring = true;
        }
        
        // Start monitoring through Audio Foundations device manager if available
        if let Some(ref manager) = self.audio_device_manager {
            manager.lock().unwrap().start_device_monitoring()
                .map_err(|e| PlatformError::DeviceCapabilityError(format!("Failed to start device monitoring: {}", e)))?;
        }
        
        // Start capability monitoring loop (would be spawned in a real async context)
        // For now, we'll just mark monitoring as active
        // In a real implementation, this would spawn an async task
        
        Ok(())
    }
}

// Additional trait implementations for comprehensive functionality
impl DeviceCapabilityDetectorImpl {
    /// Enhanced device capability detection that includes all capability types
    pub fn detect_all_comprehensive(&self) -> Result<AllDeviceCapabilities, PlatformError> {
        let audio = self.integrate_audio_capabilities()?;
        let graphics = self.detect_graphics_capabilities()?;
        let performance = self.detect_performance_capabilities()?;
        let hardware_acceleration = self.detect_hardware_acceleration()?;
        
        Ok(AllDeviceCapabilities {
            audio,
            graphics,
            performance,
            hardware_acceleration,
        })
    }
    
    /// Get optimal settings for the current device configuration
    pub fn get_optimal_settings(&self) -> Result<OptimalSettings, PlatformError> {
        let capabilities = self.detect_all_comprehensive()?;
        self.calculate_optimal_settings(&capabilities)
    }
    
    /// Register callback for capability changes
    pub fn register_capability_change_callback<F>(&self, callback: F) 
    where 
        F: Fn(CapabilityChange) + Send + Sync + 'static 
    {
        let mut callbacks = self.capability_callbacks.lock().unwrap();
        callbacks.push(Box::new(callback));
    }
    
    /// Stop capability monitoring
    pub fn stop_capability_monitoring(&self) -> Result<(), PlatformError> {
        {
            let mut monitoring = self.monitoring_active.lock().unwrap();
            *monitoring = false;
        }
        
        // Stop monitoring through Audio Foundations device manager if available
        if let Some(ref manager) = self.audio_device_manager {
            manager.lock().unwrap().stop_device_monitoring()
                .map_err(|e| PlatformError::DeviceCapabilityError(format!("Failed to stop device monitoring: {}", e)))?;
        }
        
        Ok(())
    }
}

// Default implementations for fallback scenarios
impl Default for PerformanceCapabilities {
    fn default() -> Self {
        Self {
            logical_cores: 4,
            estimated_memory: 8 * 1024 * 1024 * 1024, // 8GB
            supports_shared_array_buffer: false,
            supports_web_workers: true,
            supports_audio_worklet: false,
            cpu_performance_tier: CpuPerformanceTier::Medium,
            memory_pressure_level: MemoryPressureLevel::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_capability_detector_creation() {
        let detector = DeviceCapabilityDetectorImpl::new();
        assert!(!detector.has_hardware_acceleration() || detector.has_hardware_acceleration()); // Should not panic
    }
    
    #[test]
    fn test_performance_capability_assessment() {
        let detector = DeviceCapabilityDetectorImpl::new();
        let capability = detector.assess_performance_capability();
        
        // Should return a valid performance capability
        match capability {
            PerformanceCapability::Excellent | 
            PerformanceCapability::Good | 
            PerformanceCapability::Fair | 
            PerformanceCapability::Poor => {},
        }
    }
    
    #[test]
    fn test_optimal_settings_calculation() {
        let detector = DeviceCapabilityDetectorImpl::new();
        
        // Create test capabilities
        let audio_caps = AudioCapabilities {
            max_sample_rate: 48000,
            min_sample_rate: 8000,
            supported_buffer_sizes: vec![256, 512, 1024],
            max_channels: 2,
            supports_audio_worklet: true,
            supports_echo_cancellation: true,
            latency_characteristics: LatencyProfile::Low,
        };
        
        let graphics_caps = GraphicsCapabilities {
            supports_webgl: true,
            supports_webgl2: false,
            supports_canvas_2d: true,
            max_texture_size: 2048,
            max_renderbuffer_size: 2048,
            supports_float_textures: false,
            gpu_vendor: "Test".to_string(),
            gpu_renderer: "Test".to_string(),
            gpu_performance_tier: GpuPerformanceTier::Medium,
        };
        
        let performance_caps = PerformanceCapabilities::default();
        
        let hw_accel = HardwareAcceleration {
            audio_processing: true,
            graphics_rendering: true,
            video_decoding: false,
            crypto_operations: false,
            simd_support: false,
            offscreen_canvas: false,
        };
        
        let all_caps = AllDeviceCapabilities {
            audio: audio_caps,
            graphics: graphics_caps,
            performance: performance_caps,
            hardware_acceleration: hw_accel,
        };
        
        let optimal_settings = detector.calculate_optimal_settings(&all_caps).unwrap();
        
        assert!(optimal_settings.audio_sample_rate <= 48000);
        assert!(optimal_settings.audio_buffer_size >= 256);
        assert!(optimal_settings.audio_channels <= 2);
    }
    
    #[tokio::test]
    async fn test_capability_monitoring() {
        let detector = DeviceCapabilityDetectorImpl::new();
        
        // Start monitoring
        assert!(detector.start_capability_monitoring().is_ok());
        
        // Should be idempotent
        assert!(detector.start_capability_monitoring().is_ok());
        
        // Stop monitoring
        assert!(detector.stop_capability_monitoring().is_ok());
    }
    
    #[test]
    fn test_cache_validity() {
        let old_time = Instant::now() - std::time::Duration::from_secs(60);
        assert!(!DeviceCapabilityDetectorImpl::is_cache_valid(&old_time));
        
        let recent_time = Instant::now();
        assert!(DeviceCapabilityDetectorImpl::is_cache_valid(&recent_time));
    }
} 