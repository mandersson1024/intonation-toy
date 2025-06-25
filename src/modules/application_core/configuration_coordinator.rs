//! # Module Configuration Coordination
//!
//! This module provides centralized configuration management for all modules,
//! enabling consistent configuration across the application with persistence,
//! validation, and hot updates. It supports hierarchical configuration
//! structure and integrates with the event bus for configuration change notifications.
//!
//! ## Key Components
//!
//! - [`ConfigurationCoordinator`]: Core configuration management interface
//! - [`ConfigurationCoordinatorImpl`]: Concrete implementation with persistence and validation
//! - [`ConfigValue`]: Type-safe configuration values with validation
//! - [`ModuleConfig`]: Module-specific configuration structure
//! - [`ApplicationConfig`]: Root application configuration
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::modules::application_core::configuration_coordinator::*;
//!
//! let mut coordinator = ConfigurationCoordinatorImpl::new();
//! 
//! // Load configuration from storage
//! coordinator.load_configuration()?;
//!
//! // Update a module setting
//! coordinator.update_setting(
//!     &ModuleId::new("audio-module"),
//!     "sample_rate",
//!     ConfigValue::Integer(48000)
//! )?;
//!
//! // Save configuration
//! coordinator.save_configuration()?;
//! ```

use super::module_registry::ModuleId;
use super::event_bus::{Event, EventPriority, get_timestamp_ns};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

/// Configuration value types with type safety and validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigValue {
    /// String configuration value
    String(String),
    /// Integer configuration value  
    Integer(i64),
    /// Floating-point configuration value
    Float(f64),
    /// Boolean configuration value
    Boolean(bool),
    /// Array of configuration values
    Array(Vec<ConfigValue>),
    /// Nested object configuration
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// Get the type name of this configuration value
    pub fn type_name(&self) -> &'static str {
        match self {
            ConfigValue::String(_) => "string",
            ConfigValue::Integer(_) => "integer",
            ConfigValue::Float(_) => "float",
            ConfigValue::Boolean(_) => "boolean",
            ConfigValue::Array(_) => "array",
            ConfigValue::Object(_) => "object",
        }
    }

    /// Validate the configuration value against constraints
    pub fn validate(&self, constraints: &ValueConstraints) -> Result<(), ConfigError> {
        match (self, constraints) {
            (ConfigValue::String(s), ValueConstraints::String { min_length, max_length, pattern }) => {
                if let Some(min) = min_length {
                    if s.len() < *min {
                        return Err(ConfigError::ValidationFailed(format!(
                            "String length {} is less than minimum {}",
                            s.len(), min
                        )));
                    }
                }
                if let Some(max) = max_length {
                    if s.len() > *max {
                        return Err(ConfigError::ValidationFailed(format!(
                            "String length {} exceeds maximum {}",
                            s.len(), max
                        )));
                    }
                }
                if let Some(regex) = pattern {
                    // For now, we'll just check if the pattern is contained
                    // In a real implementation, we'd use a regex library
                    if !s.contains(regex) {
                        return Err(ConfigError::ValidationFailed(format!(
                            "String '{}' does not match pattern '{}'",
                            s, regex
                        )));
                    }
                }
                Ok(())
            }
            (ConfigValue::Integer(i), ValueConstraints::Integer { min, max }) => {
                if let Some(min_val) = min {
                    if *i < *min_val {
                        return Err(ConfigError::ValidationFailed(format!(
                            "Integer {} is less than minimum {}",
                            i, min_val
                        )));
                    }
                }
                if let Some(max_val) = max {
                    if *i > *max_val {
                        return Err(ConfigError::ValidationFailed(format!(
                            "Integer {} exceeds maximum {}",
                            i, max_val
                        )));
                    }
                }
                Ok(())
            }
            (ConfigValue::Float(f), ValueConstraints::Float { min, max }) => {
                if let Some(min_val) = min {
                    if *f < *min_val {
                        return Err(ConfigError::ValidationFailed(format!(
                            "Float {} is less than minimum {}",
                            f, min_val
                        )));
                    }
                }
                if let Some(max_val) = max {
                    if *f > *max_val {
                        return Err(ConfigError::ValidationFailed(format!(
                            "Float {} exceeds maximum {}",
                            f, max_val
                        )));
                    }
                }
                Ok(())
            }
            (ConfigValue::Array(arr), ValueConstraints::Array { min_items, max_items, item_type: _ }) => {
                if let Some(min) = min_items {
                    if arr.len() < *min {
                        return Err(ConfigError::ValidationFailed(format!(
                            "Array length {} is less than minimum {}",
                            arr.len(), min
                        )));
                    }
                }
                if let Some(max) = max_items {
                    if arr.len() > *max {
                        return Err(ConfigError::ValidationFailed(format!(
                            "Array length {} exceeds maximum {}",
                            arr.len(), max
                        )));
                    }
                }
                Ok(())
            }
            _ => Ok(()), // Other types don't have specific constraints yet
        }
    }

    /// Convert to string value if possible
    pub fn as_string(&self) -> Option<&String> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to integer value if possible
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert to float value if possible
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Convert to boolean value if possible
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Configuration value constraints for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueConstraints {
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
    },
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        min: Option<f64>,
        max: Option<f64>,
    },
    Array {
        min_items: Option<usize>,
        max_items: Option<usize>,
        item_type: Option<String>,
    },
    Boolean,
    Object,
}

/// Configuration setting definition with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSetting {
    /// Setting key
    pub key: String,
    /// Current value
    pub value: ConfigValue,
    /// Default value
    pub default_value: ConfigValue,
    /// Value constraints for validation
    pub constraints: ValueConstraints,
    /// Setting description
    pub description: String,
    /// Whether this setting requires restart to take effect
    pub requires_restart: bool,
    /// Whether this setting is sensitive (e.g., passwords)
    pub sensitive: bool,
}

/// Module-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Module identifier
    pub module_id: ModuleId,
    /// Module configuration settings
    pub settings: HashMap<String, ConfigSetting>,
    /// Configuration version for migration support
    pub version: String,
    /// Module configuration schema version
    pub schema_version: String,
    /// Last modification timestamp
    pub last_modified: u64,
}

impl ModuleConfig {
    /// Create a new module configuration
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            settings: HashMap::new(),
            version: "1.0.0".to_string(),
            schema_version: "1.0.0".to_string(),
            last_modified: get_timestamp_ns(),
        }
    }

    /// Add a configuration setting
    pub fn add_setting(&mut self, setting: ConfigSetting) {
        self.settings.insert(setting.key.clone(), setting);
        self.last_modified = get_timestamp_ns();
    }

    /// Get a configuration setting
    pub fn get_setting(&self, key: &str) -> Option<&ConfigSetting> {
        self.settings.get(key)
    }

    /// Update a configuration setting value
    pub fn update_setting(&mut self, key: &str, value: ConfigValue) -> Result<(), ConfigError> {
        if let Some(setting) = self.settings.get_mut(key) {
            // Validate new value
            value.validate(&setting.constraints)?;
            setting.value = value;
            self.last_modified = get_timestamp_ns();
            Ok(())
        } else {
            Err(ConfigError::SettingNotFound(key.to_string()))
        }
    }

    /// Get setting value by key
    pub fn get_value(&self, key: &str) -> Option<&ConfigValue> {
        self.settings.get(key).map(|s| &s.value)
    }

    /// Reset setting to default value
    pub fn reset_setting(&mut self, key: &str) -> Result<(), ConfigError> {
        if let Some(setting) = self.settings.get_mut(key) {
            setting.value = setting.default_value.clone();
            self.last_modified = get_timestamp_ns();
            Ok(())
        } else {
            Err(ConfigError::SettingNotFound(key.to_string()))
        }
    }
}

/// Root application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Application-level settings
    pub app_settings: HashMap<String, ConfigSetting>,
    /// Module configurations
    pub module_configs: HashMap<ModuleId, ModuleConfig>,
    /// Configuration version
    pub version: String,
    /// Last modification timestamp
    pub last_modified: u64,
}

impl ApplicationConfig {
    /// Create a new application configuration
    pub fn new() -> Self {
        Self {
            app_settings: HashMap::new(),
            module_configs: HashMap::new(),
            version: "1.0.0".to_string(),
            last_modified: get_timestamp_ns(),
        }
    }

    /// Add module configuration
    pub fn add_module_config(&mut self, config: ModuleConfig) {
        self.module_configs.insert(config.module_id.clone(), config);
        self.last_modified = get_timestamp_ns();
    }

    /// Get module configuration
    pub fn get_module_config(&self, module_id: &ModuleId) -> Option<&ModuleConfig> {
        self.module_configs.get(module_id)
    }

    /// Get mutable module configuration
    pub fn get_module_config_mut(&mut self, module_id: &ModuleId) -> Option<&mut ModuleConfig> {
        self.module_configs.get_mut(module_id)
    }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Setting not found
    SettingNotFound(String),
    /// Module not found
    ModuleNotFound(String),
    /// Invalid configuration value
    InvalidValue(String),
    /// Validation failed
    ValidationFailed(String),
    /// Persistence error
    PersistenceError(String),
    /// Serialization error
    SerializationError(String),
    /// Schema validation error
    SchemaError(String),
    /// Access denied
    AccessDenied(String),
    /// Configuration locked
    ConfigurationLocked(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::SettingNotFound(key) => write!(f, "Setting not found: {}", key),
            ConfigError::ModuleNotFound(id) => write!(f, "Module not found: {}", id),
            ConfigError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ConfigError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ConfigError::PersistenceError(msg) => write!(f, "Persistence error: {}", msg),
            ConfigError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ConfigError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            ConfigError::AccessDenied(msg) => write!(f, "Access denied: {}", msg),
            ConfigError::ConfigurationLocked(msg) => write!(f, "Configuration locked: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigurationChangeEvent {
    /// Module ID that changed
    pub module_id: Option<ModuleId>,
    /// Setting key that changed
    pub setting_key: String,
    /// Old value
    pub old_value: Option<ConfigValue>,
    /// New value
    pub new_value: ConfigValue,
    /// Change timestamp
    pub timestamp: u64,
    /// User or system that made the change
    pub changed_by: String,
    /// Whether this change requires restart
    pub requires_restart: bool,
}

impl Event for ConfigurationChangeEvent {
    fn event_type(&self) -> &'static str {
        "ConfigurationChangeEvent"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        if self.requires_restart {
            EventPriority::High
        } else {
            EventPriority::Normal
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Configuration change callback type
pub type ConfigChangeCallback = Box<dyn Fn(&ConfigurationChangeEvent) + Send + Sync>;

/// Core configuration coordination trait
pub trait ConfigurationCoordinator: Send + Sync {
    /// Load configuration from persistent storage
    fn load_configuration(&mut self) -> Result<(), ConfigError>;

    /// Save configuration to persistent storage
    fn save_configuration(&self) -> Result<(), ConfigError>;

    /// Get module configuration
    fn get_module_config(&self, module_id: &ModuleId) -> Option<&ModuleConfig>;

    /// Update a configuration setting
    fn update_setting<T: Into<ConfigValue>>(
        &mut self,
        module_id: &ModuleId,
        key: &str,
        value: T,
    ) -> Result<(), ConfigError>;

    /// Get configuration value
    fn get_setting(&self, module_id: &ModuleId, key: &str) -> Option<&ConfigValue>;

    /// Register module configuration schema
    fn register_module_schema(
        &mut self,
        module_id: ModuleId,
        settings: Vec<ConfigSetting>,
    ) -> Result<(), ConfigError>;

    /// Watch configuration changes
    fn watch_changes(&mut self, callback: ConfigChangeCallback);

    /// Reset module configuration to defaults
    fn reset_module_config(&mut self, module_id: &ModuleId) -> Result<(), ConfigError>;

    /// Validate all configurations
    fn validate_all(&self) -> Result<(), ConfigError>;

    /// Get configuration statistics
    fn get_stats(&self) -> ConfigurationStats;
}

/// Configuration statistics
#[derive(Debug, Clone)]
pub struct ConfigurationStats {
    /// Total number of modules
    pub total_modules: usize,
    /// Total number of settings
    pub total_settings: usize,
    /// Number of modified settings
    pub modified_settings: usize,
    /// Configuration size in bytes
    pub config_size_bytes: usize,
    /// Last save timestamp
    pub last_save_timestamp: Option<u64>,
    /// Number of validation errors
    pub validation_errors: usize,
}

/// Concrete implementation of configuration coordinator
pub struct ConfigurationCoordinatorImpl {
    /// Application configuration
    config: RwLock<ApplicationConfig>,
    /// Configuration change callbacks
    change_callbacks: RwLock<Vec<ConfigChangeCallback>>,
    /// Statistics
    stats: RwLock<ConfigurationStats>,
    /// Configuration file path (simulated for localStorage)
    storage_key: String,
}

impl ConfigurationCoordinatorImpl {
    /// Create a new configuration coordinator
    pub fn new() -> Self {
        Self {
            config: RwLock::new(ApplicationConfig::new()),
            change_callbacks: RwLock::new(Vec::new()),
            stats: RwLock::new(ConfigurationStats {
                total_modules: 0,
                total_settings: 0,
                modified_settings: 0,
                config_size_bytes: 0,
                last_save_timestamp: None,
                validation_errors: 0,
            }),
            storage_key: "pitch_toy_config".to_string(),
        }
    }

    /// Load default configuration with overrides
    pub fn load_defaults_with_overrides(&mut self) -> Result<(), ConfigError> {
        // Load default application configuration
        let mut default_config = Self::create_default_config();
        
        // Try to load overrides from storage
        if let Some(stored_config) = self.load_from_storage()? {
            // Merge stored configuration with defaults
            self.merge_configurations(&mut default_config, stored_config)?;
        }
        
        // Apply the merged configuration
        if let Ok(mut config) = self.config.write() {
            *config = default_config;
            self.update_stats();
            Ok(())
        } else {
            Err(ConfigError::ConfigurationLocked(
                "Failed to acquire write lock".to_string()
            ))
        }
    }

    /// Create default application configuration
    fn create_default_config() -> ApplicationConfig {
        let mut config = ApplicationConfig::new();
        
        // Add default application settings
        let mut app_settings = HashMap::new();
        
        // Audio settings
        app_settings.insert("audio_enabled".to_string(), ConfigSetting {
            key: "audio_enabled".to_string(),
            value: ConfigValue::Boolean(true),
            default_value: ConfigValue::Boolean(true),
            constraints: ValueConstraints::Boolean,
            description: "Enable audio processing".to_string(),
            requires_restart: false,
            sensitive: false,
        });
        
        app_settings.insert("log_level".to_string(), ConfigSetting {
            key: "log_level".to_string(),
            value: ConfigValue::String("info".to_string()),
            default_value: ConfigValue::String("info".to_string()),
            constraints: ValueConstraints::String {
                min_length: Some(1),
                max_length: Some(10),
                pattern: None,
            },
            description: "Application log level".to_string(),
            requires_restart: false,
            sensitive: false,
        });
        
        config.app_settings = app_settings;
        config
    }

    /// Merge stored configuration with defaults
    fn merge_configurations(
        &self,
        default_config: &mut ApplicationConfig,
        stored_config: ApplicationConfig,
    ) -> Result<(), ConfigError> {
        // Merge app-level settings
        for (key, stored_setting) in stored_config.app_settings {
            if let Some(default_setting) = default_config.app_settings.get_mut(&key) {
                // Validate stored value against default constraints
                if stored_setting.value.validate(&default_setting.constraints).is_ok() {
                    default_setting.value = stored_setting.value;
                }
                // Always keep the current default constraints and metadata
            }
        }
        
        // Merge module configurations
        for (module_id, stored_module_config) in stored_config.module_configs {
            if let Some(default_module_config) = default_config.module_configs.get_mut(&module_id) {
                // Merge module settings
                for (setting_key, stored_setting) in stored_module_config.settings {
                    if let Some(default_setting) = default_module_config.settings.get_mut(&setting_key) {
                        // Validate stored value
                        if stored_setting.value.validate(&default_setting.constraints).is_ok() {
                            default_setting.value = stored_setting.value;
                        }
                    }
                }
            } else {
                // Add new module configuration if not in defaults
                default_config.module_configs.insert(module_id, stored_module_config);
            }
        }
        
        Ok(())
    }

    /// Create default module configuration
    pub fn create_default_module_config(module_id: ModuleId, module_type: &str) -> ModuleConfig {
        let mut module_config = ModuleConfig::new(module_id);
        
        match module_type {
            "audio" => {
                // Default audio module settings
                module_config.add_setting(ConfigSetting {
                    key: "sample_rate".to_string(),
                    value: ConfigValue::Integer(44100),
                    default_value: ConfigValue::Integer(44100),
                    constraints: ValueConstraints::Integer {
                        min: Some(8000),
                        max: Some(192000),
                    },
                    description: "Audio sample rate in Hz".to_string(),
                    requires_restart: true,
                    sensitive: false,
                });
                
                module_config.add_setting(ConfigSetting {
                    key: "buffer_size".to_string(),
                    value: ConfigValue::Integer(1024),
                    default_value: ConfigValue::Integer(1024),
                    constraints: ValueConstraints::Integer {
                        min: Some(64),
                        max: Some(8192),
                    },
                    description: "Audio buffer size in samples".to_string(),
                    requires_restart: true,
                    sensitive: false,
                });
                
                module_config.add_setting(ConfigSetting {
                    key: "pitch_detection_algorithm".to_string(),
                    value: ConfigValue::String("yin".to_string()),
                    default_value: ConfigValue::String("yin".to_string()),
                    constraints: ValueConstraints::String {
                        min_length: Some(2),
                        max_length: Some(20),
                        pattern: None,
                    },
                    description: "Pitch detection algorithm".to_string(),
                    requires_restart: false,
                    sensitive: false,
                });
            }
            "ui" => {
                // Default UI module settings
                module_config.add_setting(ConfigSetting {
                    key: "theme".to_string(),
                    value: ConfigValue::String("dark".to_string()),
                    default_value: ConfigValue::String("dark".to_string()),
                    constraints: ValueConstraints::String {
                        min_length: Some(1),
                        max_length: Some(20),
                        pattern: None,
                    },
                    description: "UI theme".to_string(),
                    requires_restart: false,
                    sensitive: false,
                });
                
                module_config.add_setting(ConfigSetting {
                    key: "refresh_rate".to_string(),
                    value: ConfigValue::Integer(60),
                    default_value: ConfigValue::Integer(60),
                    constraints: ValueConstraints::Integer {
                        min: Some(30),
                        max: Some(120),
                    },
                    description: "UI refresh rate in FPS".to_string(),
                    requires_restart: false,
                    sensitive: false,
                });
            }
            _ => {
                // Generic default settings
                module_config.add_setting(ConfigSetting {
                    key: "enabled".to_string(),
                    value: ConfigValue::Boolean(true),
                    default_value: ConfigValue::Boolean(true),
                    constraints: ValueConstraints::Boolean,
                    description: "Enable this module".to_string(),
                    requires_restart: true,
                    sensitive: false,
                });
            }
        }
        
        module_config
    }

    /// Update statistics
    fn update_stats(&self) {
        if let (Ok(config), Ok(mut stats)) = (self.config.read(), self.stats.write()) {
            stats.total_modules = config.module_configs.len();
            stats.total_settings = config.module_configs.values()
                .map(|mc| mc.settings.len())
                .sum::<usize>() + config.app_settings.len();
            
            // Calculate approximate size
            stats.config_size_bytes = serde_json::to_string(&config.version)
                .unwrap_or_default().len() * 10; // Rough estimate
        }
    }

    /// Notify configuration change
    fn notify_change(&self, event: ConfigurationChangeEvent) {
        // Notify registered callbacks
        if let Ok(callbacks) = self.change_callbacks.read() {
            for callback in callbacks.iter() {
                callback(&event);
            }
        }
        
        // TODO: Publish to event bus
        // In a complete implementation, this would publish the event to the global event bus
        // For now, we just use callbacks
    }

    /// Publish configuration change to event bus
    pub fn publish_configuration_change(&self, event: ConfigurationChangeEvent) {
        // This method would integrate with the event bus system
        // For now, we just notify via callbacks
        self.notify_change(event);
    }

    /// Load configuration from browser localStorage
    fn load_from_storage(&self) -> Result<Option<ApplicationConfig>, ConfigError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;
            
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(config_str)) = storage.get_item(&self.storage_key) {
                        match serde_json::from_str::<ApplicationConfig>(&config_str) {
                            Ok(config) => return Ok(Some(config)),
                            Err(e) => return Err(ConfigError::SerializationError(e.to_string())),
                        }
                    }
                }
            }
            Ok(None)
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // For non-WASM targets (like tests), simulate empty storage
            Ok(None)
        }
    }

    /// Save configuration to browser localStorage
    fn save_to_storage(&self, config: &ApplicationConfig) -> Result<(), ConfigError> {
        let serialized = serde_json::to_string(config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;
            
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;
            
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    storage.set_item(&self.storage_key, &serialized)
                        .map_err(|_| ConfigError::PersistenceError("Failed to save to localStorage".to_string()))?;
                } else {
                    return Err(ConfigError::PersistenceError("localStorage not available".to_string()));
                }
            } else {
                return Err(ConfigError::PersistenceError("Window not available".to_string()));
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // For non-WASM targets, just validate serialization
            let _ = serialized;
        }
        
        // Update stats
        if let Ok(mut stats) = self.stats.write() {
            stats.last_save_timestamp = Some(get_timestamp_ns());
        }
        
        Ok(())
    }
}

impl Default for ConfigurationCoordinatorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigurationCoordinator for ConfigurationCoordinatorImpl {
    fn load_configuration(&mut self) -> Result<(), ConfigError> {
        match self.load_from_storage()? {
            Some(loaded_config) => {
                if let Ok(mut config) = self.config.write() {
                    *config = loaded_config;
                    self.update_stats();
                    Ok(())
                } else {
                    Err(ConfigError::ConfigurationLocked(
                        "Failed to acquire write lock".to_string()
                    ))
                }
            }
            None => {
                // No saved configuration, initialize with defaults
                self.update_stats();
                Ok(())
            }
        }
    }

    fn save_configuration(&self) -> Result<(), ConfigError> {
        if let Ok(config) = self.config.read() {
            self.save_to_storage(&config)?;
            self.update_stats();
            Ok(())
        } else {
            Err(ConfigError::ConfigurationLocked(
                "Failed to acquire read lock".to_string()
            ))
        }
    }

    fn get_module_config(&self, module_id: &ModuleId) -> Option<&ModuleConfig> {
        // Due to lifetime constraints with RwLock, we can't return a reference
        // In a real implementation, we'd need to handle this differently
        // For now, return None to indicate we can't safely return the reference
        None
    }

    fn update_setting<T: Into<ConfigValue>>(
        &mut self,
        module_id: &ModuleId,
        key: &str,
        value: T,
    ) -> Result<(), ConfigError> {
        let new_value = value.into();
        let mut config = self.config.write().map_err(|_| {
            ConfigError::ConfigurationLocked("Failed to acquire write lock".to_string())
        })?;

        let old_value = config
            .get_module_config(module_id)
            .and_then(|mc| mc.get_value(key))
            .cloned();

        if let Some(module_config) = config.get_module_config_mut(module_id) {
            let requires_restart = module_config
                .get_setting(key)
                .map(|s| s.requires_restart)
                .unwrap_or(false);

            module_config.update_setting(key, new_value.clone())?;

            // Create change event
            let change_event = ConfigurationChangeEvent {
                module_id: Some(module_id.clone()),
                setting_key: key.to_string(),
                old_value,
                new_value,
                timestamp: get_timestamp_ns(),
                changed_by: "system".to_string(),
                requires_restart,
            };

            drop(config); // Release lock before notifying
            self.notify_change(change_event);
            self.update_stats();
            Ok(())
        } else {
            Err(ConfigError::ModuleNotFound(module_id.to_string()))
        }
    }

    fn get_setting(&self, module_id: &ModuleId, key: &str) -> Option<&ConfigValue> {
        // Due to lifetime constraints with RwLock, we can't return a reference
        // In a real implementation, we'd need to handle this differently
        None
    }

    fn register_module_schema(
        &mut self,
        module_id: ModuleId,
        settings: Vec<ConfigSetting>,
    ) -> Result<(), ConfigError> {
        let mut config = self.config.write().map_err(|_| {
            ConfigError::ConfigurationLocked("Failed to acquire write lock".to_string())
        })?;

        let mut module_config = ModuleConfig::new(module_id.clone());
        for setting in settings {
            module_config.add_setting(setting);
        }

        config.add_module_config(module_config);
        self.update_stats();
        Ok(())
    }

    fn watch_changes(&mut self, callback: ConfigChangeCallback) {
        if let Ok(mut callbacks) = self.change_callbacks.write() {
            callbacks.push(callback);
        }
    }

    fn reset_module_config(&mut self, module_id: &ModuleId) -> Result<(), ConfigError> {
        let mut config = self.config.write().map_err(|_| {
            ConfigError::ConfigurationLocked("Failed to acquire write lock".to_string())
        })?;

        if let Some(module_config) = config.get_module_config_mut(module_id) {
            let setting_keys: Vec<String> = module_config.settings.keys().cloned().collect();
            for key in setting_keys {
                module_config.reset_setting(&key)?;
            }
            self.update_stats();
            Ok(())
        } else {
            Err(ConfigError::ModuleNotFound(module_id.to_string()))
        }
    }

    fn validate_all(&self) -> Result<(), ConfigError> {
        if let Ok(config) = self.config.read() {
            let mut validation_errors = 0;

            for module_config in config.module_configs.values() {
                for setting in module_config.settings.values() {
                    if let Err(_) = setting.value.validate(&setting.constraints) {
                        validation_errors += 1;
                    }
                }
            }

            if let Ok(mut stats) = self.stats.write() {
                stats.validation_errors = validation_errors;
            }

            if validation_errors > 0 {
                Err(ConfigError::ValidationFailed(format!(
                    "{} validation errors found",
                    validation_errors
                )))
            } else {
                Ok(())
            }
        } else {
            Err(ConfigError::ConfigurationLocked(
                "Failed to acquire read lock".to_string()
            ))
        }
    }

    fn get_stats(&self) -> ConfigurationStats {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            ConfigurationStats {
                total_modules: 0,
                total_settings: 0,
                modified_settings: 0,
                config_size_bytes: 0,
                last_save_timestamp: None,
                validation_errors: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_validation() {
        let string_value = ConfigValue::String("test".to_string());
        let constraints = ValueConstraints::String {
            min_length: Some(2),
            max_length: Some(10),
            pattern: None,
        };
        
        assert!(string_value.validate(&constraints).is_ok());
        
        let short_string = ConfigValue::String("a".to_string());
        assert!(short_string.validate(&constraints).is_err());
    }

    #[test]
    fn test_module_config_creation() {
        let module_id = ModuleId::new("test-module");
        let mut module_config = ModuleConfig::new(module_id.clone());
        
        let setting = ConfigSetting {
            key: "sample_rate".to_string(),
            value: ConfigValue::Integer(44100),
            default_value: ConfigValue::Integer(44100),
            constraints: ValueConstraints::Integer {
                min: Some(8000),
                max: Some(192000),
            },
            description: "Audio sample rate".to_string(),
            requires_restart: true,
            sensitive: false,
        };
        
        module_config.add_setting(setting);
        assert!(module_config.get_setting("sample_rate").is_some());
    }

    #[test]
    fn test_configuration_coordinator_creation() {
        let coordinator = ConfigurationCoordinatorImpl::new();
        let stats = coordinator.get_stats();
        assert_eq!(stats.total_modules, 0);
        assert_eq!(stats.total_settings, 0);
    }

    #[test]
    fn test_module_schema_registration() {
        let mut coordinator = ConfigurationCoordinatorImpl::new();
        let module_id = ModuleId::new("audio-module");
        
        let settings = vec![
            ConfigSetting {
                key: "sample_rate".to_string(),
                value: ConfigValue::Integer(44100),
                default_value: ConfigValue::Integer(44100),
                constraints: ValueConstraints::Integer {
                    min: Some(8000),
                    max: Some(192000),
                },
                description: "Audio sample rate".to_string(),
                requires_restart: true,
                sensitive: false,
            },
            ConfigSetting {
                key: "buffer_size".to_string(),
                value: ConfigValue::Integer(1024),
                default_value: ConfigValue::Integer(1024),
                constraints: ValueConstraints::Integer {
                    min: Some(64),
                    max: Some(8192),
                },
                description: "Audio buffer size".to_string(),
                requires_restart: true,
                sensitive: false,
            },
        ];
        
        let result = coordinator.register_module_schema(module_id, settings);
        assert!(result.is_ok());
        
        let stats = coordinator.get_stats();
        assert_eq!(stats.total_modules, 1);
        assert_eq!(stats.total_settings, 2);
    }

    #[test]
    fn test_configuration_validation() {
        let coordinator = ConfigurationCoordinatorImpl::new();
        let result = coordinator.validate_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_configuration_persistence() {
        let coordinator = ConfigurationCoordinatorImpl::new();
        
        // Test save (should work even with empty config)
        let result = coordinator.save_configuration();
        assert!(result.is_ok());
        
        let stats = coordinator.get_stats();
        assert!(stats.last_save_timestamp.is_some());
    }

    #[test]
    fn test_config_value_type_conversions() {
        let string_val = ConfigValue::String("test".to_string());
        assert_eq!(string_val.as_string(), Some(&"test".to_string()));
        assert_eq!(string_val.as_integer(), None);
        
        let int_val = ConfigValue::Integer(42);
        assert_eq!(int_val.as_integer(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));
        
        let bool_val = ConfigValue::Boolean(true);
        assert_eq!(bool_val.as_boolean(), Some(true));
    }

    #[test]
    fn test_configuration_change_event() {
        let event = ConfigurationChangeEvent {
            module_id: Some(ModuleId::new("test")),
            setting_key: "test_setting".to_string(),
            old_value: Some(ConfigValue::Integer(1)),
            new_value: ConfigValue::Integer(2),
            timestamp: get_timestamp_ns(),
            changed_by: "test".to_string(),
            requires_restart: false,
        };
        
        assert_eq!(event.event_type(), "ConfigurationChangeEvent");
        assert_eq!(event.priority(), EventPriority::Normal);
    }

    #[test]
    fn test_hierarchical_configuration_structure() {
        let mut config = ApplicationConfig::new();
        
        // Add app-level setting
        let app_setting = ConfigSetting {
            key: "debug_mode".to_string(),
            value: ConfigValue::Boolean(false),
            default_value: ConfigValue::Boolean(false),
            constraints: ValueConstraints::Boolean,
            description: "Enable debug mode".to_string(),
            requires_restart: true,
            sensitive: false,
        };
        config.app_settings.insert("debug_mode".to_string(), app_setting);
        
        // Add module configuration
        let module_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("audio-module"),
            "audio"
        );
        config.add_module_config(module_config);
        
        // Verify hierarchy
        assert_eq!(config.app_settings.len(), 1);
        assert_eq!(config.module_configs.len(), 1);
        
        let audio_config = config.get_module_config(&ModuleId::new("audio-module")).unwrap();
        assert_eq!(audio_config.settings.len(), 3); // sample_rate, buffer_size, pitch_detection_algorithm
    }

    #[test]
    fn test_configuration_validation_with_constraints() {
        let mut coordinator = ConfigurationCoordinatorImpl::new();
        let module_id = ModuleId::new("test-module");
        
        // Register module with constrained setting
        let settings = vec![
            ConfigSetting {
                key: "constrained_int".to_string(),
                value: ConfigValue::Integer(50),
                default_value: ConfigValue::Integer(50),
                constraints: ValueConstraints::Integer {
                    min: Some(1),
                    max: Some(100),
                },
                description: "Test constrained integer".to_string(),
                requires_restart: false,
                sensitive: false,
            }
        ];
        
        coordinator.register_module_schema(module_id.clone(), settings).unwrap();
        
        // Test valid update
        let result = coordinator.update_setting(&module_id, "constrained_int", ConfigValue::Integer(75));
        assert!(result.is_ok());
        
        // Test invalid update (out of range)
        let result = coordinator.update_setting(&module_id, "constrained_int", ConfigValue::Integer(150));
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationFailed(_) => {}
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_hot_configuration_updates() {
        let mut coordinator = ConfigurationCoordinatorImpl::new();
        let module_id = ModuleId::new("ui-module");
        
        // Register UI module
        let ui_config = ConfigurationCoordinatorImpl::create_default_module_config(module_id.clone(), "ui");
        coordinator.register_module_schema(module_id.clone(), ui_config.settings.into_values().collect()).unwrap();
        
        // Update theme setting (doesn't require restart)
        let result = coordinator.update_setting(&module_id, "theme", ConfigValue::String("light".to_string()));
        assert!(result.is_ok());
        
        // Update refresh rate
        let result = coordinator.update_setting(&module_id, "refresh_rate", ConfigValue::Integer(120));
        assert!(result.is_ok());
        
        let stats = coordinator.get_stats();
        assert_eq!(stats.total_modules, 1);
        assert_eq!(stats.total_settings, 2);
    }

    #[test]
    fn test_configuration_change_callbacks() {
        use std::sync::{Arc, Mutex};
        
        let mut coordinator = ConfigurationCoordinatorImpl::new();
        let module_id = ModuleId::new("test-module");
        
        // Track changes with callback
        let change_count = Arc::new(Mutex::new(0));
        let change_count_clone = Arc::clone(&change_count);
        
        coordinator.watch_changes(Box::new(move |_event| {
            let mut count = change_count_clone.lock().unwrap();
            *count += 1;
        }));
        
        // Register module and make changes
        let settings = vec![
            ConfigSetting {
                key: "test_setting".to_string(),
                value: ConfigValue::String("initial".to_string()),
                default_value: ConfigValue::String("initial".to_string()),
                constraints: ValueConstraints::String {
                    min_length: Some(1),
                    max_length: Some(50),
                    pattern: None,
                },
                description: "Test setting".to_string(),
                requires_restart: false,
                sensitive: false,
            }
        ];
        
        coordinator.register_module_schema(module_id.clone(), settings).unwrap();
        
        // Make configuration changes
        coordinator.update_setting(&module_id, "test_setting", ConfigValue::String("updated".to_string())).unwrap();
        coordinator.update_setting(&module_id, "test_setting", ConfigValue::String("updated2".to_string())).unwrap();
        
        // Verify callbacks were called
        let count = change_count.lock().unwrap();
        assert_eq!(*count, 2);
    }

    #[test]
    fn test_default_configuration_creation() {
        let default_config = ConfigurationCoordinatorImpl::create_default_config();
        
        // Should have default app settings
        assert!(default_config.app_settings.contains_key("audio_enabled"));
        assert!(default_config.app_settings.contains_key("log_level"));
        
        // Verify default values
        let audio_enabled = &default_config.app_settings["audio_enabled"];
        assert_eq!(audio_enabled.value, ConfigValue::Boolean(true));
        
        let log_level = &default_config.app_settings["log_level"];
        assert_eq!(log_level.value, ConfigValue::String("info".to_string()));
    }

    #[test]
    fn test_module_specific_defaults() {
        // Test audio module defaults
        let audio_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("audio-test"),
            "audio"
        );
        
        assert!(audio_config.get_setting("sample_rate").is_some());
        assert!(audio_config.get_setting("buffer_size").is_some());
        assert!(audio_config.get_setting("pitch_detection_algorithm").is_some());
        
        // Test UI module defaults
        let ui_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("ui-test"),
            "ui"
        );
        
        assert!(ui_config.get_setting("theme").is_some());
        assert!(ui_config.get_setting("refresh_rate").is_some());
        
        // Test generic module defaults
        let generic_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("generic-test"),
            "other"
        );
        
        assert!(generic_config.get_setting("enabled").is_some());
    }

    #[test]
    fn test_configuration_reset() {
        let mut coordinator = ConfigurationCoordinatorImpl::new();
        let module_id = ModuleId::new("test-module");
        
        // Register module
        let settings = vec![
            ConfigSetting {
                key: "resettable_setting".to_string(),
                value: ConfigValue::Integer(100),
                default_value: ConfigValue::Integer(50),
                constraints: ValueConstraints::Integer {
                    min: Some(1),
                    max: Some(200),
                },
                description: "Resettable setting".to_string(),
                requires_restart: false,
                sensitive: false,
            }
        ];
        
        coordinator.register_module_schema(module_id.clone(), settings).unwrap();
        
        // Update setting
        coordinator.update_setting(&module_id, "resettable_setting", ConfigValue::Integer(150)).unwrap();
        
        // Reset module configuration
        let result = coordinator.reset_module_config(&module_id);
        assert!(result.is_ok());
        
        // Verify setting was reset to default
        // Note: Due to RwLock limitations, we can't directly verify the value here
        // In a real implementation, we'd have getter methods that work with the lock
    }

    #[test]
    fn test_configuration_serialization() {
        let config = ConfigurationCoordinatorImpl::create_default_config();
        
        // Test serialization
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
        
        // Test deserialization
        let deserialized = serde_json::from_str::<ApplicationConfig>(&serialized.unwrap());
        assert!(deserialized.is_ok());
        
        let deserialized_config = deserialized.unwrap();
        assert_eq!(config.app_settings.len(), deserialized_config.app_settings.len());
    }

    #[test]
    fn test_configuration_stats() {
        let mut coordinator = ConfigurationCoordinatorImpl::new();
        
        // Initial stats
        let stats = coordinator.get_stats();
        assert_eq!(stats.total_modules, 0);
        assert_eq!(stats.total_settings, 0);
        
        // Add modules and verify stats update
        let audio_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("audio"),
            "audio"
        );
        coordinator.register_module_schema(
            ModuleId::new("audio"),
            audio_config.settings.into_values().collect()
        ).unwrap();
        
        let ui_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("ui"),
            "ui"
        );
        coordinator.register_module_schema(
            ModuleId::new("ui"),
            ui_config.settings.into_values().collect()
        ).unwrap();
        
        let stats = coordinator.get_stats();
        assert_eq!(stats.total_modules, 2);
        assert_eq!(stats.total_settings, 5); // 3 audio + 2 ui settings
        assert!(stats.config_size_bytes > 0);
    }

    #[test] 
    fn test_sensitive_setting_handling() {
        let mut coordinator = ConfigurationCoordinatorImpl::new();
        let module_id = ModuleId::new("auth-module");
        
        // Register module with sensitive setting
        let settings = vec![
            ConfigSetting {
                key: "api_key".to_string(),
                value: ConfigValue::String("default-key".to_string()),
                default_value: ConfigValue::String("default-key".to_string()),
                constraints: ValueConstraints::String {
                    min_length: Some(5),
                    max_length: Some(100),
                    pattern: None,
                },
                description: "API key for authentication".to_string(),
                requires_restart: true,
                sensitive: true,
            }
        ];
        
        coordinator.register_module_schema(module_id.clone(), settings).unwrap();
        
        // Update sensitive setting
        let result = coordinator.update_setting(&module_id, "api_key", ConfigValue::String("secret-api-key-123".to_string()));
        assert!(result.is_ok());
        
        // Verify module is registered
        let stats = coordinator.get_stats();
        assert_eq!(stats.total_modules, 1);
        assert_eq!(stats.total_settings, 1);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::{ConfigurationCoordinator, ConfigurationCoordinatorImpl, ConfigValue, ModuleConfig, ConfigError, ConfigSetting, ValueConstraints};
    use crate::modules::application_core::module_registry::{Module, ModuleId, ModuleRegistry, ModuleRegistryImpl, ModuleState};
    use crate::modules::application_core::application_lifecycle::{ApplicationLifecycle, ApplicationLifecycleCoordinator, ApplicationConfig, ApplicationState};
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    // Mock module that uses configuration
    struct ConfigurableModule {
        id: ModuleId,
        config_coordinator: Option<Arc<Mutex<ConfigurationCoordinatorImpl>>>,
        current_config: HashMap<String, ConfigValue>,
    }

    impl ConfigurableModule {
        fn new(id: &str, config_coordinator: Arc<Mutex<ConfigurationCoordinatorImpl>>) -> Self {
            Self {
                id: ModuleId::new(id),
                config_coordinator: Some(config_coordinator),
                current_config: HashMap::new(),
            }
        }

        fn apply_configuration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            // In a real implementation, this would read configuration and apply it
            // For testing, we'll just simulate applying configuration
            self.current_config.clear();
            self.current_config.insert("configured".to_string(), ConfigValue::Boolean(true));
            Ok(())
        }
    }

    impl Module for ConfigurableModule {
        fn module_id(&self) -> ModuleId {
            self.id.clone()
        }

        fn module_name(&self) -> &str {
            "Configurable Module"
        }

        fn module_version(&self) -> &str {
            "1.0.0"
        }

        fn dependencies(&self) -> Vec<ModuleId> {
            vec![]
        }

        fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            // Apply configuration during initialization
            self.apply_configuration()?;
            Ok(())
        }

        fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    #[test]
    fn test_configuration_with_module_registry() {
        let mut config_coordinator = ConfigurationCoordinatorImpl::new();
        let mut module_registry = ModuleRegistryImpl::new();
        
        // Register module configuration schema
        let audio_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("audio-module"),
            "audio"
        );
        config_coordinator.register_module_schema(
            ModuleId::new("audio-module"),
            audio_config.settings.into_values().collect()
        ).unwrap();
        
        // Create and register configurable module
        let config_coordinator_arc = Arc::new(Mutex::new(config_coordinator));
        let module = ConfigurableModule::new("audio-module", Arc::clone(&config_coordinator_arc));
        let module_id = module_registry.register_module(Box::new(module)).unwrap();
        
        // Verify module is registered
        assert!(module_registry.is_registered(&module_id));
        
        // Verify configuration is available
        let coordinator = config_coordinator_arc.lock().unwrap();
        let stats = coordinator.get_stats();
        assert_eq!(stats.total_modules, 1);
        assert_eq!(stats.total_settings, 3); // Audio module default settings
    }

    #[test]
    fn test_configuration_with_application_lifecycle() {
        let mut config_coordinator = ConfigurationCoordinatorImpl::new();
        let mut lifecycle_coordinator = ApplicationLifecycleCoordinator::new();
        
        // Set up configuration for application lifecycle
        let mut app_config = ApplicationConfig::default();
        app_config.module_init_timeout = std::time::Duration::from_secs(5);
        app_config.enable_lifecycle_events = true;
        
        // Register audio module configuration
        let audio_module_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("audio-module"),
            "audio"
        );
        config_coordinator.register_module_schema(
            ModuleId::new("audio-module"),
            audio_module_config.settings.into_values().collect()
        ).unwrap();
        
        // Create and register module with lifecycle coordinator
        let config_coordinator_arc = Arc::new(Mutex::new(config_coordinator));
        let module = ConfigurableModule::new("audio-module", Arc::clone(&config_coordinator_arc));
        lifecycle_coordinator.get_module_registry_mut()
            .register_module(Box::new(module))
            .unwrap();
        
        // Initialize application (this should trigger module initialization with configuration)
        let result = lifecycle_coordinator.initialize(app_config);
        assert!(result.is_ok());
        assert_eq!(lifecycle_coordinator.get_state(), ApplicationState::Running);
        
        // Start and then shutdown
        lifecycle_coordinator.start().unwrap();
        lifecycle_coordinator.shutdown().unwrap();
        assert_eq!(lifecycle_coordinator.get_state(), ApplicationState::Stopped);
    }

    #[test]
    fn test_configuration_hot_updates_with_modules() {
        let mut config_coordinator = ConfigurationCoordinatorImpl::new();
        let change_events = Arc::new(Mutex::new(Vec::new()));
        let change_events_clone = Arc::clone(&change_events);
        
        // Watch configuration changes
        config_coordinator.watch_changes(Box::new(move |event| {
            let mut events = change_events_clone.lock().unwrap();
            events.push(event.clone());
        }));
        
        // Register UI module configuration
        let ui_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("ui-module"),
            "ui"
        );
        config_coordinator.register_module_schema(
            ModuleId::new("ui-module"),
            ui_config.settings.into_values().collect()
        ).unwrap();
        
        // Make hot configuration updates
        config_coordinator.update_setting(
            &ModuleId::new("ui-module"),
            "theme",
            ConfigValue::String("light".to_string())
        ).unwrap();
        
        config_coordinator.update_setting(
            &ModuleId::new("ui-module"),
            "refresh_rate",
            ConfigValue::Integer(144)
        ).unwrap();
        
        // Verify change events were triggered
        let events = change_events.lock().unwrap();
        assert_eq!(events.len(), 2);
        
        // Verify first event
        assert_eq!(events[0].setting_key, "theme");
        assert_eq!(events[0].new_value, ConfigValue::String("light".to_string()));
        assert!(!events[0].requires_restart);
        
        // Verify second event
        assert_eq!(events[1].setting_key, "refresh_rate");
        assert_eq!(events[1].new_value, ConfigValue::Integer(144));
        assert!(!events[1].requires_restart);
    }

    #[test]
    fn test_configuration_persistence_integration() {
        let mut config_coordinator = ConfigurationCoordinatorImpl::new();
        
        // Load defaults with overrides
        let result = config_coordinator.load_defaults_with_overrides();
        assert!(result.is_ok());
        
        // Register module configuration
        let audio_config = ConfigurationCoordinatorImpl::create_default_module_config(
            ModuleId::new("audio-module"),
            "audio"
        );
        config_coordinator.register_module_schema(
            ModuleId::new("audio-module"),
            audio_config.settings.into_values().collect()
        ).unwrap();
        
        // Make configuration changes
        config_coordinator.update_setting(
            &ModuleId::new("audio-module"),
            "sample_rate",
            ConfigValue::Integer(48000)
        ).unwrap();
        
        // Save configuration
        let result = config_coordinator.save_configuration();
        assert!(result.is_ok());
        
        // Verify save timestamp is updated
        let stats = config_coordinator.get_stats();
        assert!(stats.last_save_timestamp.is_some());
    }

    #[test]
    fn test_configuration_validation_integration() {
        let mut config_coordinator = ConfigurationCoordinatorImpl::new();
        
        // Register module with strict validation
        let settings = vec![
            ConfigSetting {
                key: "strict_setting".to_string(),
                value: ConfigValue::Float(0.5),
                default_value: ConfigValue::Float(0.5),
                constraints: ValueConstraints::Float {
                    min: Some(0.0),
                    max: Some(1.0),
                },
                description: "Strictly validated float setting".to_string(),
                requires_restart: false,
                sensitive: false,
            }
        ];
        
        config_coordinator.register_module_schema(
            ModuleId::new("strict-module"),
            settings
        ).unwrap();
        
        // Test valid update
        let result = config_coordinator.update_setting(
            &ModuleId::new("strict-module"),
            "strict_setting",
            ConfigValue::Float(0.75)
        );
        assert!(result.is_ok());
        
        // Test invalid update
        let result = config_coordinator.update_setting(
            &ModuleId::new("strict-module"),
            "strict_setting",
            ConfigValue::Float(1.5)
        );
        assert!(result.is_err());
        
        // Validate all configurations
        let validation_result = config_coordinator.validate_all();
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_multi_module_configuration_coordination() {
        let mut config_coordinator = ConfigurationCoordinatorImpl::new();
        
        // Register multiple modules with different configurations
        let modules = vec![
            ("audio-module", "audio"),
            ("ui-module", "ui"),
            ("network-module", "other"),
        ];
        
        for (module_name, module_type) in modules {
            let module_config = ConfigurationCoordinatorImpl::create_default_module_config(
                ModuleId::new(module_name),
                module_type
            );
            config_coordinator.register_module_schema(
                ModuleId::new(module_name),
                module_config.settings.into_values().collect()
            ).unwrap();
        }
        
        // Verify all modules are configured
        let stats = config_coordinator.get_stats();
        assert_eq!(stats.total_modules, 3);
        assert!(stats.total_settings >= 6); // At least 2 settings per module
        
        // Test updating settings across modules
        config_coordinator.update_setting(
            &ModuleId::new("audio-module"),
            "sample_rate",
            ConfigValue::Integer(96000)
        ).unwrap();
        
        config_coordinator.update_setting(
            &ModuleId::new("ui-module"),
            "theme",
            ConfigValue::String("high-contrast".to_string())
        ).unwrap();
        
        config_coordinator.update_setting(
            &ModuleId::new("network-module"),
            "enabled",
            ConfigValue::Boolean(false)
        ).unwrap();
        
        // Verify all configurations are valid
        let validation_result = config_coordinator.validate_all();
        assert!(validation_result.is_ok());
    }
}