use action::{Action, ActionTrigger, ActionListener};

#[derive(Debug, Clone, PartialEq)]
pub struct RequestMicrophonePermissionAction;

pub struct ModelToEngineInterface {
    request_microphone_permission: Action<RequestMicrophonePermissionAction>,
}

impl ModelToEngineInterface {
    /// Create a new Model → Engine interface with all actions
    pub fn new() -> Self {
        Self {
            request_microphone_permission: Action::new(),
        }
    }

    /// Get a trigger for requesting microphone permission that the model can use to send requests
    pub fn request_microphone_permission_trigger(&self) -> ActionTrigger<RequestMicrophonePermissionAction> {
        self.request_microphone_permission.trigger()
    }

    /// Get a listener for microphone permission requests that the engine can use to receive requests
    pub fn request_microphone_permission_listener(&self) -> ActionListener<RequestMicrophonePermissionAction> {
        self.request_microphone_permission.listener()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[wasm_bindgen_test]
    fn test_action_interface_factory_system() {
        let interface = ModelToEngineInterface::new();
        
        // Test that trigger and listener can be extracted
        let trigger = interface.request_microphone_permission_trigger();
        let listener = interface.request_microphone_permission_listener();
        
        // Test that they work together
        let received = Rc::new(RefCell::new(Vec::new()));
        let received_clone = received.clone();
        
        listener.listen(move |action| {
            received_clone.borrow_mut().push(action);
        });
        
        trigger.fire(RequestMicrophonePermissionAction);
        
        assert_eq!(received.borrow().len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_action_interface_isolation() {
        let interface1 = ModelToEngineInterface::new();
        let interface2 = ModelToEngineInterface::new();
        
        let trigger1 = interface1.request_microphone_permission_trigger();
        let listener1 = interface1.request_microphone_permission_listener();
        let listener2 = interface2.request_microphone_permission_listener();
        
        let received1 = Rc::new(RefCell::new(0));
        let received2 = Rc::new(RefCell::new(0));
        let received1_clone = received1.clone();
        let received2_clone = received2.clone();
        
        listener1.listen(move |_| {
            *received1_clone.borrow_mut() += 1;
        });
        
        listener2.listen(move |_| {
            *received2_clone.borrow_mut() += 1;
        });
        
        trigger1.fire(RequestMicrophonePermissionAction);
        
        // Only listener1 should receive the action
        assert_eq!(*received1.borrow(), 1);
        assert_eq!(*received2.borrow(), 0);
    }
}

