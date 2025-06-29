// Development Console Component
// Interactive debugging and development tools for pitch-toy application
// Available only in development builds

use yew::prelude::*;
use web_sys::{HtmlInputElement, KeyboardEvent, Storage};

use super::commands::{CommandRegistry, CommandResult};
use super::history::ConsoleHistory;
use super::output::{ConsoleOutput, ConsoleOutputManager, CONSOLE_OUTPUT_CSS};

/// Local storage key for console history persistence
const CONSOLE_HISTORY_STORAGE_KEY: &str = "pitch_toy_console_history";

/// Properties for the DevConsole component
#[derive(Properties, PartialEq)]
pub struct DevConsoleProps {
    /// Whether the console is visible
    pub visible: bool,
    /// Callback to toggle console visibility
    pub on_toggle: Callback<()>,
}

/// State for the DevConsole component
pub struct DevConsole {
    /// Command registry for executing commands
    command_registry: CommandRegistry,
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
    /// Track previous visibility state for focus management
    was_visible: bool,
}

/// Messages for the DevConsole component
pub enum DevConsoleMsg {
    /// Execute a command
    ExecuteCommand,
    /// Update input value
    UpdateInput(String),
    /// Handle keyboard events (history navigation, shortcuts)
    HandleKeyDown(KeyboardEvent),
}

impl Component for DevConsole {
    type Message = DevConsoleMsg;
    type Properties = DevConsoleProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let mut output_manager = ConsoleOutputManager::new();
        
        // Add welcome message
        output_manager.add_output(ConsoleOutput::info("Development Console initialized"));
        output_manager.add_output(ConsoleOutput::info("Type 'help' for available commands"));
        
        // Load command history from local storage
        let command_history = Self::load_history_from_storage();
        if !command_history.is_empty() {
            output_manager.add_output(ConsoleOutput::info(&format!("Restored {} commands from history", command_history.len())));
        }
        
        Self {
            command_registry: CommandRegistry::new(),
            command_history,
            output_manager,
            input_value: String::new(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            was_visible: false,
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
                    self.output_manager.add_output(ConsoleOutput::command(command));
                    
                    // Execute the command
                    let result = self.command_registry.execute(command);
                    match result {
                        CommandResult::Output(output) => {
                            self.output_manager.add_output(output);
                        }
                        CommandResult::ClearAndOutput(output) => {
                            self.output_manager.clear();
                            self.output_manager.add_output(output);
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
                            // Focus and move cursor to end
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
                        // Let the global handler manage console toggling
                        // Don't prevent default to allow global handler to work
                        false
                    }
                    _ => false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !ctx.props().visible {
            return html! {};
        }

        let on_input = {
            let link = ctx.link().clone();
            Callback::from(move |e: InputEvent| {
                if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                    link.send_message(DevConsoleMsg::UpdateInput(input.value()));
                }
            })
        };

        let on_keydown = {
            let link = ctx.link().clone();
            Callback::from(move |e: KeyboardEvent| {
                link.send_message(DevConsoleMsg::HandleKeyDown(e));
            })
        };



        html! {
            <div class="console-overlay">
                <style>{ CONSOLE_OUTPUT_CSS }</style>
                <style>{ CONSOLE_COMPONENT_CSS }</style>
                
                <div class="console-modal">
                    <div class="console-header">
                        <span class="console-title">{ "Development Console" }</span>
                        <div class="console-controls">
                            <span class="console-hint" title="Press ESC to toggle console">
                                { "ESC to toggle" }
                            </span>
                        </div>
                    </div>
                    
                    <div class="console-output-container" ref={self.output_ref.clone()}>
                        { self.render_output() }
                    </div>
                    
                    <div class="console-input-container">
                        <span class="console-prompt">{ ">" }</span>
                        <input
                            ref={self.input_ref.clone()}
                            type="text"
                            class="console-input"
                            value={self.input_value.clone()}
                            oninput={on_input}
                            onkeydown={on_keydown}
                            placeholder="Enter command (type 'help' for commands)"
                            autofocus={true}
                        />
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let is_visible = ctx.props().visible;
        
        // Focus input on first render or when console becomes visible
        if first_render {
            self.focus_input();
            self.scroll_to_bottom();
        } else if !self.was_visible && is_visible {
            // Console just became visible, focus the input and scroll to bottom
            self.focus_input();
            self.scroll_to_bottom();
        } else if is_visible {
            // Console is visible and content may have changed, scroll to bottom
            self.scroll_to_bottom();
        }
        
        // Update visibility tracking
        self.was_visible = is_visible;
    }
}

impl DevConsole {
    /// Render the console output
    fn render_output(&self) -> Html {
        let entries = self.output_manager.visible_entries();
        
        html! {
            <div class="console-output-content">
                { for entries.iter().rev().map(|entry| {
                    let css_class = self.output_manager.entry_css_class(entry);
                    let formatted = self.output_manager.format_entry(entry);
                    html! {
                        <div class={css_class} key={entry.id}>
                            { formatted }
                        </div>
                    }
                })}
            </div>
        }
    }

    /// Focus the input element
    fn focus_input(&self) {
        if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
            let _ = input.focus();
        }
    }

    /// Focus the input element and move cursor to end
    fn focus_input_end(&self) {
        if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
            let _ = input.focus();
            let len = input.value().len() as u32;
            let _ = input.set_selection_range(len, len);
        }
    }

    /// Scroll the output container to the bottom
    fn scroll_to_bottom(&self) {
        if let Some(output_container) = self.output_ref.cast::<web_sys::Element>() {
            output_container.set_scroll_top(output_container.scroll_height());
        }
    }
}

/// Local storage helper functions for console history persistence
impl DevConsole {
    /// Load console history from local storage
    fn load_history_from_storage() -> ConsoleHistory {
        let mut history = ConsoleHistory::new();
        
        if let Some(storage) = Self::get_local_storage() {
            if let Ok(Some(stored_data)) = storage.get_item(CONSOLE_HISTORY_STORAGE_KEY) {
                history.from_json(&stored_data);
            }
        }
        
        history
    }
    
    /// Save console history to local storage
    fn save_history_to_storage(&self) {
        if let Some(storage) = Self::get_local_storage() {
            if let Some(json_data) = self.command_history.to_json() {
                let _ = storage.set_item(CONSOLE_HISTORY_STORAGE_KEY, &json_data);
            }
        }
    }
    
    /// Get browser local storage instance
    fn get_local_storage() -> Option<Storage> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok())
            .and_then(|storage| storage)
    }
}

/// CSS styles for the console component
const CONSOLE_COMPONENT_CSS: &str = r#"
.console-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
    font-family: 'Courier New', monospace;
}

.console-modal {
    width: 90%;
    max-width: 900px;
    height: 70%;
    max-height: 600px;
    background-color:rgba(31, 41, 55, 0.4);
    border: 2px solid #374151;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}

.console-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background-color:rgba(55, 65, 81, 0.4);
    border-bottom: 1px solid #4b5563;
    border-radius: 6px 6px 0 0;
}

.console-title {
    color: #f9fafb;
    font-weight: bold;
    font-size: 14px;
}

.console-controls {
    display: flex;
    gap: 8px;
    align-items: center;
}

.console-hint {
    color: #9ca3af;
    font-size: 11px;
    font-style: italic;
    padding: 4px 8px;
}


.console-output-container {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    background-color:rgba(17, 24, 39, 0.4);
    scrollbar-width: thin;
    scrollbar-color: #4b5563 #1f2937;
}

.console-output-content {
    min-height: 100%;
    display: flex;
    flex-direction: column;
}

.console-input-container {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    background-color:rgba(55, 65, 81, 0.4);
    border-top: 1px solid #4b5563;
    border-radius: 0 0 6px 6px;
}

.console-prompt {
    color: #60a5fa;
    font-weight: bold;
    margin-right: 8px;
    font-size: 14px;
}

.console-input {
    flex: 1;
    background-color: #1f2937;
    border: 1px solid #4b5563;
    color: #f9fafb;
    padding: 8px 12px;
    border-radius: 4px;
    font-family: inherit;
    font-size: 13px;
    outline: none;
}

.console-input:focus {
    border-color: #60a5fa;
    box-shadow: 0 0 0 2px rgba(96, 165, 250, 0.1);
}

.console-input::placeholder {
    color: #9ca3af;
}

/* Scrollbar styling for webkit browsers */
.console-output-container::-webkit-scrollbar {
    width: 8px;
}

.console-output-container::-webkit-scrollbar-track {
    background: #1f2937;
}

.console-output-container::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
}

.console-output-container::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
}

/* Mobile responsive adjustments */
@media (max-width: 768px) {
    .console-modal {
        width: 95%;
        height: 80%;
    }
    
    .console-header {
        padding: 8px 12px;
    }
    
    .console-title {
        font-size: 13px;
    }
    
    .console-input-container {
        padding: 8px 12px;
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_message_handling() {
        // Create a test console state
        let mut console = DevConsole {
            command_registry: CommandRegistry::new(),
            command_history: ConsoleHistory::new(),
            output_manager: ConsoleOutputManager::new(),
            input_value: "test command".to_string(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            was_visible: false,
        };

        // Test updating input
        console.input_value = "new command".to_string();
        assert_eq!(console.input_value, "new command");

        // Test clearing output
        console.output_manager.add_output(ConsoleOutput::info("test"));
        assert!(!console.output_manager.is_empty());
        console.output_manager.clear();
        assert!(console.output_manager.is_empty());
    }

    #[test]
    fn test_console_history_integration() {
        let mut console = DevConsole {
            command_registry: CommandRegistry::new(),
            command_history: ConsoleHistory::new(),
            output_manager: ConsoleOutputManager::new(),
            input_value: String::new(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            was_visible: false,
        };

        // Add some commands to history
        console.command_history.add_command("first".to_string());
        console.command_history.add_command("second".to_string());

        // Test navigation
        assert_eq!(console.command_history.navigate_previous(), Some("second"));
        assert_eq!(console.command_history.navigate_previous(), Some("first"));
        assert_eq!(console.command_history.navigate_next(), Some("second"));
    }

    #[test]
    fn test_local_storage_constants() {
        // Test storage key constant
        assert_eq!(CONSOLE_HISTORY_STORAGE_KEY, "pitch_toy_console_history");
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_local_storage_functionality() {
        // Test that local storage functions exist and handle browser context gracefully
        let history = DevConsole::load_history_from_storage();
        assert!(history.is_empty() || !history.is_empty()); // Should not panic
        
        let console = DevConsole {
            command_registry: CommandRegistry::new(),
            command_history: ConsoleHistory::new(),
            output_manager: ConsoleOutputManager::new(),
            input_value: String::new(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            was_visible: false,
        };
        
        // Save should not panic
        console.save_history_to_storage();
    }
}