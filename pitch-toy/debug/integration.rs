// Debug Integration Layer - Component communication and coordination
//
// This module provides the integration layer for coordinating between the three
// debug components: DebugConsole, LivePanel, and PermissionButton.

use yew::prelude::*;
use std::rc::Rc;
use wasm_bindgen::JsCast;

use dev_console::{ConsoleCommandRegistry, ConsoleHistory, ConsoleOutputManager};
use super::{LivePanel, PermissionButton};
use super::permission_button::AudioPermissionService;
use crate::audio::{AudioPermission, ConsoleAudioServiceImpl, ConsoleAudioService};
use crate::events::SharedEventDispatcher;

/// Properties for the integrated debug interface
#[derive(Properties)]
pub struct DebugInterfaceProps {
    /// Command registry for the console
    pub registry: Rc<ConsoleCommandRegistry>,
    /// Audio service for audio operations
    pub audio_service: Rc<ConsoleAudioServiceImpl>,
    /// Event dispatcher for real-time updates
    pub event_dispatcher: Option<SharedEventDispatcher>,
}

impl PartialEq for DebugInterfaceProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.registry, &other.registry) && 
        Rc::ptr_eq(&self.audio_service, &other.audio_service)
    }
}

/// Integrated debug interface component
pub struct DebugInterface {
    /// Whether the entire debug interface is visible
    visible: bool,
    /// Current audio permission state
    audio_permission: AudioPermission,
    /// Console input value
    console_input: String,
    /// Command history for navigation
    command_history: ConsoleHistory,
    /// Output manager for displaying results
    output_manager: ConsoleOutputManager,
    /// Reference to the output container element for auto-scrolling
    output_ref: NodeRef,
    /// Reference to the input element
    input_ref: NodeRef,
}

/// Messages for the debug interface
#[derive(Debug)]
pub enum DebugInterfaceMsg {
    /// Toggle entire debug interface visibility
    ToggleVisibility,
    /// Permission state changed
    PermissionChanged(AudioPermission),
    /// Update console input
    UpdateConsoleInput(String),
    /// Execute console command
    ExecuteConsoleCommand,
    /// Handle keyboard events
    HandleKeyDown(web_sys::KeyboardEvent),
}

impl Component for DebugInterface {
    type Message = DebugInterfaceMsg;
    type Properties = DebugInterfaceProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut output_manager = ConsoleOutputManager::new();
        
        // Add welcome message
        output_manager.add_output(dev_console::ConsoleOutput::info("Debug console initialized"));
        output_manager.add_output(dev_console::ConsoleOutput::info("Type 'help' for available commands"));
        
        // Load command history from local storage
        let command_history = Self::load_history_from_storage();
        if !command_history.is_empty() {
            output_manager.add_output(dev_console::ConsoleOutput::info(&format!("Restored {} commands from history", command_history.len())));
        }
        
        let component = Self {
            visible: true,  // Start with debug interface visible on app start
            audio_permission: AudioPermission::Uninitialized,
            console_input: String::new(),
            command_history,
            output_manager,
            output_ref: NodeRef::default(),
            input_ref: NodeRef::default(),
        };

        // Check initial permission state from browser
        let link = ctx.link().clone();
        let audio_service = ctx.props().audio_service.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let permission = audio_service.get_current_permission().await;
            link.send_message(DebugInterfaceMsg::PermissionChanged(permission));
        });

        component
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DebugInterfaceMsg::ToggleVisibility => {
                self.visible = !self.visible;
                true
            }
            DebugInterfaceMsg::PermissionChanged(permission) => {
                self.audio_permission = permission.clone();
                
                // If permission was granted, refresh the device list
                if permission == AudioPermission::Granted {
                    ctx.props().audio_service.refresh_devices();
                }
                
                true
            }
            DebugInterfaceMsg::UpdateConsoleInput(value) => {
                self.console_input = value;
                true
            }
            DebugInterfaceMsg::ExecuteConsoleCommand => {
                let command = self.console_input.trim();
                if !command.is_empty() {
                    // Add command to history
                    self.command_history.add_command(command.to_string());
                    
                    // Save history to local storage
                    self.save_history_to_storage();

                    // Echo the command
                    self.output_manager.add_output(dev_console::ConsoleOutput::echo(command));
                    
                    // Execute the command using the registry
                    let result = ctx.props().registry.execute(command);
                    match result {
                        dev_console::ConsoleCommandResult::Output(output) => {
                            self.output_manager.add_output(output);
                        }
                        dev_console::ConsoleCommandResult::ClearAndOutput(output) => {
                            self.output_manager.clear();
                            self.output_manager.add_output(output);
                        }
                        dev_console::ConsoleCommandResult::MultipleOutputs(outputs) => {
                            for output in outputs {
                                self.output_manager.add_output(output);
                            }
                        }
                    }
                    
                    // Clear input
                    self.console_input.clear();
                    
                    // Reset history navigation
                    self.command_history.reset_navigation();
                }
                true
            }
            DebugInterfaceMsg::HandleKeyDown(event) => {
                match event.key().as_str() {
                    "Enter" => {
                        event.prevent_default();
                        ctx.link().send_message(DebugInterfaceMsg::ExecuteConsoleCommand);
                        false
                    }
                    "ArrowUp" => {
                        event.prevent_default();
                        if let Some(command) = self.command_history.navigate_previous() {
                            self.console_input = command.to_string();
                            self.focus_input_end();
                        }
                        true
                    }
                    "ArrowDown" => {
                        event.prevent_default();
                        if let Some(command) = self.command_history.navigate_next() {
                            self.console_input = command.to_string();
                            self.focus_input_end();
                        }
                        true
                    }
                    _ => false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !self.visible {
            return html! {};
        }

        html! {
            <>
                <style>
                    {DEBUG_INTERFACE_CSS}
                </style>
                <div class="debug-interface">
                    {self.render_debug_components(ctx)}
                </div>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if _first_render {
            self.setup_global_keyboard_handler(ctx);
        }
        
        // Auto-scroll to bottom when new output is added
        if let Some(output_element) = self.output_ref.cast::<web_sys::Element>() {
            output_element.set_scroll_top(output_element.scroll_height());
        }
    }
}

impl DebugInterface {
    /// Set up global keyboard handler for Escape key
    fn setup_global_keyboard_handler(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if event.key() == "Escape" {
                event.prevent_default();
                link.send_message(DebugInterfaceMsg::ToggleVisibility);
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                document
                    .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                    .unwrap();
            }
        }
        
        // Keep the closure alive by leaking it (this is acceptable for a global handler)
        closure.forget();
    }

    /// Load command history from local storage
    fn load_history_from_storage() -> ConsoleHistory {
        if let Some(storage) = Self::get_local_storage() {
            if let Ok(Some(history_json)) = storage.get_item("dev_console_history") {
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
                let _ = storage.set_item("dev_console_history", &history_json);
            }
        }
    }

    /// Get local storage reference
    fn get_local_storage() -> Option<web_sys::Storage> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok())
            .flatten()
    }

    /// Focus the input element and move cursor to end
    fn focus_input_end(&self) {
        if let Some(input_element) = self.input_ref.cast::<web_sys::HtmlInputElement>() {
            input_element.focus().ok();
            if let Ok(length) = input_element.value().len().try_into() {
                input_element.set_selection_start(Some(length)).ok();
                input_element.set_selection_end(Some(length)).ok();
            }
        }
    }


    /// Render the debug components
    fn render_debug_components(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="debug-components">
                {self.render_console(ctx)}
                {self.render_permission_button(ctx)}
                {self.render_live_panel(ctx)}
            </div>
        }
    }

    /// Render the debug console
    fn render_console(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="debug-console-modal">
                <div class="debug-console-header">
                    <h3 class="debug-console-title">{"Debug Console"}</h3>
                </div>
                
                <div class="debug-console-content">
                    <div class="debug-console-output" ref={self.output_ref.clone()}>
                        <div class="debug-console-messages">
                            {for self.output_manager.entries().iter().rev().map(|entry| {
                                html! {
                                    <div class={format!("debug-console-message debug-console-message-{}", entry.output.output_type())}>
                                        {entry.output.to_string()}
                                    </div>
                                }
                            })}
                        </div>
                    </div>
                    
                    <div class="debug-console-input-container">
                        <span class="debug-console-prompt">{">"}</span>
                        <input
                            ref={self.input_ref.clone()}
                            type="text"
                            class="debug-console-input"
                            value={self.console_input.clone()}
                            placeholder="Enter command..."
                            oninput={ctx.link().callback(|e: web_sys::InputEvent| {
                                if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                    DebugInterfaceMsg::UpdateConsoleInput(input.value())
                                } else {
                                    DebugInterfaceMsg::UpdateConsoleInput(String::new())
                                }
                            })}
                            onkeydown={ctx.link().callback(DebugInterfaceMsg::HandleKeyDown)}

                        />
                    </div>
                </div>
            </div>
        }
    }

    /// Render the live panel
    fn render_live_panel(&self, ctx: &Context<Self>) -> Html {
        if let Some(event_dispatcher) = &ctx.props().event_dispatcher {
            html! {
                <LivePanel
                    event_dispatcher={event_dispatcher.clone()}
                    visible={true}
                    audio_permission={self.audio_permission.clone()}
                    audio_service={ctx.props().audio_service.clone()}
                />
            }
        } else {
            html! {}
        }
    }

    /// Render the permission button
    fn render_permission_button(&self, ctx: &Context<Self>) -> Html {
        // Create adapter for the audio service
        let service_adapter: Rc<dyn AudioPermissionService> = Rc::new(AudioServiceAdapter::new(ctx.props().audio_service.clone()));
        
        html! {
            <PermissionButton
                audio_service={service_adapter}
                on_permission_change={ctx.link().callback(DebugInterfaceMsg::PermissionChanged)}
            />
        }
    }
}

/// Create the integrated debug interface
pub fn create_debug_interface(
    registry: Rc<ConsoleCommandRegistry>,
    audio_service: Rc<ConsoleAudioServiceImpl>,
    event_dispatcher: Option<SharedEventDispatcher>,
) -> Html {
    html! {
        <DebugInterface
            registry={registry}
            audio_service={audio_service}
            event_dispatcher={event_dispatcher}
        />
    }
}

/// Adapter to make ConsoleAudioServiceImpl work with AudioPermissionService trait
pub struct AudioServiceAdapter {
    audio_service: Rc<ConsoleAudioServiceImpl>,
}

impl AudioServiceAdapter {
    pub fn new(audio_service: Rc<ConsoleAudioServiceImpl>) -> Self {
        Self { audio_service }
    }
}

impl AudioPermissionService for AudioServiceAdapter {
    fn request_permission(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AudioPermission, String>> + '_>> {
        let audio_service = self.audio_service.clone();
        Box::pin(async move {
            // Use the existing permission request method with callback
            let permission = audio_service.request_permission_with_callback(|_| {}).await;
            Ok(permission)
        })
    }
    
    fn get_current_permission(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = AudioPermission> + '_>> {
        let audio_service = self.audio_service.clone();
        Box::pin(async move {
            audio_service.get_current_permission().await
        })
    }
}

/// CSS styles for the debug interface
const DEBUG_INTERFACE_CSS: &str = r#"
.debug-interface {
    position: fixed;
    top: 10px;
    right: 10px;
    z-index: 1000;
    font-family: monospace;
    font-size: 12px;
}

.debug-toolbar {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
    padding: 8px;
    background: rgba(17, 24, 39, 0.95);
    border: 1px solid #374151;
    border-radius: 6px;
    align-items: center;
}

.debug-toggle {
    padding: 4px 8px;
    background: #374151;
    color: #d1d5db;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
}

.debug-toggle:hover {
    background: #4b5563;
}

.debug-toggle.active {
    background: #3b82f6;
    color: white;
}

.debug-toolbar-title {
    color: #f9fafb;
    font-size: 14px;
    font-weight: bold;
}

.debug-toolbar-spacer {
    flex: 1;
}

.debug-components {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: 400px;
}

.debug-console-modal {
    background: rgba(17, 24, 39, 0.95);
    border: 1px solid #374151;
    border-radius: 6px;
    padding: 0;
    max-height: 400px;
    display: flex;
    flex-direction: column;
}

.debug-console-header {
    padding: 8px 12px;
    background: #1f2937;
    border-bottom: 1px solid #374151;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.debug-console-title {
    margin: 0;
    font-size: 14px;
    color: #f9fafb;
}

.debug-console-close {
    background: none;
    border: none;
    color: #9ca3af;
    cursor: pointer;
    font-size: 18px;
    padding: 0;
    width: 20px;
    height: 20px;
}

.debug-console-close:hover {
    color: #f9fafb;
}

.debug-console-content {
    display: flex;
    flex-direction: column;
    height: 300px;
}

.debug-console-output {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    background: #111827;
}

.debug-console-messages {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.debug-console-message {
    padding: 2px 4px;
    border-radius: 2px;
    font-family: 'Courier New', monospace;
    font-size: 11px;
    white-space: pre-wrap;
    word-wrap: break-word;
}

.debug-console-message-info {
    color: #d1d5db;
}

.debug-console-message-success {
    color: #10b981;
}

.debug-console-message-warning {
    color: #f59e0b;
}

.debug-console-message-error {
    color: #ef4444;
}

.debug-console-message-command {
    color: #3b82f6;
    font-weight: bold;
}

.debug-console-input-container {
    padding: 8px;
    background: #1f2937;
    border-top: 1px solid #374151;
    display: flex;
    align-items: center;
    gap: 4px;
}

.debug-console-prompt {
    color: #3b82f6;
    font-weight: bold;
}

.debug-console-input {
    flex: 1;
    background: #111827;
    border: 1px solid #374151;
    color: #f9fafb;
    padding: 4px 6px;
    border-radius: 3px;
    font-family: 'Courier New', monospace;
    font-size: 11px;
}

.debug-console-input:focus {
    outline: none;
    border-color: #3b82f6;
}

.live-panel {
    background: rgba(17, 24, 39, 0.95);
    border: 1px solid #374151;
    padding: 0;
    color: #f9fafb;
    font-family: monospace;
    font-size: 12px;
    min-width: 300px;
}

.live-panel-header {
    padding: 8px 12px;
    background: #1f2937;
    border-bottom: 1px solid #374151;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.live-panel-title {
    margin: 0;
    font-size: 14px;
    color: #f9fafb;
}

.live-panel-refresh {
    float: right;
    background: none;
    border: none;
    color: #f9fafb;
    cursor: pointer;
    font-size: 14px;
}

.live-panel-content {
    padding: 12px;
}

.live-panel-content > div {
    margin-bottom: 15px;
}

.live-panel-section-title {
    margin: 0 0 8px 0;
    color: #d1d5db;
    font-weight: bold;
}

.permission-status {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
}

.permission-status.permission-granted {
    color: #10b981;
}

.permission-status.permission-denied {
    color: #ef4444;
}

.permission-status.permission-requesting {
    color: #f59e0b;
}

.permission-status.permission-unknown {
    color: #f59e0b;
}

.permission-status.permission-unavailable {
    color: #f59e0b;
}

.device-section h5 {
    margin: 8px 0 4px 0;
    font-size: 11px;
    color: #9ca3af;
}

.device-item {
    margin-left: 10px;
    font-size: 11px;
    margin-bottom: 2px;
}

.device-name {
    color: #d1d5db;
    font-weight: bold;
}

.device-id {
    color: #6b7280;
    margin-left: 8px;
}

.metric-item {
    margin-bottom: 4px;
    display: flex;
    gap: 8px;
}

.metric-label {
    color: #9ca3af;
    font-size: 10px;
    width: 80px;
}

.metric-value {
    color: #d1d5db;
    font-weight: bold;
}

.volume-placeholder {
    color: #6b7280;
    font-style: italic;
    font-size: 11px;
}

.pitch-placeholder {
    color: #6b7280;
    font-style: italic;
    font-size: 11px;
}

.permission-button-container {
    font-size: 11px;
}

.permission-button {
    padding: 4px 8px;
    border: 1px solid #374151;
    border-radius: 4px;
    background: #3b82f6;
    color: #ffffff;
    cursor: pointer;
    font-family: monospace;
    font-size: 11px;
}

.permission-button:disabled {
    cursor: not-allowed;
    opacity: 0.6;
}

.permission-button-uninitialized {
    border-color: #6b7280;
}

.permission-button-requesting {
    border-color: #f59e0b;
    background: rgba(245, 158, 11, 0.1);
}

.permission-button-granted {
    border-color: #10b981;
    background: rgba(16, 185, 129, 0.1);
}

.permission-button-denied {
    border-color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
}

.permission-button-unavailable {
    border-color: #f59e0b;
    background: rgba(245, 158, 11, 0.1);
}

.permission-button-content {
    display: flex;
    align-items: center;
    gap: 4px;
}

.permission-status {
    margin-top: 4px;
    font-size: 10px;
    color: #9ca3af;
}

.permission-error {
    margin-top: 4px;
    font-size: 10px;
    color: #ef4444;
}
"#;