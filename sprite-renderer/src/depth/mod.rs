//! Depth sorting and layer management
//!
//! This module provides depth-based sprite sorting and layer management
//! for proper rendering order and z-fighting prevention.

#[cfg(feature = "depth-testing")]
pub mod layers;