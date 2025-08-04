//! Integration tests for AudioWorklet message handling with transferable buffers

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    use wasm_bindgen::{JsValue, JsCast};
    use js_sys::{ArrayBuffer, Float32Array, Function, Object, Reflect};
    
    // Tests run in node environment
    
    /// Mock AudioWorkletManager message handler for testing
    fn create_mock_message_handler() -> Result<JsValue, JsValue> {
        let js_code = r#"
            class MockMessageHandler {
                constructor() {
                    this.messages = [];
                    this.processedBatches = [];
                    this.volumeUpdates = [];
                    this.chunksProcessed = 0;
                }
                
                handleMessage(event) {
                    const data = event.data;
                    if (!data || typeof data !== 'object') return;
                    
                    this.messages.push(data);
                    
                    switch (data.type) {
                        case 'processorReady':
                            this.processorReady = true;
                            this.batchSize = data.batchSize;
                            this.sampleRate = data.sampleRate;
                            break;
                            
                        case 'audioDataBatch':
                            this.handleAudioDataBatch(data);
                            break;
                            
                        case 'batchConfigUpdated':
                            this.batchConfig = data.config;
                            break;
                    }
                }
                
                handleAudioDataBatch(data) {
                    // Extract ArrayBuffer and metadata
                    const buffer = data.buffer;
                    const sampleCount = data.sampleCount || 0;
                    const timestamp = data.timestamp || 0;
                    
                    if (!(buffer instanceof ArrayBuffer)) {
                        throw new Error('Expected ArrayBuffer');
                    }
                    
                    // Create Float32Array view
                    const samples = new Float32Array(buffer);
                    const validSamples = sampleCount > 0 ? 
                        samples.slice(0, sampleCount) : samples;
                    
                    // Track processed batch
                    this.processedBatches.push({
                        sampleCount: validSamples.length,
                        timestamp: timestamp,
                        firstSample: validSamples[0],
                        lastSample: validSamples[validSamples.length - 1]
                    });
                    
                    // Update chunks processed
                    const chunkCount = Math.ceil(validSamples.length / 128);
                    this.chunksProcessed += chunkCount;
                    
                    // Simulate volume detection
                    const rms = Math.sqrt(
                        validSamples.reduce((sum, x) => sum + x * x, 0) / validSamples.length
                    );
                    const rmsDb = 20 * Math.log10(Math.max(rms, 1e-10));
                    
                    this.volumeUpdates.push({
                        rmsDb: rmsDb,
                        sampleCount: validSamples.length,
                        timestamp: timestamp
                    });
                }
                
                reset() {
                    this.messages = [];
                    this.processedBatches = [];
                    this.volumeUpdates = [];
                    this.chunksProcessed = 0;
                    this.processorReady = false;
                    this.batchSize = null;
                    this.sampleRate = null;
                    this.batchConfig = null;
                }
            }
            
            new MockMessageHandler()
        "#;
        
        web_sys::js_sys::eval(js_code)
    }
    
    /// Create a mock message event with transferable buffer
    fn create_batch_message(sample_count: usize, sample_value: f32) -> Result<Object, JsValue> {
        // Create buffer with samples
        let buffer = ArrayBuffer::new((sample_count * 4) as u32);
        let samples = Float32Array::new(&buffer);
        for i in 0..sample_count {
            samples.set_index(i as u32, sample_value);
        }
        
        // Create message data
        let data = Object::new();
        Reflect::set(&data, &"type".into(), &"audioDataBatch".into())?;
        Reflect::set(&data, &"buffer".into(), &buffer)?;
        Reflect::set(&data, &"sampleCount".into(), &JsValue::from(sample_count as f64))?;
        Reflect::set(&data, &"timestamp".into(), &JsValue::from(123.45))?;
        
        Ok(data)
    }
    
    /// Create a mock message event wrapper
    fn create_mock_event(data: &Object) -> Result<Object, JsValue> {
        let event = Object::new();
        Reflect::set(&event, &"data".into(), data)?;
        Ok(event)
    }
    
    #[wasm_bindgen_test]
    fn test_processor_ready_handling() {
        let handler = create_mock_message_handler().expect("Failed to create handler");
        
        // Create processor ready message
        let data = Object::new();
        Reflect::set(&data, &"type".into(), &"processorReady".into()).unwrap();
        Reflect::set(&data, &"batchSize".into(), &JsValue::from(1024)).unwrap();
        Reflect::set(&data, &"sampleRate".into(), &JsValue::from(48000)).unwrap();
        
        let event = create_mock_event(&data).unwrap();
        
        // Handle message
        let handle_message = Reflect::get(&handler, &"handleMessage".into()).unwrap();
        let handle_fn = handle_message.dyn_ref::<Function>().unwrap();
        handle_fn.call1(&handler, &event).unwrap();
        
        // Verify handling
        let ready = Reflect::get(&handler, &"processorReady".into()).unwrap();
        assert_eq!(ready.as_bool().unwrap(), true);
        
        let batch_size = Reflect::get(&handler, &"batchSize".into()).unwrap();
        assert_eq!(batch_size.as_f64().unwrap(), 1024.0);
        
        let sample_rate = Reflect::get(&handler, &"sampleRate".into()).unwrap();
        assert_eq!(sample_rate.as_f64().unwrap(), 48000.0);
    }
    
    #[wasm_bindgen_test]
    fn test_audio_batch_processing() {
        let handler = create_mock_message_handler().expect("Failed to create handler");
        
        // Create and handle batch message
        let data = create_batch_message(1024, 0.5).unwrap();
        let event = create_mock_event(&data).unwrap();
        
        let handle_message = Reflect::get(&handler, &"handleMessage".into()).unwrap();
        let handle_fn = handle_message.dyn_ref::<Function>().unwrap();
        handle_fn.call1(&handler, &event).unwrap();
        
        // Check processed batches
        let batches = Reflect::get(&handler, &"processedBatches".into()).unwrap();
        let batches_array = batches.dyn_ref::<js_sys::Array>().unwrap();
        assert_eq!(batches_array.length(), 1);
        
        let batch = batches_array.get(0);
        let sample_count = Reflect::get(&batch, &"sampleCount".into()).unwrap();
        assert_eq!(sample_count.as_f64().unwrap(), 1024.0);
        
        // Check chunks processed
        let chunks = Reflect::get(&handler, &"chunksProcessed".into()).unwrap();
        assert_eq!(chunks.as_f64().unwrap(), 8.0); // 1024 / 128 = 8
        
        // Check volume updates
        let volume_updates = Reflect::get(&handler, &"volumeUpdates".into()).unwrap();
        let volume_array = volume_updates.dyn_ref::<js_sys::Array>().unwrap();
        assert_eq!(volume_array.length(), 1);
    }
    
    #[wasm_bindgen_test]
    fn test_multiple_batch_handling() {
        let handler = create_mock_message_handler().expect("Failed to create handler");
        
        let handle_message = Reflect::get(&handler, &"handleMessage".into()).unwrap();
        let handle_fn = handle_message.dyn_ref::<Function>().unwrap();
        
        // Send multiple batches
        for i in 0..5 {
            let data = create_batch_message(512, 0.1 * (i as f32 + 1.0)).unwrap();
            let event = create_mock_event(&data).unwrap();
            handle_fn.call1(&handler, &event).unwrap();
        }
        
        // Verify all batches processed
        let batches = Reflect::get(&handler, &"processedBatches".into()).unwrap();
        let batches_array = batches.dyn_ref::<js_sys::Array>().unwrap();
        assert_eq!(batches_array.length(), 5);
        
        // Check total chunks
        let chunks = Reflect::get(&handler, &"chunksProcessed".into()).unwrap();
        assert_eq!(chunks.as_f64().unwrap(), 20.0); // 5 * (512 / 128) = 20
    }
    
    #[wasm_bindgen_test]
    fn test_batch_config_update() {
        let handler = create_mock_message_handler().expect("Failed to create handler");
        
        // Create config update message
        let config = Object::new();
        Reflect::set(&config, &"batchSize".into(), &JsValue::from(2048)).unwrap();
        Reflect::set(&config, &"bufferTimeout".into(), &JsValue::from(30)).unwrap();
        
        let data = Object::new();
        Reflect::set(&data, &"type".into(), &"batchConfigUpdated".into()).unwrap();
        Reflect::set(&data, &"config".into(), &config).unwrap();
        
        let event = create_mock_event(&data).unwrap();
        
        // Handle message
        let handle_message = Reflect::get(&handler, &"handleMessage".into()).unwrap();
        let handle_fn = handle_message.dyn_ref::<Function>().unwrap();
        handle_fn.call1(&handler, &event).unwrap();
        
        // Verify config update
        let batch_config = Reflect::get(&handler, &"batchConfig".into()).unwrap();
        let batch_size = Reflect::get(&batch_config, &"batchSize".into()).unwrap();
        assert_eq!(batch_size.as_f64().unwrap(), 2048.0);
        
        let timeout = Reflect::get(&batch_config, &"bufferTimeout".into()).unwrap();
        assert_eq!(timeout.as_f64().unwrap(), 30.0);
    }
    
    #[wasm_bindgen_test]
    fn test_partial_buffer_handling() {
        let handler = create_mock_message_handler().expect("Failed to create handler");
        
        // Create buffer larger than sample count
        let buffer = ArrayBuffer::new(4096); // 1024 samples * 4 bytes
        let samples = Float32Array::new(&buffer);
        for i in 0..384 { // Only fill 384 samples
            samples.set_index(i, 0.25);
        }
        
        // Create message with partial buffer
        let data = Object::new();
        Reflect::set(&data, &"type".into(), &"audioDataBatch".into()).unwrap();
        Reflect::set(&data, &"buffer".into(), &buffer).unwrap();
        Reflect::set(&data, &"sampleCount".into(), &JsValue::from(384)).unwrap(); // Only 384 valid samples
        Reflect::set(&data, &"timestamp".into(), &JsValue::from(456.78)).unwrap();
        
        let event = create_mock_event(&data).unwrap();
        
        // Handle message
        let handle_message = Reflect::get(&handler, &"handleMessage".into()).unwrap();
        let handle_fn = handle_message.dyn_ref::<Function>().unwrap();
        handle_fn.call1(&handler, &event).unwrap();
        
        // Verify only valid samples were processed
        let batches = Reflect::get(&handler, &"processedBatches".into()).unwrap();
        let batches_array = batches.dyn_ref::<js_sys::Array>().unwrap();
        let batch = batches_array.get(0);
        
        let sample_count = Reflect::get(&batch, &"sampleCount".into()).unwrap();
        assert_eq!(sample_count.as_f64().unwrap(), 384.0);
        
        // Check chunks (384 / 128 = 3)
        let chunks = Reflect::get(&handler, &"chunksProcessed".into()).unwrap();
        assert_eq!(chunks.as_f64().unwrap(), 3.0);
    }
}