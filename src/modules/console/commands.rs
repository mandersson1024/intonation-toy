// Console Command System
// Provides extensible command framework for development console

use std::collections::HashMap;
use super::output::ConsoleOutput;

// Result of command execution
pub enum CommandResult {
    Output(ConsoleOutput),
    ClearAndOutput(ConsoleOutput),
}

// Trait for extensible console commands
pub trait DevCommand {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<&str>, registry: &CommandRegistry) -> CommandResult;
}

// Command registry for managing available commands
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn DevCommand>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        
        // Register built-in commands
        registry.register(Box::new(HelpCommand));
        registry.register(Box::new(ClearCommand));
        registry.register(Box::new(StatusCommand));
        
        registry
    }
    
    pub fn register(&mut self, command: Box<dyn DevCommand>) {
        self.commands.insert(command.name().to_string(), command);
    }
    
    pub fn execute(&self, input: &str) -> CommandResult {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return CommandResult::Output(ConsoleOutput::error("Empty command"));
        }
        
        let command_name = parts[0];
        let args = parts[1..].to_vec();
        
        match self.commands.get(command_name) {
            Some(command) => command.execute(args, self),
            None => CommandResult::Output(ConsoleOutput::error(format!("Unknown command: {}", command_name))),
        }
    }
    
    pub fn get_commands(&self) -> Vec<&dyn DevCommand> {
        self.commands.values().map(|cmd| cmd.as_ref()).collect()
    }
}

// Built-in Help Command
struct HelpCommand;

impl DevCommand for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    
    fn description(&self) -> &str {
        "Display available commands and usage"
    }
    
    fn execute(&self, _args: Vec<&str>, registry: &CommandRegistry) -> CommandResult {
        let mut help_lines = vec!["Available commands:".to_string()];
        
        let mut commands = registry.get_commands();
        commands.sort_by(|a, b| a.name().cmp(b.name()));
        
        for command in commands {
            help_lines.push(format!("  {} - {}", command.name(), command.description()));
        }
        
        let help_text = help_lines.join("\n");
        CommandResult::Output(ConsoleOutput::info(help_text))
    }
}

// Built-in Clear Command
struct ClearCommand;

impl DevCommand for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }
    
    fn description(&self) -> &str {
        "Clear console output"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &CommandRegistry) -> CommandResult {
        CommandResult::ClearAndOutput(ConsoleOutput::info("Console cleared"))
    }
}

// Built-in Status Command
struct StatusCommand;

impl DevCommand for StatusCommand {
    fn name(&self) -> &str {
        "status"
    }
    
    fn description(&self) -> &str {
        "Show application status"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &CommandRegistry) -> CommandResult {
        CommandResult::Output(ConsoleOutput::info("Application Status: Development Build Running"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_basic_functionality() {
        let registry = CommandRegistry::new();
        
        // Test help command
        let result = registry.execute("help");
        match result {
            CommandResult::Output(ConsoleOutput::Info(text)) => {
                assert!(text.contains("Available commands"));
                assert!(text.contains("help - Display available commands and usage"));
                assert!(text.contains("clear - Clear console output"));
                assert!(text.contains("status - Show application status"));
            },
            _ => panic!("Expected Info output from help command"),
        }
        
        // Test clear command
        let result = registry.execute("clear");
        match result {
            CommandResult::ClearAndOutput(ConsoleOutput::Info(text)) => assert_eq!(text, "Console cleared"),
            _ => panic!("Expected ClearAndOutput result from clear command"),
        }
        
        // Test status command
        let result = registry.execute("status");
        match result {
            CommandResult::Output(ConsoleOutput::Info(text)) => assert!(text.contains("Development Build")),
            _ => panic!("Expected Info output from status command"),
        }
        
        // Test unknown command
        let result = registry.execute("unknown");
        match result {
            CommandResult::Output(ConsoleOutput::Error(text)) => assert!(text.contains("Unknown command")),
            _ => panic!("Expected Error output for unknown command"),
        }
    }
    
    #[test]
    fn test_command_parsing() {
        let registry = CommandRegistry::new();
        
        // Test empty command
        let result = registry.execute("");
        match result {
            CommandResult::Output(ConsoleOutput::Error(text)) => assert_eq!(text, "Empty command"),
            _ => panic!("Expected Error output for empty command"),
        }
        
        // Test command with whitespace
        let result = registry.execute("  help  ");
        match result {
            CommandResult::Output(ConsoleOutput::Info(_)) => (), // Success
            _ => panic!("Expected Info output from help command with whitespace"),
        }
    }
    
    #[test]
    fn test_console_output_types() {
        let info = ConsoleOutput::info("test");
        let success = ConsoleOutput::success("test");
        let error = ConsoleOutput::error("test");
        let debug = ConsoleOutput::debug("test");
        
        assert_ne!(info, success);
        assert_ne!(success, error);
        assert_ne!(error, debug);
        assert_ne!(debug, info);
    }
    
}