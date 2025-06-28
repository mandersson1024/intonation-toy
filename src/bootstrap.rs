//! Application Bootstrap Infrastructure
//! 
//! Provides parallel modular system initialization alongside the existing Yew app.
//! Enables gradual migration to modular architecture without disrupting current functionality.

use crate::modules::application_core::*;
use crate::modules::audio_foundations::AudioFoundationsModule;
use crate::legacy::services::AudioEngineService;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::modules::developer_ui::DeveloperUIModule;

/// Application bootstrap coordinator for modular system integration
pub struct ApplicationBootstrap {
    lifecycle: ApplicationLifecycleCoordinator,
    audio_service: Rc<RefCell<AudioEngineService>>, // Bridge to legacy
}

impl ApplicationBootstrap {
    /// Create new bootstrap coordinator
    pub fn new() -> Self {
        Self {
            lifecycle: ApplicationLifecycleCoordinator::new(),
            audio_service: Rc::new(RefCell::new(AudioEngineService::new())),
        }
    }
    
    /// Register all available modules with the lifecycle coordinator
    pub fn register_modules(&mut self) -> Result<(), CoreError> {
        // Register AudioFoundationsModule with legacy bridge
        let audio_module = AudioFoundationsModule::new(self.audio_service.clone());
        self.lifecycle.get_module_registry_mut()
            .register_module(Box::new(audio_module))?;
            
        // Register DeveloperUIModule (debug builds only)
        #[cfg(debug_assertions)]
        {
            let dev_ui_module = DeveloperUIModule::new()
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
        let config = ApplicationConfig::default();
        
        web_sys::console::log_1(&"Modular system: Starting initialization".into());
        self.lifecycle.initialize(config)?;
        
        web_sys::console::log_1(&"Modular system: Starting modules".into());
        self.lifecycle.start()?;
        
        web_sys::console::log_1(&"Modular system: All modules started successfully".into());
        Ok(())
    }
    
    /// Get legacy audio service for backward compatibility
    pub fn get_legacy_audio_service(&self) -> Rc<RefCell<AudioEngineService>> {
        self.audio_service.clone()
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
        self.lifecycle.shutdown()
    }
}

impl Default for ApplicationBootstrap {
    fn default() -> Self {
        Self::new()
    }
}