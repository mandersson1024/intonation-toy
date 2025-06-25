//! # Module Registry
//!
//! This module provides the central registry for all application modules, enabling
//! module registration, discovery, and dependency tracking. The registry ensures
//! unique module identification and provides O(1) lookup performance.
//!
//! ## Key Components
//!
//! - [`ModuleRegistry`]: Core registry trait for module management
//! - [`ModuleRegistryImpl`]: Concrete implementation with thread-safe operations
//! - [`ModuleInfo`]: Metadata and state information for registered modules
//! - [`ModuleState`]: Lifecycle state tracking for modules
//! - [`DependencyStatus`]: Dependency resolution status for modules
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::modules::application_core::module_registry::*;
//!
//! let mut registry = ModuleRegistryImpl::new();
//!
//! // Register a module
//! let module_id = registry.register_module(Box::new(MyModule::new()))?;
//!
//! // Lookup module by ID
//! if let Some(module) = registry.get_module::<MyModule>(&module_id) {
//!     module.do_something();
//! }
//!
//! // Check dependencies
//! let deps = registry.check_dependencies(&module_id);
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a module
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId(String);

impl ModuleId {
    /// Create a new module ID from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation of the module ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Module lifecycle states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleState {
    /// Module is created but not registered
    Unregistered,
    /// Module is registered in the registry
    Registered,
    /// Module has been initialized
    Initialized,
    /// Module is actively running
    Started,
    /// Module encountered an error
    Error(String),
}

/// Status of a module dependency
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyStatus {
    /// Dependency is satisfied (module is registered and initialized)
    Satisfied,
    /// Dependency is registered but not initialized
    Pending,
    /// Dependency is missing (not registered)
    Missing,
    /// Dependency is in error state
    Error(String),
}

/// Metadata and state information for a registered module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Unique module identifier
    pub id: ModuleId,
    /// Human-readable module name
    pub name: String,
    /// Module version
    pub version: String,
    /// Current lifecycle state
    pub state: ModuleState,
    /// List of module dependencies
    pub dependencies: Vec<ModuleId>,
    /// Module type ID for type-safe casting
    pub type_id: TypeId,
}

/// Base trait that all modules must implement
pub trait Module: ModuleAny + Send + Sync {
    /// Get the unique module ID
    fn module_id(&self) -> ModuleId;
    
    /// Get the human-readable module name
    fn module_name(&self) -> &str;
    
    /// Get the module version
    fn module_version(&self) -> &str;
    
    /// Get the list of module dependencies
    fn dependencies(&self) -> Vec<ModuleId>;
    
    /// Initialize the module
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Start the module
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Stop the module
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Shutdown the module
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Errors that can occur during module registry operations
#[derive(Debug, Clone)]
pub enum RegistryError {
    /// Module with the same ID is already registered
    DuplicateModuleId(ModuleId),
    /// Module with the given ID was not found
    ModuleNotFound(ModuleId),
    /// Invalid module metadata
    InvalidMetadata(String),
    /// Type mismatch when retrieving module
    TypeMismatch(ModuleId, String),
    /// Circular dependency detected
    CircularDependency(Vec<ModuleId>),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::DuplicateModuleId(id) => {
                write!(f, "Module with ID '{}' is already registered", id)
            }
            RegistryError::ModuleNotFound(id) => {
                write!(f, "Module with ID '{}' was not found", id)
            }
            RegistryError::InvalidMetadata(msg) => {
                write!(f, "Invalid module metadata: {}", msg)
            }
            RegistryError::TypeMismatch(id, expected) => {
                write!(f, "Type mismatch for module '{}': expected {}", id, expected)
            }
            RegistryError::CircularDependency(cycle) => {
                write!(f, "Circular dependency detected: {:?}", cycle)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Core module registry trait
pub trait ModuleRegistry: Send + Sync {
    /// Register a module in the registry
    fn register_module(&mut self, module: Box<dyn Module>) -> Result<ModuleId, RegistryError>;
    
    /// Get a module by ID with type casting
    fn get_module<T: Module + 'static>(&self, id: &ModuleId) -> Option<&T>;
    
    /// Get a mutable reference to a module by ID with type casting
    fn get_module_mut<T: Module + 'static>(&mut self, id: &ModuleId) -> Option<&mut T>;
    
    /// Check if a module is registered
    fn is_registered(&self, id: &ModuleId) -> bool;
    
    /// List all registered modules
    fn list_modules(&self) -> Vec<ModuleInfo>;
    
    /// Check dependency status for a module
    fn check_dependencies(&self, module_id: &ModuleId) -> Vec<DependencyStatus>;
    
    /// Update module state
    fn update_module_state(&mut self, id: &ModuleId, state: ModuleState) -> Result<(), RegistryError>;
    
    /// Get module info by ID
    fn get_module_info(&self, id: &ModuleId) -> Option<ModuleInfo>;
    
    /// Unregister a module
    fn unregister_module(&mut self, id: &ModuleId) -> Result<(), RegistryError>;
}

/// Internal registry entry
struct RegistryEntry {
    module: Box<dyn Module>,
    info: ModuleInfo,
}

/// Concrete implementation of the module registry
pub struct ModuleRegistryImpl {
    modules: HashMap<ModuleId, RegistryEntry>,
}

impl ModuleRegistryImpl {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Validate module metadata
    fn validate_module_metadata(&self, module: &dyn Module) -> Result<(), RegistryError> {
        let id = module.module_id();
        
        // Check for duplicate ID
        if self.modules.contains_key(&id) {
            return Err(RegistryError::DuplicateModuleId(id));
        }
        
        // Validate ID is not empty
        if id.as_str().is_empty() {
            return Err(RegistryError::InvalidMetadata("Module ID cannot be empty".to_string()));
        }
        
        // Validate name is not empty
        if module.module_name().is_empty() {
            return Err(RegistryError::InvalidMetadata("Module name cannot be empty".to_string()));
        }
        
        // Validate version is not empty
        if module.module_version().is_empty() {
            return Err(RegistryError::InvalidMetadata("Module version cannot be empty".to_string()));
        }
        
        Ok(())
    }

    /// Detect circular dependencies
    fn detect_circular_dependencies(&self, module_id: &ModuleId, dependencies: &[ModuleId]) -> Result<(), RegistryError> {
        let mut visited = std::collections::HashSet::new();
        let mut path = Vec::new();
        
        for dep_id in dependencies {
            if self.has_circular_dependency(dep_id, module_id, &mut visited, &mut path)? {
                path.push(module_id.clone());
                return Err(RegistryError::CircularDependency(path));
            }
        }
        
        Ok(())
    }

    /// Recursive helper for circular dependency detection
    fn has_circular_dependency(
        &self,
        current: &ModuleId,
        target: &ModuleId,
        visited: &mut std::collections::HashSet<ModuleId>,
        path: &mut Vec<ModuleId>,
    ) -> Result<bool, RegistryError> {
        if current == target {
            return Ok(true);
        }
        
        if visited.contains(current) {
            return Ok(false);
        }
        
        visited.insert(current.clone());
        path.push(current.clone());
        
        if let Some(entry) = self.modules.get(current) {
            for dep_id in &entry.info.dependencies {
                if self.has_circular_dependency(dep_id, target, visited, path)? {
                    return Ok(true);
                }
            }
        }
        
        path.pop();
        Ok(false)
    }
}

impl Default for ModuleRegistryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRegistry for ModuleRegistryImpl {
    fn register_module(&mut self, module: Box<dyn Module>) -> Result<ModuleId, RegistryError> {
        // Validate module metadata
        self.validate_module_metadata(&*module)?;
        
        let id = module.module_id();
        let dependencies = module.dependencies();
        
        // Check for circular dependencies
        self.detect_circular_dependencies(&id, &dependencies)?;
        
        // Create module info
        let info = ModuleInfo {
            id: id.clone(),
            name: module.module_name().to_string(),
            version: module.module_version().to_string(),
            state: ModuleState::Registered,
            dependencies: dependencies.clone(),
            type_id: (&*module).type_id(),
        };
        
        // Create registry entry
        let entry = RegistryEntry { module, info };
        
        // Register the module
        self.modules.insert(id.clone(), entry);
        
        Ok(id)
    }
    
    fn get_module<T: Module + 'static>(&self, id: &ModuleId) -> Option<&T> {
        self.modules.get(id)?.module.as_any().downcast_ref::<T>()
    }
    
    fn get_module_mut<T: Module + 'static>(&mut self, id: &ModuleId) -> Option<&mut T> {
        self.modules.get_mut(id)?.module.as_any_mut().downcast_mut::<T>()
    }
    
    fn is_registered(&self, id: &ModuleId) -> bool {
        self.modules.contains_key(id)
    }
    
    fn list_modules(&self) -> Vec<ModuleInfo> {
        self.modules.values().map(|entry| entry.info.clone()).collect()
    }
    
    fn check_dependencies(&self, module_id: &ModuleId) -> Vec<DependencyStatus> {
        if let Some(entry) = self.modules.get(module_id) {
            entry.info.dependencies.iter().map(|dep_id| {
                match self.modules.get(dep_id) {
                    Some(dep_entry) => match &dep_entry.info.state {
                        ModuleState::Initialized | ModuleState::Started => DependencyStatus::Satisfied,
                        ModuleState::Registered => DependencyStatus::Pending,
                        ModuleState::Error(err) => DependencyStatus::Error(err.clone()),
                        _ => DependencyStatus::Pending,
                    },
                    None => DependencyStatus::Missing,
                }
            }).collect()
        } else {
            Vec::new()
        }
    }
    
    fn update_module_state(&mut self, id: &ModuleId, state: ModuleState) -> Result<(), RegistryError> {
        if let Some(entry) = self.modules.get_mut(id) {
            entry.info.state = state;
            Ok(())
        } else {
            Err(RegistryError::ModuleNotFound(id.clone()))
        }
    }
    
    fn get_module_info(&self, id: &ModuleId) -> Option<ModuleInfo> {
        self.modules.get(id).map(|entry| entry.info.clone())
    }
    
    fn unregister_module(&mut self, id: &ModuleId) -> Result<(), RegistryError> {
        if self.modules.remove(id).is_some() {
            Ok(())
        } else {
            Err(RegistryError::ModuleNotFound(id.clone()))
        }
    }
}

/// Extension trait to add downcast capabilities to Module trait objects
pub trait ModuleAny: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> ModuleAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Mock module for testing
    struct MockModule {
        id: ModuleId,
        name: String,
        version: String,
        dependencies: Vec<ModuleId>,
        initialized: AtomicBool,
        started: AtomicBool,
    }

    impl MockModule {
        fn new(id: &str, name: &str, version: &str) -> Self {
            Self {
                id: ModuleId::new(id),
                name: name.to_string(),
                version: version.to_string(),
                dependencies: Vec::new(),
                initialized: AtomicBool::new(false),
                started: AtomicBool::new(false),
            }
        }

        fn with_dependencies(mut self, deps: Vec<ModuleId>) -> Self {
            self.dependencies = deps;
            self
        }
    }

    impl Module for MockModule {
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
            self.initialized.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.started.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.started.store(false, Ordering::SeqCst);
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.initialized.store(false, Ordering::SeqCst);
            self.started.store(false, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_module_registration() {
        let mut registry = ModuleRegistryImpl::new();
        let module = MockModule::new("test-module", "Test Module", "1.0.0");
        let expected_id = module.module_id();

        let result = registry.register_module(Box::new(module));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
        assert!(registry.is_registered(&expected_id));
    }

    #[test]
    fn test_duplicate_module_registration() {
        let mut registry = ModuleRegistryImpl::new();
        let module1 = MockModule::new("test-module", "Test Module", "1.0.0");
        let module2 = MockModule::new("test-module", "Another Module", "2.0.0");

        let result1 = registry.register_module(Box::new(module1));
        assert!(result1.is_ok());

        let result2 = registry.register_module(Box::new(module2));
        assert!(result2.is_err());
        assert!(matches!(result2.unwrap_err(), RegistryError::DuplicateModuleId(_)));
    }

    #[test]
    fn test_module_lookup() {
        let mut registry = ModuleRegistryImpl::new();
        let module = MockModule::new("test-module", "Test Module", "1.0.0");
        let module_id = module.module_id();

        registry.register_module(Box::new(module)).unwrap();

        let retrieved = registry.get_module::<MockModule>(&module_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().module_name(), "Test Module");
    }

    #[test]
    fn test_module_state_updates() {
        let mut registry = ModuleRegistryImpl::new();
        let module = MockModule::new("test-module", "Test Module", "1.0.0");
        let module_id = module.module_id();

        registry.register_module(Box::new(module)).unwrap();

        // Update state to Initialized
        let result = registry.update_module_state(&module_id, ModuleState::Initialized);
        assert!(result.is_ok());

        let info = registry.get_module_info(&module_id).unwrap();
        assert_eq!(info.state, ModuleState::Initialized);
    }

    #[test]
    fn test_dependency_tracking() {
        let mut registry = ModuleRegistryImpl::new();
        
        // Create modules with dependencies
        let module_a = MockModule::new("module-a", "Module A", "1.0.0");
        let module_b = MockModule::new("module-b", "Module B", "1.0.0")
            .with_dependencies(vec![ModuleId::new("module-a")]);

        let module_a_id = module_a.module_id();
        let module_b_id = module_b.module_id();

        registry.register_module(Box::new(module_a)).unwrap();
        registry.register_module(Box::new(module_b)).unwrap();

        // Check dependencies
        let deps = registry.check_dependencies(&module_b_id);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], DependencyStatus::Pending);

        // Initialize module A
        registry.update_module_state(&module_a_id, ModuleState::Initialized).unwrap();

        // Check dependencies again
        let deps = registry.check_dependencies(&module_b_id);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], DependencyStatus::Satisfied);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut registry = ModuleRegistryImpl::new();
        
        let module_a = MockModule::new("module-a", "Module A", "1.0.0")
            .with_dependencies(vec![ModuleId::new("module-b")]);
        let module_b = MockModule::new("module-b", "Module B", "1.0.0")
            .with_dependencies(vec![ModuleId::new("module-a")]);

        registry.register_module(Box::new(module_a)).unwrap();
        let result = registry.register_module(Box::new(module_b));
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RegistryError::CircularDependency(_)));
    }

    #[test]
    fn test_invalid_metadata() {
        let mut registry = ModuleRegistryImpl::new();
        
        // Test empty ID
        let module = MockModule::new("", "Test Module", "1.0.0");
        let result = registry.register_module(Box::new(module));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RegistryError::InvalidMetadata(_)));
    }

    #[test]
    fn test_module_unregistration() {
        let mut registry = ModuleRegistryImpl::new();
        let module = MockModule::new("test-module", "Test Module", "1.0.0");
        let module_id = module.module_id();

        registry.register_module(Box::new(module)).unwrap();
        assert!(registry.is_registered(&module_id));

        let result = registry.unregister_module(&module_id);
        assert!(result.is_ok());
        assert!(!registry.is_registered(&module_id));
    }

    #[test]
    fn test_list_modules() {
        let mut registry = ModuleRegistryImpl::new();
        let module1 = MockModule::new("module-1", "Module 1", "1.0.0");
        let module2 = MockModule::new("module-2", "Module 2", "2.0.0");

        registry.register_module(Box::new(module1)).unwrap();
        registry.register_module(Box::new(module2)).unwrap();

        let modules = registry.list_modules();
        assert_eq!(modules.len(), 2);
        
        let module_names: Vec<&str> = modules.iter().map(|m| m.name.as_str()).collect();
        assert!(module_names.contains(&"Module 1"));
        assert!(module_names.contains(&"Module 2"));
    }
}