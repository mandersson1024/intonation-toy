// Dev Console Module
// Provides interactive debugging and development tools for pitch-toy application

mod commands;
mod history;
mod output;
mod component;

pub use component::DevConsole;
pub use commands::{ConsoleCommand, ConsoleCommandResult, ConsoleCommandRegistry};
pub use output::ConsoleOutput;

#[cfg(test)]
mod tests {
    use super::*;
    
    struct ExternallyRegisteredCommand;
    
    impl ConsoleCommand for ExternallyRegisteredCommand {
        fn name(&self) -> &str {
            "external"
        }
        
        fn description(&self) -> &str {
            "Test command registered from outside the module"
        }
        
        fn execute(&self, _args: Vec<&str>, _registry: &commands::ConsoleCommandRegistry) -> ConsoleCommandResult {
            ConsoleCommandResult::Output(output::ConsoleOutput::success("External command executed!"))
        }
    }
    
    #[test]
    fn test_external_command_registration() {
        // Test that external commands can be registered with a registry
        let mut registry = commands::ConsoleCommandRegistry::new();
        
        let cmd = ExternallyRegisteredCommand;
        assert_eq!(cmd.name(), "external");
        assert_eq!(cmd.description(), "Test command registered from outside the module");
        
        // Test that we can register the command
        registry.register(Box::new(ExternallyRegisteredCommand));
        
        // Test that the command can be executed through the registry
        let result = registry.execute("external");
        match result {
            commands::ConsoleCommandResult::Output(output::ConsoleOutput::Success(msg)) => {
                assert_eq!(msg, "External command executed!");
            }
            _ => panic!("Expected success output from external command"),
        }
    }
}