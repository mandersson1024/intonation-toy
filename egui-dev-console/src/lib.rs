// egui-dev-console - EGUI-based development console
// Uses dev-console infrastructure with EGUI integration for three-d

pub mod egui_console;
pub mod history;
pub mod output;

// Re-export dev-console types
pub use dev_console::{ConsoleCommand, ConsoleCommandResult, ConsoleOutput, ConsoleCommandRegistry};
pub use history::ConsoleHistory;
pub use output::{ConsoleEntry, ConsoleOutputManager};
pub use egui_console::EguiDevConsole;