# Dev Console

A reusable development console library for Rust/WebAssembly applications built with Yew.

## Overview

`dev-console` provides a generic, interactive command-line interface component that can be easily integrated into any Yew-based application. It features command execution, history management, output formatting, and a pluggable command registry system.

## Features

- **Interactive Console UI**: Terminal-style interface with command input and output display
- **Command Registry System**: Pluggable architecture for registering custom commands
- **History Management**: Command history with navigation (up/down arrows) and local storage persistence
- **Output Formatting**: Typed output messages (info, success, warning, error, command echo)
- **Keyboard Shortcuts**: Arrow key navigation, Enter to execute, Escape integration
- **Responsive Design**: Clean, monospace terminal aesthetic
- **Framework Agnostic**: Generic design allows for reuse across different projects

## Architecture

The library follows a modular architecture with clear separation of concerns:

```
dev-console/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── component.rs        # Main DevConsole Yew component
│   ├── command.rs          # Command trait and result types
│   ├── command_registry.rs # Command registration and execution
│   ├── history.rs          # Command history management
│   └── output.rs           # Output formatting and management
└── Cargo.toml
```

## Quick Start

### 1. Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
dev-console = { path = "../dev-console" }
```

### 2. Basic Integration

```rust
use yew::prelude::*;
use std::rc::Rc;
use dev_console::{DevConsole, ConsoleCommandRegistry};

#[function_component(App)]
fn app() -> Html {
    let registry = use_state(|| {
        let mut reg = ConsoleCommandRegistry::new();
        // Register your commands here
        Rc::new(reg)
    });
    
    let console_visible = use_state(|| false);
    
    let toggle_console = {
        let console_visible = console_visible.clone();
        Callback::from(move |_| {
            console_visible.set(!*console_visible);
        })
    };

    html! {
        <>
            <button onclick={toggle_console.clone()}>
                {"Toggle Console"}
            </button>
            
            <DevConsole
                registry={(*registry).clone()}
                visible={*console_visible}
                on_toggle={toggle_console}
            />
        </>
    }
}
```

## Command System

### Creating Custom Commands

Implement the `ConsoleCommand` trait for your commands:

```rust
use dev_console::{ConsoleCommand, ConsoleCommandResult, ConsoleOutput};

struct EchoCommand;

impl ConsoleCommand for EchoCommand {
    fn name(&self) -> &'static str {
        "echo"
    }
    
    fn description(&self) -> &'static str {
        "Echo the provided text"
    }
    
    fn execute(&self, args: &str) -> ConsoleCommandResult {
        if args.trim().is_empty() {
            ConsoleCommandResult::Output(
                ConsoleOutput::warning("Usage: echo <text>")
            )
        } else {
            ConsoleCommandResult::Output(
                ConsoleOutput::info(&format!("Echo: {}", args))
            )
        }
    }
}
```

### Registering Commands

```rust
use dev_console::ConsoleCommandRegistry;

let mut registry = ConsoleCommandRegistry::new();
registry.register(Box::new(EchoCommand));
registry.register(Box::new(MyCustomCommand));
```

### Built-in Commands

The console automatically includes several built-in commands:

- `help` - Lists all available commands with descriptions
- `clear` - Clears the console output
- `history` - Shows command history

## Advanced Examples

### Multi-Output Command

```rust
struct StatusCommand;

impl ConsoleCommand for StatusCommand {
    fn name(&self) -> &'static str {
        "status"
    }
    
    fn description(&self) -> &'static str {
        "Show system status"
    }
    
    fn execute(&self, _args: &str) -> ConsoleCommandResult {
        ConsoleCommandResult::MultipleOutputs(vec![
            ConsoleOutput::info("System Status Report"),
            ConsoleOutput::success("✓ Audio System: Online"),
            ConsoleOutput::success("✓ Graphics: Running"),
            ConsoleOutput::warning("⚠ Memory Usage: High"),
        ])
    }
}
```

### Clear and Replace Output

```rust
struct RefreshCommand;

impl ConsoleCommand for RefreshCommand {
    fn name(&self) -> &'static str {
        "refresh"
    }
    
    fn description(&self) -> &'static str {
        "Refresh the console display"
    }
    
    fn execute(&self, _args: &str) -> ConsoleCommandResult {
        ConsoleCommandResult::ClearAndOutput(
            ConsoleOutput::success("Console refreshed")
        )
    }
}
```

### Service Integration Command

For commands that need to interact with application services:

```rust
use std::rc::Rc;

struct DeviceListCommand {
    audio_service: Rc<dyn AudioService>,
}

impl DeviceListCommand {
    pub fn new(audio_service: Rc<dyn AudioService>) -> Self {
        Self { audio_service }
    }
}

impl ConsoleCommand for DeviceListCommand {
    fn name(&self) -> &'static str {
        "devices"
    }
    
    fn description(&self) -> &'static str {
        "List available audio devices"
    }
    
    fn execute(&self, _args: &str) -> ConsoleCommandResult {
        let devices = self.audio_service.get_devices();
        let outputs: Vec<_> = devices.iter()
            .map(|device| ConsoleOutput::info(&format!("📱 {}", device.name)))
            .collect();
        
        if outputs.is_empty() {
            ConsoleCommandResult::Output(
                ConsoleOutput::warning("No audio devices found")
            )
        } else {
            ConsoleCommandResult::MultipleOutputs(outputs)
        }
    }
}
```

## Output Types

The console supports several output message types:

```rust
use dev_console::ConsoleOutput;

// Different output types for visual distinction
ConsoleOutput::info("Informational message");     // Standard info color
ConsoleOutput::success("Operation completed");    // Green success color  
ConsoleOutput::warning("Potential issue");        // Yellow warning color
ConsoleOutput::error("Something went wrong");     // Red error color
ConsoleOutput::echo("user typed command");        // Command echo styling
```

## Styling

The console comes with built-in CSS styling that provides a terminal-like appearance. The styling is automatically injected and includes:

- Dark terminal theme with monospace fonts
- Color-coded output types (info, success, warning, error)
- Responsive design that works well as a modal overlay
- Focus management and proper input handling

### Customizing Appearance

The console can be styled by overriding CSS classes:

```css
.dev-console-modal {
    /* Override modal container */
    background: rgba(0, 20, 40, 0.95);
}

.dev-console-input {
    /* Override input styling */
    background: #001122;
    color: #00ff88;
}

.dev-console-message-info {
    /* Override info message color */
    color: #88ccff;
}
```

## History Management

The console automatically manages command history:

- **Navigation**: Use ↑/↓ arrow keys to navigate through previous commands
- **Persistence**: History is automatically saved to browser local storage
- **Restoration**: Previous session history is restored on console initialization

History is stored under the key `dev_console_history` in local storage.

## Integration Patterns

### Modal Console (Recommended)

```rust
// Full-screen modal overlay (default behavior)
<DevConsole
    registry={registry}
    visible={console_visible}
    on_toggle={toggle_callback}
/>
```

### Embedded Console

Override the modal CSS to embed the console:

```css
.dev-console-modal {
    position: static;
    background: transparent;
    /* Custom positioning */
}
```

### Global Keyboard Shortcuts

```rust
use web_sys::KeyboardEvent;

// Set up global Escape key to toggle console
useEffect(|| {
    let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        if event.key() == "Escape" {
            // Toggle console visibility
        }
    }) as Box<dyn FnMut(_)>);
    
    // Add event listener to document
    // Remember to clean up on component unmount
}, []);
```

## Testing

The library includes comprehensive tests for all components:

```bash
cd dev-console
cargo test
```

Test coverage includes:
- Command registration and execution
- History management functionality  
- Output formatting and message handling
- Component prop validation

## Dependencies

Minimal dependencies for maximum reusability:

- `yew` - React-like framework for WebAssembly
- `web-sys` - Web API bindings
- `serde` - Serialization for history persistence
- `js-sys` - JavaScript interop
- Standard Rust crates (`console_error_panic_hook`, `wasm-bindgen`, etc.)

## License

MIT License - See LICENSE file for details.

## Contributing

1. Follow the existing code style and patterns
2. Add tests for new functionality
3. Update documentation for API changes
4. Maintain the generic, reusable design philosophy

## Roadmap

Future enhancements planned:
- Command auto-completion
- Command aliases and shortcuts
- Configurable themes
- Plugin system for command modules
- WebSocket integration for remote commands