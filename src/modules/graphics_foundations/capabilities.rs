use super::GraphicsError;
use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext, WebGlRenderingContext};
use wasm_bindgen::JsCast;

/// Graphics performance tiers for capability assessment
#[derive(Debug, Clone, PartialEq)]
pub enum GraphicsPerformanceTier {
    High,    // Modern discrete GPU, full feature support
    Medium,  // Integrated GPU, good performance
    Low,     // Basic graphics, limited features
    Unknown, // Cannot determine performance tier
}

/// Supported texture formats for graphics operations
#[derive(Debug, Clone, PartialEq)]
pub enum TextureFormat {
    Rgba8,
    Rgba16Float,
    Depth24Plus,
    Bgra8,
}

/// Graphics capabilities detection results
#[derive(Debug, Clone)]
pub struct GraphicsCapabilities {
    pub webgl_supported: bool,
    pub webgl2_supported: bool,
    pub webgpu_supported: bool,
    pub max_texture_size: u32,
    pub supported_formats: Vec<TextureFormat>,
    pub performance_tier: GraphicsPerformanceTier,
    pub vendor: Option<String>,
    pub renderer: Option<String>,
}

impl Default for GraphicsCapabilities {
    fn default() -> Self {
        Self {
            webgl_supported: false,
            webgl2_supported: false,
            webgpu_supported: false,
            max_texture_size: 0,
            supported_formats: Vec::new(),
            performance_tier: GraphicsPerformanceTier::Unknown,
            vendor: None,
            renderer: None,
        }
    }
}

impl GraphicsCapabilities {
    /// Detect graphics capabilities for the current browser environment
    pub fn detect() -> Result<Self, GraphicsError> {
        let mut capabilities = Self::default();
        
        // Detect WebGL support
        let (webgl_supported, webgl2_supported) = Self::detect_webgl_support()?;
        capabilities.webgl_supported = webgl_supported;
        capabilities.webgl2_supported = webgl2_supported;
        
        // Detect WebGPU support
        capabilities.webgpu_supported = Self::detect_webgpu_support();
        
        // Get WebGL context for detailed capability detection
        if webgl_supported || webgl2_supported {
            if let Ok((max_texture_size, vendor, renderer)) = Self::get_webgl_details(webgl2_supported) {
                capabilities.max_texture_size = max_texture_size;
                capabilities.vendor = vendor;
                capabilities.renderer = renderer;
            }
        }
        
        // Determine supported texture formats
        capabilities.supported_formats = Self::detect_supported_formats(&capabilities);
        
        // Assess performance tier
        capabilities.performance_tier = Self::assess_performance_tier(&capabilities);
        
        Ok(capabilities)
    }
    
    /// Detect WebGL and WebGL2 support
    fn detect_webgl_support() -> Result<(bool, bool), GraphicsError> {
        let window = window().ok_or_else(|| {
            GraphicsError::CapabilityDetectionFailed("Window object not available".to_string())
        })?;
        
        let document = window.document().ok_or_else(|| {
            GraphicsError::CapabilityDetectionFailed("Document object not available".to_string())
        })?;
        
        // Create a test canvas
        let canvas = document
            .create_element("canvas")
            .map_err(|_| GraphicsError::CapabilityDetectionFailed("Cannot create test canvas".to_string()))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| GraphicsError::CapabilityDetectionFailed("Cannot cast to canvas element".to_string()))?;
        
        // Test WebGL support
        let webgl_supported = canvas
            .get_context("webgl")
            .ok()
            .flatten()
            .and_then(|ctx| ctx.dyn_into::<WebGlRenderingContext>().ok())
            .is_some();
        
        // Test WebGL2 support
        let webgl2_supported = canvas
            .get_context("webgl2")
            .ok()
            .flatten()
            .and_then(|ctx| ctx.dyn_into::<WebGl2RenderingContext>().ok())
            .is_some();
        
        Ok((webgl_supported, webgl2_supported))
    }
    
    /// Detect WebGPU support (future capability)
    fn detect_webgpu_support() -> bool {
        // WebGPU is not yet widely supported in browsers
        // This is a placeholder for future WebGPU detection
        false
    }
    
    /// Get detailed WebGL information
    fn get_webgl_details(prefer_webgl2: bool) -> Result<(u32, Option<String>, Option<String>), GraphicsError> {
        let window = window().ok_or_else(|| {
            GraphicsError::CapabilityDetectionFailed("Window object not available".to_string())
        })?;
        
        let document = window.document().ok_or_else(|| {
            GraphicsError::CapabilityDetectionFailed("Document object not available".to_string())
        })?;
        
        let canvas = document
            .create_element("canvas")
            .map_err(|_| GraphicsError::CapabilityDetectionFailed("Cannot create test canvas".to_string()))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| GraphicsError::CapabilityDetectionFailed("Cannot cast to canvas element".to_string()))?;
        
        let context_name = if prefer_webgl2 { "webgl2" } else { "webgl" };
        
        if prefer_webgl2 {
            if let Ok(Some(context)) = canvas.get_context(context_name) {
                if let Ok(gl) = context.dyn_into::<WebGl2RenderingContext>() {
                    return Self::extract_webgl_details_webgl2(&gl);
                }
            }
        }
        
        // Fallback to WebGL 1.0
        if let Ok(Some(context)) = canvas.get_context("webgl") {
            if let Ok(gl) = context.dyn_into::<WebGlRenderingContext>() {
                return Self::extract_webgl_details_webgl1(&gl);
            }
        }
        
        Err(GraphicsError::CapabilityDetectionFailed("Cannot get WebGL context".to_string()))
    }
    
    /// Extract WebGL details from WebGL2 context
    fn extract_webgl_details_webgl2(gl: &WebGl2RenderingContext) -> Result<(u32, Option<String>, Option<String>), GraphicsError> {
        let max_texture_size = gl.get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)
            .ok()
            .and_then(|val| val.as_f64())
            .unwrap_or(0.0) as u32;
        
        let vendor = gl.get_parameter(WebGl2RenderingContext::VENDOR)
            .ok()
            .and_then(|val| val.as_string());
        
        let renderer = gl.get_parameter(WebGl2RenderingContext::RENDERER)
            .ok()
            .and_then(|val| val.as_string());
        
        Ok((max_texture_size, vendor, renderer))
    }
    
    /// Extract WebGL details from WebGL 1.0 context
    fn extract_webgl_details_webgl1(gl: &WebGlRenderingContext) -> Result<(u32, Option<String>, Option<String>), GraphicsError> {
        let max_texture_size = gl.get_parameter(WebGlRenderingContext::MAX_TEXTURE_SIZE)
            .ok()
            .and_then(|val| val.as_f64())
            .unwrap_or(0.0) as u32;
        
        let vendor = gl.get_parameter(WebGlRenderingContext::VENDOR)
            .ok()
            .and_then(|val| val.as_string());
        
        let renderer = gl.get_parameter(WebGlRenderingContext::RENDERER)
            .ok()
            .and_then(|val| val.as_string());
        
        Ok((max_texture_size, vendor, renderer))
    }
    
    /// Detect supported texture formats based on capabilities
    fn detect_supported_formats(capabilities: &GraphicsCapabilities) -> Vec<TextureFormat> {
        let mut formats = Vec::new();
        
        // Basic formats supported by WebGL
        if capabilities.webgl_supported {
            formats.push(TextureFormat::Rgba8);
            formats.push(TextureFormat::Depth24Plus);
        }
        
        // Additional formats for WebGL2
        if capabilities.webgl2_supported {
            formats.push(TextureFormat::Rgba16Float);
        }
        
        // WebGPU formats (future)
        if capabilities.webgpu_supported {
            formats.push(TextureFormat::Bgra8);
        }
        
        formats
    }
    
    /// Assess performance tier based on capabilities
    fn assess_performance_tier(capabilities: &GraphicsCapabilities) -> GraphicsPerformanceTier {
        // Basic performance tier assessment
        if capabilities.webgpu_supported {
            return GraphicsPerformanceTier::High;
        }
        
        if capabilities.webgl2_supported && capabilities.max_texture_size >= 4096 {
            return GraphicsPerformanceTier::Medium;
        }
        
        if capabilities.webgl_supported {
            return GraphicsPerformanceTier::Low;
        }
        
        GraphicsPerformanceTier::Unknown
    }
    
    /// Check if the current environment meets minimum graphics requirements
    pub fn meets_minimum_requirements(&self) -> bool {
        self.webgl_supported && self.max_texture_size >= 512
    }
    
    /// Get a human-readable description of graphics capabilities
    pub fn get_description(&self) -> String {
        let mut parts = Vec::new();
        
        if self.webgpu_supported {
            parts.push("WebGPU");
        }
        if self.webgl2_supported {
            parts.push("WebGL 2.0");
        } else if self.webgl_supported {
            parts.push("WebGL 1.0");
        }
        
        if parts.is_empty() {
            return "No graphics support detected".to_string();
        }
        
        let support_desc = parts.join(", ");
        let perf_desc = match self.performance_tier {
            GraphicsPerformanceTier::High => "High Performance",
            GraphicsPerformanceTier::Medium => "Medium Performance", 
            GraphicsPerformanceTier::Low => "Basic Performance",
            GraphicsPerformanceTier::Unknown => "Unknown Performance",
        };
        
        format!("{} ({})", support_desc, perf_desc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_capabilities() {
        let caps = GraphicsCapabilities::default();
        assert!(!caps.webgl_supported);
        assert!(!caps.webgl2_supported);
        assert!(!caps.webgpu_supported);
        assert_eq!(caps.max_texture_size, 0);
        assert!(caps.supported_formats.is_empty());
        assert_eq!(caps.performance_tier, GraphicsPerformanceTier::Unknown);
    }
    
    #[test]
    fn test_minimum_requirements() {
        let mut caps = GraphicsCapabilities::default();
        assert!(!caps.meets_minimum_requirements());
        
        caps.webgl_supported = true;
        caps.max_texture_size = 1024;
        assert!(caps.meets_minimum_requirements());
    }
    
    #[test]
    fn test_performance_tier_assessment() {
        let caps = GraphicsCapabilities {
            webgpu_supported: true,
            ..Default::default()
        };
        assert_eq!(GraphicsCapabilities::assess_performance_tier(&caps), GraphicsPerformanceTier::High);
        
        let caps = GraphicsCapabilities {
            webgl2_supported: true,
            max_texture_size: 4096,
            ..Default::default()
        };
        assert_eq!(GraphicsCapabilities::assess_performance_tier(&caps), GraphicsPerformanceTier::Medium);
        
        let caps = GraphicsCapabilities {
            webgl_supported: true,
            ..Default::default()
        };
        assert_eq!(GraphicsCapabilities::assess_performance_tier(&caps), GraphicsPerformanceTier::Low);
    }
}