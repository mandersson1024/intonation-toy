//! Input Event Types
//!
//! This module defines input-specific events that can be published and subscribed to
//! by various components of the application. These events enable loose coupling between
//! the input subsystem and other components like graphics and audio.

use event_dispatcher::Event;

/// Input-related events that can be published throughout the application
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse click detected on a GPU-rendered element
    MouseClick {
        position: (f32, f32),
        screen_position: (f32, f32),
        target_element: Option<String>,
        timestamp: f64,
    },
    /// Mouse move over canvas
    MouseMove {
        position: (f32, f32),
        screen_position: (f32, f32),
        timestamp: f64,
    },
    /// Touch start detected on a GPU-rendered element
    TouchStart {
        position: (f32, f32),
        screen_position: (f32, f32),
        target_element: Option<String>,
        timestamp: f64,
    },
    /// Touch end detected
    TouchEnd {
        position: (f32, f32),
        screen_position: (f32, f32),
        timestamp: f64,
    },
    /// Touch move detected
    TouchMove {
        position: (f32, f32),
        screen_position: (f32, f32),
        timestamp: f64,
    },
    /// Microphone permission request triggered
    MicrophonePermissionRequest {
        trigger_element: String,
        timestamp: f64,
    },
    /// Hit test result for clickable elements
    HitTestResult {
        element_id: String,
        hit: bool,
        position: (f32, f32),
        timestamp: f64,
    },
}

impl InputEvent {
    /// Get the event type as a string for subscription matching
    pub fn event_type(&self) -> &'static str {
        match self {
            InputEvent::MouseClick { .. } => "mouse_click",
            InputEvent::MouseMove { .. } => "mouse_move",
            InputEvent::TouchStart { .. } => "touch_start",
            InputEvent::TouchEnd { .. } => "touch_end", 
            InputEvent::TouchMove { .. } => "touch_move",
            InputEvent::MicrophonePermissionRequest { .. } => "microphone_permission_request",
            InputEvent::HitTestResult { .. } => "hit_test_result",
        }
    }
    
    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            InputEvent::MouseClick { position, target_element, .. } => {
                match target_element {
                    Some(element) => format!("Mouse click on {} at ({:.1}, {:.1})", element, position.0, position.1),
                    None => format!("Mouse click at ({:.1}, {:.1})", position.0, position.1),
                }
            }
            InputEvent::MouseMove { position, .. } => {
                format!("Mouse move to ({:.1}, {:.1})", position.0, position.1)
            }
            InputEvent::TouchStart { position, target_element, .. } => {
                match target_element {
                    Some(element) => format!("Touch start on {} at ({:.1}, {:.1})", element, position.0, position.1),
                    None => format!("Touch start at ({:.1}, {:.1})", position.0, position.1),
                }
            }
            InputEvent::TouchEnd { position, .. } => {
                format!("Touch end at ({:.1}, {:.1})", position.0, position.1)
            }
            InputEvent::TouchMove { position, .. } => {
                format!("Touch move to ({:.1}, {:.1})", position.0, position.1)
            }
            InputEvent::MicrophonePermissionRequest { trigger_element, .. } => {
                format!("Microphone permission request from {}", trigger_element)
            }
            InputEvent::HitTestResult { element_id, hit, position, .. } => {
                format!("Hit test for {}: {} at ({:.1}, {:.1})", 
                    element_id,
                    if *hit { "HIT" } else { "MISS" },
                    position.0, position.1)
            }
        }
    }
}

impl Event for InputEvent {
    fn event_type(&self) -> &'static str {
        self.event_type()
    }
    
    fn description(&self) -> String {
        self.description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
     use wasm_bindgen_test::wasm_bindgen_test;
   
    #[wasm_bindgen_test]
    fn test_input_event_types() {
        let mouse_click = InputEvent::MouseClick {
            position: (100.0, 200.0),
            screen_position: (150.0, 250.0),
            target_element: Some("green_square".to_string()),
            timestamp: 1000.0,
        };
        assert_eq!(mouse_click.event_type(), "mouse_click");
        
        let mouse_move = InputEvent::MouseMove {
            position: (50.0, 75.0),
            screen_position: (100.0, 125.0),
            timestamp: 1001.0,
        };
        assert_eq!(mouse_move.event_type(), "mouse_move");
        
        let touch_start = InputEvent::TouchStart {
            position: (200.0, 300.0),
            screen_position: (250.0, 350.0),
            target_element: Some("test_button".to_string()),
            timestamp: 1002.0,
        };
        assert_eq!(touch_start.event_type(), "touch_start");
        
        let mic_request = InputEvent::MicrophonePermissionRequest {
            trigger_element: "green_square".to_string(),
            timestamp: 1003.0,
        };
        assert_eq!(mic_request.event_type(), "microphone_permission_request");
    }
    
    #[wasm_bindgen_test]
    fn test_input_event_descriptions() {
        let mouse_click = InputEvent::MouseClick {
            position: (100.0, 200.0),
            screen_position: (150.0, 250.0),
            target_element: Some("green_square".to_string()),
            timestamp: 1000.0,
        };
        assert!(mouse_click.description().contains("Mouse click on green_square"));
        assert!(mouse_click.description().contains("(100.0, 200.0)"));
        
        let mouse_move = InputEvent::MouseMove {
            position: (50.0, 75.0),
            screen_position: (100.0, 125.0),
            timestamp: 1001.0,
        };
        assert!(mouse_move.description().contains("Mouse move to"));
        assert!(mouse_move.description().contains("(50.0, 75.0)"));
        
        let hit_test = InputEvent::HitTestResult {
            element_id: "green_square".to_string(),
            hit: true,
            position: (150.0, 250.0),
            timestamp: 1004.0,
        };
        assert!(hit_test.description().contains("Hit test for green_square: HIT"));
        assert!(hit_test.description().contains("(150.0, 250.0)"));
    }
    
    #[wasm_bindgen_test]
    fn test_input_event_no_target() {
        let mouse_click = InputEvent::MouseClick {
            position: (100.0, 200.0),
            screen_position: (150.0, 250.0),
            target_element: None,
            timestamp: 1000.0,
        };
        assert!(mouse_click.description().contains("Mouse click at"));
        assert!(!mouse_click.description().contains(" on "));
    }
}