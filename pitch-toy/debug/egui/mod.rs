// EGUI Debug Interface Module
// Handles the three-d + egui rendering for debug components

pub mod overlay;
pub mod live_data_panel;
pub mod hybrid_live_data_panel;

pub use overlay::EguiMicrophoneButton;
pub use live_data_panel::EguiLiveDataPanel;
pub use hybrid_live_data_panel::HybridEguiLiveDataPanel;