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

use std::cell::RefCell;

pub use input_manager::InputManager;
pub use coordinate_transform::CoordinateTransformer;
pub use hit_test::{HitTestElement, HitTestRegistry};

mod input_manager;
mod coordinate_transform;
mod hit_test;

thread_local! {
    /// Global InputManager storage (WASM is single-threaded)
    static GLOBAL_INPUT_MANAGER: RefCell<Option<InputManager>> = RefCell::new(None);
}

/// Set the global InputManager instance
pub fn set_global_input_manager(manager: InputManager) {
    GLOBAL_INPUT_MANAGER.with(|cell| {
        *cell.borrow_mut() = Some(manager);
        web_sys::console::log_1(&"✓ Global InputManager stored successfully".into());
    });
}

/// Access the global InputManager with a closure
pub fn with_global_input_manager<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut InputManager) -> R,
{
    GLOBAL_INPUT_MANAGER.with(|cell| {
        let mut guard = cell.borrow_mut();
        if let Some(ref mut manager) = *guard {
            Some(f(manager))
        } else {
            None
        }
    })
}

/// Debug function to check if InputManager is stored and active
pub fn debug_input_manager_status() {
    GLOBAL_INPUT_MANAGER.with(|cell| {
        let guard = cell.borrow();
        if guard.is_some() {
            web_sys::console::log_1(&"✓ Global InputManager is active and stored".into());
        } else {
            web_sys::console::log_1(&"✗ Global InputManager is NOT stored".into());
        }
    });
}