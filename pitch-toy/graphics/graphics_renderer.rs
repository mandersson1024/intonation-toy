use three_d::*;
use crate::graphics::GraphicsContext;

pub struct GraphicsRenderer {
    initialized: bool,
    // 2D rendering components
    background_color: Srgba,
    // 2D shader pipeline placeholder (for future implementation)
    shader_pipeline_ready: bool,
}

impl GraphicsRenderer {
    pub fn new() -> Self {
        Self {
            initialized: false,
            background_color: Srgba::new(0, 0, 0, 255), // Dark background
            shader_pipeline_ready: false,
        }
    }

    pub fn initialize(&mut self, _graphics_context: &GraphicsContext) -> Result<(), String> {
        // Task 3: Initialize 2D graphics renderer 
        // For now, we'll work with the WebGL2 context directly
        // TODO: Integrate three-d Context creation when the API is clearer
        
        // Prepare 2D shader pipeline for future implementation
        self.shader_pipeline_ready = true;
        self.initialized = true;
        
        web_sys::console::log_1(&"✓ 2D Graphics renderer initialized with shader pipeline ready".into());
        
        Ok(())
    }

    pub fn render_frame(&self, graphics_context: &GraphicsContext) -> Result<(), String> {
        if !self.initialized {
            return Err("Graphics renderer not initialized".to_string());
        }

        // Task 3: 2D render pipeline using WebGL2 directly
        let gl = &graphics_context.webgl_context;
        
        // Set viewport using three-d viewport structure
        gl.viewport(0, 0, graphics_context.viewport.width as i32, graphics_context.viewport.height as i32);
        
        // Clear color and depth buffers with dark background
        gl.clear_color(
            self.background_color.r as f32 / 255.0,
            self.background_color.g as f32 / 255.0,
            self.background_color.b as f32 / 255.0,
            1.0,
        );
        gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT | web_sys::WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        // Render basic 2D scene
        self.render_basic_2d_scene(graphics_context)?;

        Ok(())
    }

    fn render_basic_2d_scene(&self, _graphics_context: &GraphicsContext) -> Result<(), String> {
        // Task 3: Basic 2D scene with coordinate system (2D mode only)
        // For now, just clear the screen with a dark background
        // This establishes the 2D coordinate system and render pipeline
        
        // TODO: Basic 2D shader pipeline for future audio visualization:
        // - Vertex shader for 2D position and UV mapping
        // - Fragment shader with uniforms for time, resolution, and audio data
        // - Shader program compilation and linking
        // - GPU buffer management for 2D geometries
        
        Ok(())
    }

    pub fn resize(&mut self, graphics_context: &mut GraphicsContext, width: u32, height: u32) -> Result<(), String> {
        // Update viewport
        graphics_context.viewport = Viewport::new_at_origo(width, height);
        
        // Update camera viewport
        graphics_context.camera.set_viewport(graphics_context.viewport);
        
        // Update canvas size
        graphics_context.canvas.set_width(width);
        graphics_context.canvas.set_height(height);
        
        // TODO: Update render target size when three-d integration is complete
        
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    pub fn is_shader_pipeline_ready(&self) -> bool {
        self.shader_pipeline_ready
    }
}

impl Default for GraphicsRenderer {
    fn default() -> Self {
        Self::new()
    }
}