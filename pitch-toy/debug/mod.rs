// Debug Module - Specialized debugging and monitoring components
//
// This module provides egui-based debug components:
// - EguiMicrophoneButton: Standalone microphone permission management
// - HybridEguiLiveDataPanel: Real-time data visualization and monitoring

#[cfg(debug_assertions)]
pub mod debug_panel;
#[cfg(debug_assertions)]
pub mod debug_data;
