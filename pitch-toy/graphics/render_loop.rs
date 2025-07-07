use wasm_bindgen::prelude::*;
use web_sys::{window, Performance};
use crate::graphics::{GraphicsContext, GraphicsRenderer};

pub struct RenderLoop {
    renderer: GraphicsRenderer,
    performance: Performance,
    last_frame_time: f64,
    frame_count: u32,
    fps_counter: f64,
    target_fps: f64,
    frame_budget_ms: f64,
    is_running: bool,
}

impl RenderLoop {
    pub fn new() -> Result<Self, JsValue> {
        let window = window().ok_or("No window object")?;
        let performance = window.performance().ok_or("No performance object")?;
        
        Ok(Self {
            renderer: GraphicsRenderer::new(),
            performance,
            last_frame_time: 0.0,
            frame_count: 0,
            fps_counter: 0.0,
            target_fps: 60.0,
            frame_budget_ms: 16.67, // 60fps = 16.67ms per frame
            is_running: false,
        })
    }

    pub fn initialize(&mut self, graphics_context: &GraphicsContext) -> Result<(), String> {
        self.renderer.initialize(graphics_context)?;
        self.last_frame_time = self.performance.now();
        
        web_sys::console::log_1(&"Render loop initialized".into());
        
        Ok(())
    }

    pub fn render_single_frame(&mut self, graphics_context: &GraphicsContext) -> Result<(), String> {
        let timestamp = self.performance.now();
        
        // Calculate frame timing
        let delta_time = timestamp - self.last_frame_time;
        self.last_frame_time = timestamp;
        
        // Update frame statistics
        self.frame_count += 1;
        self.fps_counter += delta_time;
        
        // Log FPS every second
        if self.fps_counter >= 1000.0 {
            let fps = self.frame_count as f64 / (self.fps_counter / 1000.0);
            
            // Check if we're meeting performance targets
            if fps < self.target_fps * 0.9 {
                web_sys::console::warn_1(&format!("Frame rate below target: {:.1} fps", fps).into());
            }
            
            self.frame_count = 0;
            self.fps_counter = 0.0;
        }

        // Check frame budget
        if delta_time > self.frame_budget_ms {
            web_sys::console::warn_1(&format!("Frame budget exceeded: {:.2}ms", delta_time).into());
        }

        // Render the frame
        self.renderer.render_frame(graphics_context)?;
        
        Ok(())
    }

    pub fn get_fps(&self) -> f64 {
        if self.fps_counter > 0.0 {
            self.frame_count as f64 / (self.fps_counter / 1000.0)
        } else {
            0.0
        }
    }

    pub fn get_frame_time(&self) -> f64 {
        self.last_frame_time
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

impl Default for RenderLoop {
    fn default() -> Self {
        Self::new().unwrap()
    }
}