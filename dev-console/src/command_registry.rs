use std::collections::HashMap;
use crate::output::ConsoleOutput;
use crate::command::{ConsoleCommand, ConsoleCommandResult};

pub struct ConsoleCommandRegistry {
    commands: HashMap<String, Box<dyn ConsoleCommand>>,
}

impl ConsoleCommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        
        registry.register(Box::new(HelpCommand));
        registry.register(Box::new(ClearCommand));
        registry.register(Box::new(TestCommand));
        
        registry
    }
    
    pub fn register(&mut self, command: Box<dyn ConsoleCommand>) {
        self.commands.insert(command.name().to_string(), command);
    }
    
    pub fn execute(&self, input: &str) -> ConsoleCommandResult {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let Some((&command_name, args)) = parts.split_first() else {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Empty command"));
        };
        
        match self.commands.get(command_name) {
            Some(command) => command.execute(args.to_vec(), self),
            None => ConsoleCommandResult::Output(ConsoleOutput::error(format!("Unknown command: {}", command_name))),
        }
    }
    
    pub fn get_commands(&self) -> Vec<&dyn ConsoleCommand> {
        self.commands.values().map(|cmd| cmd.as_ref()).collect()
    }
}

struct HelpCommand;

impl ConsoleCommand for HelpCommand {
    fn name(&self) -> &str { "help" }
    fn description(&self) -> &str { "Display available commands and usage" }
    
    fn execute(&self, _args: Vec<&str>, registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut commands = registry.get_commands();
        commands.sort_by(|a, b| a.name().cmp(b.name()));
        
        let mut help_lines = vec!["Available commands:".to_string()];
        help_lines.extend(commands.iter().map(|cmd| format!("  {} - {}", cmd.name(), cmd.description())));
        
        ConsoleCommandResult::Output(ConsoleOutput::info(help_lines.join("\n")))
    }
}

struct ClearCommand;

impl ConsoleCommand for ClearCommand {
    fn name(&self) -> &str { "clear" }
    fn description(&self) -> &str { "Clear console output" }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        ConsoleCommandResult::ClearAndOutput(ConsoleOutput::info("Console cleared"))
    }
}

struct TestCommand;

impl ConsoleCommand for TestCommand {
    fn name(&self) -> &str { "test" }
    fn description(&self) -> &str { "Show examples of all console output types" }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        ConsoleCommandResult::MultipleOutputs(vec![
            ConsoleOutput::info("Console Output Examples:"),
            ConsoleOutput::empty(),
            ConsoleOutput::info("This is an informational message"),
            ConsoleOutput::success("This is a success message"),
            ConsoleOutput::warning("This is a warning message"),
            ConsoleOutput::error("This is an error message"),
            ConsoleOutput::empty(),
        ])
    }
}

