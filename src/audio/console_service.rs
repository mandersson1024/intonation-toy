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
use crate::events::{AudioEvent, SharedEventDispatcher};

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
}

/// Callback type for device change notifications
pub type DeviceChangeCallback = Box<dyn Fn(AudioDevices)>;

/// Callback type for permission change notifications
pub type PermissionChangeCallback = Box<dyn Fn(AudioPermission)>;

/// Service interface for console audio operations
/// 
/// This trait provides all the audio functionality needed by the console component
/// without exposing internal audio implementation details.
pub trait ConsoleAudioService {
    /// Request microphone permission from user
    /// Must be called from a user gesture context (button click, etc.)
    fn request_permissions(&self) -> Result<(), String>;
    
    /// Subscribe to audio device changes
    /// The callback will be called whenever audio devices are added/removed
    fn subscribe_device_changes(&self, callback: DeviceChangeCallback);
    
    /// Subscribe to permission state changes
    /// The callback will be called whenever permission state changes
    fn subscribe_permission_changes(&self, callback: PermissionChangeCallback);
    
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
    /// Event dispatcher for publishing and subscribing to events
    event_dispatcher: Option<SharedEventDispatcher>,
}

impl ConsoleAudioServiceImpl {
    /// Create new console audio service implementation
    pub fn new() -> Self {
        Self {
            audio_context_manager: None,
            event_dispatcher: None,
        }
    }
    
    /// Create console audio service with audio context manager
    pub fn with_audio_context_manager(manager: Rc<RefCell<AudioContextManager>>) -> Self {
        Self {
            audio_context_manager: Some(manager),
            event_dispatcher: None,
        }
    }
    
    /// Create console audio service with both manager and event dispatcher
    pub fn with_dependencies(
        manager: Rc<RefCell<AudioContextManager>>, 
        event_dispatcher: SharedEventDispatcher
    ) -> Self {
        Self {
            audio_context_manager: Some(manager),
            event_dispatcher: Some(event_dispatcher),
        }
    }
    
    /// Set the audio context manager
    pub fn set_audio_context_manager(&mut self, manager: Rc<RefCell<AudioContextManager>>) {
        self.audio_context_manager = Some(manager);
        
        // Set up device change listener if we also have an event dispatcher
        if self.event_dispatcher.is_some() {
            self.setup_device_change_listener();
        }
    }
    
    /// Set the event dispatcher
    pub fn set_event_dispatcher(&mut self, dispatcher: SharedEventDispatcher) {
        self.event_dispatcher = Some(dispatcher);
        
        // Set up device change listener now that we have both manager and dispatcher
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
    
    /// Publish an audio event through the event dispatcher
    pub fn publish_event(&self, event: AudioEvent) {
        if let Some(ref dispatcher) = self.event_dispatcher {
            dispatcher.borrow().publish(event);
        } else {
            dev_log!("Warning: No event dispatcher available to publish event");
        }
    }
    
    /// Set up device change listener for automatic device refresh on hardware changes
    fn setup_device_change_listener(&self) {
        if let (Some(ref manager_rc), Some(ref event_dispatcher)) = 
            (&self.audio_context_manager, &self.event_dispatcher) {
            
            dev_log!("Setting up device change listener");
            
            // Clone references for the closure
            let manager_rc_clone = manager_rc.clone();
            let event_dispatcher_clone = event_dispatcher.clone();
            
            // Set up the device change callback
            let callback = move || {
                dev_log!("Device change detected - refreshing device list");
                
                // Clone references for the async closure
                let manager_rc_async = manager_rc_clone.clone();
                let event_dispatcher_async = event_dispatcher_clone.clone();
                
                // Spawn async task to refresh devices
                wasm_bindgen_futures::spawn_local(async move {
                    match manager_rc_async.try_borrow_mut() {
                        Ok(mut manager) => {
                            if let Err(_e) = manager.refresh_audio_devices().await {
                                dev_log!("Auto device refresh failed: {:?}", _e);
                            } else {
                                dev_log!("Auto device refresh completed successfully");
                                
                                // Get updated devices and publish event
                                let updated_devices = manager.get_cached_devices().clone();
                                let event = AudioEvent::DeviceListChanged(updated_devices);
                                event_dispatcher_async.borrow().publish(event);
                                dev_log!("Published DeviceListChanged event from auto refresh");
                            }
                        }
                        Err(_) => {
                            dev_log!("AudioContextManager busy during auto device refresh");
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
            dev_log!("Cannot set up device change listener - missing manager or event dispatcher");
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
    
    fn subscribe_device_changes(&self, callback: DeviceChangeCallback) {
        dev_log!("ConsoleAudioService: Subscribing to device changes");
        
        if let Some(ref dispatcher) = self.event_dispatcher {
            dispatcher.borrow_mut().subscribe("device_list_changed", move |event| {
                if let AudioEvent::DeviceListChanged(devices) = event {
                    callback(devices);
                }
            });
            dev_log!("Device change subscription registered with event dispatcher");
        } else {
            dev_log!("Warning: No event dispatcher available for device change subscription");
        }
    }
    
    fn subscribe_permission_changes(&self, callback: PermissionChangeCallback) {
        dev_log!("ConsoleAudioService: Subscribing to permission changes");
        
        if let Some(ref dispatcher) = self.event_dispatcher {
            dispatcher.borrow_mut().subscribe("permission_changed", move |event| {
                if let AudioEvent::PermissionChanged(permission) = event {
                    callback(permission);
                }
            });
            dev_log!("Permission change subscription registered with event dispatcher");
        } else {
            dev_log!("Warning: No event dispatcher available for permission change subscription");
        }
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
        }
    }
    
    fn refresh_devices(&self) {
        dev_log!("ConsoleAudioService: Refreshing audio devices");
        
        if let Some(ref manager_rc) = self.audio_context_manager {
            // Clone the Rc so we can move it into the async closure
            let manager_rc_clone = manager_rc.clone();
            
            // Clone the event dispatcher reference for the async closure
            let event_dispatcher = self.event_dispatcher.clone();
            
            // Trigger device refresh in background
            // This is a non-blocking operation
            wasm_bindgen_futures::spawn_local(async move {
                match manager_rc_clone.try_borrow_mut() {
                    Ok(mut manager) => {
                        if let Err(_e) = manager.refresh_audio_devices().await {
                            dev_log!("Device refresh failed: {:?}", _e);
                        } else {
                            dev_log!("Device refresh completed successfully");
                            
                            // Get the updated device list and publish event
                            let updated_devices = manager.get_cached_devices().clone();
                            
                            // Publish device change event if event dispatcher is available
                            if let Some(ref dispatcher) = event_dispatcher {
                                let event = crate::events::AudioEvent::DeviceListChanged(updated_devices);
                                dispatcher.borrow().publish(event);
                                dev_log!("Published DeviceListChanged event");
                            } else {
                                dev_log!("Warning: No event dispatcher available to publish device change event");
                            }
                        }
                    }
                    Err(_) => {
                        dev_log!("AudioContextManager busy, skipping device refresh");
                    }
                }
            });
        } else {
            dev_log!("No audio context manager available for device refresh");
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

impl Default for ConsoleAudioServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_console_audio_service_creation() {
        let service = ConsoleAudioServiceImpl::new();
        
        // Service should be created successfully
        assert!(service.audio_context_manager.is_none());
    }
    
    #[test]
    fn test_audio_status_structure() {
        let status = AudioStatus {
            permission: AudioPermission::Uninitialized,
            context_state: AudioContextState::Uninitialized,
            devices: AudioDevices::new(),
            is_initialized: false,
        };
        
        assert_eq!(status.permission, AudioPermission::Uninitialized);
        assert_eq!(status.context_state, AudioContextState::Uninitialized);
        assert!(!status.is_initialized);
    }
    
    #[test]
    fn test_service_interface_methods() {
        let service = ConsoleAudioServiceImpl::new();
        
        // Test get_current_status without context manager
        let status = service.get_current_status();
        assert_eq!(status.context_state, AudioContextState::Uninitialized);
        assert!(!status.is_initialized);
        
        // Test refresh_devices doesn't panic
        service.refresh_devices();
        
        // Test subscribe methods don't panic
        service.subscribe_device_changes(Box::new(|_| {}));
        service.subscribe_permission_changes(Box::new(|_| {}));
    }
}