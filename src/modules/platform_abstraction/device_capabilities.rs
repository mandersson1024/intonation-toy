use super::*;
use crate::modules::audio_foundations::{WebDeviceManager, WebDeviceCapabilityManager, DeviceManager, DeviceCapabilityManager};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Device capability detector implementation that integrates with Audio Foundations
pub struct DeviceCapabilityDetectorImpl {
    audio_device_manager: Option<Arc<Mutex<WebDeviceManager>>>,
    audio_capability_manager: Option<Arc<Mutex<WebDeviceCapabilityManager>>>,
    capability_cache: Arc<Mutex<Option<(DeviceCapabilities, Instant)>>>,
    monitoring_active: Arc<Mutex<bool>>,
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
        }
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
    fn integrate_audio_capabilities(&self) -> Result<Vec<AudioDevice>, PlatformError> {
        if let Some(ref manager) = self.audio_device_manager {
            let audio_devices = manager
                .lock()
                .unwrap()
                .list_input_devices()
                .map_err(|e| PlatformError::DeviceCapabilityError(format!("Audio device enumeration failed: {}", e)))?;
            
            Ok(audio_devices)
        } else {
            // Fallback when no audio device manager is available
            Ok(vec![])
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
        // Check cache first
        {
            let cache = self.capability_cache.lock().unwrap();
            if let Some((ref capabilities, ref cached_time)) = *cache {
                if Self::is_cache_valid(cached_time) {
                    return Ok(capabilities.clone());
                }
            }
        }
        
        // Perform fresh detection
        let audio_devices = self.integrate_audio_capabilities()?;
        let primary_device = self.get_primary_audio_device()?;
        
        let hardware_acceleration = self.detect_hardware_acceleration();
        let performance_capability = self.assess_performance();
        
        // Get detailed capabilities from primary audio device if available
        let (max_sample_rate, min_buffer_size, max_buffer_size) = if let Some(ref device) = primary_device {
            if let Some(ref manager) = self.audio_device_manager {
                let audio_caps = manager
                    .lock()
                    .unwrap()
                    .get_device_capabilities(&device.device_id);
                
                match audio_caps {
                    Ok(caps) => {
                        let max_rate = caps.sample_rates.iter().max().copied().unwrap_or(48000);
                        let min_buf = caps.buffer_sizes.iter().min().copied().unwrap_or(256);
                        let max_buf = caps.buffer_sizes.iter().max().copied().unwrap_or(4096);
                        (max_rate as f32, min_buf, max_buf)
                    }
                    Err(_) => (48000.0, 256, 4096), // Fallback values
                }
            } else {
                (48000.0, 256, 4096) // Fallback values
            }
        } else {
            (48000.0, 256, 4096) // Fallback values
        };
        
        let capabilities = DeviceCapabilities {
            hardware_acceleration,
            max_sample_rate,
            min_buffer_size,
            max_buffer_size,
            audio_input_devices: audio_devices,
            performance_capability,
        };
        
        // Update cache
        {
            let mut cache = self.capability_cache.lock().unwrap();
            *cache = Some((capabilities.clone(), Instant::now()));
        }
        
        Ok(capabilities)
    }
    
    fn has_hardware_acceleration(&self) -> bool {
        self.detect_hardware_acceleration()
    }
    
    fn assess_performance_capability(&self) -> PerformanceCapability {
        self.assess_performance()
    }
    
    fn start_capability_monitoring(&self) -> Result<(), PlatformError> {
        {
            let mut monitoring = self.monitoring_active.lock().unwrap();
            if *monitoring {
                return Ok(()); // Already monitoring
            }
            *monitoring = true;
        }
        
        // Start monitoring through Audio Foundations device manager if available
        if let Some(ref manager) = self.audio_device_manager {
            manager
                .lock()
                .unwrap()
                .start_device_monitoring()
                .map_err(|e| PlatformError::DeviceCapabilityError(format!("Failed to start device monitoring: {}", e)))?;
        }
        
        Ok(())
    }
} 