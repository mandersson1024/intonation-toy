// Debug Console Module - Command I/O interface for development
//
// Provides a reusable synchronous command interface with:
// - Command input and execution
// - Command history management with persistence
// - Output display and formatting
// - Keyboard navigation support

mod component;
mod input_handler;
mod output_renderer;

pub use component::{DebugConsole, DebugConsoleProps, DebugConsoleMsg, CommandRegistry};