pub mod application_core;

// Re-export key types for easy access
pub use application_core::{
    ApplicationLifecycleCoordinator, 
    ApplicationConfig, 
    ApplicationState, 
    CoreError,
    ModuleId,
    ModuleState
};
