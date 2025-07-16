/**
 * AudioWorklet Processor for Real-Time Pitch Detection
 * 
 * This processor handles real-time audio processing in the dedicated audio thread.
 * It processes audio in fixed 128-sample chunks as required by the Web Audio API
 * and batches them before sending to the main thread for efficient data transfer.
 * 
 * Key Features:
 * - Fixed 128-sample chunk processing (Web Audio API standard)
 * - Batched audio data transfer (default: 1024 samples / 8 chunks)
 * - Transferable ArrayBuffers for zero-copy message passing
 * - Configurable batch size and timeout for low-latency scenarios
 * - Automatic buffer pool management to avoid allocations
 * - Error handling and processor lifecycle management
 * - Type-safe message protocol for reliable communication
 * 
 * Communication:
 * - Receives: Configuration messages (startProcessing, stopProcessing, updateBatchConfig)
 * - Sends: Batched audio data via audioDataBatch messages with transferables
 * 
 * Usage:
 * ```js
 * // Configure batching using typed messages
 * const message = protocol.createUpdateBatchConfigMessage({
 *     batchSize: 2048,      // samples per batch
 *     bufferTimeout: 30     // ms before sending partial buffer
 * });
 * processor.port.postMessage(message);
 * ```
 */

// Message Protocol (inlined for AudioWorklet compatibility)
// Message type constants matching Rust enums
const ToWorkletMessageType = {
    START_PROCESSING: 'startProcessing',
    STOP_PROCESSING: 'stopProcessing',
    UPDATE_TEST_SIGNAL_CONFIG: 'updateTestSignalConfig',
    UPDATE_BATCH_CONFIG: 'updateBatchConfig',
    UPDATE_BACKGROUND_NOISE_CONFIG: 'updateBackgroundNoiseConfig',
    GET_STATUS: 'getStatus'
};

const FromWorkletMessageType = {
    PROCESSOR_READY: 'processorReady',
    PROCESSING_STARTED: 'processingStarted',
    PROCESSING_STOPPED: 'processingStopped',
    AUDIO_DATA_BATCH: 'audioDataBatch',
    PROCESSING_ERROR: 'processingError',
    STATUS_UPDATE: 'status',
    TEST_SIGNAL_CONFIG_UPDATED: 'testSignalConfigUpdated',
    BACKGROUND_NOISE_CONFIG_UPDATED: 'backgroundNoiseConfigUpdated',
    BATCH_CONFIG_UPDATED: 'batchConfigUpdated',
    PROCESSOR_DESTROYED: 'processorDestroyed'
};

const WorkletErrorCode = {
    INITIALIZATION_FAILED: 'InitializationFailed',
    PROCESSING_FAILED: 'ProcessingFailed',
    BUFFER_OVERFLOW: 'BufferOverflow',
    INVALID_CONFIGURATION: 'InvalidConfiguration',
    MEMORY_ALLOCATION_FAILED: 'MemoryAllocationFailed',
    GENERIC: 'Generic'
};

// Simplified message protocol for AudioWorklet
class AudioWorkletMessageProtocol {
    constructor() {
        this.messageIdCounter = 0;
    }

    generateMessageId() {
        return ++this.messageIdCounter;
    }

    getCurrentTimestamp() {
        return (typeof performance !== 'undefined' && performance.now) ? performance.now() : Date.now();
    }

    createProcessorReadyMessage(options = {}) {
        return {
            type: FromWorkletMessageType.PROCESSOR_READY,
            chunkSize: options.chunkSize || 128,
            batchSize: options.batchSize || 1024,
            bufferPoolSize: options.bufferPoolSize || 4,
            sampleRate: options.sampleRate || 44100,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createProcessingStartedMessage() {
        return {
            type: FromWorkletMessageType.PROCESSING_STARTED,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createProcessingStoppedMessage() {
        return {
            type: FromWorkletMessageType.PROCESSING_STOPPED,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createAudioDataBatchMessage(buffer, options = {}) {
        return {
            type: FromWorkletMessageType.AUDIO_DATA_BATCH,
            buffer: buffer,
            sampleCount: options.sampleCount || 0,
            batchSize: options.batchSize || 1024,
            chunkCounter: options.chunkCounter || 0,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createProcessingErrorMessage(error, code = WorkletErrorCode.GENERIC) {
        return {
            type: FromWorkletMessageType.PROCESSING_ERROR,
            error: error,
            code: code,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createStatusUpdateMessage(status) {
        return {
            type: FromWorkletMessageType.STATUS_UPDATE,
            isProcessing: status.isProcessing,
            chunkCounter: status.chunkCounter,
            bufferPoolStats: status.bufferPoolStats,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createTestSignalConfigUpdatedMessage(config) {
        return {
            type: FromWorkletMessageType.TEST_SIGNAL_CONFIG_UPDATED,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createBackgroundNoiseConfigUpdatedMessage(config) {
        return {
            type: FromWorkletMessageType.BACKGROUND_NOISE_CONFIG_UPDATED,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createBatchConfigUpdatedMessage(config) {
        return {
            type: FromWorkletMessageType.BATCH_CONFIG_UPDATED,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    createProcessorDestroyedMessage() {
        return {
            type: FromWorkletMessageType.PROCESSOR_DESTROYED,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    validateMessage(message) {
        if (!message || typeof message !== 'object') {
            return false;
        }
        
        // Handle both direct messages and envelope messages with payload
        let messageType;
        if (message.payload && typeof message.payload === 'object') {
            // This is an envelope message from Rust
            messageType = message.payload.type;
        } else {
            // This is a direct message
            messageType = message.type;
        }
        
        if (!messageType || typeof messageType !== 'string') {
            return false;
        }
        
        const allMessageTypes = { ...ToWorkletMessageType, ...FromWorkletMessageType };
        return Object.values(allMessageTypes).includes(messageType);
    }

    validateBufferMetadata(buffer, metadata) {
        if (!buffer || !(buffer instanceof ArrayBuffer)) {
            return { valid: false, error: 'Invalid buffer: must be ArrayBuffer' };
        }
        if (!metadata || typeof metadata !== 'object') {
            return { valid: false, error: 'Invalid metadata: must be object' };
        }
        if (typeof metadata.sampleCount !== 'number' || metadata.sampleCount < 0) {
            return { valid: false, error: 'Invalid sampleCount: must be non-negative number' };
        }
        if (typeof metadata.batchSize !== 'number' || metadata.batchSize <= 0) {
            return { valid: false, error: 'Invalid batchSize: must be positive number' };
        }
        const expectedBufferSize = metadata.batchSize * 4;
        if (buffer.byteLength < expectedBufferSize) {
            return { valid: false, error: `Buffer too small: expected at least ${expectedBufferSize} bytes, got ${buffer.byteLength}` };
        }
        if (metadata.sampleCount > metadata.batchSize) {
            return { valid: false, error: `Sample count ${metadata.sampleCount} exceeds batch size ${metadata.batchSize}` };
        }
        return { valid: true };
    }

    getTransferableObjects(message) {
        const transferables = [];
        if (message.type === FromWorkletMessageType.AUDIO_DATA_BATCH && message.buffer) {
            transferables.push(message.buffer);
        }
        return transferables;
    }
}


class PitchDetectionProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
        
        // Constructor logging kept for debugging
        console.log('PitchDetectionProcessor: Constructor called - processor instance created');
        
        // Initialize message protocol
        this.messageProtocol = new AudioWorkletMessageProtocol();
        
        // Fixed chunk size as per Web Audio API specification
        this.chunkSize = 128;
        
        // Batch configuration for transferable buffers
        this.batchSize = 1024; // 8 chunks of 128 samples
        this.chunksPerBatch = this.batchSize / this.chunkSize;
        
        // Enhanced buffer management with lifecycle tracking
        this.bufferStats = {
            acquireCount: 0,
            transferCount: 0,
            poolExhaustedCount: 0,
            totalBytesTransferred: 0,
            averageBufferUtilization: 0.0,
            bufferLifecycle: {
                created: 0,
                transferred: 0,
                detached: 0
            }
        };
        
        // Current batch buffer tracking
        this.currentBuffer = null;
        this.currentBufferArray = null;
        this.writePosition = 0;
        
        // Timeout configuration for low-latency sending
        this.bufferTimeout = 50; // 50ms timeout for partial buffers
        this.lastBufferStartTime = 0;
        
        // Processing state
        this.isProcessing = false;
        this.chunkCounter = 0;
        
        // Test signal configuration
        this.testSignalConfig = {
            enabled: false,
            frequency: 440.0,
            amplitude: 0.3,
            waveform: 'sine',
            sample_rate: sampleRate // Use the actual sample rate from AudioWorklet
        };
        
        // Background noise configuration (independent of test signal)
        this.backgroundNoiseConfig = {
            enabled: false,
            level: 0.0,
            type: 'white_noise'  // white_noise, pink_noise
        };
        
        // Test signal generation state
        this.testSignalPhase = 0.0;
        
        // Setup message handling
        this.port.onmessage = (event) => {
            // Message logging kept for debugging
            console.log('PitchDetectionProcessor: Received message:', event.data);
            this.handleMessage(event.data);
        };
        
        // Initialize processor with typed message
        const readyMessage = this.messageProtocol.createProcessorReadyMessage({
            chunkSize: this.chunkSize,
            batchSize: this.batchSize,
            bufferPoolSize: 4, // No longer using pool but keeping for compatibility
            sampleRate: sampleRate
        });
        this.port.postMessage(readyMessage);
        
        console.log('PitchDetectionProcessor: Constructor complete, ready for processing');
    }
    
    /**
     * Acquire a new buffer for batching with lifecycle tracking
     */
    acquireNewBuffer() {
        this.bufferStats.acquireCount++;
        this.bufferStats.bufferLifecycle.created++;
        
        this.currentBuffer = new ArrayBuffer(this.batchSize * 4); // 4 bytes per float32
        this.currentBufferArray = new Float32Array(this.currentBuffer);
        this.writePosition = 0;
        this.lastBufferStartTime = this.currentTime || (typeof performance !== 'undefined' ? performance.now() : Date.now());
        
        console.log('PitchDetectionProcessor: Buffer acquired - lifecycle:', this.bufferStats.bufferLifecycle);
    }
    
    /**
     * Send the current buffer to main thread using transferable
     */
    sendCurrentBuffer() {
        if (!this.currentBuffer || !this.currentBufferArray) {
            return;
        }
        
        // Check if buffer is already detached (safety check)
        if (this.currentBuffer.byteLength === 0) {
            console.error('PitchDetectionProcessor: Attempting to send already detached buffer');
            this.bufferStats.bufferLifecycle.detached++;
            this.currentBuffer = null;
            this.currentBufferArray = null;
            this.writePosition = 0;
            return;
        }
        
        // Only send if we have data in the buffer
        if (this.writePosition > 0) {
            try {
                // Validate buffer metadata before sending
                const metadata = {
                    sampleCount: this.writePosition,
                    batchSize: this.batchSize,
                    chunkCounter: this.chunkCounter
                };
                
                const validation = this.messageProtocol.validateBufferMetadata(this.currentBuffer, metadata);
                if (!validation.valid) {
                    console.error('PitchDetectionProcessor: Buffer validation failed:', validation.error);
                    this.sendErrorMessage(`Buffer validation failed: ${validation.error}`, WorkletErrorCode.BUFFER_OVERFLOW);
                    return;
                }
                
                // Create typed message for audio data batch
                const batchMessage = this.messageProtocol.createAudioDataBatchMessage(this.currentBuffer, metadata);
                
                // Send buffer with transferable
                const transferables = this.messageProtocol.getTransferableObjects(batchMessage);
                this.port.postMessage(batchMessage, transferables);
                
                // Track buffer transfer statistics
                this.bufferStats.transferCount++;
                this.bufferStats.bufferLifecycle.transferred++;
                this.bufferStats.totalBytesTransferred += this.writePosition * 4; // 4 bytes per float32
                
                // Calculate buffer utilization
                const utilization = this.writePosition / this.batchSize;
                this.bufferStats.averageBufferUtilization = 
                    (this.bufferStats.averageBufferUtilization * (this.bufferStats.transferCount - 1) + utilization) / 
                    this.bufferStats.transferCount;
                
                console.log('PitchDetectionProcessor: Buffer transferred - utilization:', 
                    (utilization * 100).toFixed(1) + '%, average:', 
                    (this.bufferStats.averageBufferUtilization * 100).toFixed(1) + '%');
                
                // Clear references to transferred buffer immediately
                this.currentBuffer = null;
                this.currentBufferArray = null;
                this.writePosition = 0;
                
            } catch (error) {
                console.error('PitchDetectionProcessor: Error sending buffer:', error);
                // Clear buffer references on error
                this.currentBuffer = null;
                this.currentBufferArray = null;
                this.writePosition = 0;
            }
        }
    }
    
    /**
     * Handle messages from main thread using typed message protocol
     * @param {Object} message - Message from main thread
     */
    handleMessage(message) {
        // Validate incoming message
        if (!this.messageProtocol.validateMessage(message)) {
            console.error('PitchDetectionProcessor: Invalid message received:', message);
            this.sendErrorMessage('Invalid message format', WorkletErrorCode.INVALID_CONFIGURATION);
            return;
        }

        // Extract actual message from envelope if needed
        let actualMessage;
        if (message.payload && typeof message.payload === 'object') {
            // This is an envelope message from Rust
            actualMessage = message.payload;
        } else {
            // This is a direct message
            actualMessage = message;
        }

        // Type-safe message handling
        try {
            switch (actualMessage.type) {
                case ToWorkletMessageType.START_PROCESSING:
                    this.isProcessing = true;
                    const startedMessage = this.messageProtocol.createProcessingStartedMessage();
                    this.port.postMessage(startedMessage);
                    break;
                
                case ToWorkletMessageType.STOP_PROCESSING:
                    this.isProcessing = false;
                    
                    // Send any remaining buffered data before stopping
                    if (this.currentBuffer && this.currentBufferArray && this.writePosition > 0) {
                        this.sendCurrentBuffer();
                    }
                    
                    const stoppedMessage = this.messageProtocol.createProcessingStoppedMessage();
                    this.port.postMessage(stoppedMessage);
                    break;
                
                case ToWorkletMessageType.GET_STATUS:
                    const statusData = {
                        isProcessing: this.isProcessing,
                        chunkCounter: this.chunkCounter,
                        bufferPoolStats: {
                            ...this.bufferStats,
                            availableBuffers: this.currentBuffer ? 0 : 1,
                            inUseBuffers: this.currentBuffer ? 1 : 0,
                            totalBuffers: 1,
                            // Enhanced buffer pool reporting
                            bufferUtilizationPercent: (this.bufferStats.averageBufferUtilization * 100).toFixed(1),
                            totalMegabytesTransferred: (this.bufferStats.totalBytesTransferred / 1024 / 1024).toFixed(2),
                            bufferLifecycle: this.bufferStats.bufferLifecycle
                        }
                    };
                    const statusMessage = this.messageProtocol.createStatusUpdateMessage(statusData);
                    this.port.postMessage(statusMessage);
                    break;
                
                case ToWorkletMessageType.UPDATE_TEST_SIGNAL_CONFIG:
                    if (actualMessage.config) {
                        this.testSignalConfig = { ...this.testSignalConfig, ...actualMessage.config };
                        // Reset phase when configuration changes
                        this.testSignalPhase = 0.0;
                        console.log('PitchDetectionProcessor: Test signal config updated:', this.testSignalConfig);
                        const configUpdatedMessage = this.messageProtocol.createTestSignalConfigUpdatedMessage(this.testSignalConfig);
                        this.port.postMessage(configUpdatedMessage);
                    }
                    break;
                
                case ToWorkletMessageType.UPDATE_BACKGROUND_NOISE_CONFIG:
                    if (actualMessage.config) {
                        this.backgroundNoiseConfig = { ...this.backgroundNoiseConfig, ...actualMessage.config };
                        console.log('PitchDetectionProcessor: Background noise config updated:', this.backgroundNoiseConfig);
                        const noiseConfigUpdatedMessage = this.messageProtocol.createBackgroundNoiseConfigUpdatedMessage(this.backgroundNoiseConfig);
                        this.port.postMessage(noiseConfigUpdatedMessage);
                    }
                    break;
                
                case ToWorkletMessageType.UPDATE_BATCH_CONFIG:
                    if (actualMessage.config) {
                        // Update batch size if provided
                        if (actualMessage.config.batchSize && actualMessage.config.batchSize > 0) {
                            // Ensure batch size is a multiple of chunk size
                            const newBatchSize = Math.ceil(actualMessage.config.batchSize / this.chunkSize) * this.chunkSize;
                            
                            // Send any pending data before changing batch size
                            if (this.currentBuffer && this.writePosition > 0) {
                                this.sendCurrentBuffer();
                            }
                            
                            // Update configuration
                            this.batchSize = newBatchSize;
                            this.chunksPerBatch = this.batchSize / this.chunkSize;
                            
                            // Reset buffer state with new size
                            this.currentBuffer = null;
                            this.currentBufferArray = null;
                            this.writePosition = 0;
                        }
                        
                        // Update timeout if provided
                        if (actualMessage.config.bufferTimeout !== undefined) {
                            this.bufferTimeout = Math.max(0, actualMessage.config.bufferTimeout);
                        }
                        
                        console.log('PitchDetectionProcessor: Batch config updated:', {
                            batchSize: this.batchSize,
                            bufferTimeout: this.bufferTimeout
                        });
                        
                        const batchConfigUpdatedMessage = this.messageProtocol.createBatchConfigUpdatedMessage({
                            batchSize: this.batchSize,
                            chunksPerBatch: this.chunksPerBatch,
                            bufferTimeout: this.bufferTimeout
                        });
                        this.port.postMessage(batchConfigUpdatedMessage);
                    }
                    break;
                
                default:
                    console.warn('PitchDetectionProcessor: Unknown message type:', actualMessage.type);
                    this.sendErrorMessage(`Unknown message type: ${actualMessage.type}`, WorkletErrorCode.INVALID_CONFIGURATION);
            }
        } catch (error) {
            console.error('PitchDetectionProcessor: Error handling message:', error);
            this.sendErrorMessage(error.message, WorkletErrorCode.PROCESSING_FAILED);
        }
    }

    /**
     * Send an error message to the main thread
     * @param {string} errorMessage - Error message
     * @param {string} errorCode - Error code
     */
    sendErrorMessage(errorMessage, errorCode = WorkletErrorCode.GENERIC) {
        try {
            const errorMsg = this.messageProtocol.createProcessingErrorMessage(errorMessage, errorCode);
            this.port.postMessage(errorMsg);
        } catch (error) {
            console.error('PitchDetectionProcessor: Failed to send error message:', error);
        }
    }
    
    /**
     * Generate test signal samples
     * @param {number} numSamples - Number of samples to generate
     * @returns {Float32Array} - Generated test signal samples
     */
    generateTestSignal(numSamples) {
        const samples = new Float32Array(numSamples);
        const config = this.testSignalConfig;
        
        if (!config.enabled) {
            return samples; // Return silence if disabled
        }
        
        const phaseIncrement = (2 * Math.PI * config.frequency) / config.sample_rate;
        
        for (let i = 0; i < numSamples; i++) {
            let sample = 0.0;
            
            // Generate waveform
            switch (config.waveform) {
                case 'sine':
                    sample = Math.sin(this.testSignalPhase);
                    break;
                case 'square':
                    sample = Math.sin(this.testSignalPhase) >= 0 ? 1.0 : -1.0;
                    break;
                case 'sawtooth':
                    sample = 2.0 * (this.testSignalPhase / (2 * Math.PI) - Math.floor(this.testSignalPhase / (2 * Math.PI) + 0.5));
                    break;
                case 'triangle':
                    const t = this.testSignalPhase / (2 * Math.PI) - Math.floor(this.testSignalPhase / (2 * Math.PI));
                    sample = t < 0.5 ? 4.0 * t - 1.0 : 3.0 - 4.0 * t;
                    break;
                case 'white_noise':
                    sample = (Math.random() * 2.0 - 1.0);
                    break;
                case 'pink_noise':
                    // Simplified pink noise approximation
                    sample = (Math.random() * 2.0 - 1.0) * 0.5;
                    break;
                default:
                    sample = Math.sin(this.testSignalPhase);
            }
            
            // Apply amplitude scaling
            sample *= config.amplitude;
            
            // Clamp to valid range
            sample = Math.max(-1.0, Math.min(1.0, sample));
            
            samples[i] = sample;
            
            // Update phase for next sample
            this.testSignalPhase += phaseIncrement;
            
            // Keep phase in [0, 2π] range to prevent precision issues
            if (this.testSignalPhase >= 2 * Math.PI) {
                this.testSignalPhase -= 2 * Math.PI;
            }
        }
        
        return samples;
    }
    
    /**
     * Generate background noise samples
     * @param {number} numSamples - Number of samples to generate
     * @returns {Float32Array} - Generated background noise samples
     */
    generateBackgroundNoise(numSamples) {
        const samples = new Float32Array(numSamples);
        const config = this.backgroundNoiseConfig;
        
        if (!config.enabled || config.level <= 0.0) {
            return samples; // Return silence if disabled or level is 0
        }
        
        for (let i = 0; i < numSamples; i++) {
            let sample = 0.0;
            
            // Generate noise based on type
            switch (config.type) {
                case 'white_noise':
                    sample = (Math.random() * 2.0 - 1.0);
                    break;
                case 'pink_noise':
                    // Simplified pink noise approximation
                    sample = (Math.random() * 2.0 - 1.0) * 0.5;
                    break;
                default:
                    sample = (Math.random() * 2.0 - 1.0); // Default to white noise
            }
            
            // Apply level scaling
            sample *= config.level;
            
            // Clamp to valid range
            sample = Math.max(-1.0, Math.min(1.0, sample));
            
            samples[i] = sample;
        }
        
        return samples;
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
        
        // Generate processed audio: test signal OR microphone input
        let processedAudio;
        if (this.testSignalConfig.enabled) {
            // Test signal is enabled - replace mic input with test signal
            processedAudio = this.generateTestSignal(this.chunkSize);
        } else {
            // Use microphone input
            processedAudio = new Float32Array(inputChannel);
        }
        
        // Generate and mix background noise (independent of test signal/mic)
        if (this.backgroundNoiseConfig.enabled) {
            const backgroundNoise = this.generateBackgroundNoise(this.chunkSize);
            
            // Mix background noise with the processed audio
            for (let i = 0; i < this.chunkSize; i++) {
                processedAudio[i] += backgroundNoise[i];
                
                // Clamp to valid range to prevent clipping
                processedAudio[i] = Math.max(-1.0, Math.min(1.0, processedAudio[i]));
            }
        }
        
        // Pass-through processed audio to output
        if (output && output.length > 0 && output[0]) {
            const outputChannel = output[0];
            if (outputChannel && outputChannel.length === processedAudio.length) {
                outputChannel.set(processedAudio);
            }
        }
        
        // Accumulate processed audio data for batching
        if (this.isProcessing) {
            try {
                // Ensure we have a buffer to write to
                if (!this.currentBuffer || !this.currentBufferArray) {
                    this.acquireNewBuffer();
                }
                
                // If we still don't have a buffer (pool exhausted), skip this chunk
                if (!this.currentBufferArray) {
                    console.warn('PitchDetectionProcessor: No buffer available, skipping chunk');
                    this.chunkCounter++;
                    return true;
                }
                
                // Copy the processed audio into the accumulation buffer
                const remainingSpace = this.batchSize - this.writePosition;
                const samplesToWrite = Math.min(this.chunkSize, remainingSpace);
                
                // Write samples to the current position
                this.currentBufferArray.set(processedAudio.subarray(0, samplesToWrite), this.writePosition);
                this.writePosition += samplesToWrite;
                
                // Check if buffer is full or timeout has elapsed
                const currentTime = this.currentTime || (typeof performance !== 'undefined' ? performance.now() : Date.now());
                const timeElapsed = currentTime - this.lastBufferStartTime;
                const shouldSendDueToTimeout = this.writePosition > 0 && timeElapsed >= this.bufferTimeout;
                
                if (this.writePosition >= this.batchSize || shouldSendDueToTimeout) {
                    // Send the batch (full or partial due to timeout)
                    this.sendCurrentBuffer();
                    
                    // Handle any remaining samples if chunk was partially written
                    if (samplesToWrite < this.chunkSize) {
                        this.acquireNewBuffer();
                        if (this.currentBufferArray) {
                            const remainingSamples = this.chunkSize - samplesToWrite;
                            this.currentBufferArray.set(
                                processedAudio.subarray(samplesToWrite),
                                0
                            );
                            this.writePosition = remainingSamples;
                        }
                    }
                }
                
                this.chunkCounter++;
            } catch (error) {
                console.error('PitchDetectionProcessor: Error accumulating audio data:', error);
                
                // Send structured error notification to main thread
                this.sendErrorMessage(`Error accumulating audio data: ${error.message}`, WorkletErrorCode.PROCESSING_FAILED);
            }
        } else {
            // Not processing, but still increment chunk counter
            this.chunkCounter++;
        }
        
        // Keep processor alive
        return true;
    }
    
    /**
     * Called when processor is being terminated
     */
    destroy() {
        this.isProcessing = false;
        
        // Send any remaining buffered data before destroying
        if (this.currentBuffer && this.currentBufferArray && this.writePosition > 0) {
            this.sendCurrentBuffer();
        }
        
        const destroyedMessage = this.messageProtocol.createProcessorDestroyedMessage();
        this.port.postMessage(destroyedMessage);
    }
}

// Register the processor with the AudioWorklet
registerProcessor('pitch-processor', PitchDetectionProcessor);