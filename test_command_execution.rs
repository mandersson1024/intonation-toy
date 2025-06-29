// Temporary test file to examine command execution behavior
use pitch_toy::modules::console::commands::{CommandRegistry, CommandResult};
use pitch_toy::modules::console::output::ConsoleOutput;

fn main() {
    let registry = CommandRegistry::new();
    
    println!("=== Testing direct help command ===");
    let help_result = registry.execute("help");
    match help_result {
        CommandResult::Output(output) => {
            println!("Help command output: {}", output);
        },
        _ => println!("Unexpected result type from help command"),
    }
    
    println!("\n=== Testing test command ===");
    let test_result = registry.execute("test");
    match test_result {
        CommandResult::MultipleOutputs(outputs) => {
            println!("Test command returned {} outputs:", outputs.len());
            for (i, output) in outputs.iter().enumerate() {
                println!("  Output {}: {}", i + 1, output);
            }
        },
        _ => println!("Unexpected result type from test command"),
    }
    
    println!("\n=== Looking for command output in test results ===");
    if let CommandResult::MultipleOutputs(outputs) = registry.execute("test") {
        for output in outputs {
            if let ConsoleOutput::Command(cmd) = &output {
                println!("Found command output in test: {}", cmd);
                
                // Execute the command found in the test output
                println!("Executing found command: {}", cmd);
                let nested_result = registry.execute(cmd);
                match nested_result {
                    CommandResult::Output(nested_output) => {
                        println!("Nested command result: {}", nested_output);
                    },
                    _ => println!("Nested command returned unexpected result type"),
                }
            }
        }
    }
}