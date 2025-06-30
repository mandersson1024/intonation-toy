// Global Command Registry
// Manages a global registry of console commands that can be registered from anywhere in the application

use std::sync::Mutex;
use std::sync::OnceLock;

use super::commands::{ConsoleCommandRegistry, ConsoleCommandResult, ConsoleCommand};
use super::output;

// Global command registry
static GLOBAL_REGISTRY: OnceLock<Mutex<ConsoleCommandRegistry>> = OnceLock::new();

/// Register a command with the global console registry
pub fn register_command(command: Box<dyn ConsoleCommand>) {
    let registry = GLOBAL_REGISTRY.get_or_init(|| {
        Mutex::new(ConsoleCommandRegistry::new())
    });
    
    if let Ok(mut reg) = registry.lock() {
        reg.register(command);
    }
}

/// Execute a command using the global registry
pub fn execute_command(input: &str) -> ConsoleCommandResult {
    let registry = GLOBAL_REGISTRY.get_or_init(|| {
        Mutex::new(ConsoleCommandRegistry::new())
    });
    
    if let Ok(reg) = registry.lock() {
        reg.execute(input)
    } else {
        ConsoleCommandResult::Output(output::ConsoleOutput::error("Failed to access command registry"))
    }
}
