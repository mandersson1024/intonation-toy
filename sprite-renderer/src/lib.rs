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
//! ## Example
//!
//! ```rust,no_run
//! use sprite_renderer::*;
//!
//! // Initialize renderer with canvas element
//! let canvas = get_canvas_element();
//! let mut renderer = SpriteRenderer::new(&canvas)?;
//!
//! // Create a sprite
//! let sprite = Sprite::builder()
//!     .position(100.0, 100.0)
//!     .size(64.0, 64.0)
//!     .color(Color::RED)
//!     .build();
//!
//! // Render the sprite
//! let camera = Camera::default_2d(800, 600);
//! renderer.render(&[sprite], &camera)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Core modules
pub mod renderer;
pub mod sprite;
pub mod shaders;
pub mod utils;

// Feature-gated modules
#[cfg(feature = "hit-testing")]
pub mod hit_testing;

#[cfg(feature = "depth-testing")]
pub mod depth;

// Re-exports for easy usage
pub use renderer::{SpriteRenderer, Camera};
pub use sprite::{Sprite, SpriteId};
pub use shaders::{BuiltinShader, CustomShader, ShaderId};
pub use utils::{Vec2, Rectangle, Color, Transform2D};

#[cfg(feature = "hit-testing")]
pub use hit_testing::{HitBox, HitTester};

#[cfg(feature = "depth-testing")]
pub use depth::DepthManager;

// Error types
use thiserror::Error;

/// Main error type for the sprite renderer
#[derive(Error, Debug)]
pub enum RendererError {
    #[error("WebGL context creation failed")]
    WebGLContextFailed,
    
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    
    #[error("Texture loading failed: {0}")]
    TextureLoadingFailed(String),
    
    #[error("Invalid sprite data: {0}")]
    InvalidSpriteData(String),
}

/// Result type alias for sprite renderer operations
pub type Result<T> = std::result::Result<T, RendererError>;
