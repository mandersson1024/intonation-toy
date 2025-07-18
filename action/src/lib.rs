use std::rc::Rc;
use std::cell::RefCell;

type Callback<T> = Box<dyn Fn(T) + 'static>;

/// Internal shared state for an action
struct SharedAction<T> {
    listeners: RefCell<Vec<Callback<T>>>,
}

impl<T> SharedAction<T> {
    fn new() -> Self {
        Self {
            listeners: RefCell::new(Vec::new()),
        }
    }

    fn add_listener(&self, callback: Callback<T>) {
        self.listeners.borrow_mut().push(callback);
    }

    fn fire(&self, data: T) 
    where 
        T: Clone,
    {
        let listeners = self.listeners.borrow();
        for callback in listeners.iter() {
            callback(data.clone());
        }
    }
}

/// Main Action factory that creates triggers and listeners
pub struct Action<T> {
    shared: Rc<SharedAction<T>>,
}

impl<T> Action<T> {
    /// Create a new action
    pub fn new() -> Self {
        Self {
            shared: Rc::new(SharedAction::new()),
        }
    }

    /// Create a trigger handle that can fire this action
    pub fn trigger(&self) -> ActionTrigger<T> {
        ActionTrigger {
            shared: self.shared.clone(),
        }
    }

    /// Create a listener handle that can listen to this action
    pub fn listener(&self) -> ActionListener<T> {
        ActionListener {
            shared: self.shared.clone(),
        }
    }
}

impl<T> Default for Action<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle that can only trigger/fire actions
pub struct ActionTrigger<T> {
    shared: Rc<SharedAction<T>>,
}

impl<T> ActionTrigger<T> {
    /// Fire the action with the given data
    pub fn fire(&self, data: T) 
    where 
        T: Clone,
    {
        self.shared.fire(data);
    }
}

impl<T> Clone for ActionTrigger<T> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

/// Handle that can only listen to actions
pub struct ActionListener<T> {
    shared: Rc<SharedAction<T>>,
}

impl<T> ActionListener<T> {
    /// Register a callback to be called when the action is fired
    pub fn listen<F>(&self, callback: F) 
    where 
        F: Fn(T) + 'static,
    {
        self.shared.add_listener(Box::new(callback));
    }
}

impl<T> Clone for ActionListener<T> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn test_action_with_data() {
        let action = Action::<String>::new();
        let trigger = action.trigger();
        let listener = action.listener();

        let received = Rc::new(RefCell::new(Vec::new()));
        let received_clone = received.clone();

        listener.listen(move |data| {
            received_clone.borrow_mut().push(data);
        });

        trigger.fire("hello".to_string());
        trigger.fire("world".to_string());

        let received_data = received.borrow();
        assert_eq!(received_data.len(), 2);
        assert_eq!(received_data[0], "hello");
        assert_eq!(received_data[1], "world");
    }

    #[test]
    fn test_action_with_void() {
        let action = Action::<()>::new();
        let trigger = action.trigger();
        let listener = action.listener();

        let call_count = Rc::new(RefCell::new(0));
        let call_count_clone = call_count.clone();

        listener.listen(move |_| {
            *call_count_clone.borrow_mut() += 1;
        });

        trigger.fire(());
        trigger.fire(());

        assert_eq!(*call_count.borrow(), 2);
    }

    #[test]
    fn test_multiple_listeners() {
        let action = Action::<i32>::new();
        let trigger = action.trigger();
        let listener1 = action.listener();
        let listener2 = action.listener();

        let received1 = Rc::new(RefCell::new(Vec::new()));
        let received2 = Rc::new(RefCell::new(Vec::new()));
        let received1_clone = received1.clone();
        let received2_clone = received2.clone();

        listener1.listen(move |data| {
            received1_clone.borrow_mut().push(data);
        });

        listener2.listen(move |data| {
            received2_clone.borrow_mut().push(data * 2);
        });

        trigger.fire(5);

        assert_eq!(received1.borrow()[0], 5);
        assert_eq!(received2.borrow()[0], 10);
    }

    #[test]
    fn test_cloneable_handles() {
        let action = Action::<String>::new();
        let trigger1 = action.trigger();
        let trigger2 = trigger1.clone();
        let listener1 = action.listener();
        let listener2 = listener1.clone();

        let received = Rc::new(RefCell::new(Vec::new()));
        let received_clone = received.clone();

        listener1.listen(move |data| {
            received_clone.borrow_mut().push(data);
        });

        // Both triggers should work
        trigger1.fire("from_trigger1".to_string());
        trigger2.fire("from_trigger2".to_string());

        let received_data = received.borrow();
        assert_eq!(received_data.len(), 2);
        assert!(received_data.contains(&"from_trigger1".to_string()));
        assert!(received_data.contains(&"from_trigger2".to_string()));
    }
}