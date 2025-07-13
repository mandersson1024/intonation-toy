// egui-dev-console - EGUI-based development console
// Self-contained console implementation with EGUI integration for three-d

pub mod command;
pub mod command_registry;
pub mod egui_console;
pub mod history;
pub mod output;

// Re-export console types
pub use command::{ConsoleCommand, ConsoleCommandResult};
pub use command_registry::ConsoleCommandRegistry;
pub use output::{ConsoleOutput, ConsoleEntry, ConsoleOutputManager};
pub use history::ConsoleHistory;
pub use egui_console::EguiDevConsole;