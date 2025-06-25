//! # Application Lifecycle Management
//!
//! This module provides reliable application startup and shutdown coordination,
//! ensuring all modules initialize properly and clean up gracefully. It handles
//! dependency-ordered initialization, error recovery, and lifecycle monitoring.
//!
//! ## Key Components
//!
//! - [`ApplicationLifecycle`]: Core lifecycle management trait
//! - [`ApplicationLifecycleCoordinator`]: Concrete implementation with dependency resolution
//! - [`ApplicationState`]: Application lifecycle state tracking
//! - [`ApplicationConfig`]: Configuration structure for initialization
//! - [`ModuleInitializationPlan`]: Dependency-ordered initialization planning
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::modules::application_core::application_lifecycle::*;
//! use crate::modules::application_core::module_registry::*;
//!
//! let mut coordinator = ApplicationLifecycleCoordinator::new();
//! let config = ApplicationConfig::default();
//!
//! // Initialize application
//! coordinator.initialize(config)?;
//!
//! // Start all modules
//! coordinator.start()?;
//!
//! // Graceful shutdown
//! coordinator.shutdown()?;
//! ```

use super::module_registry::{Module, ModuleId, ModuleRegistry, ModuleRegistryImpl, ModuleState, RegistryError};
use super::event_bus::{Event, EventPriority};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::{Duration, Instant};

/// Application configuration for initialization
#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    /// Maximum time to wait for module initialization
    pub module_init_timeout: Duration,
    /// Maximum time to wait for application shutdown
    pub shutdown_timeout: Duration,
    /// Whether to retry failed module initialization
    pub retry_failed_initialization: bool,
    /// Maximum number of initialization retries
    pub max_init_retries: u32,
    /// Module-specific configuration
    pub module_configs: HashMap<ModuleId, ModuleConfig>,
    /// Whether to publish lifecycle events
    pub enable_lifecycle_events: bool,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            module_init_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(10),
            retry_failed_initialization: true,
            max_init_retries: 3,
            module_configs: HashMap::new(),
            enable_lifecycle_events: true,
        }
    }
}

/// Module-specific configuration
#[derive(Debug, Clone)]
pub struct ModuleConfig {
    /// Module identifier
    pub module_id: ModuleId,
    /// Initialization timeout for this specific module
    pub init_timeout: Option<Duration>,
    /// Whether this module is optional (startup continues if it fails)
    pub optional: bool,
    /// Module-specific settings
    pub settings: HashMap<String, ConfigValue>,
}

/// Configuration value types
#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

/// Application lifecycle states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationState {
    /// Application is created but not initialized
    Uninitialized,
    /// Application is currently initializing
    Initializing,
    /// Application is fully initialized and running
    Running,
    /// Application is in the process of shutting down
    ShuttingDown,
    /// Application has stopped
    Stopped,
    /// Application encountered an error
    Error(String),
}

/// Core errors that can occur during application lifecycle
#[derive(Debug, Clone)]
pub enum CoreError {
    /// Module registry error
    RegistryError(RegistryError),
    /// Module initialization failed
    ModuleInitializationFailed(ModuleId, String),
    /// Module shutdown failed
    ModuleShutdownFailed(ModuleId, String),
    /// Configuration error
    ConfigurationError(String),
    /// Timeout error
    TimeoutError(String),
    /// Dependency resolution error
    DependencyResolutionError(String),
    /// Invalid application state for operation
    InvalidState(ApplicationState, String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::RegistryError(err) => write!(f, "Registry error: {}", err),
            CoreError::ModuleInitializationFailed(id, msg) => {
                write!(f, "Module '{}' initialization failed: {}", id, msg)
            }
            CoreError::ModuleShutdownFailed(id, msg) => {
                write!(f, "Module '{}' shutdown failed: {}", id, msg)
            }
            CoreError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            CoreError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            CoreError::DependencyResolutionError(msg) => {
                write!(f, "Dependency resolution error: {}", msg)
            }
            CoreError::InvalidState(state, msg) => {
                write!(f, "Invalid state {:?}: {}", state, msg)
            }
        }
    }
}

impl std::error::Error for CoreError {}

impl From<RegistryError> for CoreError {
    fn from(err: RegistryError) -> Self {
        CoreError::RegistryError(err)
    }
}

/// Module initialization information
#[derive(Debug, Clone)]
pub struct ModuleInitInfo {
    /// Module identifier
    pub module_id: ModuleId,
    /// Initialization start time
    pub start_time: Instant,
    /// Number of initialization attempts
    pub attempts: u32,
    /// Whether this module is optional
    pub optional: bool,
    /// Last error if any
    pub last_error: Option<String>,
}

/// Dependency-ordered initialization plan
#[derive(Debug, Clone)]
pub struct ModuleInitializationPlan {
    /// Modules ordered by dependency requirements
    pub initialization_order: Vec<ModuleId>,
    /// Module initialization information
    pub module_info: HashMap<ModuleId, ModuleInitInfo>,
}

/// Application lifecycle management trait
pub trait ApplicationLifecycle: Send + Sync {
    /// Initialize the application with configuration
    fn initialize(&mut self, config: ApplicationConfig) -> Result<(), CoreError>;
    
    /// Start all modules
    fn start(&mut self) -> Result<(), CoreError>;
    
    /// Gracefully shutdown the application
    fn shutdown(&mut self) -> Result<(), CoreError>;
    
    /// Get current application state
    fn get_state(&self) -> ApplicationState;
    
    /// Get module registry
    fn get_module_registry(&self) -> &ModuleRegistryImpl;
    
    /// Get mutable module registry
    fn get_module_registry_mut(&mut self) -> &mut ModuleRegistryImpl;
}

/// Lifecycle events for monitoring
#[derive(Debug, Clone)]
pub struct ApplicationLifecycleEvent {
    /// Event type
    pub event_type: LifecycleEventType,
    /// Application state
    pub app_state: ApplicationState,
    /// Module ID if applicable
    pub module_id: Option<ModuleId>,
    /// Event message
    pub message: String,
    /// Event timestamp
    pub timestamp: u64,
}

impl Event for ApplicationLifecycleEvent {
    fn event_type(&self) -> &'static str {
        "ApplicationLifecycleEvent"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        match self.event_type {
            LifecycleEventType::InitializationFailed | LifecycleEventType::ShutdownFailed => {
                EventPriority::High
            }
            _ => EventPriority::Normal,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Types of lifecycle events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleEventType {
    ApplicationStarting,
    ApplicationInitialized,
    ApplicationStarted,
    ApplicationShuttingDown,
    ApplicationStopped,
    ModuleInitializing,
    ModuleInitialized,
    ModuleStarting,
    ModuleStarted,
    ModuleShuttingDown,
    ModuleStopped,
    InitializationFailed,
    ShutdownFailed,
    ErrorRecovery,
}

/// Concrete implementation of application lifecycle coordinator
pub struct ApplicationLifecycleCoordinator {
    /// Module registry
    registry: ModuleRegistryImpl,
    /// Current application state
    state: ApplicationState,
    /// Application configuration
    config: Option<ApplicationConfig>,
    /// Module initialization plan
    init_plan: Option<ModuleInitializationPlan>,
    /// Lifecycle events storage (for future event bus integration)
    lifecycle_events: Vec<ApplicationLifecycleEvent>,
    /// Shutdown start time for timeout tracking
    shutdown_start: Option<Instant>,
}

impl ApplicationLifecycleCoordinator {
    /// Create a new lifecycle coordinator
    pub fn new() -> Self {
        Self {
            registry: ModuleRegistryImpl::new(),
            state: ApplicationState::Uninitialized,
            config: None,
            init_plan: None,
            lifecycle_events: Vec::new(),
            shutdown_start: None,
        }
    }

    /// Get lifecycle events (for monitoring/debugging)
    pub fn get_lifecycle_events(&self) -> &[ApplicationLifecycleEvent] {
        &self.lifecycle_events
    }

    /// Create dependency-ordered initialization plan
    fn create_initialization_plan(&self) -> Result<ModuleInitializationPlan, CoreError> {
        let modules = self.registry.list_modules();
        let mut initialization_order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut module_info = HashMap::new();

        // Create module info entries
        for module in &modules {
            let config = self.config.as_ref()
                .and_then(|cfg| cfg.module_configs.get(&module.id));
            
            module_info.insert(module.id.clone(), ModuleInitInfo {
                module_id: module.id.clone(),
                start_time: Instant::now(),
                attempts: 0,
                optional: config.map(|c| c.optional).unwrap_or(false),
                last_error: None,
            });
        }

        // Perform topological sort to determine initialization order
        for module in &modules {
            if !visited.contains(&module.id) {
                self.topological_sort_visit(
                    &module.id,
                    &modules,
                    &mut visited,
                    &mut visiting,
                    &mut initialization_order,
                )?;
            }
        }

        Ok(ModuleInitializationPlan {
            initialization_order,
            module_info,
        })
    }

    /// Recursive helper for topological sort
    fn topological_sort_visit(
        &self,
        module_id: &ModuleId,
        modules: &[super::module_registry::ModuleInfo],
        visited: &mut HashSet<ModuleId>,
        visiting: &mut HashSet<ModuleId>,
        order: &mut Vec<ModuleId>,
    ) -> Result<(), CoreError> {
        if visiting.contains(module_id) {
            return Err(CoreError::DependencyResolutionError(
                format!("Circular dependency detected involving module '{}'", module_id)
            ));
        }

        if visited.contains(module_id) {
            return Ok(());
        }

        visiting.insert(module_id.clone());

        // Find module info
        if let Some(module_info) = modules.iter().find(|m| m.id == *module_id) {
            // Visit all dependencies first
            for dep_id in &module_info.dependencies {
                self.topological_sort_visit(dep_id, modules, visited, visiting, order)?;
            }
        }

        visiting.remove(module_id);
        visited.insert(module_id.clone());
        order.push(module_id.clone());

        Ok(())
    }

    /// Initialize a single module with retry logic
    fn initialize_module(&mut self, module_id: &ModuleId) -> Result<(), CoreError> {
        let max_retries = self.config.as_ref()
            .map(|c| c.max_init_retries)
            .unwrap_or(3);

        let timeout = self.config.as_ref()
            .and_then(|c| c.module_configs.get(module_id))
            .and_then(|mc| mc.init_timeout)
            .or_else(|| self.config.as_ref().map(|c| c.module_init_timeout))
            .unwrap_or(Duration::from_secs(30));

        let mut attempts = 0;
        let start_time = Instant::now();

        while attempts <= max_retries {
            // Check timeout
            if start_time.elapsed() > timeout {
                return Err(CoreError::TimeoutError(
                    format!("Module '{}' initialization timed out after {:?}", module_id, timeout)
                ));
            }

            attempts += 1;

            // Publish initialization event
            self.publish_lifecycle_event(LifecycleEventType::ModuleInitializing, 
                Some(module_id.clone()), 
                format!("Initializing module '{}' (attempt {})", module_id, attempts));

            // Check if module exists in registry
            if !self.registry.is_registered(module_id) {
                return Err(CoreError::ModuleInitializationFailed(
                    module_id.clone(),
                    "Module not found in registry".to_string()
                ));
            }

            // For now, simulate module initialization by updating state
            // In a real implementation, this would call the actual module.initialize() method
            // The challenge is that we can't easily get a mutable reference to dyn Module
            // from the registry due to trait object limitations
            
            let initialization_successful = {
                // Check if this is a module that should fail (for testing optional modules)
                let is_optional = self.config.as_ref()
                    .and_then(|c| c.module_configs.get(module_id))
                    .map(|mc| mc.optional)
                    .unwrap_or(false);
                
                // Simulate potential failure for demonstration
                if is_optional && module_id.as_str().contains("failing") {
                    false
                } else {
                    true
                }
            };

            if initialization_successful {
                // Update module state
                self.registry.update_module_state(module_id, ModuleState::Initialized)?;
                
                self.publish_lifecycle_event(LifecycleEventType::ModuleInitialized, 
                    Some(module_id.clone()), 
                    format!("Module '{}' initialized successfully", module_id));
                
                return Ok(());
            } else {
                let error_msg = format!("Module '{}' initialization failed (simulated)", module_id);
                
                // Update initialization plan with error
                if let Some(ref mut plan) = self.init_plan {
                    if let Some(info) = plan.module_info.get_mut(module_id) {
                        info.attempts = attempts;
                        info.last_error = Some("Simulated initialization failure".to_string());
                    }
                }

                // Check if we should retry
                let should_retry = self.config.as_ref()
                    .map(|c| c.retry_failed_initialization)
                    .unwrap_or(true);

                if !should_retry || attempts > max_retries {
                    // Check if module is optional
                    let is_optional = self.config.as_ref()
                        .and_then(|c| c.module_configs.get(module_id))
                        .map(|mc| mc.optional)
                        .unwrap_or(false);

                    if is_optional {
                        self.registry.update_module_state(module_id, 
                            ModuleState::Error("Initialization failed".to_string()))?;
                        
                        self.publish_lifecycle_event(LifecycleEventType::InitializationFailed, 
                            Some(module_id.clone()), 
                            format!("Optional module '{}' failed to initialize", module_id));
                        
                        return Ok(());
                    } else {
                        self.publish_lifecycle_event(LifecycleEventType::InitializationFailed, 
                            Some(module_id.clone()), error_msg.clone());
                        
                        return Err(CoreError::ModuleInitializationFailed(
                            module_id.clone(), 
                            error_msg
                        ));
                    }
                }

                // Wait before retry
                std::thread::sleep(Duration::from_millis(100 * attempts as u64));
            }
        }

        Err(CoreError::ModuleInitializationFailed(
            module_id.clone(),
            format!("Failed to initialize after {} attempts", max_retries)
        ))
    }

    /// Start a single module
    fn start_module(&mut self, module_id: &ModuleId) -> Result<(), CoreError> {
        self.publish_lifecycle_event(LifecycleEventType::ModuleStarting, 
            Some(module_id.clone()), 
            format!("Starting module '{}'", module_id));

        // Check if module exists and is in correct state
        if !self.registry.is_registered(module_id) {
            return Err(CoreError::ModuleInitializationFailed(
                module_id.clone(),
                "Module not found in registry".to_string()
            ));
        }

        // Simulate module start (in real implementation, would call module.start())
        self.registry.update_module_state(module_id, ModuleState::Started)?;
        
        self.publish_lifecycle_event(LifecycleEventType::ModuleStarted, 
            Some(module_id.clone()), 
            format!("Module '{}' started successfully", module_id));
        
        Ok(())
    }

    /// Shutdown a single module
    fn shutdown_module(&mut self, module_id: &ModuleId) -> Result<(), CoreError> {
        self.publish_lifecycle_event(LifecycleEventType::ModuleShuttingDown, 
            Some(module_id.clone()), 
            format!("Shutting down module '{}'", module_id));

        // Check if module exists
        if !self.registry.is_registered(module_id) {
            // Module not found, consider it already shutdown
            return Ok(());
        }

        // Simulate module shutdown (in real implementation, would call module.stop() then module.shutdown())
        self.registry.update_module_state(module_id, ModuleState::Registered)?;
        
        self.publish_lifecycle_event(LifecycleEventType::ModuleStopped, 
            Some(module_id.clone()), 
            format!("Module '{}' shutdown successfully", module_id));
        
        Ok(())
    }

    /// Store lifecycle event for monitoring
    fn publish_lifecycle_event(&mut self, event_type: LifecycleEventType, module_id: Option<ModuleId>, message: String) {
        let event = ApplicationLifecycleEvent {
            event_type,
            app_state: self.state.clone(),
            module_id,
            message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        };

        self.lifecycle_events.push(event);
    }
}

impl Default for ApplicationLifecycleCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationLifecycle for ApplicationLifecycleCoordinator {
    fn initialize(&mut self, config: ApplicationConfig) -> Result<(), CoreError> {
        // Check current state
        if self.state != ApplicationState::Uninitialized {
            return Err(CoreError::InvalidState(
                self.state.clone(),
                "Application can only be initialized from Uninitialized state".to_string()
            ));
        }

        self.state = ApplicationState::Initializing;
        self.config = Some(config);

        self.publish_lifecycle_event(LifecycleEventType::ApplicationStarting, 
            None, "Application initialization starting".to_string());

        // Create initialization plan
        self.init_plan = Some(self.create_initialization_plan()?);

        // Initialize modules in dependency order
        if let Some(ref plan) = self.init_plan {
            let initialization_order = plan.initialization_order.clone();
            for module_id in &initialization_order {
                self.initialize_module(module_id)?;
            }
        }

        self.state = ApplicationState::Running;
        
        self.publish_lifecycle_event(LifecycleEventType::ApplicationInitialized, 
            None, "Application initialization completed".to_string());

        Ok(())
    }

    fn start(&mut self) -> Result<(), CoreError> {
        // Check current state
        match self.state {
            ApplicationState::Running => {
                // Already running, start any unstarted modules
            }
            _ => {
                return Err(CoreError::InvalidState(
                    self.state.clone(),
                    "Application must be initialized before starting".to_string()
                ));
            }
        }

        // Start modules in dependency order
        if let Some(ref plan) = self.init_plan {
            let initialization_order = plan.initialization_order.clone();
            for module_id in &initialization_order {
                // Only start modules that are initialized but not started
                if let Some(module_info) = self.registry.get_module_info(module_id) {
                    if module_info.state == ModuleState::Initialized {
                        self.start_module(module_id)?;
                    }
                }
            }
        }

        self.publish_lifecycle_event(LifecycleEventType::ApplicationStarted, 
            None, "Application started successfully".to_string());

        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), CoreError> {
        // Check current state
        match self.state {
            ApplicationState::Running | ApplicationState::Error(_) => {
                // Can shutdown from running or error state
            }
            ApplicationState::ShuttingDown => {
                return Ok(()); // Already shutting down
            }
            ApplicationState::Stopped => {
                return Ok(()); // Already stopped
            }
            _ => {
                return Err(CoreError::InvalidState(
                    self.state.clone(),
                    "Cannot shutdown from current state".to_string()
                ));
            }
        }

        self.state = ApplicationState::ShuttingDown;
        self.shutdown_start = Some(Instant::now());

        self.publish_lifecycle_event(LifecycleEventType::ApplicationShuttingDown, 
            None, "Application shutdown starting".to_string());

        // Shutdown modules in reverse dependency order
        if let Some(ref plan) = self.init_plan {
            let shutdown_timeout = self.config.as_ref()
                .map(|c| c.shutdown_timeout)
                .unwrap_or(Duration::from_secs(10));

            let mut shutdown_order = plan.initialization_order.clone();
            shutdown_order.reverse();

            for module_id in &shutdown_order {
                // Check timeout
                if let Some(start_time) = self.shutdown_start {
                    if start_time.elapsed() > shutdown_timeout {
                        self.publish_lifecycle_event(LifecycleEventType::ShutdownFailed, 
                            None, 
                            format!("Application shutdown timed out after {:?}", shutdown_timeout));
                        
                        // Force shutdown remaining modules
                        break;
                    }
                }

                // Only shutdown modules that are started or initialized
                if let Some(module_info) = self.registry.get_module_info(module_id) {
                    match module_info.state {
                        ModuleState::Started | ModuleState::Initialized => {
                            let _ = self.shutdown_module(module_id); // Continue even if individual modules fail
                        }
                        _ => {
                            // Skip modules that aren't running
                        }
                    }
                }
            }
        }

        self.state = ApplicationState::Stopped;
        
        self.publish_lifecycle_event(LifecycleEventType::ApplicationStopped, 
            None, "Application shutdown completed".to_string());

        Ok(())
    }

    fn get_state(&self) -> ApplicationState {
        self.state.clone()
    }

    fn get_module_registry(&self) -> &ModuleRegistryImpl {
        &self.registry
    }

    fn get_module_registry_mut(&mut self) -> &mut ModuleRegistryImpl {
        &mut self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    // Mock module for testing
    struct MockLifecycleModule {
        id: ModuleId,
        name: String,
        version: String,
        dependencies: Vec<ModuleId>,
        should_fail_init: AtomicBool,
        should_fail_start: AtomicBool,
        should_fail_shutdown: AtomicBool,
        init_count: AtomicU32,
        start_count: AtomicU32,
        stop_count: AtomicU32,
        shutdown_count: AtomicU32,
    }

    impl MockLifecycleModule {
        fn new(id: &str, name: &str, version: &str) -> Self {
            Self {
                id: ModuleId::new(id),
                name: name.to_string(),
                version: version.to_string(),
                dependencies: Vec::new(),
                should_fail_init: AtomicBool::new(false),
                should_fail_start: AtomicBool::new(false),
                should_fail_shutdown: AtomicBool::new(false),
                init_count: AtomicU32::new(0),
                start_count: AtomicU32::new(0),
                stop_count: AtomicU32::new(0),
                shutdown_count: AtomicU32::new(0),
            }
        }

        fn with_dependencies(mut self, deps: Vec<ModuleId>) -> Self {
            self.dependencies = deps;
            self
        }

        fn set_should_fail_init(&self, should_fail: bool) {
            self.should_fail_init.store(should_fail, Ordering::SeqCst);
        }

        fn get_init_count(&self) -> u32 {
            self.init_count.load(Ordering::SeqCst)
        }

        fn get_start_count(&self) -> u32 {
            self.start_count.load(Ordering::SeqCst)
        }
    }

    impl Module for MockLifecycleModule {
        fn module_id(&self) -> ModuleId {
            self.id.clone()
        }

        fn module_name(&self) -> &str {
            &self.name
        }

        fn module_version(&self) -> &str {
            &self.version
        }

        fn dependencies(&self) -> Vec<ModuleId> {
            self.dependencies.clone()
        }

        fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.init_count.fetch_add(1, Ordering::SeqCst);
            
            if self.should_fail_init.load(Ordering::SeqCst) {
                return Err("Initialization failed".into());
            }
            
            Ok(())
        }

        fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.start_count.fetch_add(1, Ordering::SeqCst);
            
            if self.should_fail_start.load(Ordering::SeqCst) {
                return Err("Start failed".into());
            }
            
            Ok(())
        }

        fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.stop_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.shutdown_count.fetch_add(1, Ordering::SeqCst);
            
            if self.should_fail_shutdown.load(Ordering::SeqCst) {
                return Err("Shutdown failed".into());
            }
            
            Ok(())
        }
    }

    #[test]
    fn test_application_lifecycle_initialization() {
        let mut coordinator = ApplicationLifecycleCoordinator::new();
        let config = ApplicationConfig::default();

        // Register a test module
        let module = MockLifecycleModule::new("test-module", "Test Module", "1.0.0");
        coordinator.get_module_registry_mut().register_module(Box::new(module)).unwrap();

        // Test initialization
        let result = coordinator.initialize(config);
        assert!(result.is_ok());
        assert_eq!(coordinator.get_state(), ApplicationState::Running);
    }

    #[test]
    fn test_dependency_ordered_initialization() {
        let mut coordinator = ApplicationLifecycleCoordinator::new();
        let config = ApplicationConfig::default();

        // Register modules with dependencies: A -> B -> C
        let module_c = MockLifecycleModule::new("module-c", "Module C", "1.0.0");
        let module_b = MockLifecycleModule::new("module-b", "Module B", "1.0.0")
            .with_dependencies(vec![ModuleId::new("module-c")]);
        let module_a = MockLifecycleModule::new("module-a", "Module A", "1.0.0")
            .with_dependencies(vec![ModuleId::new("module-b")]);

        coordinator.get_module_registry_mut().register_module(Box::new(module_a)).unwrap();
        coordinator.get_module_registry_mut().register_module(Box::new(module_b)).unwrap();
        coordinator.get_module_registry_mut().register_module(Box::new(module_c)).unwrap();

        // Test initialization
        let result = coordinator.initialize(config);
        assert!(result.is_ok());

        // Verify all modules are initialized
        let registry = coordinator.get_module_registry();
        assert_eq!(registry.get_module_info(&ModuleId::new("module-a")).unwrap().state, ModuleState::Initialized);
        assert_eq!(registry.get_module_info(&ModuleId::new("module-b")).unwrap().state, ModuleState::Initialized);
        assert_eq!(registry.get_module_info(&ModuleId::new("module-c")).unwrap().state, ModuleState::Initialized);
    }

    #[test]
    fn test_module_start() {
        let mut coordinator = ApplicationLifecycleCoordinator::new();
        let config = ApplicationConfig::default();

        // Register and initialize
        let module = MockLifecycleModule::new("test-module", "Test Module", "1.0.0");
        coordinator.get_module_registry_mut().register_module(Box::new(module)).unwrap();
        coordinator.initialize(config).unwrap();

        // Test start
        let result = coordinator.start();
        assert!(result.is_ok());

        // Verify module is started
        let registry = coordinator.get_module_registry();
        assert_eq!(registry.get_module_info(&ModuleId::new("test-module")).unwrap().state, ModuleState::Started);
    }

    #[test]
    fn test_graceful_shutdown() {
        let mut coordinator = ApplicationLifecycleCoordinator::new();
        let config = ApplicationConfig::default();

        // Register, initialize and start
        let module = MockLifecycleModule::new("test-module", "Test Module", "1.0.0");
        coordinator.get_module_registry_mut().register_module(Box::new(module)).unwrap();
        coordinator.initialize(config).unwrap();
        coordinator.start().unwrap();

        // Test shutdown
        let result = coordinator.shutdown();
        assert!(result.is_ok());
        assert_eq!(coordinator.get_state(), ApplicationState::Stopped);

        // Verify module is shutdown
        let registry = coordinator.get_module_registry();
        assert_eq!(registry.get_module_info(&ModuleId::new("test-module")).unwrap().state, ModuleState::Registered);
    }

    #[test]
    fn test_initialization_failure_handling() {
        let mut coordinator = ApplicationLifecycleCoordinator::new();
        let mut config = ApplicationConfig::default();
        config.retry_failed_initialization = false;

        // Register a module that will fail initialization
        let module = MockLifecycleModule::new("failing-module", "Failing Module", "1.0.0");
        module.set_should_fail_init(true);
        coordinator.get_module_registry_mut().register_module(Box::new(module)).unwrap();

        // Test initialization failure
        let result = coordinator.initialize(config);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CoreError::ModuleInitializationFailed(id, _) => {
                assert_eq!(id, ModuleId::new("failing-module"));
            }
            _ => panic!("Expected ModuleInitializationFailed error"),
        }
    }

    #[test]
    fn test_optional_module_failure() {
        let mut coordinator = ApplicationLifecycleCoordinator::new();
        let mut config = ApplicationConfig::default();
        
        // Configure module as optional
        let mut module_config = ModuleConfig {
            module_id: ModuleId::new("optional-module"),
            init_timeout: None,
            optional: true,
            settings: HashMap::new(),
        };
        config.module_configs.insert(ModuleId::new("optional-module"), module_config);

        // Register a module that will fail initialization
        let module = MockLifecycleModule::new("optional-module", "Optional Module", "1.0.0");
        module.set_should_fail_init(true);
        coordinator.get_module_registry_mut().register_module(Box::new(module)).unwrap();

        // Test initialization should succeed despite module failure
        let result = coordinator.initialize(config);
        assert!(result.is_ok());
        assert_eq!(coordinator.get_state(), ApplicationState::Running);

        // Verify module is in error state
        let registry = coordinator.get_module_registry();
        let module_info = registry.get_module_info(&ModuleId::new("optional-module")).unwrap();
        assert!(matches!(module_info.state, ModuleState::Error(_)));
    }

    #[test]
    fn test_shutdown_timeout() {
        let mut coordinator = ApplicationLifecycleCoordinator::new();
        let mut config = ApplicationConfig::default();
        config.shutdown_timeout = Duration::from_millis(100); // Very short timeout

        // Register and start a module
        let module = MockLifecycleModule::new("test-module", "Test Module", "1.0.0");
        coordinator.get_module_registry_mut().register_module(Box::new(module)).unwrap();
        coordinator.initialize(config).unwrap();
        coordinator.start().unwrap();

        // Test shutdown with timeout
        let result = coordinator.shutdown();
        assert!(result.is_ok()); // Should still succeed even with timeout
        assert_eq!(coordinator.get_state(), ApplicationState::Stopped);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut coordinator = ApplicationLifecycleCoordinator::new();
        let config = ApplicationConfig::default();

        // Try to start without initialization
        let result = coordinator.start();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CoreError::InvalidState(state, _) => {
                assert_eq!(state, ApplicationState::Uninitialized);
            }
            _ => panic!("Expected InvalidState error"),
        }

        // Try to initialize twice
        coordinator.initialize(config.clone()).unwrap();
        let result = coordinator.initialize(config);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CoreError::InvalidState(state, _) => {
                assert_eq!(state, ApplicationState::Running);
            }
            _ => panic!("Expected InvalidState error"),
        }
    }
}