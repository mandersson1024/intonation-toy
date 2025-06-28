//! Application Bootstrap Infrastructure
//! 
//! Provides parallel modular system initialization alongside the existing Yew app.
//! Enables gradual migration to modular architecture without disrupting current functionality.

use crate::modules::application_core::*;
use crate::modules::audio_foundations::{AudioFoundationsModule, AudioFoundationsConfig, PitchAlgorithm};
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::modules::developer_ui::DeveloperUIModule;

/// Application bootstrap coordinator for modular system integration
pub struct ApplicationBootstrap {
    lifecycle: ApplicationLifecycleCoordinator,
}

impl ApplicationBootstrap {
    /// Create new bootstrap coordinator
    pub fn new() -> Self {
        Self {
            lifecycle: ApplicationLifecycleCoordinator::new(),
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
        let config = ApplicationConfig::default();
        
        web_sys::console::log_1(&"Modular system: Starting initialization".into());
        self.lifecycle.initialize(config)?;
        
        web_sys::console::log_1(&"Modular system: Starting modules".into());
        self.lifecycle.start()?;
        
        web_sys::console::log_1(&"Modular system: All modules started successfully".into());
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
        self.lifecycle.shutdown()
    }
}

impl Default for ApplicationBootstrap {
    fn default() -> Self {
        Self::new()
    }
}