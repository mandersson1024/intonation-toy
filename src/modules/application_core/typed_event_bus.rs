//! # Typed Event Bus Implementation
//!
//! This module implements a type-safe event bus system with priority-based event processing,
//! compile-time type safety, efficient event routing, and automatic handler lifecycle management.
//!
//! The TypedEventBus provides compile-time guarantees that event handlers can only be
//! registered for compatible event types, preventing runtime type errors and ensuring
//! type-safe event processing throughout the application.

use super::event_bus::{
    Event, EventBus, EventBusError, EventBusMetrics, EventBusState,
    EventHandler, EventPriority, SubscriptionId, HandlerInfo, get_timestamp_ns
};
use super::performance_monitor::{EventBusPerformanceMonitor, MonitorConfig};
use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Default queue capacity per priority level
const DEFAULT_QUEUE_CAPACITY: usize = 10000;

/// Maximum queue capacity per priority level 
const MAX_QUEUE_CAPACITY: usize = 50000;

/// Processing interval for the event loop (microseconds)
const PROCESSING_INTERVAL_US: u64 = 100;

/// Type-safe event container with proper handler invocation
struct TypedEventContainer {
    event: Box<dyn Any + Send + Sync>,
    type_id: TypeId,
    event_type_name: &'static str,
    timestamp: u64,
    priority: EventPriority,
    processor: Box<dyn Fn(&dyn Any, &mut HashMap<TypeId, Vec<TypedHandlerContainer>>) -> Result<(), EventBusError> + Send + Sync>,
}

impl TypedEventContainer {
    fn new<T: Event + 'static>(event: T) -> Self {
        let timestamp = event.timestamp();
        let priority = event.priority();
        let event_type_name = event.event_type();
        
        // Create processor closure that can safely handle the event
        let processor = Box::new(move |event_any: &dyn Any, handlers: &mut HashMap<TypeId, Vec<TypedHandlerContainer>>| -> Result<(), EventBusError> {
            if let Some(typed_event) = event_any.downcast_ref::<T>() {
                if let Some(type_handlers) = handlers.get_mut(&TypeId::of::<T>()) {
                    for handler_container in type_handlers.iter_mut() {
                        if let Some(handler) = handler_container.handler.downcast_mut::<Box<dyn EventHandler<T>>>() {
                            if let Err(e) = handler.handle_event(typed_event) {
                                return Err(EventBusError::ProcessingFailed(format!("Handler error: {}", e)));
                            }
                        }
                    }
                }
            }
            Ok(())
        });
        
        Self {
            event: Box::new(event),
            type_id: TypeId::of::<T>(),
            event_type_name,
            timestamp,
            priority,
            processor,
        }
    }
}

/// Type-safe handler container with proper lifecycle management
struct TypedHandlerContainer {
    handler: Box<dyn Any + Send + Sync>,
    type_id: TypeId,
    subscription_id: SubscriptionId,
    handler_info: HandlerInfo,
}

impl TypedHandlerContainer {
    fn new<T: Event + 'static>(handler: Box<dyn EventHandler<T>>, subscription_id: SubscriptionId) -> Self {
        let handler_info = handler.handler_info();
        Self {
            handler: Box::new(handler),
            type_id: TypeId::of::<T>(),
            subscription_id,
            handler_info,
        }
    }
}


/// Priority queue implementation for events
struct EventQueues {
    critical: VecDeque<TypedEventContainer>,
    high: VecDeque<TypedEventContainer>,
    normal: VecDeque<TypedEventContainer>,
    low: VecDeque<TypedEventContainer>,
    capacity: usize,
}

impl EventQueues {
    fn new(capacity: usize) -> Self {
        Self {
            critical: VecDeque::with_capacity(capacity),
            high: VecDeque::with_capacity(capacity),
            normal: VecDeque::with_capacity(capacity),
            low: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    fn push_event(&mut self, event: TypedEventContainer) -> Result<(), EventBusError> {
        let queue = match event.priority {
            EventPriority::Critical => &mut self.critical,
            EventPriority::High => &mut self.high,
            EventPriority::Normal => &mut self.normal,
            EventPriority::Low => &mut self.low,
        };
        
        if queue.len() >= self.capacity {
            return Err(EventBusError::QueueFull);
        }
        
        queue.push_back(event);
        Ok(())
    }
    
    fn pop_next_event(&mut self) -> Option<TypedEventContainer> {
        // Process in priority order: Critical -> High -> Normal -> Low
        if let Some(event) = self.critical.pop_front() {
            return Some(event);
        }
        if let Some(event) = self.high.pop_front() {
            return Some(event);
        }
        if let Some(event) = self.normal.pop_front() {
            return Some(event);
        }
        self.low.pop_front()
    }
    
    fn get_depths(&self) -> [usize; 4] {
        [
            self.critical.len(),
            self.high.len(),
            self.normal.len(),
            self.low.len(),
        ]
    }
    
    fn total_events(&self) -> usize {
        self.critical.len() + self.high.len() + self.normal.len() + self.low.len()
    }
}

/// Module registration information for tracking
#[derive(Debug, Clone)]
pub struct ModuleRegistration {
    pub module_name: String,
    pub subscription_ids: Vec<SubscriptionId>,
    pub registered_event_types: Vec<String>,
    pub registration_timestamp: u64,
}

/// Thread-safe typed event bus implementation with compile-time type safety
pub struct TypedEventBus {
    state: Arc<RwLock<EventBusState>>,
    queues: Arc<Mutex<EventQueues>>,
    handlers: Arc<RwLock<HashMap<TypeId, Vec<TypedHandlerContainer>>>>,
    metrics: Arc<RwLock<EventBusMetrics>>,
    next_subscription_id: Arc<Mutex<u64>>,
    processing_thread: Option<thread::JoinHandle<()>>,
    stop_signal: Arc<Mutex<bool>>,
    modules: Arc<RwLock<HashMap<String, ModuleRegistration>>>,
    performance_monitor: Arc<EventBusPerformanceMonitor>,
}

impl TypedEventBus {
    /// Creates a new typed event bus with default settings
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_QUEUE_CAPACITY)
    }
    
    /// Creates a new typed event bus with specified queue capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.min(MAX_QUEUE_CAPACITY);
        
        Self {
            state: Arc::new(RwLock::new(EventBusState::Stopped)),
            queues: Arc::new(Mutex::new(EventQueues::new(capacity))),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(EventBusMetrics::default())),
            next_subscription_id: Arc::new(Mutex::new(1)),
            processing_thread: None,
            stop_signal: Arc::new(Mutex::new(false)),
            modules: Arc::new(RwLock::new(HashMap::new())),
            performance_monitor: Arc::new(EventBusPerformanceMonitor::new()),
        }
    }
    
    /// Creates a new typed event bus with custom performance monitoring configuration
    pub fn with_monitor_config(capacity: usize, monitor_config: MonitorConfig) -> Self {
        let capacity = capacity.min(MAX_QUEUE_CAPACITY);
        
        Self {
            state: Arc::new(RwLock::new(EventBusState::Stopped)),
            queues: Arc::new(Mutex::new(EventQueues::new(capacity))),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(EventBusMetrics::default())),
            next_subscription_id: Arc::new(Mutex::new(1)),
            processing_thread: None,
            stop_signal: Arc::new(Mutex::new(false)),
            modules: Arc::new(RwLock::new(HashMap::new())),
            performance_monitor: Arc::new(EventBusPerformanceMonitor::with_config(monitor_config)),
        }
    }
    
    /// Gets access to the performance monitor for configuration and querying
    pub fn get_performance_monitor(&self) -> Arc<EventBusPerformanceMonitor> {
        Arc::clone(&self.performance_monitor)
    }
    
    /// Registers a module with the typed event bus
    pub fn register_module<T: Event + 'static>(
        &mut self, 
        module_name: String,
        handlers: Vec<Box<dyn EventHandler<T>>>
    ) -> Result<ModuleRegistration, EventBusError> {
        let mut subscription_ids = Vec::new();
        let mut registered_event_types = Vec::new();
        
        // Subscribe all handlers
        for handler in handlers {
            let subscription_id = self.subscribe::<T>(handler)?;
            subscription_ids.push(subscription_id);
            registered_event_types.push(std::any::type_name::<T>().to_string());
        }
        
        let registration = ModuleRegistration {
            module_name: module_name.clone(),
            subscription_ids,
            registered_event_types,
            registration_timestamp: get_timestamp_ns(),
        };
        
        // Store module registration
        {
            let mut modules = self.modules.write().map_err(|_| EventBusError::Internal("Modules lock poisoned".to_string()))?;
            modules.insert(module_name, registration.clone());
        }
        
        Ok(registration)
    }
    
    /// Unregisters a module and cleans up all its subscriptions
    pub fn unregister_module(&mut self, module_name: &str) -> Result<(), EventBusError> {
        let registration = {
            let mut modules = self.modules.write().map_err(|_| EventBusError::Internal("Modules lock poisoned".to_string()))?;
            modules.remove(module_name)
        };
        
        if let Some(registration) = registration {
            // Unsubscribe all handlers for this module
            for subscription_id in registration.subscription_ids {
                self.unsubscribe(subscription_id)?;
            }
        }
        
        Ok(())
    }
    
    /// Gets information about registered modules
    pub fn get_registered_modules(&self) -> Result<Vec<ModuleRegistration>, EventBusError> {
        let modules = self.modules.read().map_err(|_| EventBusError::Internal("Modules lock poisoned".to_string()))?;
        Ok(modules.values().cloned().collect())
    }
    
    /// Updates latency metrics for a priority level
    fn update_latency_metrics(&self, priority: EventPriority, latency_ns: u64) -> Result<(), EventBusError> {
        let mut metrics = self.metrics.write().map_err(|_| EventBusError::Internal("Metrics lock poisoned".to_string()))?;
        let priority_index = priority.as_usize();
        
        // Simple moving average update
        let current_avg = metrics.avg_latency_by_priority[priority_index];
        metrics.avg_latency_by_priority[priority_index] = if current_avg == 0 {
            latency_ns
        } else {
            (current_avg + latency_ns) / 2
        };
        
        Ok(())
    }
    
    /// Updates queue depth metrics
    fn update_queue_metrics(&self) -> Result<(), EventBusError> {
        let queues = self.queues.lock().map_err(|_| EventBusError::Internal("Queue lock poisoned".to_string()))?;
        let mut metrics = self.metrics.write().map_err(|_| EventBusError::Internal("Metrics lock poisoned".to_string()))?;
        
        metrics.queue_depths = queues.get_depths();
        
        Ok(())
    }
    
    /// Main event processing loop with proper type-safe handler invocation
    fn event_processing_loop(
        queues: Arc<Mutex<EventQueues>>,
        handlers: Arc<RwLock<HashMap<TypeId, Vec<TypedHandlerContainer>>>>,
        metrics: Arc<RwLock<EventBusMetrics>>,
        stop_signal: Arc<Mutex<bool>>,
        state: Arc<RwLock<EventBusState>>,
        performance_monitor: Arc<EventBusPerformanceMonitor>,
    ) {
        let processing_interval = Duration::from_micros(PROCESSING_INTERVAL_US);
        
        loop {
            // Check stop signal
            {
                let stop = stop_signal.lock().unwrap();
                if *stop {
                    break;
                }
            }
            
            // Process next event
            let event_opt = {
                let mut queues = queues.lock().unwrap();
                queues.pop_next_event()
            };
            
            if let Some(event) = event_opt {
                let start_time = Instant::now();
                
                // Process the event with appropriate handlers
                let processing_success = {
                    let mut handlers = handlers.write().unwrap();
                    match (event.processor)(event.event.as_ref(), &mut *handlers) {
                        Ok(()) => true,
                        Err(e) => {
                            // Log processing error (in a real implementation, you'd use proper logging)
                            eprintln!("Event processing error for {}: {:?}", event.event_type_name, e);
                            false
                        }
                    }
                };
                
                let processing_time = start_time.elapsed().as_nanos() as u64;
                
                // Record detailed event processing metrics
                if let Err(e) = performance_monitor.record_event_processing(
                    event.event_type_name,
                    processing_time,
                    processing_success
                ) {
                    eprintln!("Failed to record event processing metrics: {}", e);
                }
                
                // Update base metrics
                {
                    let mut metrics = metrics.write().unwrap();
                    let priority_index = event.priority.as_usize();
                    let current_avg = metrics.avg_latency_by_priority[priority_index];
                    metrics.avg_latency_by_priority[priority_index] = if current_avg == 0 {
                        processing_time
                    } else {
                        (current_avg + processing_time) / 2
                    };
                    metrics.total_events_processed += 1;
                    
                    // Update error counts if processing failed
                    if !processing_success {
                        let error_count = metrics.error_counts
                            .entry(event.event_type_name.to_string())
                            .or_insert(0);
                        *error_count += 1;
                    }
                }
            } else {
                // No events to process, sleep briefly
                thread::sleep(processing_interval);
            }
            
            // Update queue depth metrics and push to performance monitor
            {
                let queues = queues.lock().unwrap();
                let mut metrics = metrics.write().unwrap();
                metrics.queue_depths = queues.get_depths();
                
                // Update performance monitor with current metrics
                if let Err(e) = performance_monitor.update_metrics(metrics.clone()) {
                    eprintln!("Failed to update performance monitor metrics: {}", e);
                }
            }
        }
        
        // Set state to stopped
        {
            let mut state = state.write().unwrap();
            *state = EventBusState::Stopped;
        }
    }
}

impl EventBus for TypedEventBus {
    fn publish<T: Event + 'static>(&self, event: T) -> Result<(), EventBusError> {
        // Check if bus is running
        {
            let state = self.state.read().map_err(|_| EventBusError::Internal("State lock poisoned".to_string()))?;
            if *state != EventBusState::Running {
                return Err(EventBusError::NotRunning);
            }
        }
        
        let typed_event = TypedEventContainer::new(event);
        
        // Queue the event
        {
            let mut queues = self.queues.lock().map_err(|_| EventBusError::Internal("Queue lock poisoned".to_string()))?;
            queues.push_event(typed_event)?;
        }
        
        // Update metrics
        self.update_queue_metrics()?;
        
        Ok(())
    }
    
    fn subscribe<T: Event + 'static>(&mut self, handler: Box<dyn EventHandler<T>>) -> Result<SubscriptionId, EventBusError> {
        let subscription_id = {
            let mut next_id = self.next_subscription_id.lock().map_err(|_| EventBusError::Internal("ID lock poisoned".to_string()))?;
            let id = SubscriptionId::new(*next_id);
            *next_id += 1;
            id
        };
        
        let typed_handler = TypedHandlerContainer::new(handler, subscription_id);
        
        {
            let mut handlers = self.handlers.write().map_err(|_| EventBusError::Internal("Handlers lock poisoned".to_string()))?;
            let type_handlers = handlers.entry(TypeId::of::<T>()).or_insert_with(Vec::new);
            type_handlers.push(typed_handler);
        }
        
        // Update subscription count
        {
            let mut metrics = self.metrics.write().map_err(|_| EventBusError::Internal("Metrics lock poisoned".to_string()))?;
            metrics.active_subscriptions += 1;
        }
        
        Ok(subscription_id)
    }
    
    fn unsubscribe(&mut self, subscription_id: SubscriptionId) -> Result<(), EventBusError> {
        let mut handlers = self.handlers.write().map_err(|_| EventBusError::Internal("Handlers lock poisoned".to_string()))?;
        
        for type_handlers in handlers.values_mut() {
            if let Some(index) = type_handlers.iter().position(|h| h.subscription_id == subscription_id) {
                type_handlers.remove(index);
                
                // Update subscription count
                {
                    let mut metrics = self.metrics.write().map_err(|_| EventBusError::Internal("Metrics lock poisoned".to_string()))?;
                    metrics.active_subscriptions = metrics.active_subscriptions.saturating_sub(1);
                }
                
                return Ok(());
            }
        }
        
        Err(EventBusError::InvalidSubscription)
    }
    
    fn get_metrics(&self) -> EventBusMetrics {
        self.metrics.read().unwrap().clone()
    }
    
    fn start(&mut self) -> Result<(), EventBusError> {
        {
            let mut state = self.state.write().map_err(|_| EventBusError::Internal("State lock poisoned".to_string()))?;
            if *state == EventBusState::Running {
                return Err(EventBusError::AlreadyRunning);
            }
            *state = EventBusState::Starting;
        }
        
        // Reset stop signal
        {
            let mut stop = self.stop_signal.lock().map_err(|_| EventBusError::Internal("Stop signal lock poisoned".to_string()))?;
            *stop = false;
        }
        
        // Start processing thread
        let queues = Arc::clone(&self.queues);
        let handlers = Arc::clone(&self.handlers);
        let metrics = Arc::clone(&self.metrics);
        let stop_signal = Arc::clone(&self.stop_signal);
        let state = Arc::clone(&self.state);
        let performance_monitor = Arc::clone(&self.performance_monitor);
        
        let processing_thread = thread::spawn(move || {
            Self::event_processing_loop(queues, handlers, metrics, stop_signal, state, performance_monitor);
        });
        
        self.processing_thread = Some(processing_thread);
        
        // Set state to running
        {
            let mut state = self.state.write().map_err(|_| EventBusError::Internal("State lock poisoned".to_string()))?;
            *state = EventBusState::Running;
        }
        
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), EventBusError> {
        {
            let mut state = self.state.write().map_err(|_| EventBusError::Internal("State lock poisoned".to_string()))?;
            if *state != EventBusState::Running {
                return Err(EventBusError::NotRunning);
            }
            *state = EventBusState::Stopping;
        }
        
        // Signal stop
        {
            let mut stop = self.stop_signal.lock().map_err(|_| EventBusError::Internal("Stop signal lock poisoned".to_string()))?;
            *stop = true;
        }
        
        // Wait for processing thread to finish
        if let Some(thread_handle) = self.processing_thread.take() {
            thread_handle.join().map_err(|_| EventBusError::Internal("Failed to join processing thread".to_string()))?;
        }
        
        Ok(())
    }
    
    fn state(&self) -> EventBusState {
        self.state.read().unwrap().clone()
    }
}

impl Default for TypedEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Ensure TypedEventBus is Send and Sync
unsafe impl Send for TypedEventBus {}
unsafe impl Sync for TypedEventBus {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    
    #[derive(Debug, Clone)]
    struct TestEvent {
        id: u32,
        priority: EventPriority,
        timestamp: u64,
    }
    
    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }
        
        fn timestamp(&self) -> u64 {
            self.timestamp
        }
        
        fn priority(&self) -> EventPriority {
            self.priority
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[derive(Debug, Clone)]
    struct AnotherTestEvent {
        data: String,
        timestamp: u64,
    }
    
    impl Event for AnotherTestEvent {
        fn event_type(&self) -> &'static str {
            "AnotherTestEvent"
        }
        
        fn timestamp(&self) -> u64 {
            self.timestamp
        }
        
        fn priority(&self) -> EventPriority {
            EventPriority::Normal
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    struct TestHandler {
        processed_count: Arc<AtomicU32>,
        expected_id: Option<u32>,
    }
    
    impl EventHandler<TestEvent> for TestHandler {
        fn handle_event(&mut self, event: &TestEvent) -> Result<(), Box<dyn std::error::Error>> {
            if let Some(expected) = self.expected_id {
                assert_eq!(event.id, expected);
            }
            self.processed_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    
    struct AnotherTestHandler {
        processed_count: Arc<AtomicU32>,
    }
    
    impl EventHandler<AnotherTestEvent> for AnotherTestHandler {
        fn handle_event(&mut self, event: &AnotherTestEvent) -> Result<(), Box<dyn std::error::Error>> {
            assert!(!event.data.is_empty());
            self.processed_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    
    #[test]
    fn test_typed_event_bus_creation() {
        let bus = TypedEventBus::new();
        assert_eq!(bus.state(), EventBusState::Stopped);
    }
    
    #[test]
    fn test_subscription_management() {
        let mut bus = TypedEventBus::new();
        let processed_count = Arc::new(AtomicU32::new(0));
        
        let handler = TestHandler {
            processed_count: Arc::clone(&processed_count),
            expected_id: None,
        };
        
        let subscription_id = bus.subscribe::<TestEvent>(Box::new(handler)).unwrap();
        assert!(subscription_id.as_u64() > 0);
        
        let result = bus.unsubscribe(subscription_id);
        assert!(result.is_ok());
        
        // Unsubscribing again should fail
        let result = bus.unsubscribe(subscription_id);
        assert!(matches!(result, Err(EventBusError::InvalidSubscription)));
    }
    
    #[test]
    fn test_module_registration() {
        let mut bus = TypedEventBus::new();
        let processed_count = Arc::new(AtomicU32::new(0));
        
        let handlers = vec![
            Box::new(TestHandler {
                processed_count: Arc::clone(&processed_count),
                expected_id: None,
            }) as Box<dyn EventHandler<TestEvent>>,
        ];
        
        let registration = bus.register_module("test_module".to_string(), handlers).unwrap();
        assert_eq!(registration.module_name, "test_module");
        assert_eq!(registration.subscription_ids.len(), 1);
        
        // Verify module is registered
        let modules = bus.get_registered_modules().unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].module_name, "test_module");
        
        // Unregister module
        bus.unregister_module("test_module").unwrap();
        let modules = bus.get_registered_modules().unwrap();
        assert_eq!(modules.len(), 0);
    }
    
    #[test]
    fn test_event_processing() {
        let mut bus = TypedEventBus::new();
        let test_count = Arc::new(AtomicU32::new(0));
        let another_count = Arc::new(AtomicU32::new(0));
        
        // Subscribe handlers for different event types
        let test_handler = TestHandler {
            processed_count: Arc::clone(&test_count),
            expected_id: Some(42),
        };
        let another_handler = AnotherTestHandler {
            processed_count: Arc::clone(&another_count),
        };
        
        bus.subscribe::<TestEvent>(Box::new(test_handler)).unwrap();
        bus.subscribe::<AnotherTestEvent>(Box::new(another_handler)).unwrap();
        
        // Start the bus
        bus.start().unwrap();
        
        // Publish events of different types
        let test_event = TestEvent {
            id: 42,
            priority: EventPriority::High,
            timestamp: get_timestamp_ns(),
        };
        let another_event = AnotherTestEvent {
            data: "test_data".to_string(),
            timestamp: get_timestamp_ns(),
        };
        
        bus.publish(test_event).unwrap();
        bus.publish(another_event).unwrap();
        
        // Wait for processing
        thread::sleep(Duration::from_millis(50));
        
        // Verify events were processed by correct handlers
        assert_eq!(test_count.load(Ordering::SeqCst), 1);
        assert_eq!(another_count.load(Ordering::SeqCst), 1);
        
        bus.stop().unwrap();
    }
    
    #[test]
    fn test_multiple_handlers_same_type() {
        let mut bus = TypedEventBus::new();
        let count1 = Arc::new(AtomicU32::new(0));
        let count2 = Arc::new(AtomicU32::new(0));
        
        // Subscribe multiple handlers for the same event type
        let handler1 = TestHandler {
            processed_count: Arc::clone(&count1),
            expected_id: None,
        };
        let handler2 = TestHandler {
            processed_count: Arc::clone(&count2),
            expected_id: None,
        };
        
        bus.subscribe::<TestEvent>(Box::new(handler1)).unwrap();
        bus.subscribe::<TestEvent>(Box::new(handler2)).unwrap();
        
        bus.start().unwrap();
        
        // Publish single event
        let event = TestEvent {
            id: 1,
            priority: EventPriority::Normal,
            timestamp: get_timestamp_ns(),
        };
        
        bus.publish(event).unwrap();
        
        // Wait for processing
        thread::sleep(Duration::from_millis(50));
        
        // Both handlers should have processed the event
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
        
        bus.stop().unwrap();
    }
    
    #[test]
    fn test_error_handling_in_handlers() {
        let mut bus = TypedEventBus::new();
        
        struct FailingHandler;
        impl EventHandler<TestEvent> for FailingHandler {
            fn handle_event(&mut self, _event: &TestEvent) -> Result<(), Box<dyn std::error::Error>> {
                Err("Handler intentionally failed".into())
            }
        }
        
        bus.subscribe::<TestEvent>(Box::new(FailingHandler)).unwrap();
        bus.start().unwrap();
        
        let event = TestEvent {
            id: 1,
            priority: EventPriority::Normal,
            timestamp: get_timestamp_ns(),
        };
        
        // Event should be published successfully even if handler fails
        assert!(bus.publish(event).is_ok());
        
        // Wait for processing
        thread::sleep(Duration::from_millis(50));
        
        // Metrics should still be updated
        let metrics = bus.get_metrics();
        assert_eq!(metrics.total_events_processed, 1);
        
        bus.stop().unwrap();
    }
}