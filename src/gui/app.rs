use anyhow::Result;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use crate::bridge::{MessageBus, GuiHandle, AudioHandle, AudioMessage};
use crate::audio::AudioEngine;
use super::renderer::Renderer;

/// Main application state and coordinator
pub struct PitchVisualizerApp {
    /// Window and event loop
    window: Window,
    event_loop: Option<EventLoop<()>>,
    
    /// Graphics renderer (wgpu + egui)
    renderer: Renderer,
    
    /// Communication with audio thread
    gui_handle: GuiHandle,
    
    /// Audio thread handle
    audio_thread: Option<thread::JoinHandle<()>>,
    
    /// Application state
    state: AppState,
}

/// Application state for the GUI
#[derive(Debug)]
pub struct AppState {
    /// Latest audio analysis result
    pub current_audio: Option<AudioMessage>,
    
    /// Reference frequency setting
    pub reference_frequency: f32,
    
    /// Whether audio processing is enabled
    pub audio_enabled: bool,
    
    /// User interface state
    pub ui_state: UiState,
}

/// UI-specific state
#[derive(Debug)]
pub struct UiState {
    /// Show debug information
    pub show_debug: bool,
    
    /// Reference frequency input text
    pub reference_input: String,
    
    /// Selected reference note
    pub reference_note: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_audio: None,
            reference_frequency: crate::audio::constants::A4_FREQUENCY,
            audio_enabled: true,
            ui_state: UiState {
                show_debug: false,
                reference_input: "440.0".to_string(),
                reference_note: "A4".to_string(),
            },
        }
    }
}

impl PitchVisualizerApp {
    /// Create a new pitch visualizer application
    pub fn new() -> Result<Self> {
        // Create window and event loop
        let (window, event_loop) = super::create_window()?;
        let window = Arc::new(window);
        
        // Initialize graphics renderer
        let renderer = Renderer::new(window.clone())?;
        
        // Create message bus for audio-GUI communication
        let message_bus = MessageBus::new();
        let (audio_handle, gui_handle) = message_bus.split();
        
        // Start audio processing thread
        let audio_thread = Self::start_audio_thread(audio_handle)?;
        
        Ok(Self {
            window: Arc::try_unwrap(window).map_err(|_| anyhow::anyhow!("Failed to unwrap window"))?,
            event_loop: Some(event_loop),
            renderer,
            gui_handle,
            audio_thread: Some(audio_thread),
            state: AppState::default(),
        })
    }
    
    /// Start the audio processing thread
    fn start_audio_thread(audio_handle: AudioHandle) -> Result<thread::JoinHandle<()>> {
        let handle = thread::Builder::new()
            .name("audio_thread".to_string())
            .spawn(move || {
                if let Err(e) = Self::run_audio_thread(audio_handle) {
                    log::error!("Audio thread error: {}", e);
                }
            })?;
        
        Ok(handle)
    }
    
    /// Run the audio processing thread
    fn run_audio_thread(audio_handle: AudioHandle) -> Result<()> {
        let mut audio_engine = AudioEngine::new()?;
        
        // Main audio processing loop
        loop {
            // Process control messages from GUI
            audio_handle.process_control_messages(|message| {
                if let Err(e) = audio_engine.handle_control_message(message) {
                    log::warn!("Failed to handle control message: {}", e);
                }
            });
            
            // Process audio and send results to GUI
            if let Some(result) = audio_engine.process_audio()? {
                if let Err(e) = audio_handle.send_audio_result(result) {
                    log::warn!("Failed to send audio result: {}", e);
                }
            }
            
            // Small sleep to prevent busy waiting
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    
    /// Run the main application loop
    pub fn run(mut self) -> Result<()> {
        let event_loop = self.event_loop.take()
            .ok_or_else(|| anyhow::anyhow!("Event loop already consumed"))?;
        
        let mut last_frame = Instant::now();
        
        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            
            match event {
                Event::WindowEvent { event, .. } => {
                    // Handle window events
                    match event {
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            self.renderer.resize(physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            // Update application state
                            self.update();
                            
                            // Render frame
                            match self.renderer.render(&self.state) {
                                Ok(_) => {}
                                Err(e) => {
                                    log::error!("Render error: {}", e);
                                }
                            }
                        }
                        _ => {}
                    }
                    
                    // Forward events to renderer for egui handling
                    self.renderer.handle_event(&event);
                }
                Event::AboutToWait => {
                    // Request redraw at target framerate
                    let now = Instant::now();
                    let frame_time = std::time::Duration::from_secs_f32(1.0 / 60.0);
                    
                    if now.duration_since(last_frame) >= frame_time {
                        self.window.request_redraw();
                        last_frame = now;
                    }
                }
                _ => {}
            }
        })?;
        
        Ok(())
    }
    
    /// Update application state
    fn update(&mut self) {
        // Get latest audio analysis result
        if let Some(audio_result) = self.gui_handle.get_latest_audio_result() {
            self.state.current_audio = Some(audio_result);
        }
        
        // Handle UI state changes
        if self.state.ui_state.reference_input != self.state.reference_frequency.to_string() {
            if let Ok(freq) = self.state.ui_state.reference_input.parse::<f32>() {
                if freq > 0.0 && freq < 20000.0 {
                    self.state.reference_frequency = freq;
                    let _ = self.gui_handle.set_reference_frequency(freq);
                }
            }
        }
    }
}

impl Drop for PitchVisualizerApp {
    fn drop(&mut self) {
        // Properly shutdown audio thread
        if let Some(handle) = self.audio_thread.take() {
            // Audio thread will stop when the handle is dropped
            let _ = handle.join();
        }
    }
} 