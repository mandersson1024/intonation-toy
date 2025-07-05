// Permission Button Module - Standalone microphone permission management
//
// Provides standalone microphone permission management with:
// - Microphone permission request handling
// - Permission state visualization
// - User gesture context preservation
// - Error state display

mod component;

pub use component::{PermissionButton, PermissionButtonProps, PermissionButtonMsg, AudioPermissionService};