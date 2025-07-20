//! Observable data structures for reactive programming patterns
//!
//! This crate provides traits and implementations for observable data with clear
//! ownership semantics. The owner holds a `DataSource` and can modify values,
//! while observers get `ObservableData` handles that can only read and observe.
//! For thread-safe modifications, use `DataSetter` instances that can be shared
//! across threads.
//!
//! # Example
//!
//! ```rust
//! use observable_data::{ObservableData, DataSource, DataSetter};
//!
//! // Owner creates and holds the data source
//! let data_source: DataSource<i32> = DataSource::new(42);
//!
//! // Create observer handles to distribute (read-only access)
//! let observer1: ObservableData<i32> = data_source.observer();
//! let observer2: ObservableData<i32> = data_source.observer();
//!
//! // Create setter handle for modifications
//! let setter = data_source.setter();
//!
//! println!("Current observer1 value: {}", observer1.get());
//! println!("Current observer2 value: {}", observer2.get());
//!
//! // Observers can read and observe, but not modify
//! observer1.observe(Box::new(|value: &i32| {
//!     println!("Observer 1 saw: {}", value);
//! }));
//!
//! observer2.observe(Box::new(|value: &i32| {
//!     println!("Observer 2 saw: {}", value);
//! }));
//!
//! // Use the setter to modify values (triggers all listeners)
//! setter.set(100);
//! ```

use std::rc::Rc;
use std::cell::RefCell;

/// Trait for setting data values.
/// This provides a way to update data without requiring mutable access to the DataSource.
pub trait DataSetter<T> {
    fn set(&self, value: T);
}

type Callback<T> = Box<dyn Fn(&T)>;

trait DataObserverTrait<T> {
    fn get(&self) -> T where T: Clone;
    fn listen(&self, callback: Callback<T>);
}

/// A handle to observable data that can be read and listened to.
/// This is the main type that gets distributed to observers.
pub struct DataObserver<T> {
    inner: Rc<dyn DataObserverTrait<T>>,
}

impl<T> DataObserver<T> {
    pub fn get(&self) -> T where T: Clone {
        self.inner.get()
    }

    pub fn observe(&self, callback: Callback<T>) {
        self.inner.listen(callback)
    }

    pub fn observe_now<F>(&self, callback: F) where T: Clone, F: Fn(&T) + 'static {
        let current_value = self.get();
        callback(&current_value);
        self.observe(Box::new(callback));
    }
}

impl<T> Clone for DataObserver<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct SharedData<T> {
    value: RefCell<T>,
    listeners: RefCell<Vec<Callback<T>>>,
}

pub struct DataSource<T> {
    data: Rc<SharedData<T>>,
}

/// Wrapper for DataSource that implements DataSetter
#[derive(Clone)]
pub struct DataSourceSetter<T> {
    data: Rc<SharedData<T>>,
}

impl<T: Clone> DataSetter<T> for DataSourceSetter<T> {
    fn set(&self, data: T) {
        // Update the value
        *self.data.value.borrow_mut() = data;
        
        // Notify all listeners
        let value = self.data.value.borrow();
        let listeners = self.data.listeners.borrow();
        for callback in listeners.iter() {
            callback(&*value);
        }
    }
}

impl<T: 'static + Clone> DataSource<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Rc::new(SharedData {
                value: RefCell::new(data),
                listeners: RefCell::new(Vec::new()),
            }),
        }
    }

    pub fn observer(&self) -> DataObserver<T> {
        DataObserver {
            inner: self.data.clone(),
        }
    }

    /// Create a setter that can be shared
    pub fn setter(&self) -> DataSourceSetter<T> {
        DataSourceSetter {
            data: self.data.clone(),
        }
    }
}

impl<T: Clone> DataObserverTrait<T> for SharedData<T> {
    fn get(&self) -> T {
        self.value.borrow().clone()
    }

    fn listen(&self, callback: Callback<T>) {
        self.listeners.borrow_mut().push(callback);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use wasm_bindgen_test::*;

    // No wasm_bindgen_test_configure! needed for Node.js

    #[wasm_bindgen_test]
    fn test_data_source_creation_and_get() {
        let data_source = DataSource::new(42);
        let observer: DataObserver<i32> = data_source.observer();
        assert_eq!(observer.get(), 42);
    }

    #[wasm_bindgen_test]
    fn test_data_source_set_updates_value() {
        let data_source = DataSource::new(10);
        let observer: DataObserver<i32> = data_source.observer();
        let setter = data_source.setter();
        
        setter.set(20);
        assert_eq!(observer.get(), 20);
        
        setter.set(30);
        assert_eq!(observer.get(), 30);
    }

    #[wasm_bindgen_test]
    fn test_multiple_observers_see_same_value() {
        let data_source = DataSource::new(100);
        let observer1: DataObserver<i32> = data_source.observer();
        let observer2: DataObserver<i32> = data_source.observer();
        let setter = data_source.setter();
        
        assert_eq!(observer1.get(), 100);
        assert_eq!(observer2.get(), 100);
        
        setter.set(200);
        assert_eq!(observer1.get(), 200);
        assert_eq!(observer2.get(), 200);
    }

    #[wasm_bindgen_test]
    fn test_listener_called_on_set() {
        let data_source = DataSource::new(1);
        let observer: DataObserver<i32> = data_source.observer();
        let setter = data_source.setter();
        
        // Track if callback was called
        let called = Rc::new(RefCell::new(false));
        let called_clone = called.clone();
        
        observer.observe(Box::new(move |_| {
            *called_clone.borrow_mut() = true;
        }));
        
        // Trigger update
        setter.set(2);
        
        // Verify listener was called
        assert!(*called.borrow());
    }

    #[wasm_bindgen_test]
    fn test_multiple_listeners_all_called() {
        let data_source = DataSource::new(0);
        let observer: DataObserver<i32> = data_source.observer();
        let setter = data_source.setter();
        
        let called1 = Rc::new(RefCell::new(false));
        let called2 = Rc::new(RefCell::new(false));
        
        let called1_clone = called1.clone();
        let called2_clone = called2.clone();
        
        observer.observe(Box::new(move |_| {
            *called1_clone.borrow_mut() = true;
        }));
        
        observer.observe(Box::new(move |_| {
            *called2_clone.borrow_mut() = true;
        }));
        
        setter.set(5);
        
        assert!(*called1.borrow());
        assert!(*called2.borrow());
    }

    #[wasm_bindgen_test]
    fn test_listeners_from_different_observers() {
        let data_source = DataSource::new(0);
        let observer1: DataObserver<i32> = data_source.observer();
        let observer2: DataObserver<i32> = data_source.observer();
        let setter = data_source.setter();
        
        let called1 = Rc::new(RefCell::new(false));
        let called2 = Rc::new(RefCell::new(false));
        
        let called1_clone = called1.clone();
        let called2_clone = called2.clone();
        
        observer1.observe(Box::new(move |_| {
            *called1_clone.borrow_mut() = true;
        }));
        
        observer2.observe(Box::new(move |_| {
            *called2_clone.borrow_mut() = true;
        }));
        
        setter.set(7);
        
        assert!(*called1.borrow());
        assert!(*called2.borrow());
    }

    #[wasm_bindgen_test]
    fn test_string_data_type() {
        let data_source = DataSource::new("hello".to_string());
        let observer: DataObserver<String> = data_source.observer();
        let setter = data_source.setter();
        
        let called = Rc::new(RefCell::new(false));
        let called_clone = called.clone();
        
        observer.observe(Box::new(move |_| {
            *called_clone.borrow_mut() = true;
        }));
        
        assert_eq!(observer.get(), "hello");
        
        setter.set("world".to_string());
        assert_eq!(observer.get(), "world");
        assert!(*called.borrow());
    }

    #[wasm_bindgen_test]
    fn test_observer_cloning() {
        let data_source = DataSource::new(0);
        let observer = data_source.observer();
        let setter = data_source.setter();
        
        // Verify we can clone observers
        let observer_clone = observer.clone();
        assert_eq!(observer_clone.get(), 0);
        
        // Verify both observers see updates
        setter.set(42);
        assert_eq!(observer.get(), 42);
        assert_eq!(observer_clone.get(), 42);
    }

    #[wasm_bindgen_test]
    fn test_observe_now() {
        let data_source = DataSource::new(10);
        let observer: DataObserver<i32> = data_source.observer();
        let setter = data_source.setter();
        
        let immediate_called = Rc::new(RefCell::new(false));
        let future_called = Rc::new(RefCell::new(false));
        
        let immediate_clone = immediate_called.clone();
        let future_clone = future_called.clone();
        
        // Use observe_now - should trigger immediately and listen for future
        observer.observe_now(move |value| {
            if *value == 10 {
                *immediate_clone.borrow_mut() = true;
            } else if *value == 20 {
                *future_clone.borrow_mut() = true;
            }
        });
        
        // Should have immediately been called with current value
        assert!(*immediate_called.borrow());
        assert!(!*future_called.borrow());
        
        // Trigger update for future listener
        setter.set(20);
        
        // Should now have been called for future value too
        assert!(*future_called.borrow());
    }

    #[wasm_bindgen_test]
    fn test_data_setter() {
        let data_source = DataSource::new(42);
        let observer: DataObserver<i32> = data_source.observer();
        let setter = data_source.setter();
        
        // Test initial value
        assert_eq!(observer.get(), 42);
        
        // Test setting via setter
        setter.set(100);
        assert_eq!(observer.get(), 100);
        
        // Test that listeners are called
        let called = Rc::new(RefCell::new(false));
        let called_clone = called.clone();
        
        observer.observe(Box::new(move |_| {
            *called_clone.borrow_mut() = true;
        }));
        
        setter.set(200);
        assert_eq!(observer.get(), 200);
        assert!(*called.borrow());
    }

    #[wasm_bindgen_test]
    fn test_data_setter_functionality() {
        let data_source = DataSource::new(0);
        let observer = data_source.observer();
        let setter = data_source.setter();
        
        // Verify setter works correctly
        setter.set(42);
        assert_eq!(observer.get(), 42);
        
        // Verify setter can be cloned and used
        let setter_clone = setter.clone();
        setter_clone.set(100);
        assert_eq!(observer.get(), 100);
    }
}