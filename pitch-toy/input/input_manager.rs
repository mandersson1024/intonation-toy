//! Input Manager Implementation
//!
//! This module provides the main InputManager that handles mouse/touch interactions
//! with GPU-rendered elements and integrates with the event dispatcher system.

use web_sys::{HtmlCanvasElement, MouseEvent, TouchEvent};
use wasm_bindgen::{prelude::*, JsCast};
use std::rc::Rc;
use std::cell::RefCell;

use super::{CoordinateTransformer, HitTestRegistry, HitTestElement};

/// Main input manager for handling canvas input events
pub struct InputManager {
    canvas: HtmlCanvasElement,
    coordinate_transformer: Rc<RefCell<CoordinateTransformer>>,
    hit_test_registry: Rc<RefCell<HitTestRegistry>>,
    
    // Closures need to be stored to prevent deallocation
    mouse_click_closure: Option<Closure<dyn Fn(MouseEvent)>>,
    mouse_move_closure: Option<Closure<dyn Fn(MouseEvent)>>,
    touch_start_closure: Option<Closure<dyn Fn(TouchEvent)>>,
    touch_end_closure: Option<Closure<dyn Fn(TouchEvent)>>,
    touch_move_closure: Option<Closure<dyn Fn(TouchEvent)>>,
}

impl InputManager {
    /// Create a new InputManager
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // Initialize coordinate transformer with default graphics dimensions
        // These will be updated when graphics renderer provides actual dimensions
        let coordinate_transformer = Rc::new(RefCell::new(
            CoordinateTransformer::new(&canvas, 2.0, 2.0)
        ));
        let hit_test_registry = Rc::new(RefCell::new(
            HitTestRegistry::new()
        ));
        
        let mut input_manager = Self {
            canvas,
            coordinate_transformer,
            hit_test_registry,
            mouse_click_closure: None,
            mouse_move_closure: None,
            touch_start_closure: None,
            touch_end_closure: None,
            touch_move_closure: None,
        };
        
        input_manager.setup_event_listeners()?;
        
        Ok(input_manager)
    }
    
    /// Setup canvas event listeners
    fn setup_event_listeners(&mut self) -> Result<(), JsValue> {
        let canvas = self.canvas.clone();
        
        // Create mouse click handler
        {
            let canvas_clone = canvas.clone();
            let coordinate_transformer = self.coordinate_transformer.clone();
            let hit_test_registry = self.hit_test_registry.clone();
            
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                event.prevent_default();
                
                let rect = canvas_clone.get_bounding_client_rect();
                let screen_x = event.client_x() as f32 - rect.left() as f32;
                let screen_y = event.client_y() as f32 - rect.top() as f32;
                
                let transformer = coordinate_transformer.borrow();
                let (graphics_x, graphics_y) = transformer.screen_to_graphics(screen_x, screen_y);
                
                // Perform hit test
                let registry = hit_test_registry.borrow();
                let hit_element = registry.hit_test_first(graphics_x, graphics_y);
                
                // Log to console for debugging
                web_sys::console::log_3(
                    &"Mouse click detected:".into(),
                    &format!("Screen: ({:.1}, {:.1})", screen_x, screen_y).into(),
                    &format!("Graphics: ({:.1}, {:.1})", graphics_x, graphics_y).into(),
                );
                
                if let Some(element_id) = &hit_element {
                    web_sys::console::log_2(
                        &"Hit element:".into(),
                        &element_id.into(),
                    );
                    
                    // Trigger microphone permission request for green square
                    if element_id == "green_square" {
                        web_sys::console::log_1(&"Triggering microphone permission request".into());
                        // TODO: Will implement permission request in Task 4
                    }
                }
                
                // TODO: Publish input event through event dispatcher
                // This will be implemented when we have proper event integration
                
            }) as Box<dyn Fn(MouseEvent)>);
            
            canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            self.mouse_click_closure = Some(closure);
        }
        
        // Create mouse move handler with throttling
        {
            let canvas_clone = canvas.clone();
            let coordinate_transformer = self.coordinate_transformer.clone();
            let last_move_time = std::cell::Cell::new(0.0);
            
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                // Throttle mouse move events to every 16ms (60fps)
                let current_time = Self::get_timestamp();
                if current_time - last_move_time.get() < 16.0 {
                    return;
                }
                last_move_time.set(current_time);
                
                let rect = canvas_clone.get_bounding_client_rect();
                let screen_x = event.client_x() as f32 - rect.left() as f32;
                let screen_y = event.client_y() as f32 - rect.top() as f32;
                
                let transformer = coordinate_transformer.borrow();
                if transformer.is_within_canvas(screen_x, screen_y) {
                    let (_graphics_x, _graphics_y) = transformer.screen_to_graphics(screen_x, screen_y);
                    // TODO: Publish mouse move event
                }
            }) as Box<dyn Fn(MouseEvent)>);
            
            canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            self.mouse_move_closure = Some(closure);
        }
        
        // Create touch start handler
        {
            let canvas_clone = canvas.clone();
            let coordinate_transformer = self.coordinate_transformer.clone();
            let hit_test_registry = self.hit_test_registry.clone();
            
            let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
                event.prevent_default();
                
                if let Some(touch) = event.touches().get(0) {
                    let rect = canvas_clone.get_bounding_client_rect();
                    let screen_x = touch.client_x() as f32 - rect.left() as f32;
                    let screen_y = touch.client_y() as f32 - rect.top() as f32;
                    
                    let transformer = coordinate_transformer.borrow();
                    let (graphics_x, graphics_y) = transformer.screen_to_graphics(screen_x, screen_y);
                    
                    // Perform hit test
                    let registry = hit_test_registry.borrow();
                    let hit_element = registry.hit_test_first(graphics_x, graphics_y);
                    
                    web_sys::console::log_3(
                        &"Touch start detected:".into(),
                        &format!("Screen: ({:.1}, {:.1})", screen_x, screen_y).into(),
                        &format!("Graphics: ({:.1}, {:.1})", graphics_x, graphics_y).into(),
                    );
                    
                    if let Some(element_id) = &hit_element {
                        web_sys::console::log_2(
                            &"Touch hit element:".into(),
                            &element_id.into(),
                        );
                        
                        // Trigger microphone permission request for green square
                        if element_id == "green_square" {
                            web_sys::console::log_1(&"Triggering microphone permission request (touch)".into());
                            // TODO: Will implement permission request in Task 4
                        }
                    }
                }
            }) as Box<dyn Fn(TouchEvent)>);
            
            canvas.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())?;
            self.touch_start_closure = Some(closure);
        }
        
        // Create touch end handler
        {
            let canvas_clone = canvas.clone();
            let coordinate_transformer = self.coordinate_transformer.clone();
            
            let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
                event.prevent_default();
                
                if let Some(touch) = event.changed_touches().get(0) {
                    let rect = canvas_clone.get_bounding_client_rect();
                    let screen_x = touch.client_x() as f32 - rect.left() as f32;
                    let screen_y = touch.client_y() as f32 - rect.top() as f32;
                    
                    let transformer = coordinate_transformer.borrow();
                    if transformer.is_within_canvas(screen_x, screen_y) {
                        let (_graphics_x, _graphics_y) = transformer.screen_to_graphics(screen_x, screen_y);
                        web_sys::console::log_1(&"Touch end detected".into());
                        // TODO: Publish touch end event
                    }
                }
            }) as Box<dyn Fn(TouchEvent)>);
            
            canvas.add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())?;
            self.touch_end_closure = Some(closure);
        }
        
        // Create touch move handler with throttling
        {
            let canvas_clone = canvas.clone();
            let coordinate_transformer = self.coordinate_transformer.clone();
            let last_move_time = std::cell::Cell::new(0.0);
            
            let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
                event.prevent_default();
                
                // Throttle touch move events to every 16ms (60fps)
                let current_time = Self::get_timestamp();
                if current_time - last_move_time.get() < 16.0 {
                    return;
                }
                last_move_time.set(current_time);
                
                if let Some(touch) = event.touches().get(0) {
                    let rect = canvas_clone.get_bounding_client_rect();
                    let screen_x = touch.client_x() as f32 - rect.left() as f32;
                    let screen_y = touch.client_y() as f32 - rect.top() as f32;
                    
                    let transformer = coordinate_transformer.borrow();
                    if transformer.is_within_canvas(screen_x, screen_y) {
                        let (_graphics_x, _graphics_y) = transformer.screen_to_graphics(screen_x, screen_y);
                        // TODO: Publish touch move event
                    }
                }
            }) as Box<dyn Fn(TouchEvent)>);
            
            canvas.add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())?;
            self.touch_move_closure = Some(closure);
        }
        
        Ok(())
    }
    
    /// Register a hit test element
    pub fn register_hit_test_element(&mut self, id: &str, center_x: f32, center_y: f32, width: f32, height: f32) {
        let element = HitTestElement::from_center_and_size(
            id.to_string(),
            center_x,
            center_y,
            width,
            height,
        );
        self.hit_test_registry.borrow_mut().register_element(element);
        
        web_sys::console::log_4(
            &"Registered hit test element:".into(),
            &id.into(),
            &format!("center: ({:.1}, {:.1})", center_x, center_y).into(),
            &format!("size: ({:.1}, {:.1})", width, height).into(),
        );
    }
    
    /// Update coordinate transformer dimensions
    pub fn update_graphics_dimensions(&mut self, width: f32, height: f32) {
        self.coordinate_transformer.borrow_mut().update_graphics_size(width, height);
        web_sys::console::log_2(
            &"Updated graphics dimensions:".into(),
            &format!("{}x{}", width, height).into(),
        );
    }
    
    /// Update canvas size (call when canvas is resized)
    pub fn update_canvas_size(&mut self) {
        self.coordinate_transformer.borrow_mut().update_canvas_size(&self.canvas);
        let (width, height) = self.coordinate_transformer.borrow().canvas_dimensions();
        web_sys::console::log_2(
            &"Updated canvas dimensions:".into(),
            &format!("{}x{}", width, height).into(),
        );
    }
    
    /// Get hit test registry for external manipulation
    pub fn hit_test_registry(&self) -> Rc<RefCell<HitTestRegistry>> {
        self.hit_test_registry.clone()
    }
    
    /// Get coordinate transformer for external use
    pub fn coordinate_transformer(&self) -> Rc<RefCell<CoordinateTransformer>> {
        self.coordinate_transformer.clone()
    }
    
    /// Get current performance timestamp
    fn get_timestamp() -> f64 {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        // Clean up event listeners when InputManager is dropped
        if let Some(closure) = self.mouse_click_closure.take() {
            let _ = self.canvas.remove_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        }
        if let Some(closure) = self.mouse_move_closure.take() {
            let _ = self.canvas.remove_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref());
        }
        if let Some(closure) = self.touch_start_closure.take() {
            let _ = self.canvas.remove_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref());
        }
        if let Some(closure) = self.touch_end_closure.take() {
            let _ = self.canvas.remove_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref());
        }
        if let Some(closure) = self.touch_move_closure.take() {
            let _ = self.canvas.remove_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: These tests require a DOM environment and are primarily for structure validation
    // Actual functionality testing should be done in the browser environment
    
    #[test]
    fn test_coordinate_transformer_interface() {
        // Test that coordinate transformer has expected interface
        // DOM operations tested in wasm-pack tests
        let registry = HitTestRegistry::new();
        assert_eq!(registry.element_count(), 0);
        
        // Test that we can create elements with expected interface
        let element = HitTestElement::from_center_and_size(
            "test".to_string(), 0.0, 0.0, 1.0, 1.0
        );
        assert_eq!(element.id, "test");
        assert_eq!(element.center(), (0.0, 0.0));
        assert_eq!(element.size(), (1.0, 1.0));
    }
    
    #[test]
    fn test_hit_test_registry_basic_operations() {
        let mut registry = HitTestRegistry::new();
        
        let element = HitTestElement::from_center_and_size(
            "test_element".to_string(),
            0.0, 0.0, 1.0, 1.0
        );
        registry.register_element(element);
        
        assert_eq!(registry.element_count(), 1);
        assert!(registry.hit_test_first(0.0, 0.0).is_some());
        assert!(registry.hit_test_first(1.0, 1.0).is_none());
    }
}