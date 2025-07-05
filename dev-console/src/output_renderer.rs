// Output Renderer - Output formatting and display for debug console
//
// This module handles the rendering and formatting of console output messages.
// It provides utilities for styling and displaying different types of console output.

use yew::prelude::*;
use crate::output::ConsoleOutput;

/// Output renderer for debug console messages
pub struct OutputRenderer;

impl OutputRenderer {
    /// Render a single console output message
    pub fn render_output(output: &ConsoleOutput) -> Html {
        html! {
            <div class={format!("debug-console-message debug-console-message-{}", Self::get_css_class(output))}>
                {output.to_string()}
            </div>
        }
    }

    /// Render multiple console output messages
    pub fn render_outputs(outputs: &[ConsoleOutput]) -> Html {
        html! {
            <div class="debug-console-messages">
                {for outputs.iter().map(|output| Self::render_output(output))}
            </div>
        }
    }

    /// Get CSS class for console output type
    fn get_css_class(output: &ConsoleOutput) -> &'static str {
        match output {
            ConsoleOutput::Info(_) => "info",
            ConsoleOutput::Success(_) => "success",
            ConsoleOutput::Warning(_) => "warning",
            ConsoleOutput::Error(_) => "error",
            ConsoleOutput::Echo(_) => "echo",
            ConsoleOutput::Empty => "empty",
        }
    }

    /// Get icon for console output type
    pub fn get_icon(output: &ConsoleOutput) -> &'static str {
        match output {
            ConsoleOutput::Info(_) => "ℹ️",
            ConsoleOutput::Success(_) => "✅",
            ConsoleOutput::Warning(_) => "⚠️",
            ConsoleOutput::Error(_) => "❌",
            ConsoleOutput::Echo(_) => "▶️",
            ConsoleOutput::Empty => "",
        }
    }
}