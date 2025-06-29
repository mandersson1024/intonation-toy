// Development Console Module
// Provides interactive debugging and development tools for pitch-toy application
// Available only in development builds

pub mod commands;
pub mod history;
pub mod output;
pub mod component;

// Re-export main component for easy access
pub use component::DevConsole;