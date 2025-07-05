// Debug Console Component - Reusable command I/O interface
//
// This component provides a focused command interface without audio dependencies.
// It handles command execution, history management, and output display.

use yew::prelude::*;
use web_sys::{HtmlInputElement, KeyboardEvent};
use wasm_bindgen::JsCast;
use std::rc::Rc;

use crate::console::{ConsoleCommandResult, ConsoleCommandRegistry, ConsoleHistory, ConsoleOutput, ConsoleOutputManager};

/// Local storage key for console history persistence
const CONSOLE_HISTORY_STORAGE_KEY: &str = "pitch_toy_debug_console_history";

/// Command registry trait for dependency injection
pub trait CommandRegistry: Send + Sync {
    fn execute(&self, command: &str) -> ConsoleCommandResult;
    fn list_commands(&self) -> Vec<String>;
}

/// Implement CommandRegistry for ConsoleCommandRegistry
impl CommandRegistry for ConsoleCommandRegistry {
    fn execute(&self, command: &str) -> ConsoleCommandResult {
        ConsoleCommandRegistry::execute(self, command)
    }
    
    fn list_commands(&self) -> Vec<String> {
        // TODO: Implement command listing - for now return empty vec
        Vec::new()
    }
}

/// Properties for the DebugConsole component
#[derive(Properties)]
pub struct DebugConsoleProps {
    /// Command registry for executing commands
    pub registry: Rc<dyn CommandRegistry>,
    /// Whether the console is visible
    pub visible: bool,
    /// Callback for toggling visibility
    pub on_toggle: Callback<()>,
}

impl PartialEq for DebugConsoleProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer equality and visibility state
        Rc::ptr_eq(&self.registry, &other.registry) && self.visible == other.visible
    }
}

/// State for the DebugConsole component
pub struct DebugConsole {
    /// Command registry for executing commands
    registry: Rc<dyn CommandRegistry>,
    /// Command history for navigation
    command_history: ConsoleHistory,
    /// Output manager for displaying results
    output_manager: ConsoleOutputManager,
    /// Current input value
    input_value: String,
    /// Reference to the input element
    input_ref: NodeRef,
    /// Reference to the output container element for auto-scrolling
    output_ref: NodeRef,
}

/// Messages for the DebugConsole component
#[derive(Debug)]
pub enum DebugConsoleMsg {
    /// Execute a command
    ExecuteCommand,
    /// Update input value
    UpdateInput(String),
    /// Handle keyboard events (history navigation, shortcuts)
    HandleKeyDown(KeyboardEvent),
    /// Toggle console visibility
    ToggleVisibility,
}

impl Component for DebugConsole {
    type Message = DebugConsoleMsg;
    type Properties = DebugConsoleProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut output_manager = ConsoleOutputManager::new();
        
        // Add welcome message
        output_manager.add_output(ConsoleOutput::info("Debug Console initialized"));
        output_manager.add_output(ConsoleOutput::info("Type 'help' for available commands"));
        
        // Load command history from local storage
        let command_history = Self::load_history_from_storage();
        if !command_history.is_empty() {
            output_manager.add_output(ConsoleOutput::info(&format!("Restored {} commands from history", command_history.len())));
        }
        
        Self {
            registry: Rc::clone(&ctx.props().registry),
            command_history,
            output_manager,
            input_value: String::new(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DebugConsoleMsg::ExecuteCommand => {
                let command = self.input_value.trim();
                if !command.is_empty() {
                    // Add command to history
                    self.command_history.add_command(command.to_string());
                    
                    // Save history to local storage
                    self.save_history_to_storage();

                    // Echo the command
                    self.output_manager.add_output(ConsoleOutput::echo(command));
                    
                    // Execute the command using the provided registry
                    let result = self.registry.execute(command);
                    match result {
                        ConsoleCommandResult::Output(output) => {
                            self.output_manager.add_output(output);
                        }
                        ConsoleCommandResult::ClearAndOutput(output) => {
                            self.output_manager.clear();
                            self.output_manager.add_output(output);
                        }
                        ConsoleCommandResult::MultipleOutputs(outputs) => {
                            for output in outputs {
                                self.output_manager.add_output(output);
                            }
                        }
                    }
                    
                    // Clear input
                    self.input_value.clear();
                    
                    // Reset history navigation
                    self.command_history.reset_navigation();
                }
                true
            }
            
            DebugConsoleMsg::UpdateInput(value) => {
                self.input_value = value;
                true
            }
            
            DebugConsoleMsg::HandleKeyDown(event) => {
                match event.key().as_str() {
                    "Enter" => {
                        event.prevent_default();
                        ctx.link().send_message(DebugConsoleMsg::ExecuteCommand);
                        false
                    }
                    "ArrowUp" => {
                        event.prevent_default();
                        if let Some(command) = self.command_history.navigate_previous() {
                            self.input_value = command.to_string();
                            self.focus_input_end();
                        }
                        true
                    }
                    "ArrowDown" => {
                        event.prevent_default();
                        if let Some(command) = self.command_history.navigate_next() {
                            self.input_value = command.to_string();
                            self.focus_input_end();
                        }
                        true
                    }
                    "Escape" => {
                        event.prevent_default();
                        ctx.props().on_toggle.emit(());
                        false
                    }
                    _ => false
                }
            }
            
            DebugConsoleMsg::ToggleVisibility => {
                ctx.props().on_toggle.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !ctx.props().visible {
            return html! {};
        }

        html! {
            <div class="debug-console-modal">
                <div class="debug-console-header">
                    <h3 class="debug-console-title">{"Debug Console"}</h3>
                </div>
                
                <div class="debug-console-content">
                    <div class="debug-console-output" ref={self.output_ref.clone()}>
                        {self.render_output()}
                    </div>
                    
                    <div class="debug-console-input-container">
                        <span class="debug-console-prompt">{"$ "}</span>
                        <input
                            type="text"
                            class="debug-console-input"
                            ref={self.input_ref.clone()}
                            value={self.input_value.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let target = e.target().unwrap();
                                let input = target.unchecked_into::<HtmlInputElement>();
                                DebugConsoleMsg::UpdateInput(input.value())
                            })}
                            onkeydown={ctx.link().callback(DebugConsoleMsg::HandleKeyDown)}
                            placeholder="Type a command..."
                        />
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        // Auto-scroll to bottom when new output is added
        if let Some(output_element) = self.output_ref.cast::<web_sys::Element>() {
            output_element.set_scroll_top(output_element.scroll_height());
        }
    }
}

impl DebugConsole {
    /// Render the console output
    fn render_output(&self) -> Html {
        html! {
            <div class="debug-console-messages">
                {for self.output_manager.entries().iter().map(|entry| {
                    html! {
                        <div class={format!("debug-console-message debug-console-message-{}", entry.output.output_type())}>
                            {entry.output.to_string()}
                        </div>
                    }
                })}
            </div>
        }
    }

    /// Focus the input element and move cursor to end
    fn focus_input_end(&self) {
        if let Some(input_element) = self.input_ref.cast::<HtmlInputElement>() {
            let _ = input_element.focus();
            let value_length = input_element.value().len() as u32;
            input_element.set_selection_range(value_length, value_length).ok();
        }
    }

    /// Load command history from local storage
    fn load_history_from_storage() -> ConsoleHistory {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        
        match storage.get_item(CONSOLE_HISTORY_STORAGE_KEY) {
            Ok(Some(json_str)) => {
                match serde_json::from_str::<Vec<String>>(&json_str) {
                    Ok(commands) => {
                        let mut history = ConsoleHistory::new();
                        for command in commands {
                            history.add_command(command);
                        }
                        history
                    }
                    Err(_) => ConsoleHistory::new(),
                }
            }
            _ => ConsoleHistory::new(),
        }
    }

    /// Save command history to local storage
    fn save_history_to_storage(&self) {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        
        if let Some(json_str) = self.command_history.to_json() {
            storage.set_item(CONSOLE_HISTORY_STORAGE_KEY, &json_str).ok();
        }
    }
}

/// Extension trait for ConsoleOutput to provide CSS classes
trait ConsoleOutputExt {
    fn css_class(&self) -> &'static str;
}

impl ConsoleOutputExt for ConsoleOutput {
    fn css_class(&self) -> &'static str {
        match self {
            ConsoleOutput::Info(_) => "info",
            ConsoleOutput::Success(_) => "success",
            ConsoleOutput::Warning(_) => "warning",
            ConsoleOutput::Error(_) => "error",
            ConsoleOutput::Echo(_) => "echo",
            ConsoleOutput::Empty => "empty",
        }
    }
}