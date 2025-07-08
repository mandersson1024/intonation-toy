//! Spatial indexing and collision detection
//!
//! This module provides hit testing functionality including spatial indexing
//! for performance optimization and collision detection algorithms.

#[cfg(feature = "hit-testing")]
pub mod bounds;
#[cfg(feature = "hit-testing")]
pub mod spatial_index;