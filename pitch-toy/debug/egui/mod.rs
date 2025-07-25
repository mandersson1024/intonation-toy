// EGUI Debug Interface Module
// Handles the three-d + egui rendering for debug components
// 
// Note: Microphone permission is now handled directly from lib.rs
// Debug controls use the new presenter action collection system
// All debug functionality is only available in debug builds

pub mod overlay;
pub mod hybrid_live_data_panel;
pub mod data_types;

pub use overlay::EguiDebugControls;
pub use hybrid_live_data_panel::HybridEguiLiveDataPanel;