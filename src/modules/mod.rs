//! # Modules
//!
//! This module contains the core application modules that provide the foundational
//! services and infrastructure for the pitch-toy application.

pub mod application_core;
pub mod audio_foundations;
pub mod platform_abstraction;
pub mod data_management;

// Developer UI module - conditionally compiled for debug builds only
#[cfg(debug_assertions)]
pub mod developer_ui;

pub use application_core::*;
pub use audio_foundations::*;
pub use platform_abstraction::*;

// Re-export developer UI for debug builds
#[cfg(debug_assertions)]
pub use developer_ui::exports::*;