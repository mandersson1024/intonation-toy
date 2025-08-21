use crate::output::ConsoleOutput;

#[derive(Debug)]
pub enum ConsoleCommandResult {
    Output(ConsoleOutput),
    ClearAndOutput(ConsoleOutput),
    MultipleOutputs(Vec<ConsoleOutput>),
}

pub trait ConsoleCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<&str>, registry: &crate::command_registry::ConsoleCommandRegistry) -> ConsoleCommandResult;
}