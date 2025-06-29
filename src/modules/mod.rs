// Module system for pitch-toy application
// Provides structured organization for audio processing, graphics, and development tools

pub mod common;

#[cfg(debug_assertions)]
pub mod debug;

#[cfg(debug_assertions)]
pub mod console;

// Future modules for upcoming stories (following YAGNI principle):
// TODO: audio/audio_processor.rs - implement when audio processing is needed
// TODO: events/event_dispatcher.rs - implement when event system is needed
// TODO: graphics/graphics_renderer.rs - implement when wgpu rendering is needed
// TODO: presentation/presentation_layer.rs - implement when visualization is needed
// TODO: themes/theme_manager.rs - implement when theming is needed