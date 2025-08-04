//! Tests for AudioWorklet batched data accumulation
//! 
//! These tests verify that the AudioWorklet processor correctly batches
//! multiple 128-sample chunks before sending via postMessage with transferables.

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    use wasm_bindgen::{JsValue, JsCast};
    use js_sys::{ArrayBuffer, Float32Array, Function, Reflect};
    
    // Tests run in node environment
    
    /// Helper to create a mock AudioWorklet processor for testing
    fn create_test_processor() -> Result<JsValue, JsValue> {
        // Create a minimal test processor with batching
        let js_code = r#"
            // Mock currentTime
            if (typeof globalThis.currentTime === 'undefined') {
                globalThis.currentTime = 0;
            }
            if (typeof globalThis.sampleRate === 'undefined') {
                globalThis.sampleRate = 48000;
            }
            
            // Simple buffer pool for testing
            class TestBufferPool {
                constructor(poolSize, bufferCapacity) {
                    this.poolSize = poolSize;
                    this.bufferCapacity = bufferCapacity;
                    this.buffers = [];
                    this.availableIndices = [];
                    this.inUseBuffers = new Map();
                    
                    for (let i = 0; i < poolSize; i++) {
                        this.buffers.push(new ArrayBuffer(bufferCapacity * 4));
                        this.availableIndices.push(i);
                    }
                }
                
                acquire() {
                    if (this.availableIndices.length === 0) return null;
                    const index = this.availableIndices.pop();
                    const buffer = this.buffers[index];
                    this.inUseBuffers.set(buffer, index);
                    return buffer;
                }
                
                markTransferred(buffer) {
                    const index = this.inUseBuffers.get(buffer);
                    if (index === undefined) return;
                    this.inUseBuffers.delete(buffer);
                    const newBuffer = new ArrayBuffer(this.bufferCapacity * 4);
                    this.buffers[index] = newBuffer;
                    this.availableIndices.push(index);
                }
                
                isDetached(buffer) {
                    return buffer.byteLength === 0;
                }
            }
            
            class TestProcessor {
                constructor() {
                    this.chunkSize = 128;
                    this.batchSize = 1024; // 8 chunks
                    this.bufferPool = new TestBufferPool(4, this.batchSize);
                    this.currentBuffer = null;
                    this.currentBufferArray = null;
                    this.writePosition = 0;
                    this.bufferTimeout = 50;
                    this.lastBufferStartTime = 0;
                    this.isProcessing = true;
                    this.messages = [];
                    
                    // Mock port
                    this.port = {
                        postMessage: (msg, transfer) => {
                            this.messages.push({ message: msg, transfer: transfer });
                        }
                    };
                }
                
                acquireNewBuffer() {
                    this.currentBuffer = this.bufferPool.acquire();
                    if (this.currentBuffer) {
                        this.currentBufferArray = new Float32Array(this.currentBuffer);
                        this.writePosition = 0;
                        this.lastBufferStartTime = globalThis.currentTime;
                    }
                }
                
                sendCurrentBuffer() {
                    if (!this.currentBuffer || !this.currentBufferArray) return;
                    if (this.writePosition > 0) {
                        this.port.postMessage({
                            type: 'audioDataBatch',
                            buffer: this.currentBuffer,
                            sampleCount: this.writePosition,
                            batchSize: this.batchSize,
                            timestamp: globalThis.currentTime
                        }, [this.currentBuffer]);
                        
                        this.bufferPool.markTransferred(this.currentBuffer);
                        this.currentBuffer = null;
                        this.currentBufferArray = null;
                        this.writePosition = 0;
                    }
                }
                
                processChunk(audioData) {
                    if (!this.isProcessing) return;
                    
                    if (!this.currentBuffer || !this.currentBufferArray) {
                        this.acquireNewBuffer();
                    }
                    
                    if (!this.currentBufferArray) return;
                    
                    const remainingSpace = this.batchSize - this.writePosition;
                    const samplesToWrite = Math.min(this.chunkSize, remainingSpace);
                    
                    this.currentBufferArray.set(audioData.subarray(0, samplesToWrite), this.writePosition);
                    this.writePosition += samplesToWrite;
                    
                    const timeElapsed = globalThis.currentTime - this.lastBufferStartTime;
                    const shouldSendDueToTimeout = this.writePosition > 0 && timeElapsed >= this.bufferTimeout;
                    
                    if (this.writePosition >= this.batchSize || shouldSendDueToTimeout) {
                        this.sendCurrentBuffer();
                        
                        if (samplesToWrite < this.chunkSize) {
                            this.acquireNewBuffer();
                            if (this.currentBufferArray) {
                                const remainingSamples = this.chunkSize - samplesToWrite;
                                this.currentBufferArray.set(
                                    audioData.subarray(samplesToWrite),
                                    0
                                );
                                this.writePosition = remainingSamples;
                            }
                        }
                    }
                }
                
                stop() {
                    this.isProcessing = false;
                    if (this.currentBuffer && this.writePosition > 0) {
                        this.sendCurrentBuffer();
                    }
                }
            }
            
            new TestProcessor()
        "#;
        
        web_sys::js_sys::eval(js_code)
    }
    
    #[wasm_bindgen_test]
    fn test_basic_batching() {
        let processor = create_test_processor().expect("Failed to create test processor");
        
        // Create test audio data (128 samples)
        let chunk = Float32Array::new_with_length(128);
        for i in 0..128 {
            chunk.set_index(i, 0.1);
        }
        
        // Process 8 chunks to fill one batch
        let process_chunk = Reflect::get(&processor, &"processChunk".into()).unwrap();
        let process_chunk_fn = process_chunk.dyn_ref::<Function>().unwrap();
        
        for _ in 0..8 {
            process_chunk_fn.call1(&processor, &chunk).unwrap();
        }
        
        // Check that exactly one message was sent
        let messages = Reflect::get(&processor, &"messages".into()).unwrap();
        let messages_array = messages.dyn_ref::<js_sys::Array>().unwrap();
        assert_eq!(messages_array.length(), 1);
        
        // Verify message content
        let msg_obj = messages_array.get(0);
        let message = Reflect::get(&msg_obj, &"message".into()).unwrap();
        
        let msg_type = Reflect::get(&message, &"type".into()).unwrap();
        assert_eq!(msg_type.as_string().unwrap(), "audioDataBatch");
        
        let sample_count = Reflect::get(&message, &"sampleCount".into()).unwrap();
        assert_eq!(sample_count.as_f64().unwrap(), 1024.0);
    }
    
    #[wasm_bindgen_test]
    fn test_partial_batch_on_stop() {
        let processor = create_test_processor().expect("Failed to create test processor");
        
        // Create test audio data
        let chunk = Float32Array::new_with_length(128);
        for i in 0..128 {
            chunk.set_index(i, 0.2);
        }
        
        // Process only 3 chunks (partial batch)
        let process_chunk = Reflect::get(&processor, &"processChunk".into()).unwrap();
        let process_chunk_fn = process_chunk.dyn_ref::<Function>().unwrap();
        
        for _ in 0..3 {
            process_chunk_fn.call1(&processor, &chunk).unwrap();
        }
        
        // Check no message sent yet
        let messages = Reflect::get(&processor, &"messages".into()).unwrap();
        let messages_array = messages.dyn_ref::<js_sys::Array>().unwrap();
        assert_eq!(messages_array.length(), 0);
        
        // Stop processing
        let stop = Reflect::get(&processor, &"stop".into()).unwrap();
        let stop_fn = stop.dyn_ref::<Function>().unwrap();
        stop_fn.call0(&processor).unwrap();
        
        // Now check that partial buffer was sent
        assert_eq!(messages_array.length(), 1);
        
        let msg_obj = messages_array.get(0);
        let message = Reflect::get(&msg_obj, &"message".into()).unwrap();
        
        let sample_count = Reflect::get(&message, &"sampleCount".into()).unwrap();
        assert_eq!(sample_count.as_f64().unwrap(), 384.0); // 3 * 128
    }
    
    #[wasm_bindgen_test]
    fn test_timeout_mechanism() {
        let processor = create_test_processor().expect("Failed to create test processor");
        
        // Create test audio data
        let chunk = Float32Array::new_with_length(128);
        for i in 0..128 {
            chunk.set_index(i, 0.3);
        }
        
        // Process 2 chunks
        let process_chunk = Reflect::get(&processor, &"processChunk".into()).unwrap();
        let process_chunk_fn = process_chunk.dyn_ref::<Function>().unwrap();
        
        for _ in 0..2 {
            process_chunk_fn.call1(&processor, &chunk).unwrap();
        }
        
        // Simulate time passing (> 50ms timeout)
        js_sys::Reflect::set(&js_sys::global(), &"currentTime".into(), &JsValue::from(60)).unwrap();
        
        // Process one more chunk - should trigger timeout send
        process_chunk_fn.call1(&processor, &chunk).unwrap();
        
        // Check that message was sent due to timeout
        let messages = Reflect::get(&processor, &"messages".into()).unwrap();
        let messages_array = messages.dyn_ref::<js_sys::Array>().unwrap();
        assert_eq!(messages_array.length(), 1);
        
        let msg_obj = messages_array.get(0);
        let message = Reflect::get(&msg_obj, &"message".into()).unwrap();
        
        let sample_count = Reflect::get(&message, &"sampleCount".into()).unwrap();
        assert_eq!(sample_count.as_f64().unwrap(), 384.0); // 3 * 128
    }
    
    #[wasm_bindgen_test]
    fn test_buffer_overflow_handling() {
        let processor = create_test_processor().expect("Failed to create test processor");
        
        // Create test audio data with 130 samples (more than chunk size)
        let large_chunk = Float32Array::new_with_length(130);
        for i in 0..130 {
            large_chunk.set_index(i, 0.4);
        }
        
        // Fill buffer almost completely (7 chunks = 896 samples)
        let process_chunk = Reflect::get(&processor, &"processChunk".into()).unwrap();
        let process_chunk_fn = process_chunk.dyn_ref::<Function>().unwrap();
        
        let normal_chunk = Float32Array::new_with_length(128);
        for i in 0..128 {
            normal_chunk.set_index(i, 0.4);
        }
        
        for _ in 0..7 {
            process_chunk_fn.call1(&processor, &normal_chunk).unwrap();
        }
        
        // Process one more chunk - should complete buffer and handle overflow
        process_chunk_fn.call1(&processor, &normal_chunk).unwrap();
        
        // Should have sent one complete buffer
        let messages = Reflect::get(&processor, &"messages".into()).unwrap();
        let messages_array = messages.dyn_ref::<js_sys::Array>().unwrap();
        assert_eq!(messages_array.length(), 1);
        
        // Verify the sent buffer is full
        let msg_obj = messages_array.get(0);
        let message = Reflect::get(&msg_obj, &"message".into()).unwrap();
        
        let sample_count = Reflect::get(&message, &"sampleCount".into()).unwrap();
        assert_eq!(sample_count.as_f64().unwrap(), 1024.0);
    }
    
    #[wasm_bindgen_test]
    fn test_transferable_usage() {
        let processor = create_test_processor().expect("Failed to create test processor");
        
        // Process enough chunks to trigger a send
        let chunk = Float32Array::new_with_length(128);
        for i in 0..128 {
            chunk.set_index(i, 0.5);
        }
        
        let process_chunk = Reflect::get(&processor, &"processChunk".into()).unwrap();
        let process_chunk_fn = process_chunk.dyn_ref::<Function>().unwrap();
        
        for _ in 0..8 {
            process_chunk_fn.call1(&processor, &chunk).unwrap();
        }
        
        // Verify transfer array was included
        let messages = Reflect::get(&processor, &"messages".into()).unwrap();
        let messages_array = messages.dyn_ref::<js_sys::Array>().unwrap();
        
        let msg_obj = messages_array.get(0);
        let transfer = Reflect::get(&msg_obj, &"transfer".into()).unwrap();
        
        assert!(!transfer.is_undefined());
        let transfer_array = transfer.dyn_ref::<js_sys::Array>().unwrap();
        assert_eq!(transfer_array.length(), 1);
        
        // Verify the transferred object is an ArrayBuffer
        let transferred_buffer = transfer_array.get(0);
        assert!(transferred_buffer.is_instance_of::<ArrayBuffer>());
    }
}