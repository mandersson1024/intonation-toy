## Overview

The Console Module implements a terminal-style debugging interface rendered as a modal overlay using Yew components. It provides:

- **Interactive Command System**: Extensible command framework with built-in development commands
- **Persistent Command History**: Browser storage-backed history with navigation support  
- **Real-time System Control**: Runtime configuration and debugging capabilities

## Public API

The console module provides a **minimal, clean API** that encapsulates all internal complexity:

```rust
pub use component::DevConsole;
pub use commands::{ConsoleCommand, ConsoleCommandResult};
pub use command_registry::register_command;

// In your Yew application root
#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div>
            <DevConsole />
        </div>
    }
}

// Define a custom command
struct MyCustomCommand;

impl ConsoleCommand for MyCustomCommand {
    fn name(&self) -> &str {
        "mycmd"
    }
    
    fn description(&self) -> &str {
        "My custom development command"
    }
    
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        ConsoleCommandResult::Output(ConsoleOutput::success("Custom command executed!"))
    }
}

// Register the command
register_command(Box::new(MyCustomCommand));
```

## User Interaction

- **Toggle Visibility**: Press `ESC` key to show/hide console
- **Command Entry**: Type commands in the input field and press `Enter`
- **History Navigation**: Use `↑`/`↓` arrow keys to navigate command history
- **Auto-focus**: Console automatically focuses input when shown

## Architecture

### System Architecture

```
┌─────────────────────────────────────────┐
│             Dev Console                 │
│              (Yew UI)                   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│           Command Registry              │
│         (DevCommand Trait)              │
└─────────────────┬───────────────────────┘
                  │
        ┌─────────┼─────────┐
        │         │         │
        ▼         ▼         ▼
┌─────────┐ ┌──────────┐ ┌─────────┐
│ Built-in│ │  Custom  │ │ Future  │
│Commands │ │ Commands │ │Commands │
└─────────┘ └──────────┘ └─────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│           Output System                 │
│      (Typed Results & Styling)          │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│         Command History                 │
│       (Persistent Storage)              │
└─────────────────────────────────────────┘
```

### Component Interaction Flow

```
User Input (ESC) → Console Toggle → Command Entry
                                        │
                                        ▼
                              Command Parsing
                                        │
                                        ▼
                              Registry Lookup
                                        │
                                        ▼
                              Command Execution
                                        │
                                        ▼
                              Output Generation
                                        │
                                        ▼
                          History Storage & Display
```
