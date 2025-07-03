// Development Console Component
// Interactive debugging and development tools for pitch-toy application
// Available only in development builds

use yew::prelude::*;
use web_sys::{HtmlInputElement, KeyboardEvent, Storage};
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;

use super::command_registry::{ConsoleCommandResult, ConsoleCommandRegistry};
use super::history::ConsoleHistory;
use super::output::{ConsoleOutput, ConsoleOutputManager, CONSOLE_OUTPUT_CSS};
use crate::modules::audio::{AudioPermission, permission::PermissionManager, get_audio_context_manager};

/// Local storage key for console history persistence
const CONSOLE_HISTORY_STORAGE_KEY: &str = "pitch_toy_console_history";

/// Properties for the DevConsole component
#[derive(Properties)]
pub struct DevConsoleProps {
    /// Command registry to use for executing commands
    pub registry: Rc<ConsoleCommandRegistry>,
}

impl PartialEq for DevConsoleProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer equality since registries are immutable after creation
        Rc::ptr_eq(&self.registry, &other.registry)
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
    /// Whether the console is currently visible
    visible: bool,
    /// Track previous visibility state for focus management
    was_visible: bool,
    /// Current audio permission state
    audio_permission: AudioPermission,
}

/// Messages for the DevConsole component
pub enum DevConsoleMsg {
    /// Execute a command
    ExecuteCommand,
    /// Update input value
    UpdateInput(String),
    /// Handle keyboard events (history navigation, shortcuts)
    HandleKeyDown(KeyboardEvent),
    /// Toggle console visibility
    ToggleVisibility,
    /// Request audio permission
    RequestAudioPermission,
    /// Update audio permission state
    UpdateAudioPermission(AudioPermission),
    /// Periodic device refresh tick
    RefreshDevices,
    /// Device refresh completed - trigger re-render
    DevicesRefreshed,
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
        
        let console = Self {
            registry: Rc::clone(&ctx.props().registry),
            command_history,
            output_manager,
            input_value: String::new(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            visible: true, // Start visible by default (matching current behavior)
            was_visible: false,
            audio_permission: AudioPermission::Uninitialized,
        };
        
        // Set up device change listener on the audio context manager
        let link_for_devices = ctx.link().clone();
        if let Some(manager_rc) = get_audio_context_manager() {
            let mut manager = manager_rc.borrow_mut();
            let _ = manager.setup_device_change_listener(move || {
                link_for_devices.send_message(DevConsoleMsg::RefreshDevices);
            });
        }
        
        // Check microphone permission status on component creation
        let link = ctx.link().clone();
        spawn_local(async move {
            let permission = PermissionManager::check_microphone_permission().await;
            link.send_message(DevConsoleMsg::UpdateAudioPermission(permission));
        });
        
        console
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
                        false
                    }
                    _ => false
                }
            }
            
            DevConsoleMsg::ToggleVisibility => {
                web_sys::console::log_3(&"DevConsole: Toggling visibility from".into(), &self.visible.into(), &format!("to {}", !self.visible).into());
                self.visible = !self.visible;
                true
            }
            
            DevConsoleMsg::RequestAudioPermission => {
                // Update state to requesting immediately
                self.audio_permission = AudioPermission::Requesting;
                
                // Request permission - must be in same call stack as user gesture
                let link = ctx.link().clone();
                spawn_local(async move {
                    let _result = PermissionManager::request_permission_with_callback(move |permission_state| {
                        link.send_message(DevConsoleMsg::UpdateAudioPermission(permission_state));
                    }).await;
                });
                
                true
            }
            
            DevConsoleMsg::UpdateAudioPermission(permission) => {
                let old_permission = self.audio_permission.clone();
                self.audio_permission = permission;
                
                // Refresh device list when permission is granted to show device labels
                if old_permission != AudioPermission::Granted && self.audio_permission == AudioPermission::Granted {
                    ctx.link().send_message(DevConsoleMsg::RefreshDevices);
                }
                
                // Only re-render if permission actually changed
                old_permission != self.audio_permission
            }
            
            DevConsoleMsg::RefreshDevices => {
                // Refresh audio devices in background
                let link = ctx.link().clone();
                spawn_local(async move {
                    if let Some(manager_rc) = get_audio_context_manager() {
                        // Try to borrow mutably, but handle the case where it's already borrowed
                        match manager_rc.try_borrow_mut() {
                            Ok(mut manager) => {
                                let result = manager.refresh_audio_devices().await;
                                // After refresh completes, trigger a re-render to show updated devices
                                if result.is_ok() {
                                    link.send_message(DevConsoleMsg::DevicesRefreshed);
                                }
                            }
                            Err(_) => {
                                // Manager is currently borrowed, try again in a moment
                                web_sys::console::log_1(&"AudioContextManager busy, skipping device refresh".into());
                            }
                        }
                    }
                });
                false // No need to re-render immediately
            }
            
            DevConsoleMsg::DevicesRefreshed => {
                // Device refresh completed, trigger re-render to show updated devices
                true
            }
            
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !self.visible {
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
                        <div class="console-left">
                            <span class="console-title">{ "Dev Console" }</span>
                            <span class="console-hint" title="Press ESC to toggle console">
                                { "(ESC to toggle)" }
                            </span>
                        </div>
                        <div class="console-controls">
                            { self.render_audio_permission_ui(ctx) }
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
                    
                    { self.render_device_list() }
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let is_visible = self.visible;
        
        // Set up global keyboard listener on first render
        if first_render {
            self.setup_global_keyboard_listener(ctx);
        }
        
        // Focus input on first render or when console becomes visible
        if first_render && is_visible {
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

    /// Render the audio permission UI based on current state
    fn render_audio_permission_ui(&self, ctx: &Context<Self>) -> Html {
        match self.audio_permission {
            AudioPermission::Uninitialized | AudioPermission::Unavailable => {
                let onclick = {
                    let link = ctx.link().clone();
                    Callback::from(move |_| {
                        link.send_message(DevConsoleMsg::RequestAudioPermission);
                    })
                };
                
                html! {
                    <button class="audio-permission-button" onclick={onclick}>
                        { "Request Audio Permission" }
                    </button>
                }
            }
            AudioPermission::Requesting => {
                html! {
                    <span class="audio-permission-status requesting">
                        { "Requesting permission..." }
                    </span>
                }
            }
            AudioPermission::Granted => {
                html! {
                    <span class="audio-permission-status granted">
                        { "Audio permission granted" }
                    </span>
                }
            }
            AudioPermission::Denied => {
                html! {
                    <span class="audio-permission-status denied">
                        { "Audio permission denied" }
                    </span>
                }
            }
        }
    }

    /// Render the audio device list
    fn render_device_list(&self) -> Html {
        if let Some(manager_rc) = get_audio_context_manager() {
            // Try to borrow, but handle the case where it's already borrowed
            let devices = match manager_rc.try_borrow() {
                Ok(manager) => manager.get_cached_devices().clone(),
                Err(_) => {
                    // Manager is currently borrowed (probably refreshing), show loading state
                    return html! {
                        <div class="console-device-list">
                            <div class="device-section">
                                <div class="device-item empty">{ "Refreshing devices..." }</div>
                            </div>
                        </div>
                    };
                }
            };
            
            html! {
                <div class="console-device-list">
                    <div class="device-sections">
                        <div class="device-section">
                            <h4>{ "🎤 Input Devices" }</h4>
                            if devices.input_devices.is_empty() {
                                <div class="device-item empty">{ "No input devices found" }</div>
                            } else {
                                { for devices.input_devices.iter().map(|(device_id, label)| {
                                    html! {
                                        <div class="device-item" title={device_id.clone()}>
                                            { label.clone() }
                                        </div>
                                    }
                                })}
                            }
                        </div>
                        
                        <div class="device-section">
                            <h4>{ "🔊 Output Devices" }</h4>
                            if devices.output_devices.is_empty() {
                                <div class="device-item empty">{ "No output devices found" }</div>
                            } else {
                                { for devices.output_devices.iter().map(|(device_id, label)| {
                                    html! {
                                        <div class="device-item" title={device_id.clone()}>
                                            { label.clone() }
                                        </div>
                                    }
                                })}
                            }
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="console-device-list">
                    <div class="device-section">
                        <div class="device-item empty">{ "Audio system not initialized" }</div>
                    </div>
                </div>
            }
        }
    }

    /// Render the console output
    fn render_output(&self) -> Html {
        let entries = self.output_manager.entries();
        
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

    /// Set up global keyboard event listener for Escape key
    fn setup_global_keyboard_listener(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
                if keyboard_event.key() == "Escape" {
                    web_sys::console::log_1(&"Global Escape key detected - toggling console".into());
                    keyboard_event.prevent_default();
                    
                    // Toggle console visibility
                    link.send_message(DevConsoleMsg::ToggleVisibility);
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        let window = web_sys::window()
            .expect("Failed to get window");
        web_sys::console::log_1(&"DevConsole: Setting up global keydown event listener".into());
        window
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .expect("Failed to add keydown event listener to window");
        
        // Store closure to prevent it from being dropped
        closure.forget();
        
        // Note: In a real application, we should properly manage the closure cleanup
        // For this development console, the memory leak is acceptable
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

.console-left {
    display: flex;
    gap: 12px;
    align-items: center;
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
}

.audio-permission-button {
    background-color: #3b82f6;
    color: white;
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    margin-right: 8px;
    transition: background-color 0.2s;
}

.audio-permission-button:hover {
    background-color: #2563eb;
}

.audio-permission-status {
    font-size: 11px;
    padding: 6px 12px;
    border-radius: 4px;
    margin-right: 8px;
    font-weight: 500;
}

.audio-permission-status.granted {
    color: #10b981;
    background-color: rgba(16, 185, 129, 0.1);
}

.audio-permission-status.denied {
    color: #ef4444;
    background-color: rgba(239, 68, 68, 0.1);
}

.audio-permission-status.requesting {
    color: #f59e0b;
    background-color: rgba(245, 158, 11, 0.1);
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

/* Device list styling */
.console-device-list {
    background-color: rgba(31, 41, 55, 0.6);
    border-top: 1px solid #4b5563;
    padding: 8px 16px;
    flex: 1;
    overflow-y: auto;
    font-size: 11px;
}

.device-sections {
    display: flex;
    gap: 16px;
}

.device-section {
    flex: 1;
    margin-bottom: 8px;
}

.device-section:last-child {
    margin-bottom: 0;
}

.device-section h4 {
    color: #9ca3af;
    font-size: 10px;
    font-weight: 600;
    margin: 0 0 4px 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.device-item {
    color: #f9fafb;
    padding: 2px 4px;
    border-radius: 2px;
    margin-bottom: 1px;
    font-family: inherit;
    cursor: default;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.device-item:hover {
    background-color: rgba(75, 85, 99, 0.3);
}

.device-item.empty {
    color: #6b7280;
    font-style: italic;
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
    
    .console-device-list {
        font-size: 10px;
    }
    
    .device-sections {
        gap: 8px;
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::console::ConsoleCommandRegistry;

    #[test]
    fn test_console_message_handling() {
        // Create a test console state
        let mut console = DevConsole {
            registry: Rc::new(ConsoleCommandRegistry::new()),
            command_history: ConsoleHistory::new(),
            output_manager: ConsoleOutputManager::new(),
            input_value: "test command".to_string(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            visible: true,
            was_visible: false,
            audio_permission: AudioPermission::Uninitialized,
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
            registry: Rc::new(ConsoleCommandRegistry::new()),
            command_history: ConsoleHistory::new(),
            output_manager: ConsoleOutputManager::new(),
            input_value: String::new(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            visible: true,
            was_visible: false,
            audio_permission: AudioPermission::Uninitialized,
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

    #[test]
    fn test_audio_permission_state_transitions() {
        let mut console = DevConsole {
            registry: Rc::new(ConsoleCommandRegistry::new()),
            command_history: ConsoleHistory::new(),
            output_manager: ConsoleOutputManager::new(),
            input_value: String::new(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            visible: true,
            was_visible: false,
            audio_permission: AudioPermission::Uninitialized,
        };

        // Test initial state
        assert_eq!(console.audio_permission, AudioPermission::Uninitialized);

        // Test state change to requesting
        console.audio_permission = AudioPermission::Requesting;
        assert_eq!(console.audio_permission, AudioPermission::Requesting);

        // Test state change to granted
        console.audio_permission = AudioPermission::Granted;
        assert_eq!(console.audio_permission, AudioPermission::Granted);

        // Test state change to denied
        console.audio_permission = AudioPermission::Denied;
        assert_eq!(console.audio_permission, AudioPermission::Denied);
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_local_storage_functionality() {
        // Test that local storage functions exist and handle browser context gracefully
        let history = DevConsole::load_history_from_storage();
        assert!(history.is_empty() || !history.is_empty()); // Should not panic
        
        let console = DevConsole {
            registry: Rc::new(ConsoleCommandRegistry::new()),
            command_history: ConsoleHistory::new(),
            output_manager: ConsoleOutputManager::new(),
            input_value: String::new(),
            input_ref: NodeRef::default(),
            output_ref: NodeRef::default(),
            visible: true,
            was_visible: false,
            audio_permission: AudioPermission::Uninitialized,
        };
        
        // Save should not panic
        console.save_history_to_storage();
    }
}