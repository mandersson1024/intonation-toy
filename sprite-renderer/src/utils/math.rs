//! Mathematical utilities and structures

/// 2D vector structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Create a new 2D vector
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    /// Zero vector
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    
    /// Unit vector along X axis
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    
    /// Unit vector along Y axis
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Rectangle structure for bounds and hit testing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// 2D transformation matrix
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    // Placeholder - will be implemented in future stories
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform2D {
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}