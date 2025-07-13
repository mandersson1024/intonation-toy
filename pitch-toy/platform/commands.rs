// Platform Console Commands
// Commands for platform information and API status

use egui_dev_console::{ConsoleCommandRegistry, ConsoleCommand, ConsoleCommandResult, ConsoleOutput};
use crate::{platform::Platform, common::dev_log};

/// Register all platform commands into the console registry
pub fn register_platform_commands(registry: &mut ConsoleCommandRegistry) {
    registry.register(Box::new(ApiStatusCommand));
}

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