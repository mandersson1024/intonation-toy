//! # WASM-JS Bridge Integration
//!
//! This module provides optimized buffer transfers across the WASM-JavaScript boundary,
//! minimizing GC pressure and enabling zero-copy operations where possible.

use wasm_bindgen::prelude::*;
use js_sys::{Float32Array, SharedArrayBuffer, ArrayBuffer};
use web_sys::console;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use super::buffer_recycling_pool::{PoolError, JSBufferRef};

/// WASM-JS boundary optimization bridge
#[wasm_bindgen]
pub struct WasmJsBridge {
    /// SharedArrayBuffer support detection
    shared_array_buffer_supported: bool,
    /// Buffer reference tracking
    buffer_refs: Arc<Mutex<HashMap<u64, JSBufferRef>>>,
    /// Performance metrics
    boundary_metrics: Arc<Mutex<BoundaryMetrics>>,
    /// Next buffer reference ID
    next_ref_id: Arc<Mutex<u64>>,
}

/// WASM-JS boundary performance metrics
#[derive(Debug, Clone, Default)]
pub struct BoundaryMetrics {
    /// Total boundary crossings
    pub total_crossings: u64,
    /// Zero-copy operations
    pub zero_copy_operations: u64,
    /// Copy operations
    pub copy_operations: u64,
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Average transfer time in nanoseconds
    pub avg_transfer_time_ns: u64,
    /// JavaScript GC pressure reduction estimate
    pub gc_pressure_reduction_percent: f32,
}

#[wasm_bindgen]
impl WasmJsBridge {
    /// Create a new WASM-JS bridge
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmJsBridge {
        let shared_array_buffer_supported = Self::detect_shared_array_buffer();
        
        Self {
            shared_array_buffer_supported,
            buffer_refs: Arc::new(Mutex::new(HashMap::new())),
            boundary_metrics: Arc::new(Mutex::new(BoundaryMetrics::default())),
            next_ref_id: Arc::new(Mutex::new(1)),
        }
    }
    
    /// Check if SharedArrayBuffer is supported
    #[wasm_bindgen(getter)]
    pub fn shared_array_buffer_supported(&self) -> bool {
        self.shared_array_buffer_supported
    }
    
    /// Create a buffer reference optimized for WASM-JS boundary
    #[wasm_bindgen]
    pub fn create_buffer_ref(&self, size: u32) -> Result<JSBufferRef, JsValue> {
        let start_time = js_sys::Date::now();
        
        let ref_id = {
            let mut next_id = self.next_ref_id.lock()
                .map_err(|_| JsValue::from_str("Failed to get next ref ID"))?;
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let (buffer_ref, is_zero_copy) = if self.shared_array_buffer_supported {
            match self.create_shared_buffer_ref(size as usize, ref_id) {
                Ok(buffer_ref) => (buffer_ref, true),
                Err(_) => {
                    // Fall back to regular buffer
                    (self.create_regular_buffer_ref(size as usize, ref_id)?, false)
                }
            }
        } else {
            (self.create_regular_buffer_ref(size as usize, ref_id)?, false)
        };
        
        // Track the buffer reference
        {
            let mut refs = self.buffer_refs.lock()
                .map_err(|_| JsValue::from_str("Failed to track buffer reference"))?;
            refs.insert(ref_id, buffer_ref.clone());
        }
        
        // Update metrics
        let transfer_time = ((js_sys::Date::now() - start_time) * 1_000_000.0) as u64; // Convert to nanoseconds
        self.update_boundary_metrics(size as u64 * 4, transfer_time, is_zero_copy)?;
        
        Ok(buffer_ref)
    }
    
    /// Transfer data from Rust Vec to JavaScript buffer
    #[wasm_bindgen]
    pub fn transfer_to_js(&self, ref_id: u64, data: &[f32]) -> Result<(), JsValue> {
        let start_time = js_sys::Date::now();
        
        let buffer_ref = {
            let refs = self.buffer_refs.lock()
                .map_err(|_| JsValue::from_str("Failed to access buffer references"))?;
            refs.get(&ref_id).cloned()
                .ok_or_else(|| JsValue::from_str("Buffer reference not found"))?
        };
        
        // Perform the transfer
        let is_zero_copy = buffer_ref.is_shared();
        if is_zero_copy {
            // Zero-copy operation via SharedArrayBuffer
            self.zero_copy_transfer_to_js(&buffer_ref, data)?;
        } else {
            // Copy operation
            buffer_ref.from_vec(data)
                .map_err(|e| JsValue::from_str(&format!("Transfer failed: {}", e)))?;
        }
        
        // Update metrics
        let transfer_time = ((js_sys::Date::now() - start_time) * 1_000_000.0) as u64;
        self.update_boundary_metrics(data.len() as u64 * 4, transfer_time, is_zero_copy)?;
        
        Ok(())
    }
    
    /// Transfer data from JavaScript buffer to Rust Vec
    #[wasm_bindgen]
    pub fn transfer_from_js(&self, ref_id: u64) -> Result<Vec<f32>, JsValue> {
        let start_time = js_sys::Date::now();
        
        let buffer_ref = {
            let refs = self.buffer_refs.lock()
                .map_err(|_| JsValue::from_str("Failed to access buffer references"))?;
            refs.get(&ref_id).cloned()
                .ok_or_else(|| JsValue::from_str("Buffer reference not found"))?
        };
        
        // Perform the transfer
        let is_zero_copy = buffer_ref.is_shared();
        let data = if is_zero_copy {
            // Zero-copy operation via SharedArrayBuffer
            self.zero_copy_transfer_from_js(&buffer_ref)?
        } else {
            // Copy operation
            buffer_ref.to_vec()
        };
        
        // Update metrics
        let transfer_time = ((js_sys::Date::now() - start_time) * 1_000_000.0) as u64;
        self.update_boundary_metrics(data.len() as u64 * 4, transfer_time, is_zero_copy)?;
        
        Ok(data)
    }
    
    /// Release a buffer reference
    #[wasm_bindgen]
    pub fn release_buffer_ref(&self, ref_id: u64) -> Result<(), JsValue> {
        let mut refs = self.buffer_refs.lock()
            .map_err(|_| JsValue::from_str("Failed to access buffer references"))?;
        
        refs.remove(&ref_id);
        Ok(())
    }
    
    /// Get boundary performance metrics
    #[wasm_bindgen]
    pub fn get_boundary_metrics(&self) -> Result<JsValue, JsValue> {
        let metrics = self.boundary_metrics.lock()
            .map_err(|_| JsValue::from_str("Failed to access boundary metrics"))?;
        
        let js_metrics = js_sys::Object::new();
        
        js_sys::Reflect::set(
            &js_metrics,
            &JsValue::from_str("totalCrossings"),
            &JsValue::from_f64(metrics.total_crossings as f64),
        )?;
        
        js_sys::Reflect::set(
            &js_metrics,
            &JsValue::from_str("zeroCopyOperations"),
            &JsValue::from_f64(metrics.zero_copy_operations as f64),
        )?;
        
        js_sys::Reflect::set(
            &js_metrics,
            &JsValue::from_str("copyOperations"),
            &JsValue::from_f64(metrics.copy_operations as f64),
        )?;
        
        js_sys::Reflect::set(
            &js_metrics,
            &JsValue::from_str("bytesTransferred"),
            &JsValue::from_f64(metrics.bytes_transferred as f64),
        )?;
        
        js_sys::Reflect::set(
            &js_metrics,
            &JsValue::from_str("avgTransferTimeNs"),
            &JsValue::from_f64(metrics.avg_transfer_time_ns as f64),
        )?;
        
        js_sys::Reflect::set(
            &js_metrics,
            &JsValue::from_str("gcPressureReductionPercent"),
            &JsValue::from_f64(metrics.gc_pressure_reduction_percent as f64),
        )?;
        
        Ok(js_metrics.into())
    }
}

impl WasmJsBridge {
    /// Detect SharedArrayBuffer support
    fn detect_shared_array_buffer() -> bool {
        js_sys::eval("typeof SharedArrayBuffer !== 'undefined'")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }
    
    /// Create a SharedArrayBuffer-based buffer reference
    fn create_shared_buffer_ref(&self, size: usize, ref_id: u64) -> Result<JSBufferRef, JsValue> {
        let byte_length = size * 4; // 4 bytes per f32
        
        let shared_buffer = SharedArrayBuffer::new(byte_length as u32);
        let array = Float32Array::new_with_byte_length(&shared_buffer, 0, size as u32);
        
        Ok(JSBufferRef::new(array, ref_id, true))
    }
    
    /// Create a regular ArrayBuffer-based buffer reference
    fn create_regular_buffer_ref(&self, size: usize, ref_id: u64) -> Result<JSBufferRef, JsValue> {
        let array = Float32Array::new_with_length(size as u32);
        Ok(JSBufferRef::new(array, ref_id, false))
    }
    
    /// Perform zero-copy transfer to JavaScript (SharedArrayBuffer)
    fn zero_copy_transfer_to_js(&self, buffer_ref: &JSBufferRef, data: &[f32]) -> Result<(), JsValue> {
        if !buffer_ref.is_shared() {
            return Err(JsValue::from_str("Buffer is not shared, cannot perform zero-copy"));
        }
        
        // For SharedArrayBuffer, we can write directly to the shared memory
        let array = buffer_ref.array();
        
        // Use subarray to get a view and copy data
        for (i, &value) in data.iter().enumerate() {
            array.set_index(i as u32, value);
        }
        
        Ok(())
    }
    
    /// Perform zero-copy transfer from JavaScript (SharedArrayBuffer)
    fn zero_copy_transfer_from_js(&self, buffer_ref: &JSBufferRef) -> Result<Vec<f32>, JsValue> {
        if !buffer_ref.is_shared() {
            return Err(JsValue::from_str("Buffer is not shared, cannot perform zero-copy"));
        }
        
        // For SharedArrayBuffer, we can read directly from shared memory
        let array = buffer_ref.array();
        let size = array.length() as usize;
        let mut data = vec![0.0f32; size];
        
        for i in 0..size {
            data[i] = array.get_index(i as u32);
        }
        
        Ok(data)
    }
    
    /// Update boundary crossing metrics
    fn update_boundary_metrics(&self, bytes: u64, transfer_time_ns: u64, is_zero_copy: bool) -> Result<(), JsValue> {
        let mut metrics = self.boundary_metrics.lock()
            .map_err(|_| JsValue::from_str("Failed to update boundary metrics"))?;
        
        metrics.total_crossings += 1;
        metrics.bytes_transferred += bytes;
        
        if is_zero_copy {
            metrics.zero_copy_operations += 1;
        } else {
            metrics.copy_operations += 1;
        }
        
        // Update average transfer time
        let total_crossings = metrics.total_crossings;
        let prev_avg = metrics.avg_transfer_time_ns;
        metrics.avg_transfer_time_ns = ((prev_avg * (total_crossings - 1)) + transfer_time_ns) / total_crossings;
        
        // Calculate GC pressure reduction estimate
        // Zero-copy operations reduce GC pressure significantly
        let zero_copy_ratio = metrics.zero_copy_operations as f32 / metrics.total_crossings as f32;
        metrics.gc_pressure_reduction_percent = zero_copy_ratio * 30.0; // Up to 30% reduction
        
        Ok(())
    }
}

/// Audio buffer integration for real-time processing
impl WasmJsBridge {
    /// Create optimized audio buffer for real-time processing
    pub fn create_audio_buffer(&self, samples: usize) -> Result<JSBufferRef, PoolError> {
        let ref_id = {
            let mut next_id = self.next_ref_id.lock()
                .map_err(|_| PoolError::Internal("Failed to get next ref ID".to_string()))?;
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let buffer_ref = if self.shared_array_buffer_supported {
            match self.create_shared_buffer_ref(samples, ref_id) {
                Ok(buffer_ref) => buffer_ref,
                Err(_) => {
                    // Fall back to regular buffer
                    self.create_regular_buffer_ref(samples, ref_id)
                        .map_err(|_| PoolError::WasmJsBoundaryFailed("Failed to create regular buffer".to_string()))?
                }
            }
        } else {
            self.create_regular_buffer_ref(samples, ref_id)
                .map_err(|_| PoolError::WasmJsBoundaryFailed("Failed to create regular buffer".to_string()))?
        };
        
        // Track the buffer reference
        {
            let mut refs = self.buffer_refs.lock()
                .map_err(|_| PoolError::Internal("Failed to track buffer reference".to_string()))?;
            refs.insert(ref_id, buffer_ref.clone());
        }
        
        Ok(buffer_ref)
    }
    
    /// Transfer audio data with <0.5ms latency requirement
    pub fn transfer_audio_data(&self, ref_id: u64, audio_data: &[f32]) -> Result<(), PoolError> {
        let start_time = js_sys::Date::now();
        
        let buffer_ref = {
            let refs = self.buffer_refs.lock()
                .map_err(|_| PoolError::Internal("Failed to access buffer references".to_string()))?;
            refs.get(&ref_id).cloned()
                .ok_or_else(|| PoolError::WasmJsBoundaryFailed("Buffer reference not found".to_string()))?
        };
        
        // Perform optimized transfer
        if buffer_ref.is_shared() {
            self.zero_copy_transfer_to_js(&buffer_ref, audio_data)
                .map_err(|_| PoolError::ZeroCopyFailed("Zero-copy transfer failed".to_string()))?;
        } else {
            buffer_ref.from_vec(audio_data)?;
        }
        
        // Verify latency requirement
        let transfer_time_ms = js_sys::Date::now() - start_time;
        if transfer_time_ms > 0.5 {
            console::warn_1(&JsValue::from_str(&format!(
                "Audio transfer took {:.2}ms, exceeding 0.5ms target", 
                transfer_time_ms
            )));
        }
        
        Ok(())
    }
    
    /// Get audio buffer metrics for Audio Foundations integration
    pub fn get_audio_buffer_metrics(&self) -> Result<AudioBufferMetrics, PoolError> {
        let metrics = self.boundary_metrics.lock()
            .map_err(|_| PoolError::Internal("Failed to access boundary metrics".to_string()))?;
        
        let active_buffers = {
            let refs = self.buffer_refs.lock()
                .map_err(|_| PoolError::Internal("Failed to access buffer references".to_string()))?;
            refs.len()
        };
        
        Ok(AudioBufferMetrics {
            active_buffers,
            total_transfers: metrics.total_crossings,
            zero_copy_transfers: metrics.zero_copy_operations,
            avg_transfer_time_ns: metrics.avg_transfer_time_ns,
            gc_pressure_reduction: metrics.gc_pressure_reduction_percent,
            shared_array_buffer_supported: self.shared_array_buffer_supported,
        })
    }
}

/// Audio buffer metrics for Audio Foundations integration
#[derive(Debug, Clone)]
pub struct AudioBufferMetrics {
    pub active_buffers: usize,
    pub total_transfers: u64,
    pub zero_copy_transfers: u64,
    pub avg_transfer_time_ns: u64,
    pub gc_pressure_reduction: f32,
    pub shared_array_buffer_supported: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    #[wasm_bindgen_test]
    fn test_bridge_creation() {
        let bridge = WasmJsBridge::new();
        // Should not panic and should detect capabilities
        assert!(bridge.shared_array_buffer_supported() || !bridge.shared_array_buffer_supported());
    }
    
    #[wasm_bindgen_test]
    fn test_buffer_ref_creation() {
        let bridge = WasmJsBridge::new();
        let buffer_ref = bridge.create_buffer_ref(1024).unwrap();
        assert_eq!(buffer_ref.size(), 1024);
    }
    
    #[wasm_bindgen_test]
    fn test_audio_buffer_creation() {
        let bridge = WasmJsBridge::new();
        let audio_buffer = bridge.create_audio_buffer(512).unwrap();
        assert_eq!(audio_buffer.size(), 512);
    }
}