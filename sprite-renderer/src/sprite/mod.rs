//! Sprite definitions and management
//!
//! This module provides core sprite structures and management functionality,
//! including sprite definitions, texture atlas management, and animation systems.

pub mod sprite;
pub mod atlas;
pub mod animation;

// Re-export core sprite types
pub use sprite::{Sprite, SpriteId, SpriteBuilder};

/// Texture identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u64);

impl TextureId {
    /// Create a new texture ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for TextureId {
    fn default() -> Self {
        Self::new()
    }
}