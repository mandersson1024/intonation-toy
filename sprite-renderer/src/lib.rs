//! # Sprite Renderer
//!
//! A high-performance 2D sprite rendering library for modern web browsers.
//! Built with WebGL acceleration and designed for WebAssembly deployment.
//!
//! ## Features
//!
//! - GPU-accelerated 2D sprite rendering via WebGL
//! - Built-in and custom shader support
//! - Optional hit testing with spatial indexing
//! - Optional depth management and layer sorting
//! - Texture atlas support for performance optimization
//! - Zero-dependency library interface
//!
//! ## Architecture
//!
//! The library is organized into several modules:
//!
//! - [`renderer`]: Core rendering engine and WebGL context management
//! - [`sprite`]: Sprite definitions, builders, and management
//! - [`shaders`]: Shader compilation and management system
//! - [`hit_testing`]: Spatial indexing and collision detection (optional)
//! - [`depth`]: Depth sorting and layer management (optional)
//! - [`utils`]: Common utilities and helper functions
//!
//! ## Performance Tips
//!
//! - Use batch rendering for multiple sprites with `SpriteRenderer::render()`
//! - Group sprites by texture to minimize GPU state changes
//! - Enable depth testing only when needed with the `depth-testing` feature
//! - Reuse sprite objects and avoid frequent allocations
//! - Use texture atlases to reduce texture switching overhead
//!
//! ## Example
//!
//! ```rust,no_run
//! use sprite_renderer::*;
//! use wasm_bindgen::JsCast;
//!
//! // Initialize renderer with canvas element
//! let document = web_sys::window().unwrap().document().unwrap();
//! let canvas = document.get_element_by_id("canvas").unwrap()
//!     .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
//! let mut renderer = SpriteRenderer::new(&canvas)?;
//!
//! // Create a sprite
//! let sprite = Sprite::builder()
//!     .position(100.0, 100.0)
//!     .size(64.0, 64.0)
//!     .color(Color::RED)
//!     .build()?;
//!
//! // Render the sprite
//! let camera = Camera::default_2d(800, 600);
//! renderer.render(&[sprite], &camera)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Core modules

/// Core rendering engine and WebGL context management
/// 
/// This module provides the main [`SpriteRenderer`] struct and [`Camera`] for
/// managing the rendering pipeline and viewport.
pub mod renderer;

/// Sprite definitions, builders, and management
/// 
/// This module provides the [`Sprite`] struct and related types for creating
/// and managing sprite instances.
pub mod sprite;

/// Shader compilation and management system
/// 
/// This module provides built-in shaders and support for custom shader programs.
pub mod shaders;

/// Common utilities and helper functions
/// 
/// This module provides mathematical types, color utilities, and other common
/// functionality used throughout the library.
pub mod utils;

// Feature-gated modules

/// Spatial indexing and collision detection (optional)
/// 
/// This module provides efficient hit testing and collision detection for sprites.
/// Only available when the `hit-testing` feature is enabled.
#[cfg(feature = "hit-testing")]
pub mod hit_testing;

/// Depth sorting and layer management (optional)
/// 
/// This module provides depth-based sprite sorting and layer management for
/// proper rendering order and z-fighting prevention.
/// Only available when the `depth-testing` feature is enabled.
#[cfg(feature = "depth-testing")]
pub mod depth;

// Re-exports for easy usage
pub use renderer::{SpriteRenderer, Camera};
pub use sprite::{Sprite, SpriteId, SpriteBuilder};
pub use shaders::{BuiltinShader, CustomShader, ShaderId};
pub use utils::{Vec2, Rectangle, Color, Transform2D, Mat4};

#[cfg(feature = "hit-testing")]
pub use hit_testing::{HitBox, HitTester};

#[cfg(feature = "depth-testing")]
pub use depth::DepthManager;

// Error types
use thiserror::Error;

/// Error types for sprite renderer operations
/// 
/// This enum represents all possible errors that can occur during sprite rendering
/// operations, from WebGL context creation to texture loading and shader compilation.
/// All errors implement the standard `Error` trait and provide detailed error messages
/// to help with debugging and error handling.
///
/// # Examples
///
/// ```rust,no_run
/// use sprite_renderer::{RendererError, Result};
///
/// fn handle_renderer_error(result: Result<()>) {
///     match result {
///         Ok(()) => println!("Success!"),
///         Err(RendererError::WebGLContextFailed) => {
///             eprintln!("Failed to create WebGL context. Check browser support.");
///         }
///         Err(RendererError::ShaderCompilationFailed(msg)) => {
///             eprintln!("Shader compilation error: {}", msg);
///         }
///         Err(RendererError::TextureLoadingFailed(msg)) => {
///             eprintln!("Texture loading error: {}", msg);
///         }
///         Err(RendererError::InvalidSpriteData(msg)) => {
///             eprintln!("Invalid sprite data: {}", msg);
///         }
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum RendererError {
    /// WebGL context creation failed
    ///
    /// This error occurs when the browser cannot create a WebGL context,
    /// typically due to hardware limitations, driver issues, or browser settings.
    /// 
    /// Common causes:
    /// - WebGL is disabled in browser settings
    /// - Graphics drivers are outdated or incompatible
    /// - Hardware acceleration is disabled
    /// - Running in a headless environment without proper WebGL support
    #[error("WebGL context creation failed")]
    WebGLContextFailed,
    
    /// Shader compilation failed
    ///
    /// This error occurs when a vertex or fragment shader fails to compile.
    /// The error message contains details about the compilation failure.
    ///
    /// Common causes:
    /// - Syntax errors in shader code
    /// - Unsupported shader language features
    /// - Hardware-specific shader limitations
    /// - Incorrect shader version directives
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    
    /// Texture loading failed
    ///
    /// This error occurs when a texture cannot be loaded or uploaded to the GPU.
    /// The error message contains details about the failure.
    ///
    /// Common causes:
    /// - Image file format not supported
    /// - Image file corrupted or invalid
    /// - Insufficient GPU memory for texture
    /// - Texture dimensions exceed hardware limits
    #[error("Texture loading failed: {0}")]
    TextureLoadingFailed(String),
    
    /// Invalid sprite data provided
    ///
    /// This error occurs when sprite data is malformed or contains invalid values.
    /// The error message describes the specific validation failure.
    ///
    /// Common causes:
    /// - Negative sprite dimensions
    /// - Invalid texture coordinates (outside 0.0-1.0 range)
    /// - Null or empty sprite data
    /// - Incompatible sprite format
    #[error("Invalid sprite data: {0}")]
    InvalidSpriteData(String),
}

/// Result type alias for sprite renderer operations
///
/// This is a convenient type alias for `Result<T, RendererError>` to reduce
/// boilerplate code when working with sprite renderer functions.
///
/// # Examples
///
/// ```rust,no_run
/// use sprite_renderer::{Result, SpriteRenderer};
///
/// fn create_renderer() -> Result<SpriteRenderer> {
///     // This function returns Result<SpriteRenderer, RendererError>
///     # todo!("Implementation")
/// }
/// ```
pub type Result<T> = std::result::Result<T, RendererError>;
