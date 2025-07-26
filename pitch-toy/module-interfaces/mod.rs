//! Module Interfaces for Three-Layer Architecture
//!
//! This module defines the communication interfaces between the three layers
//! of the pitch-toy architecture: Engine, Model, and Presentation.
//!
//! ## Interface Communication Flow
//!
//! - **Engine → Model**: Data passed via update() return values
//! - **Model → Presentation**: Data passed via update() return values  

pub mod engine_to_model;
pub mod model_to_presentation;