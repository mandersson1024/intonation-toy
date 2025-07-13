// Console Command System
// Provides extensible command framework for development console

use std::collections::HashMap;
use crate::output::ConsoleOutput;
use crate::command::{ConsoleCommand, ConsoleCommandResult};

// Command registry for managing available commands
pub struct ConsoleCommandRegistry {
    commands: HashMap<String, Box<dyn ConsoleCommand>>,
}

impl ConsoleCommandRegistry {
    /// Create a new registry with only built-in commands (no module dependencies)
    /// Built-in commands: help, clear, test
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        
        // Register built-in commands that require no external module dependencies
        registry.register(Box::new(HelpCommand));
        registry.register(Box::new(ClearCommand));
        registry.register(Box::new(TestCommand));
        
        registry
    }
    
    pub fn register(&mut self, command: Box<dyn ConsoleCommand>) {
        self.commands.insert(command.name().to_string(), command);
    }
    
    pub fn execute(&self, input: &str) -> ConsoleCommandResult {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Empty command"));
        }
        
        let command_name = parts[0];
        let args = parts[1..].to_vec();
        
        // First, try to find the command directly
        if let Some(command) = self.commands.get(command_name) {
            return command.execute(args, self);
        }
        
        // If not found, check if it's a compound command (aaa-bbb format)
        if command_name.contains('-') {
            let compound_parts: Vec<&str> = command_name.split('-').collect();
            if compound_parts.len() >= 2 {
                let base_command = compound_parts[0];
                let sub_command = compound_parts[1..].join("-");
                
                // Try to find the base command
                if let Some(command) = self.commands.get(base_command) {
                    // Convert compound command to base command with arguments
                    let mut new_args = vec![sub_command.as_str()];
                    new_args.extend(args);
                    return command.execute(new_args, self);
                }
            }
        }
        
        // If still not found, check if it's a base command without arguments
        // and show documentation for its variants
        if args.is_empty() {
            let variants = self.get_command_variants(command_name);
            if !variants.is_empty() {
                let mut outputs = vec![
                    ConsoleOutput::info(&format!("Available {} commands:", command_name)),
                    ConsoleOutput::empty(),
                ];
                
                for variant in variants {
                    outputs.push(ConsoleOutput::info(&format!("  {} - {}", variant.name(), variant.description())));
                }
                
                return ConsoleCommandResult::MultipleOutputs(outputs);
            }
        }
        
        ConsoleCommandResult::Output(ConsoleOutput::error(format!("Unknown command: {}", command_name)))
    }
    
    pub fn get_commands(&self) -> Vec<&dyn ConsoleCommand> {
        self.commands.values().map(|cmd| cmd.as_ref()).collect()
    }
    
    /// Get all command variants for a given base command name
    /// Returns commands that start with the base name followed by a hyphen
    pub fn get_command_variants(&self, base_name: &str) -> Vec<&dyn ConsoleCommand> {
        let prefix = format!("{}-", base_name);
        self.commands.values()
            .filter(|cmd| cmd.name().starts_with(&prefix))
            .map(|cmd| cmd.as_ref())
            .collect()
    }
}

// Built-in Help Command
struct HelpCommand;

impl ConsoleCommand for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    
    fn description(&self) -> &str {
        "Display available commands and usage"
    }
    
    fn execute(&self, _args: Vec<&str>, registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut help_lines = vec!["Available commands:".to_string()];
        
        let mut commands = registry.get_commands();
        commands.sort_by(|a, b| a.name().cmp(b.name()));
        
        // Filter out compound commands (those containing hyphens) to show only base commands
        for command in commands {
            if !command.name().contains('-') {
                help_lines.push(format!("  {} - {}", command.name(), command.description()));
            }
        }
        
        let help_text = help_lines.join("\n");
        ConsoleCommandResult::Output(ConsoleOutput::info(help_text))
    }
}

// Built-in Clear Command
struct ClearCommand;

impl ConsoleCommand for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }
    
    fn description(&self) -> &str {
        "Clear console output"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        ConsoleCommandResult::ClearAndOutput(ConsoleOutput::info("Console cleared"))
    }
}


// Built-in Test Command - Shows examples of all ConsoleOutput variants
struct TestCommand;

impl ConsoleCommand for TestCommand {
    fn name(&self) -> &str {
        "test"
    }
    
    fn description(&self) -> &str {
        "Show examples of all console output types"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        // This command demonstrates all available ConsoleOutput variants
        // by returning multiple outputs with proper styling
        
        let outputs = vec![
            ConsoleOutput::info("Console Output Examples:"),
            ConsoleOutput::empty(),
            ConsoleOutput::info("This is an informational message"),
            ConsoleOutput::success("This is a success message"),
            ConsoleOutput::warning("This is a warning message"),
            ConsoleOutput::error("This is an error message"),
            ConsoleOutput::empty(),
        ];
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}