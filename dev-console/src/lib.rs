// Development Console Library
// Provides reusable development and debugging tools

mod command;
mod command_registry;
mod history;
mod output;
mod component;

// Only export what's actually used by other crates
pub use command::{ConsoleCommand, ConsoleCommandResult};
pub use command_registry::ConsoleCommandRegistry;
pub use output::{ConsoleOutput, ConsoleOutputManager};
pub use history::ConsoleHistory; 