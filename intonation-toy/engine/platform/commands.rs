// Platform Console Commands
// Commands for platform information and API status

use egui_dev_console::{ConsoleCommandRegistry, ConsoleCommand, ConsoleCommandResult, ConsoleOutput};
use crate::{engine::platform::Platform, common::dev_log, shared_types::Theme};

/// Register all platform commands into the console registry
pub fn register_platform_commands(registry: &mut ConsoleCommandRegistry) {
    registry.register(Box::new(ApiStatusCommand));
    registry.register(Box::new(ThemeCommand));
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

// Theme Command
struct ThemeCommand;

impl ConsoleCommand for ThemeCommand {
    fn name(&self) -> &str {
        "theme"
    }
    
    fn description(&self) -> &str {
        "Switch UI color theme (light|dark|autumn|sunset)"
    }
    
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show current theme and available options
            let current = crate::theme::get_current_theme().name();
            
            let mut outputs = Vec::new();
            outputs.push(ConsoleOutput::info(&format!("Current theme: {}", current)));
            outputs.push(ConsoleOutput::info("Available themes: light, dark, autumn, sunset"));
            outputs.push(ConsoleOutput::info("Usage: theme <theme_name>"));
            
            return ConsoleCommandResult::MultipleOutputs(outputs);
        }
        
        let theme_name = args[0].to_lowercase();
        let new_theme = match theme_name.as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "autumn" => Theme::Autumn,
            "sunset" => Theme::Sunset,
            _ => {
                return ConsoleCommandResult::MultipleOutputs(vec![
                    ConsoleOutput::error(&format!("Unknown theme '{}'. Available themes: light, dark, autumn, sunset", theme_name))
                ]);
            }
        };
        
        // Set the new theme
        crate::theme::set_current_theme(new_theme);
        
        // Reapply styles
        crate::web::styling::reapply_current_theme();
        
        ConsoleCommandResult::MultipleOutputs(vec![
            ConsoleOutput::success(&format!("Theme set to {}", theme_name))
        ])
    }
}