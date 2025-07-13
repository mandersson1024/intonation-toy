// egui-dev-console - EGUI-based development console
// Clone of dev-console crate adapted for EGUI integration with three-d

pub mod command;
pub mod output;
pub mod history;
pub mod command_registry;
pub mod egui_console;

pub use command::{ConsoleCommand, ConsoleCommandResult};
pub use output::{ConsoleOutput, ConsoleEntry, ConsoleOutputManager};
pub use history::ConsoleHistory;
pub use command_registry::ConsoleCommandRegistry;
pub use egui_console::EguiDevConsole;