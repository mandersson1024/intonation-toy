//! Console Audio Service Interface
//!
//! This module provides a clean interface between the console component and the audio subsystem,
//! allowing the console to access audio functionality without tight coupling to audio internals.
//!
//! The service interface respects browser constraints (user gestures for permission requests)
//! while providing a clean abstraction for the console component.

use std::rc::Rc;
use std::cell::RefCell;
use super::{AudioPermission, AudioDevices, AudioContextState, AudioContextManager, permission::PermissionManager};
use crate::common::dev_log;



/// Audio status information for console display
#[derive(Debug, Clone)]
pub struct AudioStatus {
    /// Current audio permission state
    pub permission: AudioPermission,
    /// Current audio context state
    pub context_state: AudioContextState,
    /// Currently available audio devices
    pub devices: AudioDevices,
    /// Whether the audio system is properly initialized
    pub is_initialized: bool,
    /// Buffer pool performance metrics (if available)
    pub buffer_pool_metrics: Option<BufferPoolMetrics>,
}

/// Buffer pool performance metrics for UI display
#[derive(Debug, Clone)]
pub struct BufferPoolMetrics {
    /// Pool size configuration
    pub pool_size: u32,
    /// Currently available buffers
    pub available_buffers: u32,
    /// Buffers currently in use
    pub in_use_buffers: u32,
    /// Pool hit rate percentage
    pub pool_hit_rate: f32,
    /// Total allocations made
    pub allocation_count: u32,
    /// Average buffer acquisition time in ms
    pub avg_acquisition_time: f32,
    /// Number of dropped chunks due to pool exhaustion
    pub dropped_chunks: u32,
    /// Average audio processing time in ms
    pub avg_processing_time: f32,
}



/// Service interface for console audio operations
/// 
/// This trait provides all the audio functionality needed by the console component
/// without exposing internal audio implementation details.
pub trait ConsoleAudioService {
    /// Request microphone permission from user
    /// Must be called from a user gesture context (button click, etc.)
    fn request_permissions(&self) -> Result<(), String>;
    

    
    /// Get current audio system status
    fn get_current_status(&self) -> AudioStatus;
    
    /// Refresh audio device list
    /// This triggers a background refresh of available audio devices
    fn refresh_devices(&self);
    
    /// Get current audio permission state
    fn get_current_permission(&self) -> impl std::future::Future<Output = AudioPermission>;
    
    /// Request permission with callback
    /// Must be called from a user gesture context (button click, etc.)
    fn request_permission_with_callback<F>(&self, callback: F) -> impl std::future::Future<Output = AudioPermission>
    where 
        F: Fn(AudioPermission) + 'static;
}

/// Implementation of ConsoleAudioService
pub struct ConsoleAudioServiceImpl {
    /// Audio context manager for context operations
    audio_context_manager: Option<Rc<RefCell<AudioContextManager>>>,
}

impl ConsoleAudioServiceImpl {
    /// Create new console audio service implementation
    pub fn new() -> Self {
        Self {
            audio_context_manager: None,
        }
    }
    
    /// Create console audio service with audio context manager
    pub fn with_audio_context_manager(manager: Rc<RefCell<AudioContextManager>>) -> Self {
        Self {
            audio_context_manager: Some(manager),
        }
    }
    
    /// Create console audio service with both manager and event dispatcher
    /// Set the audio context manager
    pub fn set_audio_context_manager(&mut self, manager: Rc<RefCell<AudioContextManager>>) {
        self.audio_context_manager = Some(manager);
        
        // Set up device change listener
        self.setup_device_change_listener();
    }
    
    
    
    /// Get current audio devices from context manager
    fn get_current_devices(&self) -> AudioDevices {
        if let Some(ref manager_rc) = self.audio_context_manager {
            match manager_rc.try_borrow() {
                Ok(manager) => manager.get_cached_devices().clone(),
                Err(_) => {
                    dev_log!("AudioContextManager busy, returning empty device list");
                    AudioDevices::new()
                }
            }
        } else {
            AudioDevices::new()
        }
    }
    
    /// Get current audio context state
    fn get_current_context_state(&self) -> AudioContextState {
        if let Some(ref manager_rc) = self.audio_context_manager {
            match manager_rc.try_borrow() {
                Ok(manager) => manager.state().clone(),
                Err(_) => {
                    dev_log!("AudioContextManager busy, returning Uninitialized state");
                    AudioContextState::Uninitialized
                }
            }
        } else {
            AudioContextState::Uninitialized
        }
    }
    

    
    /// Set up device change listener for automatic device refresh on hardware changes
    fn setup_device_change_listener(&self) {
        if let Some(manager_rc) = &self.audio_context_manager {
            dev_log!("Setting up device change listener");
            
            // Clone references for the closure
            let manager_rc_clone = manager_rc.clone();
            
            // Set up the device change callback
            let callback = move || {
                dev_log!("Device change detected - refreshing device list");
                
                // Clone references for the async closure
                let manager_rc_async = manager_rc_clone.clone();
                
                // Spawn async task to refresh devices
                wasm_bindgen_futures::spawn_local(async move {
                    // Try to refresh devices using the local manager first
                    let mut refresh_success = false;
                    match manager_rc_async.try_borrow_mut() {
                        Ok(mut manager) => {
                            if let Err(_e) = manager.refresh_audio_devices().await {
                                dev_log!("Auto device refresh failed: {:?}", _e);
                            } else {
                                dev_log!("Auto device refresh completed successfully");
                                refresh_success = true;
                            }
                        }
                        Err(_) => {
                            dev_log!("Local AudioContextManager busy during auto device refresh");
                        }
                    }
                    
                    // If local refresh failed, try using the global AudioContextManager
                    if !refresh_success {
                        if let Some(global_manager_rc) = super::get_audio_context_manager() {
                            match global_manager_rc.try_borrow_mut() {
                                Ok(mut manager) => {
                                    if let Err(_e) = manager.refresh_audio_devices().await {
                                        dev_log!("Global auto device refresh failed: {:?}", _e);
                                    } else {
                                        dev_log!("Global auto device refresh completed successfully");
                                    }
                                }
                                Err(_) => {
                                    dev_log!("Global AudioContextManager busy during auto device refresh");
                                }
                            }
                        } else {
                            dev_log!("No global AudioContextManager available for device refresh");
                        }
                    }
                });
            };
            
            // Set up the listener in the AudioContextManager
            match manager_rc.try_borrow_mut() {
                Ok(mut manager) => {
                    if let Err(_e) = manager.setup_device_change_listener(callback) {
                        dev_log!("Failed to set up device change listener: {:?}", _e);
                    } else {
                        dev_log!("Device change listener set up successfully");
                    }
                }
                Err(_) => {
                    dev_log!("AudioContextManager busy, cannot set up device change listener");
                }
            }
        } else {
            dev_log!("Cannot set up device change listener - missing manager");
        }
    }
}

impl ConsoleAudioService for ConsoleAudioServiceImpl {
    fn request_permissions(&self) -> Result<(), String> {
        dev_log!("ConsoleAudioService: Requesting microphone permission");
        
        // Note: The actual permission request must be handled asynchronously
        // This method just validates that the request can be made
        // The actual request will be handled by the console component using PermissionManager
        
        // Check if we have the necessary dependencies
        if !PermissionManager::is_supported() {
            return Err("getUserMedia API not supported".to_string());
        }
        
        Ok(())
    }
    

    
    fn get_current_status(&self) -> AudioStatus {
        let context_state = self.get_current_context_state();
        let devices = self.get_current_devices();
        let is_initialized = self.audio_context_manager.is_some();
        
        // Note: Permission state requires async check, so we'll return a placeholder
        // In practice, the console component will manage permission state separately
        // until we implement the event system in Phase 2
        
        AudioStatus {
            permission: AudioPermission::Uninitialized, // Placeholder - will be managed by console
            context_state,
            devices,
            is_initialized,
            buffer_pool_metrics: None, // TODO: Implement buffer pool metrics collection
        }
    }
    
    fn refresh_devices(&self) {
        dev_log!("ConsoleAudioService: Refreshing audio devices");
        
        if let Some(ref manager_rc) = self.audio_context_manager {
            // Clone the Rc so we can move it into the async closure
            let manager_rc_clone = manager_rc.clone();
            
            // Trigger device refresh in background
            // This is a non-blocking operation
            wasm_bindgen_futures::spawn_local(async move {
                let mut refresh_success = false;
                match manager_rc_clone.try_borrow_mut() {
                    Ok(mut manager) => {
                        if let Err(_e) = manager.refresh_audio_devices().await {
                            dev_log!("Device refresh failed: {:?}", _e);
                        } else {
                            dev_log!("Device refresh completed successfully");
                            refresh_success = true;
                        }
                    }
                    Err(_) => {
                        dev_log!("Local AudioContextManager busy during device refresh");
                    }
                }
                
                // If local refresh failed, try using the global AudioContextManager
                if !refresh_success {
                    if let Some(global_manager_rc) = super::get_audio_context_manager() {
                        match global_manager_rc.try_borrow_mut() {
                            Ok(mut manager) => {
                                if let Err(_e) = manager.refresh_audio_devices().await {
                                    dev_log!("Global device refresh failed: {:?}", _e);
                                } else {
                                    dev_log!("Global device refresh completed successfully");
                                }
                            }
                            Err(_) => {
                                dev_log!("Global AudioContextManager busy during device refresh");
                            }
                        }
                    } else {
                        dev_log!("No global AudioContextManager available for device refresh");
                    }
                }
            });
        } else {
            // Try using the global AudioContextManager directly
            dev_log!("No local audio context manager - trying global manager");
            if let Some(global_manager_rc) = super::get_audio_context_manager() {
                wasm_bindgen_futures::spawn_local(async move {
                    match global_manager_rc.try_borrow_mut() {
                        Ok(mut manager) => {
                            if let Err(_e) = manager.refresh_audio_devices().await {
                                dev_log!("Global device refresh failed: {:?}", _e);
                            } else {
                                dev_log!("Global device refresh completed successfully");
                            }
                        }
                        Err(_) => {
                            dev_log!("Global AudioContextManager busy during device refresh");
                        }
                    }
                });
            } else {
                dev_log!("No audio context manager available for device refresh");
            }
        }
    }
    
    fn get_current_permission(&self) -> impl std::future::Future<Output = AudioPermission> {
        PermissionManager::check_microphone_permission()
    }
    
    fn request_permission_with_callback<F>(&self, callback: F) -> impl std::future::Future<Output = AudioPermission>
    where 
        F: Fn(AudioPermission) + 'static,
    {
        PermissionManager::request_permission_with_callback(callback)
    }
}

impl ConsoleAudioServiceImpl {
    /// Get current audio devices for engine data collection
    pub fn get_audio_devices(&self) -> AudioDevices {
        self.get_current_devices()
    }
}


impl Default for ConsoleAudioServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
     use wasm_bindgen_test::wasm_bindgen_test;
   
    #[wasm_bindgen_test]
    fn test_console_audio_service_creation() {
        let service = ConsoleAudioServiceImpl::new();
        
        // Service should be created successfully
        assert!(service.audio_context_manager.is_none());
    }
    
    #[wasm_bindgen_test]
    fn test_audio_status_structure() {
        let status = AudioStatus {
            permission: AudioPermission::Uninitialized,
            context_state: AudioContextState::Uninitialized,
            devices: AudioDevices::new(),
            is_initialized: false,
            buffer_pool_metrics: None,
        };
        
        assert_eq!(status.permission, AudioPermission::Uninitialized);
        assert_eq!(status.context_state, AudioContextState::Uninitialized);
        assert!(!status.is_initialized);
    }
    
    #[wasm_bindgen_test]
    fn test_service_interface_methods() {
        let service = ConsoleAudioServiceImpl::new();
        
        // Test get_current_status without context manager
        let status = service.get_current_status();
        assert_eq!(status.context_state, AudioContextState::Uninitialized);
        assert!(!status.is_initialized);
        
        // Test refresh_devices doesn't panic
        service.refresh_devices();
    }
}