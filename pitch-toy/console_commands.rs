// Console Commands Orchestrator
// Central location for registering all non-built-in console commands
// Provides a clean separation between console infrastructure and module-specific commands
//
// Commands should not be unit tested

use egui_dev_console::ConsoleCommandRegistry;

/// Creates a fully configured console command registry with all module commands
pub fn create_console_registry() -> ConsoleCommandRegistry {
    let mut registry = ConsoleCommandRegistry::new();
    
    // Register platform module commands
    crate::platform::commands::register_platform_commands(&mut registry);
    
    registry
}



