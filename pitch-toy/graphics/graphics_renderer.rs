use three_d::*;
use crate::graphics::GraphicsContext;

#[cfg(debug_assertions)]
use crate::graphics::TestScene;

pub struct GraphicsRenderer {
    initialized: bool,
    // 2D rendering components
    background_color: Srgba,
    // Test scene for demonstrating 2D coordinate system (debug only)
    #[cfg(debug_assertions)]
    test_scene: Option<TestScene>,
}

impl GraphicsRenderer {
    pub fn new() -> Self {
        Self {
            initialized: false,
            background_color: Srgba::new(0, 0, 0, 255), // Dark background
            #[cfg(debug_assertions)]
            test_scene: None,
        }
    }

    pub fn initialize(&mut self, #[cfg_attr(not(debug_assertions), allow(unused_variables))] graphics_context: &GraphicsContext) -> Result<(), String> {
        // Task 3: Initialize 2D graphics renderer
        
        // Initialize test scene to demonstrate 2D coordinate system (debug only)
        #[cfg(debug_assertions)]
        {
            let gl = &graphics_context.webgl_context;
            let mut test_scene = TestScene::new();
            test_scene.initialize(gl)?;
            self.test_scene = Some(test_scene);
            web_sys::console::log_1(&"✓ 2D Graphics renderer initialized with debug test scene".into());
        }
        
        #[cfg(not(debug_assertions))]
        {
            web_sys::console::log_1(&"✓ 2D Graphics renderer initialized (production - no test scene)".into());
        }
        
        self.initialized = true;
        
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

        // Render test scene
        #[cfg(debug_assertions)]
        self.render_test_scene(gl)?;

        Ok(())
    }

    #[cfg(debug_assertions)]
    fn render_test_scene(&self, gl: &web_sys::WebGl2RenderingContext) -> Result<(), String> {
        // Enable depth testing for proper 2D layering
        gl.enable(web_sys::WebGl2RenderingContext::DEPTH_TEST);
        
        // Render test scene if available (debug only)
        if let Some(test_scene) = &self.test_scene {
            test_scene.render(gl)?;
        }
        
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
}

impl Default for GraphicsRenderer {
    fn default() -> Self {
        Self::new()
    }
}