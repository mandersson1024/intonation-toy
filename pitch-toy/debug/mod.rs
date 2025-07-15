// Debug Module - Specialized debugging and monitoring components
//
// This module provides egui-based debug components:
// - EguiMicrophoneButton: Standalone microphone permission management
// - EguiLiveDataPanel: Real-time data visualization and monitoring

pub mod microphone_button;
pub mod egui;

use super::audio::AudioPermission;
