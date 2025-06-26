// Component module exports
pub mod audio_control_panel;
pub mod audio_engine;
pub mod debug_interface;
pub mod debug_panel;
pub mod performance_monitor;
pub mod test_signal_generator;
pub mod error_display;
// pub mod audio_permission;  // Module doesn't exist yet
pub mod error_toast;
pub mod fallback_ui;
pub mod metrics_display;
pub mod microphone_panel;
pub mod microphone_permission;

// Re-export components for easy access
pub use audio_control_panel::AudioControlPanel;
pub use audio_engine::{AudioEngineComponent, use_audio_engine};
pub use debug_interface::DebugInterface;
pub use debug_panel::DebugPanel;
pub use performance_monitor::PerformanceMonitor;
pub use test_signal_generator::TestSignalGenerator;
pub use error_display::ErrorDisplayComponent;
// pub use audio_permission::AudioPermissionComponent;  // Module doesn't exist yet
pub use error_toast::{ErrorToastComponent, ErrorToastContainer};
pub use fallback_ui::FallbackUIComponent;
pub use metrics_display::MetricsDisplay;
pub use microphone_panel::MicrophonePanel;
pub use microphone_permission::MicrophonePermission; 