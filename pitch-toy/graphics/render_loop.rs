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
    // Frame timing metrics
    frame_times: Vec<f64>,
    max_frame_time: f64,
    min_frame_time: f64,
    avg_frame_time: f64,
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
            // Frame timing metrics
            frame_times: Vec::with_capacity(60), // Store last 60 frame times
            max_frame_time: 0.0,
            min_frame_time: f64::MAX,
            avg_frame_time: 0.0,
        })
    }

    pub fn initialize(&mut self, graphics_context: &GraphicsContext) -> Result<(), String> {
        self.renderer.initialize(graphics_context)?;
        self.last_frame_time = self.performance.now();
        
        web_sys::console::log_1(&"Render loop initialized".into());
        
        Ok(())
    }

    pub fn render_single_frame(&mut self, graphics_context: &GraphicsContext) -> Result<(), String> {
        let frame_start = self.performance.now();
        
        // Calculate frame timing
        let delta_time = frame_start - self.last_frame_time;
        self.last_frame_time = frame_start;
        
        // Update frame statistics
        self.frame_count += 1;
        self.fps_counter += delta_time;
        
        // Store frame timing for detailed metrics
        self.frame_times.push(delta_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        
        // Update min/max frame times
        self.max_frame_time = self.max_frame_time.max(delta_time);
        self.min_frame_time = self.min_frame_time.min(delta_time);
        
        // Calculate average frame time
        self.avg_frame_time = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
        
        // Log FPS every second with detailed metrics
        if self.fps_counter >= 1000.0 {
            let fps = self.frame_count as f64 / (self.fps_counter / 1000.0);
            
            // Check if we're meeting performance targets
            if fps < self.target_fps * 0.9 {
                web_sys::console::warn_1(&format!(
                    "Frame rate below target: {:.1} fps (avg: {:.2}ms, max: {:.2}ms)", 
                    fps, self.avg_frame_time, self.max_frame_time
                ).into());
            } else {
                web_sys::console::log_1(&format!(
                    "Performance: {:.1} fps (avg: {:.2}ms, min: {:.2}ms, max: {:.2}ms)", 
                    fps, self.avg_frame_time, self.min_frame_time, self.max_frame_time
                ).into());
            }
            
            self.frame_count = 0;
            self.fps_counter = 0.0;
            self.max_frame_time = 0.0;
            self.min_frame_time = f64::MAX;
        }

        // Check frame budget
        if delta_time > self.frame_budget_ms {
            web_sys::console::warn_1(&format!("Frame budget exceeded: {:.2}ms", delta_time).into());
        }

        // Render the frame
        self.renderer.render_frame(graphics_context)?;
        
        // Calculate total frame time including rendering
        let frame_end = self.performance.now();
        let total_frame_time = frame_end - frame_start;
        
        // Check total frame time against budget
        if total_frame_time > self.frame_budget_ms {
            web_sys::console::warn_1(&format!("Total frame time exceeded budget: {:.2}ms", total_frame_time).into());
        }
        
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
    
    pub fn get_avg_frame_time(&self) -> f64 {
        self.avg_frame_time
    }
    
    pub fn get_max_frame_time(&self) -> f64 {
        self.max_frame_time
    }
    
    pub fn get_min_frame_time(&self) -> f64 {
        self.min_frame_time
    }
    
    pub fn get_frame_budget_ms(&self) -> f64 {
        self.frame_budget_ms
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