/**
 * AudioWorklet Processor for Real-Time Pitch Detection
 * 
 * This processor handles real-time audio processing in the dedicated audio thread.
 * It processes audio in fixed 128-sample chunks as required by the Web Audio API
 * and forwards audio data to the main thread for pitch detection processing.
 * 
 * Key Features:
 * - Fixed 128-sample chunk processing (Web Audio API standard)
 * - Real-time audio data forwarding via MessagePort
 * - Low-latency processing with minimal overhead
 * - Error handling and processor lifecycle management
 * 
 * Communication:
 * - Receives: Configuration and control messages from main thread
 * - Sends: Audio data chunks and processing status to main thread
 */

class PitchDetectionProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
        
        // Constructor logging kept for debugging
        console.log('PitchDetectionProcessor: Constructor called - processor instance created');
        
        // Fixed chunk size as per Web Audio API specification
        this.chunkSize = 128;
        
        // Processing state
        this.isProcessing = false;
        this.chunkCounter = 0;
        
        // Setup message handling
        this.port.onmessage = (event) => {
            // Message logging kept for debugging
            console.log('PitchDetectionProcessor: Received message:', event.data);
            this.handleMessage(event.data);
        };
        
        // Initialize processor
        this.port.postMessage({
            type: 'processorReady',
            chunkSize: this.chunkSize,
            timestamp: this.currentTime || 0
        });
        
        console.log('PitchDetectionProcessor: Constructor complete, ready for processing');
    }
    
    /**
     * Handle messages from main thread
     * @param {Object} message - Message from main thread
     */
    handleMessage(message) {
        switch (message.type) {
            case 'startProcessing':
                this.isProcessing = true;
                this.port.postMessage({
                    type: 'processingStarted',
                    timestamp: this.currentTime || 0
                });
                break;
                
            case 'stopProcessing':
                this.isProcessing = false;
                this.port.postMessage({
                    type: 'processingStopped',
                    timestamp: this.currentTime || 0
                });
                break;
                
            case 'getStatus':
                this.port.postMessage({
                    type: 'status',
                    isProcessing: this.isProcessing,
                    chunkCounter: this.chunkCounter,
                    timestamp: this.currentTime || 0
                });
                break;
                
            default:
                console.warn('PitchDetectionProcessor: Unknown message type:', message.type);
        }
    }
    
    /**
     * Process audio data in 128-sample chunks
     * This method is called by the Web Audio API for each processing quantum
     * 
     * @param {Float32Array[][]} inputs - Input audio data (channels x samples)
     * @param {Float32Array[][]} outputs - Output audio data (channels x samples)
     * @param {Object} parameters - Audio parameters (unused in this implementation)
     * @returns {boolean} - True to keep processor alive, false to terminate
     */
    process(inputs, outputs, parameters) {
        // Debug logging disabled - verification complete
        // if (this.chunkCounter < 5) {
        //     console.log(`PitchDetectionProcessor: process() called - chunk ${this.chunkCounter}, inputs: ${inputs.length}, outputs: ${outputs.length}`);
        // }
        
        const input = inputs[0];
        const output = outputs[0];
        
        // Debug logging disabled - remove spam
        // if (this.chunkCounter % 100 === 0) {
        //     console.log(`AudioWorklet: Processing chunk ${this.chunkCounter}, input channels: ${input ? input.length : 0}, processing: ${this.isProcessing}`);
        // }
        
        // Check if we have valid input
        if (!input || input.length === 0) {
            // No input available, pass through silence
            // Debug logging disabled
            // if (this.chunkCounter % 100 === 0) {
            //     console.log(`AudioWorklet: No input available - inputs array length: ${inputs.length}`);
            // }
            if (output && output.length > 0) {
                const outputChannel = output[0];
                if (outputChannel) {
                    outputChannel.fill(0);
                }
            }
            this.chunkCounter++;
            return true;
        }
        
        // Get first channel (mono processing for pitch detection)
        const inputChannel = input[0];
        
        if (!inputChannel || inputChannel.length !== this.chunkSize) {
            // Invalid input chunk size, pass through silence
            // Debug logging disabled
            // if (this.chunkCounter % 100 === 0) {
            //     console.log(`AudioWorklet: Invalid input - channel: ${!!inputChannel}, length: ${inputChannel ? inputChannel.length : 0}, expected: ${this.chunkSize}`);
            // }
            if (output && output.length > 0 && output[0]) {
                output[0].fill(0);
            }
            this.chunkCounter++;
            return true;
        }
        
        // Debug logging disabled - signal detection confirmed working
        // if (this.chunkCounter % 100 === 0) {
        //     const hasSignal = inputChannel.some(sample => Math.abs(sample) > 0.001);
        //     const maxLevel = Math.max(...inputChannel.map(Math.abs));
        //     console.log(`AudioWorklet: Input signal - hasSignal: ${hasSignal}, maxLevel: ${maxLevel.toFixed(4)}`);
        // }
        
        // Pass-through audio (copy input to output)
        if (output && output.length > 0 && output[0]) {
            const outputChannel = output[0];
            if (outputChannel && outputChannel.length === inputChannel.length) {
                outputChannel.set(inputChannel);
            }
        }
        
        // Forward audio data to main thread for processing
        if (this.isProcessing) {
            try {
                // Create a copy of the input data for thread-safe transfer
                const audioData = new Float32Array(inputChannel);
                
                this.port.postMessage({
                    type: 'audioData',
                    samples: audioData,
                    chunkSize: this.chunkSize,
                    chunkCounter: this.chunkCounter,
                    timestamp: this.currentTime || 0
                });
                
                this.chunkCounter++;
            } catch (error) {
                console.error('PitchDetectionProcessor: Error sending audio data:', error);
                
                // Send error notification to main thread
                this.port.postMessage({
                    type: 'processingError',
                    error: error.message,
                    timestamp: this.currentTime || 0
                });
            }
        }
        
        // Keep processor alive
        return true;
    }
    
    /**
     * Called when processor is being terminated
     */
    destroy() {
        this.isProcessing = false;
        this.port.postMessage({
            type: 'processorDestroyed',
            timestamp: this.currentTime || 0
        });
    }
}

// Register the processor with the AudioWorklet
registerProcessor('pitch-processor', PitchDetectionProcessor);