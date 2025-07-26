// EGUI Debug Interface Module
// Handles the three-d + egui rendering for debug components
// 
// Note: Microphone permission is now handled directly from lib.rs
// Debug controls use the new presenter action collection system
// All debug functionality is only available in debug builds

pub(crate) mod overlay;
pub(crate) mod hybrid_live_data_panel;
pub(crate) mod data_types;

pub(crate) use overlay::EguiDebugControls;
pub(crate) use hybrid_live_data_panel::HybridEguiLiveDataPanel;