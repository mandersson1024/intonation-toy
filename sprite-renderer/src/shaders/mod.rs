//! Shader compilation and management
//!
//! This module provides shader compilation, management, and built-in shader
//! implementations for common sprite rendering operations.

pub mod builtin;
pub mod solid_color;
pub mod textured;

use std::collections::HashMap;

/// Shader identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderId(pub u64);

impl ShaderId {
    /// Create a new shader ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ShaderId {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in shader types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinShader {
    SolidColor,
    Textured,
    TexturedWithColor,
}

impl From<BuiltinShader> for ShaderId {
    fn from(_builtin: BuiltinShader) -> Self {
        // Placeholder implementation
        ShaderId::new()
    }
}

/// Custom shader definition
#[derive(Debug, Clone)]
pub struct CustomShader {
    pub vertex_source: String,
    pub fragment_source: String,
    pub uniforms: HashMap<String, UniformValue>,
}

/// Uniform value types
#[derive(Debug, Clone)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    Bool(bool),
}