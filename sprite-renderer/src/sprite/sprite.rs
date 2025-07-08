//! Core sprite implementation
//!
//! This module contains the main Sprite struct and related functionality
//! for creating, validating, and manipulating sprites.

use crate::utils::{Vec2, Color};
use crate::{RendererError, Result};

/// Unique identifier for sprites
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteId(pub u64);

impl SpriteId {
    /// Create a new unique sprite ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for SpriteId {
    fn default() -> Self {
        Self::new()
    }
}

/// Core sprite structure with position, size, rotation, and color properties
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Unique identifier for this sprite
    pub id: SpriteId,
    /// Position in 2D space (x, y)
    pub position: Vec2,
    /// Size dimensions (width, height)
    pub size: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// RGBA color values (0.0 to 1.0)
    pub color: Color,
    /// Depth layer for z-ordering
    pub depth: f32,
    /// Visibility flag
    pub visible: bool,
}

impl Sprite {
    /// Create a new sprite with default values
    pub fn new() -> Self {
        Self {
            id: SpriteId::new(),
            position: Vec2::ZERO,
            size: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            color: Color::WHITE,
            depth: 0.0,
            visible: true,
        }
    }

    /// Create a sprite builder for fluent construction
    pub fn builder() -> SpriteBuilder {
        SpriteBuilder::new()
    }

    /// Validate sprite properties
    pub fn validate(&self) -> Result<()> {
        // Validate size dimensions are positive
        if self.size.x <= 0.0 || self.size.y <= 0.0 {
            return Err(RendererError::InvalidSpriteData(
                format!("Sprite size must be positive, got width={}, height={}", 
                    self.size.x, self.size.y)
            ));
        }

        // Validate color ranges (0.0 to 1.0)
        if self.color.r < 0.0 || self.color.r > 1.0 ||
           self.color.g < 0.0 || self.color.g > 1.0 ||
           self.color.b < 0.0 || self.color.b > 1.0 ||
           self.color.a < 0.0 || self.color.a > 1.0 {
            return Err(RendererError::InvalidSpriteData(
                format!("Color values must be in range [0.0, 1.0], got r={}, g={}, b={}, a={}", 
                    self.color.r, self.color.g, self.color.b, self.color.a)
            ));
        }

        Ok(())
    }

    /// Translate the sprite by the given offset
    pub fn translate(&mut self, offset: Vec2) {
        self.position.x += offset.x;
        self.position.y += offset.y;
    }

    /// Rotate the sprite by the given angle in radians
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }

    /// Scale the sprite by the given factors
    pub fn scale(&mut self, scale_x: f32, scale_y: f32) {
        self.size.x *= scale_x;
        self.size.y *= scale_y;
    }

    /// Set the sprite's position
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Set the sprite's size
    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    /// Set the sprite's rotation
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    /// Set the sprite's color
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for creating sprites with fluent API
pub struct SpriteBuilder {
    sprite: Sprite,
}

impl SpriteBuilder {
    /// Create a new sprite builder with default values
    pub fn new() -> Self {
        Self {
            sprite: Sprite::new(),
        }
    }

    /// Set the sprite position
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.sprite.position = Vec2::new(x, y);
        self
    }

    /// Set the sprite position using Vec2
    pub fn position_vec(mut self, position: Vec2) -> Self {
        self.sprite.position = position;
        self
    }

    /// Set the sprite size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.sprite.size = Vec2::new(width, height);
        self
    }

    /// Set the sprite size using Vec2
    pub fn size_vec(mut self, size: Vec2) -> Self {
        self.sprite.size = size;
        self
    }

    /// Set the sprite rotation in radians
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.sprite.rotation = rotation;
        self
    }

    /// Set the sprite color
    pub fn color(mut self, color: Color) -> Self {
        self.sprite.color = color;
        self
    }

    /// Set the sprite color using RGB values
    pub fn color_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.sprite.color = Color::rgb(r, g, b);
        self
    }

    /// Set the sprite color using RGBA values
    pub fn color_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.sprite.color = Color::new(r, g, b, a);
        self
    }

    /// Set the sprite depth
    pub fn depth(mut self, depth: f32) -> Self {
        self.sprite.depth = depth;
        self
    }

    /// Set the sprite visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.sprite.visible = visible;
        self
    }

    /// Build the sprite and validate it
    pub fn build(self) -> Result<Sprite> {
        self.sprite.validate()?;
        Ok(self.sprite)
    }

    /// Build the sprite without validation
    pub fn build_unchecked(self) -> Sprite {
        self.sprite
    }
}

impl Default for SpriteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_creation() {
        let sprite = Sprite::new();
        assert_eq!(sprite.position, Vec2::ZERO);
        assert_eq!(sprite.size, Vec2::new(1.0, 1.0));
        assert_eq!(sprite.rotation, 0.0);
        assert_eq!(sprite.color, Color::WHITE);
        assert!(sprite.visible);
    }

    #[test]
    fn test_sprite_builder() {
        let sprite = Sprite::builder()
            .position(10.0, 20.0)
            .size(50.0, 100.0)
            .rotation(1.57) // ~90 degrees
            .color_rgb(1.0, 0.0, 0.0)
            .build()
            .unwrap();

        assert_eq!(sprite.position, Vec2::new(10.0, 20.0));
        assert_eq!(sprite.size, Vec2::new(50.0, 100.0));
        assert_eq!(sprite.rotation, 1.57);
        assert_eq!(sprite.color, Color::RED);
    }

    #[test]
    fn test_sprite_validation_positive_size() {
        let mut sprite = Sprite::new();
        sprite.size = Vec2::new(-1.0, 1.0);
        assert!(sprite.validate().is_err());

        sprite.size = Vec2::new(1.0, -1.0);
        assert!(sprite.validate().is_err());

        sprite.size = Vec2::new(1.0, 1.0);
        assert!(sprite.validate().is_ok());
    }

    #[test]
    fn test_sprite_validation_color_range() {
        let mut sprite = Sprite::new();
        sprite.color = Color::new(-0.1, 0.5, 0.5, 1.0);
        assert!(sprite.validate().is_err());

        sprite.color = Color::new(1.1, 0.5, 0.5, 1.0);
        assert!(sprite.validate().is_err());

        sprite.color = Color::new(0.5, 0.5, 0.5, 1.0);
        assert!(sprite.validate().is_ok());
    }

    #[test]
    fn test_sprite_transformations() {
        let mut sprite = Sprite::new();
        
        // Test translation
        sprite.translate(Vec2::new(10.0, 20.0));
        assert_eq!(sprite.position, Vec2::new(10.0, 20.0));

        // Test rotation
        sprite.rotate(1.57);
        assert_eq!(sprite.rotation, 1.57);

        // Test scaling
        sprite.scale(2.0, 3.0);
        assert_eq!(sprite.size, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_sprite_id_uniqueness() {
        let sprite1 = Sprite::new();
        let sprite2 = Sprite::new();
        assert_ne!(sprite1.id, sprite2.id);
    }
}