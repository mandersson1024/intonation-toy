// Development Console Component
// Interactive debugging and development tools
// Generic console component without audio dependencies

use yew::prelude::*;
use web_sys::{HtmlInputElement, KeyboardEvent, Storage};
use std::rc::Rc;

use super::command_registry::{ConsoleCommandResult, ConsoleCommandRegistry};
use super::history::ConsoleHistory;
use super::output::{ConsoleOutput, ConsoleOutputManager, CONSOLE_OUTPUT_CSS};

/// Local storage key for console history persistence
const CONSOLE_HISTORY_STORAGE_KEY: &str = "dev_console_history";

/// Properties for the DevConsole component
#[derive(Properties)]
pub struct DevConsoleProps {
    /// Command registry to use for executing commands
    pub registry: Rc<ConsoleCommandRegistry>,
    /// Whether the console is visible
    pub visible: bool,
    /// Callback for toggling visibility
    pub on_toggle: Callback<()>,
}

impl PartialEq for DevConsoleProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer equality since registries are immutable after creation
        Rc::ptr_eq(&self.registry, &other.registry) && self.visible == other.visible
    }
}

/// State for the DevConsole component
pub struct DevConsole {
    /// Command registry for executing commands
    registry: Rc<ConsoleCommandRegistry>,
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

/// Messages for the DevConsole component
#[derive(Debug)]
pub enum DevConsoleMsg {
    /// Execute a command
    ExecuteCommand,
    /// Update input value
    UpdateInput(String),
    /// Handle keyboard events (history navigation, shortcuts)
    HandleKeyDown(KeyboardEvent),
    /// Toggle console visibility
    ToggleVisibility,
}

impl Component for DevConsole {
    type Message = DevConsoleMsg;
    type Properties = DevConsoleProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut output_manager = ConsoleOutputManager::new();
        
        // Add welcome message
        output_manager.add_output(ConsoleOutput::info("Dev Console initialized"));
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
            DevConsoleMsg::ExecuteCommand => {
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
            
            DevConsoleMsg::UpdateInput(value) => {
                self.input_value = value;
                true
            }
            
            DevConsoleMsg::HandleKeyDown(event) => {
                match event.key().as_str() {
                    "Enter" => {
                        event.prevent_default();
                        ctx.link().send_message(DevConsoleMsg::ExecuteCommand);
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
            
            DevConsoleMsg::ToggleVisibility => {
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
            <div class="dev-console-modal">
                <style>{ CONSOLE_OUTPUT_CSS }</style>
                <style>{ CONSOLE_COMPONENT_CSS }</style>
                
                <div class="dev-console-header">
                    <div class="dev-console-left">
                        <span class="dev-console-title">{ "Development Console" }</span>
                        <span class="dev-console-hint" title="Press ESC to toggle console">
                            { "(ESC to toggle)" }
                        </span>
                    </div>
                </div>
                
                <div class="dev-console-output-container" ref={self.output_ref.clone()}>
                    { self.render_output() }
                </div>
                
                <div class="dev-console-input-container">
                    <span class="dev-console-prompt">{ ">" }</span>
                    <input
                        ref={self.input_ref.clone()}
                        type="text"
                        class="dev-console-input"
                        value={self.input_value.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                                DevConsoleMsg::UpdateInput(input.value())
                            } else {
                                DevConsoleMsg::UpdateInput(String::new())
                            }
                        })}
                        onkeydown={ctx.link().callback(DevConsoleMsg::HandleKeyDown)}
                        placeholder="Enter command (type 'help' for commands)"
                        autofocus={true}
                    />
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

impl DevConsole {
    /// Render the console output
    fn render_output(&self) -> Html {
        html! {
            <div class="dev-console-messages">
                {for self.output_manager.entries().iter().map(|entry| {
                    html! {
                        <div class={format!("dev-console-message dev-console-message-{}", entry.output.output_type())}>
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
            input_element.focus().ok();
            if let Ok(length) = input_element.value().len().try_into() {
                input_element.set_selection_start(Some(length)).ok();
                input_element.set_selection_end(Some(length)).ok();
            }
        }
    }

    /// Load command history from local storage
    fn load_history_from_storage() -> ConsoleHistory {
        if let Some(storage) = Self::get_local_storage() {
            if let Ok(Some(history_json)) = storage.get_item(CONSOLE_HISTORY_STORAGE_KEY) {
                if let Ok(history) = serde_json::from_str(&history_json) {
                    return history;
                }
            }
        }
        ConsoleHistory::new()
    }

    /// Save command history to local storage
    fn save_history_to_storage(&self) {
        if let Some(storage) = Self::get_local_storage() {
            if let Ok(history_json) = serde_json::to_string(&self.command_history) {
                let _ = storage.set_item(CONSOLE_HISTORY_STORAGE_KEY, &history_json);
            }
        }
    }

    /// Get local storage reference
    fn get_local_storage() -> Option<Storage> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok())
            .flatten()
    }
}

// CSS for the console component
const CONSOLE_COMPONENT_CSS: &str = r#"
.dev-console-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    z-index: 1000;
    display: flex;
    flex-direction: column;
    font-family: 'Courier New', monospace;
    font-size: 14px;
}

.dev-console-header {
    background: #2c3e50;
    color: white;
    padding: 10px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #34495e;
}

.dev-console-left {
    display: flex;
    align-items: center;
    gap: 10px;
}

.dev-console-title {
    font-weight: bold;
    font-size: 16px;
}

.dev-console-hint {
    font-size: 12px;
    opacity: 0.7;
}

.dev-console-output-container {
    flex: 1;
    background: #1a1a1a;
    color: #e0e0e0;
    padding: 10px;
    overflow-y: auto;
    max-height: 70vh;
}

.dev-console-input-container {
    background: #2c3e50;
    padding: 10px;
    display: flex;
    align-items: center;
    gap: 5px;
    border-top: 1px solid #34495e;
}

.dev-console-prompt {
    color: #3498db;
    font-weight: bold;
}

.dev-console-input {
    flex: 1;
    background: #1a1a1a;
    color: #e0e0e0;
    border: none;
    padding: 5px;
    font-family: 'Courier New', monospace;
    font-size: 14px;
    outline: none;
}

.dev-console-input:focus {
    background: #2a2a2a;
}

.dev-console-messages {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.dev-console-message {
    padding: 2px 0;
    word-wrap: break-word;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_console_message_handling() {
        // Test that console messages are handled correctly
        let registry = ConsoleCommandRegistry::new();
        let props = DevConsoleProps {
            registry: Rc::new(registry),
            visible: true,
            on_toggle: Callback::from(|_| {}),
        };
        
        assert!(props.visible);
    }
    
    #[test]
    fn test_console_history_integration() {
        // Test that console history works correctly
        let history = ConsoleHistory::new();
        assert_eq!(history.len(), 0);
        
        let mut history = ConsoleHistory::new();
        history.add_command("test".to_string());
        assert_eq!(history.len(), 1);
    }
    
    #[test]
    fn test_local_storage_constants() {
        // Test that storage key is defined
        assert!(!CONSOLE_HISTORY_STORAGE_KEY.is_empty());
    }
}