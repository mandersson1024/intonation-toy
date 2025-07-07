use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use js_sys::Error;

#[derive(Clone)]
pub struct GraphicsContext {
    pub webgl_context: WebGl2RenderingContext,
    pub canvas: HtmlCanvasElement,
    pub width: u32,
    pub height: u32,
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
        // Fail-fast WebGL context detection
        let webgl_context = self.get_webgl_context(&canvas)?;
        
        let graphics_context = GraphicsContext {
            webgl_context,
            canvas: canvas.clone(),
            width: canvas.width(),
            height: canvas.height(),
        };

        self.graphics_context = Some(graphics_context);
        
        web_sys::console::log_1(&"✓ WebGL2 context initialized successfully with fail-fast validation".into());
        
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
        // Check for required extensions and features
        let required_extensions = vec![
            "EXT_color_buffer_float",
            "OES_texture_float_linear",
        ];

        for extension in required_extensions {
            if context.get_extension(extension)?.is_none() {
                web_sys::console::warn_1(&format!("Optional WebGL extension not available: {}", extension).into());
            }
        }

        // Validate context parameters
        let max_texture_size = context.get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)?;
        if max_texture_size.as_f64().unwrap_or(0.0) < 1024.0 {
            return Err(Error::new("WebGL max texture size too small").into());
        }

        Ok(())
    }

    pub fn handle_context_loss(&mut self) -> Result<(), JsValue> {
        // Clear current context
        self.graphics_context = None;
        
        // Log context loss
        web_sys::console::error_1(&"WebGL context lost, attempting recovery...".into());
        
        Ok(())
    }

    pub fn handle_context_restore(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        // Attempt to reinitialize context
        self.initialize_context(canvas)?;
        
        web_sys::console::log_1(&"WebGL context restored successfully".into());
        
        Ok(())
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}