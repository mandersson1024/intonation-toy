// Console Command System
// Provides extensible command framework for development console

use std::collections::HashMap;
use super::output::ConsoleOutput;

// Result of command execution
#[derive(Debug)]
pub enum ConsoleCommandResult {
    Output(ConsoleOutput),
    ClearAndOutput(ConsoleOutput),
    MultipleOutputs(Vec<ConsoleOutput>),
}

// Trait for extensible console commands
pub trait ConsoleCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<&str>, registry: &ConsoleCommandRegistry) -> ConsoleCommandResult;
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_basic_functionality() {
        let registry = ConsoleCommandRegistry::new();
        
        // Test help command
        let result = registry.execute("help");
        match result {
            ConsoleCommandResult::Output(ConsoleOutput::Info(text)) => {
                assert!(text.contains("Available commands"));
                assert!(text.contains("help - Display available commands and usage"));
                assert!(text.contains("clear - Clear console output"));
                assert!(text.contains("test - Show examples of all console output types"));
                // Module commands should NOT be present in built-ins only registry
                assert!(!text.contains("api-status - Show application and API status"));
                assert!(!text.contains("mic-status"));
                // Compound commands should NOT appear in help output
                assert!(!text.contains("audio-context"));
            },
            _ => panic!("Expected Info output from help command"),
        }
        
        // Test clear command
        let result = registry.execute("clear");
        match result {
            ConsoleCommandResult::ClearAndOutput(ConsoleOutput::Info(text)) => assert_eq!(text, "Console cleared"),
            _ => panic!("Expected ClearAndOutput result from clear command"),
        }
        
        
        // Test unknown command
        let result = registry.execute("unknown");
        match result {
            ConsoleCommandResult::Output(ConsoleOutput::Error(text)) => assert!(text.contains("Unknown command")),
            _ => panic!("Expected Error output for unknown command"),
        }
        
        // Test test command
        let result = registry.execute("test");
        match result {
            ConsoleCommandResult::MultipleOutputs(outputs) => {
                assert!(!outputs.is_empty());
                // Should contain examples of different output types
                assert!(outputs.iter().any(|o| matches!(o, ConsoleOutput::Info(_))));
                assert!(outputs.iter().any(|o| matches!(o, ConsoleOutput::Success(_))));
                assert!(outputs.iter().any(|o| matches!(o, ConsoleOutput::Warning(_))));
                assert!(outputs.iter().any(|o| matches!(o, ConsoleOutput::Error(_))));
            },
            _ => panic!("Expected MultipleOutputs result from test command"),
        }
    }
    
    #[test]
    fn test_command_parsing() {
        let registry = ConsoleCommandRegistry::new();
        
        // Test empty command
        let result = registry.execute("");
        match result {
            ConsoleCommandResult::Output(ConsoleOutput::Error(text)) => assert_eq!(text, "Empty command"),
            _ => panic!("Expected Error output for empty command"),
        }
        
        // Test command with whitespace
        let result = registry.execute("  help  ");
        match result {
            ConsoleCommandResult::Output(ConsoleOutput::Info(_)) => (), // Success
            _ => panic!("Expected Info output from help command with whitespace"),
        }
    }
    
    #[test]
    fn test_console_output_types() {
        let info = ConsoleOutput::info("test");
        let error = ConsoleOutput::error("test");
        let command = ConsoleOutput::echo("test");
        
        assert_ne!(info, error);
        assert_ne!(error, command);
        assert_ne!(command, info);
    }

    #[test]
    fn test_compound_command_parsing() {
        // Create a test command that handles subcommands
        struct MyBaseCommand;
        impl ConsoleCommand for MyBaseCommand {
            fn name(&self) -> &str { "mybase" }
            fn description(&self) -> &str { "Test base command" }
            fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
                if args.is_empty() {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: mybase <subcommand>"));
                }
                ConsoleCommandResult::Output(ConsoleOutput::info(&format!("Executed with args: {:?}", args)))
            }
        }

        let mut registry = ConsoleCommandRegistry::new();
        registry.register(Box::new(MyBaseCommand));

        // Test compound command parsing (mybase-sub -> mybase sub)
        let result = registry.execute("mybase-sub");
        match result {
            ConsoleCommandResult::Output(ConsoleOutput::Info(text)) => {
                assert!(text.contains("Executed with args: [\"sub\"]"));
            },
            ConsoleCommandResult::Output(ConsoleOutput::Error(text)) => {
                panic!("Got error instead of expected output: {}", text);
            },
            _ => panic!("Expected Info output from compound command, got: {:?}", result),
        }

        // Test compound command with multiple parts and args (mybase-sub-part arg1 -> mybase sub-part arg1)
        let result = registry.execute("mybase-sub-part arg1");
        match result {
            ConsoleCommandResult::Output(ConsoleOutput::Info(text)) => {
                assert!(text.contains("Executed with args: [\"sub-part\", \"arg1\"]"));
            },
            _ => panic!("Expected Info output from compound command with args"),
        }
    }

    #[test]
    fn test_command_variants_discovery() {
        // Create test commands with a common prefix
        struct MyCommand1;
        impl ConsoleCommand for MyCommand1 {
            fn name(&self) -> &str { "myprefix-cmd1" }
            fn description(&self) -> &str { "Test command 1" }
            fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
                ConsoleCommandResult::Output(ConsoleOutput::info("cmd1"))
            }
        }

        struct MyCommand2;
        impl ConsoleCommand for MyCommand2 {
            fn name(&self) -> &str { "myprefix-cmd2" }
            fn description(&self) -> &str { "Test command 2" }
            fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
                ConsoleCommandResult::Output(ConsoleOutput::info("cmd2"))
            }
        }

        struct OtherCommand;
        impl ConsoleCommand for OtherCommand {
            fn name(&self) -> &str { "other-cmd" }
            fn description(&self) -> &str { "Other command" }
            fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
                ConsoleCommandResult::Output(ConsoleOutput::info("other"))
            }
        }

        let mut registry = ConsoleCommandRegistry::new();
        registry.register(Box::new(MyCommand1));
        registry.register(Box::new(MyCommand2));
        registry.register(Box::new(OtherCommand));

        // Test that entering just the base command without arguments shows variants
        let result = registry.execute("myprefix");
        match result {
            ConsoleCommandResult::MultipleOutputs(outputs) => {
                let output_text = outputs.iter()
                    .map(|o| match o {
                        ConsoleOutput::Info(text) => text.clone(),
                        ConsoleOutput::Empty => String::new(),
                        _ => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(output_text.contains("Available myprefix commands:"));
                assert!(output_text.contains("myprefix-cmd1 - Test command 1"));
                assert!(output_text.contains("myprefix-cmd2 - Test command 2"));
                assert!(!output_text.contains("other-cmd")); // Should not include non-matching commands
            },
            _ => panic!("Expected MultipleOutputs when showing command variants"),
        }
    }

    #[test]
    fn test_help_filters_compound_commands() {
        // Create a test registry with both base and compound commands
        struct BaseTestCommand;
        impl ConsoleCommand for BaseTestCommand {
            fn name(&self) -> &str { "base" }
            fn description(&self) -> &str { "Base command" }
            fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
                ConsoleCommandResult::Output(ConsoleOutput::info("base"))
            }
        }

        struct CompoundTestCommand;
        impl ConsoleCommand for CompoundTestCommand {
            fn name(&self) -> &str { "base-sub" }
            fn description(&self) -> &str { "Compound command" }
            fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
                ConsoleCommandResult::Output(ConsoleOutput::info("compound"))
            }
        }

        let mut registry = ConsoleCommandRegistry::new();
        registry.register(Box::new(BaseTestCommand));
        registry.register(Box::new(CompoundTestCommand));

        // Test that help only shows base commands, not compound ones
        let result = registry.execute("help");
        match result {
            ConsoleCommandResult::Output(ConsoleOutput::Info(text)) => {
                assert!(text.contains("base - Base command"));
                assert!(!text.contains("base-sub - Compound command")); // Should NOT show compound commands
            },
            _ => panic!("Expected Info output from help command"),
        }
    }   
}