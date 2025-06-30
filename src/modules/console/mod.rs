// Dev Console Module
// Provides interactive debugging and development tools for pitch-toy application

mod commands;
mod history;
mod output;
mod component;
mod command_registry;

pub use component::DevConsole;
pub use commands::{DevCommand, CommandResult};
pub use command_registry::register_command;

#[cfg(test)]
mod tests {
    use super::*;
    
    struct ExternallyRegisteredCommand;
    
    impl DevCommand for ExternallyRegisteredCommand {
        fn name(&self) -> &str {
            "external"
        }
        
        fn description(&self) -> &str {
            "Test command registered from outside the module"
        }
        
        fn execute(&self, _args: Vec<&str>, _registry: &commands::CommandRegistry) -> CommandResult {
            CommandResult::Output(output::ConsoleOutput::success("External command executed!"))
        }
    }
    
    #[test]
    fn test_external_command_registration() {
        // Test the API we want for external command registration
        let cmd = ExternallyRegisteredCommand;
        assert_eq!(cmd.name(), "external");
        assert_eq!(cmd.description(), "Test command registered from outside the module");
        
        // Test that we can register the command
        register_command(Box::new(ExternallyRegisteredCommand));
        
        // Test that the command can be executed through the global registry
        let result = command_registry::execute_command("external");
        match result {
            commands::CommandResult::Output(output::ConsoleOutput::Success(msg)) => {
                assert_eq!(msg, "External command executed!");
            }
            _ => panic!("Expected success output from external command"),
        }
    }
}