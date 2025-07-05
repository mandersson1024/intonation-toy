// Console Commands Orchestrator
// Central location for registering all non-built-in console commands
// Provides a clean separation between console infrastructure and module-specific commands
//
// Commands should not be unit tested

use dev_console::{ConsoleCommandRegistry, ConsoleCommand, ConsoleCommandResult, ConsoleOutput};

use crate::{platform::Platform, common::dev_log};

/// Creates a fully configured console command registry with all module commands
pub fn create_console_registry() -> ConsoleCommandRegistry {
    let mut registry = ConsoleCommandRegistry::new();
    
    // Register platform module commands
    registry.register(Box::new(ApiStatusCommand));
    
    registry
}

/// Creates a fully configured console command registry with all module commands
/// including audio commands
pub fn create_console_registry_with_audio() -> ConsoleCommandRegistry {
    let mut registry = create_console_registry();
    
    // Register audio module commands
    crate::audio::register_audio_commands(&mut registry);
    
    registry
}

// Platform Commands Implementation
// These commands require access to platform module and are therefore not built-in

// API Status Command
struct ApiStatusCommand;

impl ConsoleCommand for ApiStatusCommand {
    fn name(&self) -> &str {
        "api-status"
    }
    
    fn description(&self) -> &str {
        "Show application and API status"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        // Application status
        let build_type = if cfg!(debug_assertions) { "Development" } else { "Production" };
        outputs.push(ConsoleOutput::info(&format!("Application Status: {} Build Running", build_type)));
        
        // Platform information
        let platform_info = Platform::get_platform_info();
        outputs.push(ConsoleOutput::info(&format!("Platform: {}", platform_info)));
        
        // Critical API status
        outputs.push(ConsoleOutput::info("Critical API Status:"));
        
        let api_statuses = Platform::get_api_status();
        for status in api_statuses {
            let status_symbol = if status.supported { "✓" } else { "✗" };
            let details = status.details.as_deref().unwrap_or("");
            
            let formatted_string = format!(
                "  {} {:<20}: {}",
                status_symbol,
                format!("{}", status.api),
                details
            );
            
            // Log to web console for debugging
            dev_log!("{}", &formatted_string);
            
            let output = if status.supported {
                ConsoleOutput::success(&formatted_string)
            } else {
                ConsoleOutput::error(&formatted_string)
            };
            outputs.push(output);
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}



