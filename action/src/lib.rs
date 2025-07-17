use std::sync::{Arc, Mutex};

type Callback<T> = Box<dyn Fn(T) + Send + Sync + 'static>;

/// Internal shared state for an action
struct SharedAction<T> {
    listeners: Mutex<Vec<Callback<T>>>,
}

impl<T> SharedAction<T> {
    fn new() -> Self {
        Self {
            listeners: Mutex::new(Vec::new()),
        }
    }

    fn add_listener(&self, callback: Callback<T>) {
        let mut listeners = self.listeners.lock().unwrap();
        listeners.push(callback);
    }

    fn fire(&self, data: T) 
    where 
        T: Clone,
    {
        let listeners = self.listeners.lock().unwrap();
        for callback in listeners.iter() {
            callback(data.clone());
        }
    }
}

/// Main Action factory that creates triggers and listeners
pub struct Action<T> {
    shared: Arc<SharedAction<T>>,
}

impl<T> Action<T> {
    /// Create a new action
    pub fn new() -> Self {
        Self {
            shared: Arc::new(SharedAction::new()),
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
    shared: Arc<SharedAction<T>>,
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
    shared: Arc<SharedAction<T>>,
}

impl<T> ActionListener<T> {
    /// Register a callback to be called when the action is fired
    pub fn listen<F>(&self, callback: F) 
    where 
        F: Fn(T) + Send + Sync + 'static,
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_action_with_data() {
        let action = Action::<String>::new();
        let trigger = action.trigger();
        let listener = action.listener();

        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();

        listener.listen(move |data| {
            received_clone.lock().unwrap().push(data);
        });

        trigger.fire("hello".to_string());
        trigger.fire("world".to_string());

        let received_data = received.lock().unwrap();
        assert_eq!(received_data.len(), 2);
        assert_eq!(received_data[0], "hello");
        assert_eq!(received_data[1], "world");
    }

    #[test]
    fn test_action_with_void() {
        let action = Action::<()>::new();
        let trigger = action.trigger();
        let listener = action.listener();

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        listener.listen(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        trigger.fire(());
        trigger.fire(());

        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_multiple_listeners() {
        let action = Action::<i32>::new();
        let trigger = action.trigger();
        let listener1 = action.listener();
        let listener2 = action.listener();

        let received1 = Arc::new(Mutex::new(Vec::new()));
        let received2 = Arc::new(Mutex::new(Vec::new()));
        let received1_clone = received1.clone();
        let received2_clone = received2.clone();

        listener1.listen(move |data| {
            received1_clone.lock().unwrap().push(data);
        });

        listener2.listen(move |data| {
            received2_clone.lock().unwrap().push(data * 2);
        });

        trigger.fire(5);

        assert_eq!(received1.lock().unwrap()[0], 5);
        assert_eq!(received2.lock().unwrap()[0], 10);
    }

    #[test]
    fn test_cloneable_handles() {
        let action = Action::<String>::new();
        let trigger1 = action.trigger();
        let trigger2 = trigger1.clone();
        let listener1 = action.listener();
        let listener2 = listener1.clone();

        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();

        listener1.listen(move |data| {
            received_clone.lock().unwrap().push(data);
        });

        // Both triggers should work
        trigger1.fire("from_trigger1".to_string());
        trigger2.fire("from_trigger2".to_string());

        let received_data = received.lock().unwrap();
        assert_eq!(received_data.len(), 2);
        assert!(received_data.contains(&"from_trigger1".to_string()));
        assert!(received_data.contains(&"from_trigger2".to_string()));
    }
}