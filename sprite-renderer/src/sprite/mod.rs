//! Sprite definitions and management
//!
//! This module provides core sprite structures and management functionality,
//! including sprite definitions, texture atlas management, and animation systems.

pub mod sprite;
pub mod atlas;
pub mod animation;

use crate::utils::{Vec2, Color};
use crate::shaders::ShaderId;

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

/// Core sprite structure
#[derive(Debug, Clone)]
pub struct Sprite {
    pub id: SpriteId,
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub color: Color,
    pub texture: Option<TextureId>,
    pub shader: ShaderId,
    pub depth: f32,
    pub visible: bool,
}

impl Sprite {
    /// Create a sprite builder
    pub fn builder() -> SpriteBuilder {
        SpriteBuilder::new()
    }
}

/// Builder pattern for creating sprites
pub struct SpriteBuilder {
    sprite: Sprite,
}

impl SpriteBuilder {
    /// Create a new sprite builder
    pub fn new() -> Self {
        Self {
            sprite: Sprite {
                id: SpriteId::new(),
                position: Vec2::new(0.0, 0.0),
                size: Vec2::new(1.0, 1.0),
                rotation: 0.0,
                color: Color::WHITE,
                texture: None,
                shader: ShaderId::default(),
                depth: 0.0,
                visible: true,
            }
        }
    }
    
    /// Set sprite position
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.sprite.position = Vec2::new(x, y);
        self
    }
    
    /// Set sprite size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.sprite.size = Vec2::new(width, height);
        self
    }
    
    /// Set sprite color
    pub fn color(mut self, color: Color) -> Self {
        self.sprite.color = color;
        self
    }
    
    /// Build the sprite
    pub fn build(self) -> Sprite {
        self.sprite
    }
}

impl Default for SpriteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Texture identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u64);