// EGUI Development Console Component
// Provides the main EGUI-based console interface

use crate::{ConsoleCommandRegistry, ConsoleOutput, ConsoleCommandResult, ConsoleHistory, ConsoleOutputManager, ConsoleCommand};
use web_sys::Storage;

/// Local storage key for console history persistence (same as original dev-console)
const CONSOLE_HISTORY_STORAGE_KEY: &str = "dev_console_history";

pub struct EguiDevConsole {
    command_registry: ConsoleCommandRegistry,
    output_manager: ConsoleOutputManager,
    history: ConsoleHistory,
    input_text: String,
    is_visible: bool
}

impl EguiDevConsole {
    pub fn new() -> Self {
        let mut output_manager = ConsoleOutputManager::new();
        
        // Add welcome message
        output_manager.add_output(ConsoleOutput::info("EGUI Dev Console initialized"));
        output_manager.add_output(ConsoleOutput::info("Type 'help' for available commands"));
        
        // Load command history from local storage
        let command_history = Self::load_history_from_storage();
        if !command_history.is_empty() {
            output_manager.add_output(ConsoleOutput::info(&format!("Restored {} commands from history", command_history.len())));
        }
        
        Self {
            command_registry: ConsoleCommandRegistry::new(),
            output_manager,
            history: command_history,
            input_text: String::new(),
            is_visible: true
        }
    }

    pub fn new_with_registry(registry: ConsoleCommandRegistry) -> Self {
        let mut output_manager = ConsoleOutputManager::new();
        
        // Add welcome message
        output_manager.add_output(ConsoleOutput::info("EGUI Dev Console initialized"));
        output_manager.add_output(ConsoleOutput::info("Type 'help' for available commands"));
        
        // Load command history from local storage
        let command_history = Self::load_history_from_storage();
        if !command_history.is_empty() {
            output_manager.add_output(ConsoleOutput::info(&format!("Restored {} commands from history", command_history.len())));
        }
        
        Self {
            command_registry: registry,
            output_manager,
            history: command_history,
            input_text: String::new(),
            is_visible: true
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn render(&mut self, ctx: &three_d::egui::Context) {
        // Only render console if visible
        if !self.is_visible {
            return;
        }

        three_d::egui::Window::new("Dev Console")
            .default_width(600.0)
            .default_height(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_console(ui);
            });
    }

    fn render_console(&mut self, ui: &mut three_d::egui::Ui) {
        ui.vertical(|ui| {
            // Output area
            three_d::egui::ScrollArea::vertical()
                .max_height(600.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    self.render_output(ui);
                });

            ui.separator();

            // Input area
            ui.horizontal(|ui| {
                ui.label(">");
                
                let response = ui.text_edit_singleline(&mut self.input_text);
                
                // Handle enter key
                if response.lost_focus() && ui.input(|i| i.key_pressed(three_d::egui::Key::Enter)) {
                    self.execute_command();
                    response.request_focus();
                }

                // Handle history navigation
                if response.has_focus() {
                    if ui.input(|i| i.key_pressed(three_d::egui::Key::ArrowUp)) {
                        if let Some(cmd) = self.history.navigate_previous() {
                            self.input_text = cmd.to_string();
                        }
                    }
                    if ui.input(|i| i.key_pressed(three_d::egui::Key::ArrowDown)) {
                        if let Some(cmd) = self.history.navigate_next() {
                            self.input_text = cmd.to_string();
                        }
                    }
                }
            });
        });
    }

    fn render_output(&self, ui: &mut three_d::egui::Ui) {
        for entry in self.output_manager.entries().iter().rev() {
            let output = &entry.output;
            
            let color = match output {
                ConsoleOutput::Info(_) => three_d::egui::Color32::WHITE,
                ConsoleOutput::Success(_) => three_d::egui::Color32::GREEN,
                ConsoleOutput::Warning(_) => three_d::egui::Color32::YELLOW,
                ConsoleOutput::Error(_) => three_d::egui::Color32::RED,
                ConsoleOutput::Echo(_) => three_d::egui::Color32::LIGHT_BLUE,
                ConsoleOutput::Empty => three_d::egui::Color32::WHITE,
            };

            if !output.message().is_empty() {
                ui.colored_label(color, output.message());
            } else {
                ui.label("");
            }
        }
    }

    fn execute_command(&mut self) {
        let command = self.input_text.trim().to_string();
        
        if command.is_empty() {
            return;
        }

        // Add command to history
        self.history.add_command(command.clone());

        // Save history to local storage
        self.save_history_to_storage();

        // Echo the command
        self.output_manager.add_output(ConsoleOutput::echo(&command));

        // Execute the command
        let result = self.command_registry.execute(&command);
        
        // Handle the result
        match result {
            ConsoleCommandResult::Output(output) => {
                self.output_manager.add_output(output);
            }
            ConsoleCommandResult::MultipleOutputs(outputs) => {
                for output in outputs {
                    self.output_manager.add_output(output);
                }
            }
            ConsoleCommandResult::ClearAndOutput(output) => {
                self.output_manager.clear();
                self.output_manager.add_output(output);
            }
        }

        // Clear input
        self.input_text.clear();

        // Reset history navigation
        self.history.reset_navigation();
    }

    pub fn register_command(&mut self, command: Box<dyn ConsoleCommand>) {
        self.command_registry.register(command);
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
            if let Ok(history_json) = serde_json::to_string(&self.history) {
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

