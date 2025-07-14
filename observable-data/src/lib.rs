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

use std::sync::{Arc, RwLock, Mutex};

/// Trait for setting data values in a thread-safe manner.
/// This provides a way to update data without requiring mutable access to the DataSource.
pub trait DataSetter<T>: Send + Sync {
    fn set(&self, value: T);
}

type Callback<T> = Box<dyn Fn(&T) + Send + Sync>;

trait ObservableDataTrait<T>: Send + Sync {
    fn get(&self) -> T where T: Clone;
    fn listen(&self, callback: Callback<T>);
}

/// A thread-safe handle to observable data that can be read and listened to.
/// This is the main type that gets distributed to observers.
pub struct ObservableData<T> {
    inner: Arc<dyn ObservableDataTrait<T>>,
}

impl<T> ObservableData<T> {
    pub fn get(&self) -> T where T: Clone {
        self.inner.get()
    }

    pub fn observe(&self, callback: Callback<T>) {
        self.inner.listen(callback)
    }

    pub fn observe_now<F>(&self, callback: F) where T: Clone, F: Fn(&T) + Send + Sync + 'static {
        let current_value = self.get();
        callback(&current_value);
        self.observe(Box::new(callback));
    }
}

impl<T> Clone for ObservableData<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct SharedData<T> {
    value: RwLock<T>,
    listeners: Mutex<Vec<Callback<T>>>,
}

pub struct DataSource<T> {
    data: Arc<SharedData<T>>,
}

/// Thread-safe wrapper for DataSource that implements DataSetter
#[derive(Clone)]
pub struct DataSourceSetter<T> {
    data: Arc<SharedData<T>>,
}

impl<T: Clone + Send + Sync> DataSetter<T> for DataSourceSetter<T> {
    fn set(&self, data: T) {
        // Update the value
        {
            let mut value = self.data.value.write().unwrap();
            *value = data;
        }
        
        // Notify all listeners
        let value = self.data.value.read().unwrap();
        let listeners = self.data.listeners.lock().unwrap();
        for callback in listeners.iter() {
            callback(&*value);
        }
    }
}

impl<T: 'static + Clone + Send + Sync> DataSource<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(SharedData {
                value: RwLock::new(data),
                listeners: Mutex::new(Vec::new()),
            }),
        }
    }

    pub fn observer(&self) -> ObservableData<T> {
        ObservableData {
            inner: self.data.clone(),
        }
    }

    /// Create a thread-safe setter that can be shared across threads
    pub fn setter(&self) -> DataSourceSetter<T> {
        DataSourceSetter {
            data: self.data.clone(),
        }
    }
}

impl<T: Clone + Send + Sync> ObservableDataTrait<T> for SharedData<T> {
    fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    fn listen(&self, callback: Callback<T>) {
        self.listeners.lock().unwrap().push(callback);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use wasm_bindgen_test::*;

    // No wasm_bindgen_test_configure! needed for Node.js

    #[wasm_bindgen_test]
    fn test_data_source_creation_and_get() {
        let data_source = DataSource::new(42);
        let observer: ObservableData<i32> = data_source.observer();
        assert_eq!(observer.get(), 42);
    }

    #[wasm_bindgen_test]
    fn test_data_source_set_updates_value() {
        let data_source = DataSource::new(10);
        let observer: ObservableData<i32> = data_source.observer();
        let setter = data_source.setter();
        
        setter.set(20);
        assert_eq!(observer.get(), 20);
        
        setter.set(30);
        assert_eq!(observer.get(), 30);
    }

    #[wasm_bindgen_test]
    fn test_multiple_observers_see_same_value() {
        let data_source = DataSource::new(100);
        let observer1: ObservableData<i32> = data_source.observer();
        let observer2: ObservableData<i32> = data_source.observer();
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
        let observer: ObservableData<i32> = data_source.observer();
        let setter = data_source.setter();
        
        // Track if callback was called
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();
        
        observer.observe(Box::new(move |_| {
            *called_clone.lock().unwrap() = true;
        }));
        
        // Trigger update
        setter.set(2);
        
        // Verify listener was called
        assert!(*called.lock().unwrap());
    }

    #[wasm_bindgen_test]
    fn test_multiple_listeners_all_called() {
        let data_source = DataSource::new(0);
        let observer: ObservableData<i32> = data_source.observer();
        let setter = data_source.setter();
        
        let called1 = Arc::new(Mutex::new(false));
        let called2 = Arc::new(Mutex::new(false));
        
        let called1_clone = called1.clone();
        let called2_clone = called2.clone();
        
        observer.observe(Box::new(move |_| {
            *called1_clone.lock().unwrap() = true;
        }));
        
        observer.observe(Box::new(move |_| {
            *called2_clone.lock().unwrap() = true;
        }));
        
        setter.set(5);
        
        assert!(*called1.lock().unwrap());
        assert!(*called2.lock().unwrap());
    }

    #[wasm_bindgen_test]
    fn test_listeners_from_different_observers() {
        let data_source = DataSource::new(0);
        let observer1: ObservableData<i32> = data_source.observer();
        let observer2: ObservableData<i32> = data_source.observer();
        let setter = data_source.setter();
        
        let called1 = Arc::new(Mutex::new(false));
        let called2 = Arc::new(Mutex::new(false));
        
        let called1_clone = called1.clone();
        let called2_clone = called2.clone();
        
        observer1.observe(Box::new(move |_| {
            *called1_clone.lock().unwrap() = true;
        }));
        
        observer2.observe(Box::new(move |_| {
            *called2_clone.lock().unwrap() = true;
        }));
        
        setter.set(7);
        
        assert!(*called1.lock().unwrap());
        assert!(*called2.lock().unwrap());
    }

    #[wasm_bindgen_test]
    fn test_string_data_type() {
        let data_source = DataSource::new("hello".to_string());
        let observer: ObservableData<String> = data_source.observer();
        let setter = data_source.setter();
        
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();
        
        observer.observe(Box::new(move |_| {
            *called_clone.lock().unwrap() = true;
        }));
        
        assert_eq!(observer.get(), "hello");
        
        setter.set("world".to_string());
        assert_eq!(observer.get(), "world");
        assert!(*called.lock().unwrap());
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
        let observer: ObservableData<i32> = data_source.observer();
        let setter = data_source.setter();
        
        let immediate_called = Arc::new(Mutex::new(false));
        let future_called = Arc::new(Mutex::new(false));
        
        let immediate_clone = immediate_called.clone();
        let future_clone = future_called.clone();
        
        // Use observe_now - should trigger immediately and listen for future
        observer.observe_now(move |value| {
            if *value == 10 {
                *immediate_clone.lock().unwrap() = true;
            } else if *value == 20 {
                *future_clone.lock().unwrap() = true;
            }
        });
        
        // Should have immediately been called with current value
        assert!(*immediate_called.lock().unwrap());
        assert!(!*future_called.lock().unwrap());
        
        // Trigger update for future listener
        setter.set(20);
        
        // Should now have been called for future value too
        assert!(*future_called.lock().unwrap());
    }

    #[wasm_bindgen_test]
    fn test_data_setter() {
        let data_source = DataSource::new(42);
        let observer: ObservableData<i32> = data_source.observer();
        let setter = data_source.setter();
        
        // Test initial value
        assert_eq!(observer.get(), 42);
        
        // Test setting via setter
        setter.set(100);
        assert_eq!(observer.get(), 100);
        
        // Test that listeners are called
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();
        
        observer.observe(Box::new(move |_| {
            *called_clone.lock().unwrap() = true;
        }));
        
        setter.set(200);
        assert_eq!(observer.get(), 200);
        assert!(*called.lock().unwrap());
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