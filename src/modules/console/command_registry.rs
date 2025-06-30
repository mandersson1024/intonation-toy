// Global Command Registry
// Manages a global registry of console commands that can be registered from anywhere in the application

use std::sync::Mutex;
use std::sync::OnceLock;

use super::commands::{CommandRegistry, CommandResult, Command};
use super::output;

// Global command registry
static GLOBAL_REGISTRY: OnceLock<Mutex<CommandRegistry>> = OnceLock::new();

/// Register a command with the global console registry
pub fn register_command(command: Box<dyn Command>) {
    let registry = GLOBAL_REGISTRY.get_or_init(|| {
        Mutex::new(CommandRegistry::new())
    });
    
    if let Ok(mut reg) = registry.lock() {
        reg.register(command);
    }
}

/// Execute a command using the global registry
pub fn execute_command(input: &str) -> CommandResult {
    let registry = GLOBAL_REGISTRY.get_or_init(|| {
        Mutex::new(CommandRegistry::new())
    });
    
    if let Ok(reg) = registry.lock() {
        reg.execute(input)
    } else {
        CommandResult::Output(output::ConsoleOutput::error("Failed to access command registry"))
    }
}
