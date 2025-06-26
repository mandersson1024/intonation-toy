// Services module for business logic
pub mod error_manager;
pub mod audio_engine;
pub mod browser_compat;
pub mod error_manager_root;
pub mod performance_monitor;

// Re-export for easy access
pub use error_manager::{ErrorManager, ApplicationError, ErrorCategory, ErrorSeverity, RecoveryStrategy};
pub use audio_engine::{AudioEngineService, AudioEngineState, AudioData};
pub use browser_compat::*;
pub use performance_monitor::*; 