use crate::{ConsoleCommandRegistry, ConsoleOutput, ConsoleCommandResult, ConsoleHistory, ConsoleOutputManager, ConsoleCommand};
use web_sys::Storage;
const CONSOLE_HISTORY_STORAGE_KEY: &str = "dev_console_history";

pub struct DevConsole {
    command_registry: ConsoleCommandRegistry,
    output_manager: ConsoleOutputManager,
    history: ConsoleHistory,
    input_text: String,
    is_visible: bool
}

impl DevConsole {
    pub fn new(registry: ConsoleCommandRegistry) -> Self {
        let mut output_manager = ConsoleOutputManager::new();
        output_manager.add_output(ConsoleOutput::info("EGUI Dev Console initialized"));
        output_manager.add_output(ConsoleOutput::info("Type 'help' for available commands"));
        
        let command_history = Self::load_history_from_storage();
        if !command_history.is_empty() {
            output_manager.add_output(ConsoleOutput::info(format!("Restored {} commands from history", command_history.len())));
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
        if !self.is_visible {
            return;
        }

        let screen_rect = ctx.screen_rect();
        three_d::egui::Window::new("Dev Console")
            .default_pos([screen_rect.width() - 600.0, 0.0])
            .default_size([600.0, screen_rect.height()])
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    three_d::egui::ScrollArea::vertical()
                        .max_height(600.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            self.render_output(ui);
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(">");
                        
                        let response = ui.text_edit_singleline(&mut self.input_text);
                        
                        if response.lost_focus() && ui.input(|i| i.key_pressed(three_d::egui::Key::Enter)) {
                            self.execute_command();
                            response.request_focus();
                        }

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
            });
    }


    fn render_output(&self, ui: &mut three_d::egui::Ui) {
        for output in self.output_manager.entries().iter().rev() {
            
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

        self.history.add_command(command.clone());
        self.save_history_to_storage();
        self.output_manager.add_output(ConsoleOutput::echo(&command));

        let result = self.command_registry.execute(&command);
        match result {
            ConsoleCommandResult::Output(output) => self.output_manager.add_output(output),
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

        self.input_text.clear();
        self.history.reset_navigation();
    }

    pub fn register_command(&mut self, command: Box<dyn ConsoleCommand>) {
        self.command_registry.register(command);
    }

    fn load_history_from_storage() -> ConsoleHistory {
        Self::get_local_storage()
            .and_then(|storage| storage.get_item(CONSOLE_HISTORY_STORAGE_KEY).ok()?)
            .and_then(|history_json| serde_json::from_str(&history_json).ok())
            .unwrap_or_else(ConsoleHistory::new)
    }

    fn save_history_to_storage(&self) {
        if let (Some(storage), Ok(history_json)) = (Self::get_local_storage(), serde_json::to_string(&self.history)) {
            let _ = storage.set_item(CONSOLE_HISTORY_STORAGE_KEY, &history_json);
        }
    }

    fn get_local_storage() -> Option<Storage> {
        web_sys::window()?.local_storage().ok()?
    }
}

