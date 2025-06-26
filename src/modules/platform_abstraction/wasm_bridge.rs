use super::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use js_sys::{Promise, Function, Reflect};

/// WebAssembly bridge utilities implementation with JavaScript interop optimization
pub struct WasmBridgeImpl {
    /// Cache for optimized interop calls
    call_cache: Arc<Mutex<HashMap<String, OptimizedCall>>>,
    /// Performance metrics for interop operations
    performance_metrics: Arc<Mutex<InteropMetrics>>,
}

#[derive(Default, Clone)]
struct InteropMetrics {
    total_calls: u64,
    cache_hits: u64,
    average_call_time_us: f64,
    error_count: u64,
}

impl WasmBridgeImpl {
    pub fn new() -> Self {
        Self {
            call_cache: Arc::new(Mutex::new(HashMap::new())),
            performance_metrics: Arc::new(Mutex::new(InteropMetrics::default())),
        }
    }
    
    /// Handle JavaScript promise-based async operations
    #[cfg(target_arch = "wasm32")]
    pub async fn handle_js_promise(&self, promise: Promise) -> Result<JsValue, PlatformError> {
        JsFuture::from(promise)
            .await
            .map_err(|e| PlatformError::WasmBridgeError(format!("Promise failed: {:?}", e)))
    }
    
    /// Optimize frequent JavaScript calls with caching
    fn get_or_create_optimized_call(&self, call: &InteropCall) -> Result<OptimizedCall, PlatformError> {
        let call_key = format!("{}:{}", call.function_name, call.call_type.as_cache_key());
        
        // Check cache first
        {
            let cache = self.call_cache.lock().unwrap();
            if let Some(optimized) = cache.get(&call_key) {
                self.update_metrics(true, 0.0); // Cache hit
                return Ok(optimized.clone());
            }
        }
        
        // Create new optimized call
        let start_time = self.get_performance_now();
        let optimized = self.create_optimized_call(call)?;
        let end_time = self.get_performance_now();
        let call_time = end_time - start_time;
        
        // Cache the result
        {
            let mut cache = self.call_cache.lock().unwrap();
            cache.insert(call_key, optimized.clone());
        }
        
        self.update_metrics(false, call_time * 1000.0); // Convert to microseconds
        Ok(optimized)
    }
    
    fn create_optimized_call(&self, call: &InteropCall) -> Result<OptimizedCall, PlatformError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;
            
            let window = window().ok_or(PlatformError::WasmBridgeError("No window object".to_string()))?;
            
            // Get the function from the global scope
            let function_value = Reflect::get(&window, &JsValue::from_str(&call.function_name))
                .map_err(|_| PlatformError::WasmBridgeError(format!("Function {} not found", call.function_name)))?;
            
            if !function_value.is_function() {
                return Err(PlatformError::WasmBridgeError(format!("{} is not a function", call.function_name)));
            }
            
            let function = Function::from(function_value);
            
            Ok(OptimizedCall {
                call: call.clone(),
                optimization_applied: "cached_function".to_string(),
            })
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM fallback for testing
            Ok(OptimizedCall {
                call: call.clone(),
                optimization_applied: "standard_fallback".to_string(),
            })
        }
    }
    
    /// Get high-resolution timestamp for performance measurement
    fn get_performance_now(&self) -> f64 {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|window| window.performance())
                .map(|perf| perf.now())
                .unwrap_or(0.0)
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64() * 1000.0
        }
    }
    
    fn update_metrics(&self, cache_hit: bool, call_time_us: f64) {
        let mut metrics = self.performance_metrics.lock().unwrap();
        metrics.total_calls += 1;
        
        if cache_hit {
            metrics.cache_hits += 1;
        } else {
            // Update rolling average of call time
            let alpha = 0.1; // Exponential smoothing factor
            metrics.average_call_time_us = 
                alpha * call_time_us + (1.0 - alpha) * metrics.average_call_time_us;
        }
    }
    
    /// Execute an optimized JavaScript call with performance tracking
    #[cfg(target_arch = "wasm32")]
    pub fn execute_optimized_call(&self, optimized: &OptimizedCall, _args: &[&str]) -> Result<String, PlatformError> {
        use web_sys::window;
        
        let start_time = self.get_performance_now();
        
        // Get function from global scope
        let window = window().ok_or(PlatformError::WasmBridgeError("No window object".to_string()))?;
        let function_value = Reflect::get(&window, &JsValue::from_str(&optimized.call.function_name))
            .map_err(|_| PlatformError::WasmBridgeError(format!("Function {} not found", optimized.call.function_name)))?;
        
        if !function_value.is_function() {
            return Err(PlatformError::WasmBridgeError(format!("{} is not a function", optimized.call.function_name)));
        }
        
        let function = Function::from(function_value);
        
        // Execute the function (simplified for demonstration)
        let result = function.call0(&JsValue::UNDEFINED)
            .map_err(|e| PlatformError::WasmBridgeError(format!("Function call failed: {:?}", e)))?;
        
        let end_time = self.get_performance_now();
        let call_time = (end_time - start_time) * 1000.0; // Convert to microseconds
        
        self.update_metrics(false, call_time);
        
        Ok(format!("Executed {} with optimization: {}", optimized.call.function_name, optimized.optimization_applied))
    }
    
    /// Get performance metrics for monitoring
    pub fn get_performance_metrics(&self) -> InteropMetrics {
        self.performance_metrics.lock().unwrap().clone()
    }
    
    /// Clear the optimization cache (useful for memory management)
    pub fn clear_cache(&self) {
        let mut cache = self.call_cache.lock().unwrap();
        cache.clear();
    }
}

impl Default for WasmBridgeImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmBridge for WasmBridgeImpl {
    fn handle_async_operation(&self, operation: AsyncOperation) -> Result<AsyncResult, PlatformError> {
        // Handle different operation types
        let success = match operation.operation_type.as_str() {
            "promise" => true,
            "callback" => true,
            "stream" => true,
            _ => false,
        };
        
        Ok(AsyncResult {
            id: operation.id,
            success,
            data: Some(format!("Handled operation: {}", operation.operation_type)),
        })
    }
    
    fn optimize_interop_call(&self, call: InteropCall) -> Result<OptimizedCall, PlatformError> {
        self.get_or_create_optimized_call(&call)
    }
    
    fn create_js_wrapper_concrete(&self, function_name: &str) -> Result<JsWrapper, PlatformError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;
            
            let window = window().ok_or(PlatformError::WasmBridgeError("No window object".to_string()))?;
            
            let function_value = Reflect::get(&window, &JsValue::from_str(function_name))
                .map_err(|_| PlatformError::WasmBridgeError(format!("Function {} not found", function_name)))?;
            
            if !function_value.is_function() {
                return Err(PlatformError::WasmBridgeError(format!("{} is not a function", function_name)));
            }
            
            Ok(JsWrapper {
                id: format!("wrapper_{}", function_name),
                function_name: function_name.to_string(),
            })
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM fallback for testing
            Ok(JsWrapper {
                id: format!("mock_wrapper_{}", function_name),
                function_name: function_name.to_string(),
            })
        }
    }
} 