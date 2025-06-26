// Services module for business logic
pub mod error_manager;
pub mod audio_engine;

// Re-export for easy access
pub use error_manager::{ErrorManager, ApplicationError, ErrorCategory, ErrorSeverity, RecoveryStrategy};
pub use audio_engine::{AudioEngineService, AudioEngineState, AudioData}; 