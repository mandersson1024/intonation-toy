use three_d::*;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use js_sys::Error;

pub struct GraphicsContext {
    pub canvas: HtmlCanvasElement,
    pub camera: Camera,
    pub viewport: Viewport,
    pub webgl_context: WebGl2RenderingContext,
}

pub struct ContextManager {
    graphics_context: Option<GraphicsContext>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            graphics_context: None,
        }
    }

    pub fn initialize_context(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        // Task 2: Enhanced WebGL context detection with three-d camera and viewport setup
        let webgl_context = self.get_webgl_context(&canvas)?;
        
        // Set up viewport  
        let width = canvas.width();
        let height = canvas.height();
        let viewport = Viewport::new_at_origo(width, height);
        
        // Set up three-d camera with proper perspective
        let camera = Camera::new_perspective(
            viewport,
            vec3(0.0, 0.0, 5.0),     // Eye position
            vec3(0.0, 0.0, 0.0),     // Target
            vec3(0.0, 1.0, 0.0),     // Up vector
            degrees(45.0),           // Field of view
            0.1,                     // Near plane
            100.0,                   // Far plane
        );

        let graphics_context = GraphicsContext {
            canvas: canvas.clone(),
            camera,
            viewport,
            webgl_context,
        };

        // Add canvas event listeners for context loss/restore
        self.add_context_event_listeners(&canvas)?;
        
        self.graphics_context = Some(graphics_context);
        
        web_sys::console::log_1(&"✓ three-d context with camera and viewport initialized successfully".into());
        
        Ok(())
    }
    
    fn add_context_event_listeners(&self, canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        
        // Add webglcontextlost event listener
        let context_lost_callback = Closure::wrap(Box::new(move |event: web_sys::Event| {
            web_sys::console::warn_1(&"WebGL context lost!".into());
            event.prevent_default();
        }) as Box<dyn FnMut(_)>);
        
        canvas.add_event_listener_with_callback(
            "webglcontextlost", 
            context_lost_callback.as_ref().unchecked_ref()
        )?;
        context_lost_callback.forget();
        
        // Add webglcontextrestored event listener  
        let context_restored_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            web_sys::console::log_1(&"WebGL context restored!".into());
        }) as Box<dyn FnMut(_)>);
        
        canvas.add_event_listener_with_callback(
            "webglcontextrestored",
            context_restored_callback.as_ref().unchecked_ref()
        )?;
        context_restored_callback.forget();
        
        Ok(())
    }

    pub fn get_context(&self) -> Option<&GraphicsContext> {
        self.graphics_context.as_ref()
    }

    pub fn get_context_mut(&mut self) -> Option<&mut GraphicsContext> {
        self.graphics_context.as_mut()
    }

    fn get_webgl_context(&self, canvas: &HtmlCanvasElement) -> Result<WebGl2RenderingContext, JsValue> {
        // Try to get WebGL2 context first
        let context = canvas
            .get_context("webgl2")?
            .ok_or_else(|| Error::new("WebGL2 not supported"))?
            .dyn_into::<WebGl2RenderingContext>()?;

        // Validate WebGL context
        if context.is_context_lost() {
            return Err(Error::new("WebGL context is lost").into());
        }

        // Check for essential WebGL features
        self.validate_webgl_features(&context)?;

        Ok(context)
    }

    fn validate_webgl_features(&self, context: &WebGl2RenderingContext) -> Result<(), JsValue> {
        // Task 2: Enhanced WebGL feature validation following fail-fast policy
        
        // Check critical WebGL parameters
        let max_texture_size = context.get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)?;
        if max_texture_size.as_f64().unwrap_or(0.0) < 1024.0 {
            return Err(Error::new("CRITICAL: WebGL max texture size too small (< 1024)").into());
        }
        
        let max_renderbuffer_size = context.get_parameter(WebGl2RenderingContext::MAX_RENDERBUFFER_SIZE)?;
        if max_renderbuffer_size.as_f64().unwrap_or(0.0) < 1024.0 {
            return Err(Error::new("CRITICAL: WebGL max renderbuffer size too small").into());
        }
        
        // Check for essential WebGL 2.0 features
        let max_vertex_attribs = context.get_parameter(WebGl2RenderingContext::MAX_VERTEX_ATTRIBS)?;
        if max_vertex_attribs.as_f64().unwrap_or(0.0) < 8.0 {
            return Err(Error::new("CRITICAL: Insufficient vertex attributes support").into());
        }
        
        // Check for helpful extensions (warn if missing but don't fail)
        let beneficial_extensions = vec![
            "EXT_color_buffer_float",
            "OES_texture_float_linear", 
            "WEBGL_depth_texture",
            "EXT_texture_filter_anisotropic",
        ];

        for extension in beneficial_extensions {
            if context.get_extension(extension)?.is_none() {
                web_sys::console::warn_1(&format!("Beneficial WebGL extension not available: {}", extension).into());
            } else {
                web_sys::console::log_1(&format!("✓ WebGL extension available: {}", extension).into());
            }
        }
        
        // Validate context state
        if context.is_context_lost() {
            return Err(Error::new("CRITICAL: WebGL context is in lost state").into());
        }
        
        web_sys::console::log_1(&"✓ WebGL feature validation passed".into());
        Ok(())
    }

    pub fn handle_context_loss(&mut self) -> Result<(), JsValue> {
        // Task 2: Enhanced context loss handling
        web_sys::console::error_1(&"⚠️ WebGL context lost detected".into());
        
        // Clear current context
        self.graphics_context = None;
        
        // Log loss details for debugging
        web_sys::console::error_1(&"Graphics context cleared due to WebGL context loss".into());
        web_sys::console::error_1(&"Waiting for context restore event...".into());
        
        Ok(())
    }

    pub fn handle_context_restore(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        // Task 2: Enhanced context restoration with validation
        web_sys::console::log_1(&"🔄 Attempting WebGL context restoration...".into());
        
        // Attempt to reinitialize context with full validation
        match self.initialize_context(canvas) {
            Ok(_) => {
                web_sys::console::log_1(&"✅ WebGL context restored successfully".into());
                Ok(())
            }
            Err(e) => {
                web_sys::console::error_1(&"❌ Failed to restore WebGL context".into());
                web_sys::console::error_1(&format!("Restoration error: {:?}", e).into());
                Err(e)
            }
        }
    }
    
    pub fn is_context_valid(&self) -> bool {
        if let Some(ref context) = self.graphics_context {
            !context.webgl_context.is_context_lost()
        } else {
            false
        }
    }
    
    pub fn force_context_validation(&self) -> Result<(), JsValue> {
        if let Some(ref context) = self.graphics_context {
            if context.webgl_context.is_context_lost() {
                return Err(Error::new("WebGL context is lost").into());
            }
            self.validate_webgl_features(&context.webgl_context)?;
            Ok(())
        } else {
            Err(Error::new("No graphics context available").into())
        }
    }
    
    pub fn resize_context(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        if let Some(ref mut context) = self.graphics_context {
            // Update viewport
            context.viewport = Viewport::new_at_origo(width, height);
            
            // Update camera viewport
            context.camera.set_viewport(context.viewport);
            
            // Update canvas size
            context.canvas.set_width(width);
            context.canvas.set_height(height);
            
            web_sys::console::log_1(&format!("✓ Graphics context resized to {}x{}", width, height).into());
            Ok(())
        } else {
            Err(Error::new("No graphics context available for resize").into())
        }
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}