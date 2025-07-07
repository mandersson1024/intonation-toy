use crate::graphics::GraphicsContext;
use web_sys::WebGl2RenderingContext;

pub struct GraphicsRenderer {
    initialized: bool,
}

impl GraphicsRenderer {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    pub fn initialize(&mut self, _graphics_context: &GraphicsContext) -> Result<(), String> {
        // Initialize renderer with graphics context
        self.initialized = true;
        
        web_sys::console::log_1(&"Graphics renderer initialized".into());
        
        Ok(())
    }

    pub fn render_frame(&self, graphics_context: &GraphicsContext) -> Result<(), String> {
        if !self.initialized {
            return Err("Graphics renderer not initialized".to_string());
        }

        // Clear the screen with a dark background using WebGL2
        let gl = &graphics_context.webgl_context;
        
        // Set viewport
        gl.viewport(0, 0, graphics_context.width as i32, graphics_context.height as i32);
        
        // Clear color and depth buffers
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        // Basic render pipeline - render a simple scene
        self.render_basic_scene(graphics_context)?;

        Ok(())
    }

    fn render_basic_scene(&self, _graphics_context: &GraphicsContext) -> Result<(), String> {
        // For now, just clear the screen with a dark background
        // This will be expanded with actual 3D scene rendering in future tasks
        
        // TODO: Add basic 3D scene with camera, lighting, and coordinate system
        // TODO: Add shader pipeline for future audio visualization
        
        Ok(())
    }

    pub fn resize(&mut self, graphics_context: &mut GraphicsContext, width: u32, height: u32) -> Result<(), String> {
        // Update context dimensions
        graphics_context.width = width;
        graphics_context.height = height;
        
        // Update canvas size
        graphics_context.canvas.set_width(width);
        graphics_context.canvas.set_height(height);
        
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for GraphicsRenderer {
    fn default() -> Self {
        Self::new()
    }
}