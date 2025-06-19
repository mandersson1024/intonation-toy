//! GUI module for the pitch visualizer
//! 
//! This module provides the graphical user interface using egui for controls
//! and wgpu for custom background rendering. It's designed to run at 60 FPS
//! with real-time audio visualization.

use anyhow::Result;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

pub mod app;
pub mod renderer;
pub mod widgets;

pub use app::PitchVisualizerApp;

/// GUI configuration constants
pub mod constants {
    /// Target frame rate for GUI updates
    pub const TARGET_FPS: u32 = 60;
    
    /// Default window width
    pub const WINDOW_WIDTH: u32 = 800;
    
    /// Default window height  
    pub const WINDOW_HEIGHT: u32 = 600;
    
    /// Minimum window width
    pub const MIN_WINDOW_WIDTH: u32 = 400;
    
    /// Minimum window height
    pub const MIN_WINDOW_HEIGHT: u32 = 300;
}

/// GUI-related errors
#[derive(Debug, thiserror::Error)]
pub enum GuiError {
    #[error("Window creation failed: {0}")]
    WindowCreationFailed(String),
    
    #[error("Graphics initialization failed: {0}")]
    GraphicsInitFailed(String),
    
    #[error("Rendering error: {0}")]
    RenderingError(String),
}

/// Create the main application window
pub fn create_window() -> Result<(winit::window::Window, EventLoop<()>)> {
    use constants::*;
    
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Pitch Visualizer")
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_min_inner_size(winit::dpi::LogicalSize::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT))
        .build(&event_loop)?;
    
    Ok((window, event_loop))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        use constants::*;
        
        assert_eq!(TARGET_FPS, 60);
        assert!(WINDOW_WIDTH >= MIN_WINDOW_WIDTH);
        assert!(WINDOW_HEIGHT >= MIN_WINDOW_HEIGHT);
    }
} 