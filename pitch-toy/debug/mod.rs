// Debug Module - Specialized debugging and monitoring components
//
// This module provides three specialized debug components:
// - DebugConsole: Command I/O interface for development
// - LivePanel: Real-time data visualization and monitoring
// - PermissionButton: Standalone microphone permission management
//
// These components replace the monolithic DevConsole with focused,
// reusable implementations following separation of concerns.

// pub mod console;
pub mod live_panel;
pub mod permission_button;
pub mod integration;

// Re-export main component types
// pub use console::DebugConsole;
pub use live_panel::LivePanel;
pub use permission_button::PermissionButton;
pub use integration::{DebugInterface, create_debug_interface};
