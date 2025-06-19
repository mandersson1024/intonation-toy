use anyhow::Result;
use log::info;

mod audio;
mod gui;
mod bridge;

use gui::PitchVisualizerApp;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting Pitch Visualizer");

    // Create and run the application
    let app = PitchVisualizerApp::new()?;
    app.run()
} 