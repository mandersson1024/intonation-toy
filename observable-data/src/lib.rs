//! Observable data structures for reactive programming patterns
//!
//! This crate provides traits and implementations for observable data with clear
//! ownership semantics. The owner holds a `DataSource` and can modify values,
//! while observers get `ObservableData` handles that can only read and listen.
//!
//! # Example
//!
//! ```rust
//! use observable_data::{ObservableData, DataSource};
//! use std::rc::Rc;
//!
//! // Owner creates and holds the data source
//! let mut data_source: DataSource<i32> = DataSource::new(42);
//!
//! // Create observer handles to distribute (read-only access)
//! let observer1: Rc<dyn ObservableData<i32>> = data_source.observer();
//! let observer2: Rc<dyn ObservableData<i32>> = data_source.observer();
//!
//! // Observers can read and listen, but not modify
//! observer1.listen(Box::new(|value: &i32| {
//!     println!("Observer 1 saw: {}", value);
//! }));
//!
//! observer2.listen(Box::new(|value: &i32| {
//!     println!("Observer 2 saw: {}", value);
//! }));
//!
//! println!("Current value: {}", observer2.get());
//!
//! // Only the owner can modify (triggers all listeners)
//! data_source.set(100);
//! ```

use std::cell::RefCell;
use std::rc::Rc;

type Callback<T> = Box<dyn Fn(&T)>;

pub trait ObservableData<T> {
    fn get(&self) -> &T;
    fn listen(&self, callback: Callback<T>);
}

/// Internal shared state between DataSource owner and observer handles.
/// 
/// This struct is crucial for the ownership model because it:
/// - Encapsulates the actual data and listener storage
/// - Enables safe sharing via Rc (reference counting)
/// - Uses RefCell for interior mutability without requiring &mut self
/// - Remains private to prevent external access to mutation methods
struct SharedData<T> {
    value: RefCell<T>,
    listeners: RefCell<Vec<Callback<T>>>,
}

/// The owner of the data - can create observers and modify values.
/// 
/// Key design properties:
/// - Does NOT implement ObservableData (cannot read/listen directly)
/// - Only provides mutation via set() and observer creation via observer()
/// - Enforces single ownership pattern for write access
pub struct DataSource<T> {
    data: Rc<SharedData<T>>,
}

impl<T: 'static> DataSource<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            data: Rc::new(SharedData {
                value: RefCell::new(initial_value),
                listeners: RefCell::new(Vec::new()),
            }),
        }
    }

    pub fn set(&mut self, data: T) {
        *self.data.value.borrow_mut() = data;
        
        // Notify all listeners
        let value_ref = self.data.value.borrow();
        for callback in self.data.listeners.borrow().iter() {
            callback(&*value_ref);
        }
    }

    pub fn observer(&self) -> Rc<dyn ObservableData<T>> {
        self.data.clone()
    }
}

impl<T> ObservableData<T> for SharedData<T> {
    fn get(&self) -> &T {
        unsafe { &*self.value.as_ptr() }
    }

    fn listen(&self, callback: Callback<T>) {
        self.listeners.borrow_mut().push(callback);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn test_data_source_creation_and_get() {
        let data_source = DataSource::new(42);
        let observer = data_source.observer();
        assert_eq!(*observer.get(), 42);
    }

    #[test]
    fn test_data_source_set_updates_value() {
        let mut data_source = DataSource::new(10);
        let observer = data_source.observer();
        
        data_source.set(20);
        assert_eq!(*observer.get(), 20);
        
        data_source.set(30);
        assert_eq!(*observer.get(), 30);
    }

    #[test]
    fn test_multiple_observers_see_same_value() {
        let mut data_source = DataSource::new(100);
        let observer1 = data_source.observer();
        let observer2 = data_source.observer();
        
        assert_eq!(*observer1.get(), 100);
        assert_eq!(*observer2.get(), 100);
        
        data_source.set(200);
        assert_eq!(*observer1.get(), 200);
        assert_eq!(*observer2.get(), 200);
    }

    #[test]
    fn test_listener_called_on_set() {
        let mut data_source = DataSource::new(1);
        let observer = data_source.observer();
        
        let received_values = Rc::new(RefCell::new(Vec::new()));
        let values_clone = received_values.clone();
        
        observer.listen(Box::new(move |value| {
            values_clone.borrow_mut().push(*value);
        }));
        
        data_source.set(2);
        data_source.set(3);
        data_source.set(4);
        
        assert_eq!(*received_values.borrow(), vec![2, 3, 4]);
    }

    #[test]
    fn test_multiple_listeners_all_called() {
        let mut data_source = DataSource::new(0);
        let observer = data_source.observer();
        
        let values1 = Rc::new(RefCell::new(Vec::new()));
        let values2 = Rc::new(RefCell::new(Vec::new()));
        
        let values1_clone = values1.clone();
        let values2_clone = values2.clone();
        
        observer.listen(Box::new(move |value| {
            values1_clone.borrow_mut().push(*value);
        }));
        
        observer.listen(Box::new(move |value| {
            values2_clone.borrow_mut().push(*value);
        }));
        
        data_source.set(5);
        data_source.set(10);
        
        assert_eq!(*values1.borrow(), vec![5, 10]);
        assert_eq!(*values2.borrow(), vec![5, 10]);
    }

    #[test]
    fn test_listeners_from_different_observers() {
        let mut data_source = DataSource::new(0);
        let observer1 = data_source.observer();
        let observer2 = data_source.observer();
        
        let values1 = Rc::new(RefCell::new(Vec::new()));
        let values2 = Rc::new(RefCell::new(Vec::new()));
        
        let values1_clone = values1.clone();
        let values2_clone = values2.clone();
        
        observer1.listen(Box::new(move |value| {
            values1_clone.borrow_mut().push(*value);
        }));
        
        observer2.listen(Box::new(move |value| {
            values2_clone.borrow_mut().push(*value);
        }));
        
        data_source.set(7);
        
        assert_eq!(*values1.borrow(), vec![7]);
        assert_eq!(*values2.borrow(), vec![7]);
    }

    #[test]
    fn test_string_data_type() {
        let mut data_source = DataSource::new("hello".to_string());
        let observer = data_source.observer();
        
        let received_values = Rc::new(RefCell::new(Vec::new()));
        let values_clone = received_values.clone();
        
        observer.listen(Box::new(move |value| {
            values_clone.borrow_mut().push(value.clone());
        }));
        
        assert_eq!(observer.get(), "hello");
        
        data_source.set("world".to_string());
        assert_eq!(observer.get(), "world");
        assert_eq!(*received_values.borrow(), vec!["world".to_string()]);
    }
}