// Development Console Library
// Provides reusable development and debugging tools

mod command_registry;
mod history;
mod output;
mod component;
mod input_handler;
mod output_renderer;
mod component_debug;

// Only export what's actually used by other crates
pub use command_registry::{ConsoleCommand, ConsoleCommandResult, ConsoleCommandRegistry};
pub use output::{ConsoleOutput, ConsoleOutputManager};
pub use component_debug::{DebugConsole, CommandRegistry}; 