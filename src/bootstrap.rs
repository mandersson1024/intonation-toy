//! Application Bootstrap Infrastructure
//! 
//! Provides parallel modular system initialization alongside the existing Yew app.
//! Enables gradual migration to modular architecture without disrupting current functionality.

use crate::modules::application_core::*;
use crate::modules::audio_foundations::{AudioFoundationsModule, AudioFoundationsConfig, PitchAlgorithm};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// Service Layer Migration imports
use crate::modules::audio_foundations::{
    AudioService, ModularAudioService, ModularAudioServiceFactory, LegacyAudioBridge
};
use crate::modules::application_core::{
    ErrorService, ModularErrorService, ModularErrorServiceFactory, LegacyErrorBridge
};
use crate::legacy::active::services::audio_engine::AudioEngineService;
use crate::legacy::active::services::error_manager::ErrorManager;

#[cfg(debug_assertions)]
use crate::modules::developer_ui::DeveloperUIModule;

/// Application bootstrap coordinator for modular system integration
pub struct ApplicationBootstrap {
    lifecycle: ApplicationLifecycleCoordinator,
    
    // Service Layer Migration - Step 2.1
    modular_audio_service: Option<Arc<Mutex<dyn AudioService>>>,
    modular_error_service: Option<Arc<Mutex<dyn ErrorService>>>,
    legacy_audio_bridge: Option<Rc<RefCell<LegacyAudioBridge>>>,
    legacy_error_bridge: Option<Rc<RefCell<LegacyErrorBridge>>>,
}

impl ApplicationBootstrap {
    /// Create new bootstrap coordinator
    pub fn new() -> Self {
        Self {
            lifecycle: ApplicationLifecycleCoordinator::new(),
            modular_audio_service: None,
            modular_error_service: None,
            legacy_audio_bridge: None,
            legacy_error_bridge: None,
        }
    }
    
    /// Register all available modules with the lifecycle coordinator
    pub fn register_modules(&mut self) -> Result<(), CoreError> {
        // Register AudioFoundationsModule with native implementation
        let audio_config = AudioFoundationsConfig {
            sample_rate: 44100.0,
            buffer_size: 2048,
            pitch_detection_algorithm: PitchAlgorithm::McLeod,
            event_publishing_interval_ms: 50, // 20Hz for real-time updates
            device_monitoring_enabled: true,
            performance_metrics_enabled: cfg!(debug_assertions),
        };
        
        let audio_module = AudioFoundationsModule::new()
            .with_config(audio_config);
            
        self.lifecycle.get_module_registry_mut()
            .register_module(Box::new(audio_module))?;
            
        // Register DeveloperUIModule with event subscriptions (debug builds only)
        #[cfg(debug_assertions)]
        {
            let dev_ui_module = DeveloperUIModule::new_with_subscriptions()
                .map_err(|e| CoreError::ModuleInitializationFailed(
                    ModuleId::new("developer_ui"), 
                    e.to_string()
                ))?;
            self.lifecycle.get_module_registry_mut()
                .register_module(Box::new(dev_ui_module))?;
        }
        
        web_sys::console::log_1(&"Modular system: All modules registered successfully".into());
        Ok(())
    }
    
    /// Initialize and start the modular system
    pub fn initialize_and_start(&mut self) -> Result<(), CoreError> {
        // Initialize services first
        self.initialize_services()?;
        
        let config = ApplicationConfig::default();
        
        web_sys::console::log_1(&"Modular system: Starting initialization".into());
        self.lifecycle.initialize(config)?;
        
        web_sys::console::log_1(&"Modular system: Starting modules".into());
        self.lifecycle.start()?;
        
        web_sys::console::log_1(&"Modular system: All modules started successfully".into());
        Ok(())
    }
    
    /// Initialize modular services and create legacy bridges
    fn initialize_services(&mut self) -> Result<(), CoreError> {
        web_sys::console::log_1(&"Service Migration: Initializing modular services".into());
        
        // Create modular services
        let audio_factory = ModularAudioServiceFactory::new();
        let error_factory = ModularErrorServiceFactory::new();
        
        // Create modular audio service
        let audio_service = Arc::new(Mutex::new(audio_factory.create_audio_service()));
        self.modular_audio_service = Some(audio_service.clone());
        
        // Create modular error service with event bus integration
        let event_bus = self.lifecycle.get_event_bus();
        let error_service = match error_factory.create_error_service_with_event_bus(event_bus) {
            Ok(service) => Arc::new(Mutex::new(service)),
            Err(_) => {
                // Fallback to service without event bus
                Arc::new(Mutex::new(error_factory.create_error_service()))
            }
        };
        self.modular_error_service = Some(error_service.clone());
        
        // Create legacy bridges
        let audio_bridge = LegacyAudioBridge::new(audio_service);
        self.legacy_audio_bridge = Some(Rc::new(RefCell::new(audio_bridge)));
        
        let error_bridge = LegacyErrorBridge::new(error_service);
        self.legacy_error_bridge = Some(Rc::new(RefCell::new(error_bridge)));
        
        web_sys::console::log_1(&"Service Migration: Modular services and bridges initialized".into());
        Ok(())
    }
    
    /// Get real-time audio metrics from native implementation
    pub fn get_audio_metrics(&self) -> Option<crate::modules::audio_foundations::AudioPerformanceMetrics> {
        let registry = self.lifecycle.get_module_registry();
        if let Some(audio_module) = registry.get_module::<AudioFoundationsModule>(&ModuleId::new("audio-foundations")) {
            Some(audio_module.get_performance_metrics().clone())
        } else {
            None
        }
    }
    
    /// Get available audio devices from native implementation
    pub fn get_available_devices(&self) -> Vec<crate::types::AudioDeviceInfo> {
        let registry = self.lifecycle.get_module_registry();
        if let Some(audio_module) = registry.get_module::<AudioFoundationsModule>(&ModuleId::new("audio-foundations")) {
            audio_module.get_available_devices().to_vec()
        } else {
            vec![]
        }
    }
    
    /// Get module states for health monitoring
    pub fn get_module_states(&self) -> HashMap<ModuleId, ModuleState> {
        let mut states = HashMap::new();
        for module_info in self.lifecycle.get_module_registry().list_modules() {
            states.insert(module_info.id.clone(), module_info.state.clone());
        }
        states
    }
    
    /// Check if all modules are healthy (started)
    pub fn is_healthy(&self) -> bool {
        let states = self.get_module_states();
        states.values().all(|state| matches!(state, ModuleState::Started))
    }
    
    /// Get current application state
    pub fn get_application_state(&self) -> ApplicationState {
        self.lifecycle.get_state()
    }
    
    /// Gracefully shutdown the modular system
    pub fn shutdown(&mut self) -> Result<(), CoreError> {
        web_sys::console::log_1(&"Modular system: Starting shutdown".into());
        
        // Clean up service bridges
        self.legacy_audio_bridge = None;
        self.legacy_error_bridge = None;
        self.modular_audio_service = None;
        self.modular_error_service = None;
        
        self.lifecycle.shutdown()
    }
    
    // =============================================================================
    // Service Layer Migration - Step 2.1: Legacy Bridge Methods
    // =============================================================================
    
    /// Get legacy AudioEngineService interface (bridge method)
    /// 
    /// Returns a legacy-compatible AudioEngineService that uses the modular
    /// audio service underneath. This enables legacy components to continue
    /// working without modification during the migration period.
    pub fn get_legacy_audio_service(&self) -> Option<Rc<RefCell<AudioEngineService>>> {
        // This method needs to be implemented differently since LegacyAudioBridge
        // doesn't directly implement AudioEngineService trait. For now, we'll
        // return None and guide users to use the bridge directly.
        None
    }
    
    /// Get legacy ErrorManager interface (bridge method)
    /// 
    /// Returns a legacy-compatible ErrorManager that uses the modular
    /// error service underneath. This enables legacy components to continue
    /// working without modification during the migration period.
    pub fn get_legacy_error_manager(&self) -> Option<Rc<RefCell<ErrorManager>>> {
        // This method needs to be implemented differently since LegacyErrorBridge
        // doesn't directly implement ErrorManager trait. For now, we'll
        // return None and guide users to use the bridge directly.
        None
    }
    
    /// Get the legacy audio bridge (direct access)
    /// 
    /// Provides direct access to the legacy audio bridge for components
    /// that need to transition gradually to the new interface.
    pub fn get_audio_bridge(&self) -> Option<Rc<RefCell<LegacyAudioBridge>>> {
        self.legacy_audio_bridge.clone()
    }
    
    /// Get the legacy error bridge (direct access)
    /// 
    /// Provides direct access to the legacy error bridge for components
    /// that need to transition gradually to the new interface.
    pub fn get_error_bridge(&self) -> Option<Rc<RefCell<LegacyErrorBridge>>> {
        self.legacy_error_bridge.clone()
    }
    
    /// Get modular audio service (direct access)
    /// 
    /// Provides direct access to the modular audio service for components
    /// that are ready to use the new interface directly.
    pub fn get_modular_audio_service(&self) -> Option<Arc<Mutex<dyn AudioService>>> {
        self.modular_audio_service.clone()
    }
    
    /// Get modular error service (direct access)
    /// 
    /// Provides direct access to the modular error service for components
    /// that are ready to use the new interface directly.
    pub fn get_modular_error_service(&self) -> Option<Arc<Mutex<dyn ErrorService>>> {
        self.modular_error_service.clone()
    }
    
    /// Check if service migration is enabled
    /// 
    /// Returns true if the modular services have been initialized and
    /// are available for use.
    pub fn is_service_migration_enabled(&self) -> bool {
        self.modular_audio_service.is_some() && self.modular_error_service.is_some()
    }
    
    /// Get service migration status
    /// 
    /// Returns detailed information about the service migration state
    /// for debugging and monitoring purposes.
    pub fn get_service_migration_status(&self) -> HashMap<String, bool> {
        let mut status = HashMap::new();
        status.insert("modular_audio_service".to_string(), self.modular_audio_service.is_some());
        status.insert("modular_error_service".to_string(), self.modular_error_service.is_some());
        status.insert("legacy_audio_bridge".to_string(), self.legacy_audio_bridge.is_some());
        status.insert("legacy_error_bridge".to_string(), self.legacy_error_bridge.is_some());
        status.insert("service_migration_enabled".to_string(), self.is_service_migration_enabled());
        status
    }
}

impl Default for ApplicationBootstrap {
    fn default() -> Self {
        Self::new()
    }
}