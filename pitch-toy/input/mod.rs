//! Input Manager Module
//!
//! This module provides an InputManager system that handles mouse/touch interactions
//! with GPU-rendered elements. It integrates with the three-d graphics pipeline and
//! event dispatcher system to enable interactive elements in the 2D graphics rendering.
//!
//! ## Architecture
//!
//! - **InputManager**: Main manager for handling input events on the canvas
//! - **InputEvent**: Event types for input interactions  
//! - **CoordinateTransform**: Transforms screen coordinates to graphics space
//! - **HitTestElement**: Registry of clickable areas for collision detection
//!
//! ## Usage
//!
//! ```rust,no_run
//! use pitch_toy::input::{InputManager, InputEvent, InputEventType};
//! use pitch_toy::events::get_global_event_dispatcher;
//!
//! // Create InputManager with canvas element and event dispatcher
//! let canvas = /* get canvas from graphics renderer */;
//! let event_dispatcher = get_global_event_dispatcher();
//! let input_manager = InputManager::new(canvas, event_dispatcher);
//!
//! // Register hit test elements
//! input_manager.register_hit_test_element("green_square", (0.0, 0.0), (100.0, 100.0));
//! ```

pub use input_manager::InputManager;
pub use coordinate_transform::CoordinateTransformer;
pub use hit_test::{HitTestElement, HitTestRegistry};

mod input_manager;
mod coordinate_transform;
mod hit_test;