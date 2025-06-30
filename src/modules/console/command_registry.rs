// Global Command Registry
// Manages a global registry of console commands that can be registered from anywhere in the application

use std::sync::Mutex;
use std::sync::OnceLock;

use super::commands::{CommandRegistry, CommandResult, DevCommand};
use super::output;

// Global command registry
static GLOBAL_REGISTRY: OnceLock<Mutex<CommandRegistry>> = OnceLock::new();

/// Register a command with the global console registry
pub fn register_command(command: Box<dyn DevCommand>) {
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

/// Get list of available commands from global registry
pub fn get_available_commands() -> Vec<(String, String)> {
    let registry = GLOBAL_REGISTRY.get_or_init(|| {
        Mutex::new(CommandRegistry::new())
    });
    
    if let Ok(reg) = registry.lock() {
        reg.get_commands().into_iter()
            .map(|cmd| (cmd.name().to_string(), cmd.description().to_string()))
            .collect()
    } else {
        vec![]
    }
}