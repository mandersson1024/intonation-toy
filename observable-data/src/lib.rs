//! Observable data structures for reactive programming patterns
//!
//! This crate provides traits and implementations for observable data that can
//! notify listeners when values change.
//!
//! # Example
//!
//! ```rust
//! use observable_data::{ObservableData, DataSource};
//!
//! // Create a data source with initial value
//! let data_source = DataSource::new(42);
//!
//! // Add a listener that gets called when the value changes
//! data_source.listen(Box::new(|value| {
//!     println!("Value changed to: {}", value);
//! }));
//!
//! // Get the current value
//! println!("Current value: {}", data_source.get());
//!
//! // Update the value (triggers all listeners)
//! data_source.set(100);
//! ```

use std::cell::RefCell;
use std::rc::Rc;

type Callback<T> = Box<dyn Fn(&T)>;

pub trait ObservableData<T> {
    fn get(&self) -> &T;
    fn listen(&self, callback: Callback<T>);
}

pub struct DataSource<T> {
    value: RefCell<T>,
    listeners: RefCell<Vec<Callback<T>>>,
}

impl<T> DataSource<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            value: RefCell::new(initial_value),
            listeners: RefCell::new(Vec::new()),
        }
    }

    pub fn set(&self, data: T) {
        *self.value.borrow_mut() = data;
        
        // Notify all listeners
        let value_ref = self.value.borrow();
        for callback in self.listeners.borrow().iter() {
            callback(&*value_ref);
        }
    }
}

impl<T> ObservableData<T> for DataSource<T> {
    fn get(&self) -> &T {
        unsafe { &*self.value.as_ptr() }
    }

    fn listen(&self, callback: Callback<T>) {
        self.listeners.borrow_mut().push(callback);
    }
}