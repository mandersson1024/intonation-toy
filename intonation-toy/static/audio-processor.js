/**
 * AudioWorklet Processor for Real-Time Pitch Detection
 * 
 * This processor handles real-time audio processing in the dedicated audio thread.
 * It processes audio in fixed 128-sample chunks as required by the Web Audio API
 * and batches them before sending to the main thread for efficient data transfer.
 * 
 * Key Features:
 * - Fixed 128-sample chunk processing (Web Audio API standard)
 * - Batched audio data transfer (default: 4096 samples / 32 chunks)
 * - Transferable ArrayBuffers for zero-copy message passing
 * - Configurable batch size and timeout for low-latency scenarios
 * - Buffer pool management with ping-pong recycling pattern
 * - Error handling and processor lifecycle management
 * - Type-safe message protocol for reliable communication
 * 
 * Communication:
 * - Receives: Configuration messages (startProcessing, stopProcessing, updateBatchConfig, returnBuffer)
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
 * 
 * // Buffer pool usage with ping-pong pattern
 * const buffer = processor.bufferPool.acquire();
 * if (buffer) {
 *     // Fill buffer with audio data
 *     processor.port.postMessage(message, [buffer]);
 *     processor.bufferPool.markTransferred(buffer);
 * }
 * ```
 */

// Buffer size constants - IMPORTANT: Must match Rust BUFFER_SIZE constant in engine/audio/buffer.rs
const AUDIO_CHUNK_SIZE = 128;  // Fixed AudioWorklet chunk size
const BUFFER_SIZE = AUDIO_CHUNK_SIZE * 32;  // 4096 samples - matches Rust constant

// AudioWorklet compatibility helpers
// Note: performance and other APIs are not available in AudioWorklet context
function getCurrentTime() {
    return Date.now();
}

// TransferableBufferPool class (inlined for AudioWorklet compatibility)
// Note: importScripts is not available in AudioWorklet context
class TransferableBufferPool {
    constructor(poolSize = 4, bufferCapacity = BUFFER_SIZE, options = {}) {
        this.poolSize = poolSize;
        this.bufferCapacity = bufferCapacity;
        this.buffers = [];
        this.availableIndices = [];
        this.inUseBuffers = new Map();
        
        // Buffer lifecycle management
        this.bufferStates = [];
        this.bufferTimestamps = [];
        this.bufferIds = [];
        this.nextBufferId = 1;
        
        // Configuration options
        this.options = {
            timeout: options.timeout || 5000,
            enableTimeouts: options.enableTimeouts !== false,
            enableValidation: options.enableValidation !== false,
            ...options
        };
        
        // Timeout checker properties (setInterval not available in AudioWorklet)
        this.timeoutCheckEnabled = false;
        this.lastTimeoutCheck = 0;
        
        // Buffer states enum
        this.BUFFER_STATES = {
            AVAILABLE: 'available',
            IN_FLIGHT: 'in_flight',
            PROCESSING: 'processing',
            TIMED_OUT: 'timed_out'
        };
        
        // Performance counters
        this.perfCounters = {
            allocationCount: 0,
            totalAcquisitionTime: 0,
            fastestAcquisition: Infinity,
            slowestAcquisition: 0,
            poolHitRate: 0
        };
        
        // Statistics
        this.stats = {
            acquireCount: 0,
            transferCount: 0,
            poolExhaustedCount: 0,
            timeoutCount: 0,
            validationFailures: 0,
            returnedBuffers: 0,
            bufferReuseRate: 0,
            averageTurnoverTime: 0
        };
        
        // Initialize pool with pre-allocated buffers
        for (let i = 0; i < poolSize; i++) {
            this.buffers.push(new ArrayBuffer(bufferCapacity * 4));
            this.availableIndices.push(i);
            this.bufferStates.push(this.BUFFER_STATES.AVAILABLE);
            this.bufferTimestamps.push(0);
            this.bufferIds.push(0);
            this.perfCounters.allocationCount++;
        }
        
        // Start timeout checker if enabled
        if (this.options.enableTimeouts) {
            this.startTimeoutChecker();
        }
    }
    
    acquire() {
        const startTime = getCurrentTime();
        this.stats.acquireCount++;
        // Debug logging removed
        // Milestone logging removed
        
        // Check for timed out buffers periodically (since setInterval is not available in AudioWorklet)
        if (this.timeoutCheckEnabled && this.lastTimeoutCheck > 0) {
            const timeSinceLastCheck = Date.now() - this.lastTimeoutCheck;
            if (timeSinceLastCheck > 2000) { // Check every 2 seconds
                this.checkForTimedOutBuffers();
                this.lastTimeoutCheck = Date.now();
            }
        }
        
        
        if (this.availableIndices.length === 0) {
            this.stats.poolExhaustedCount++;
            console.warn('TransferableBufferPool: Pool exhausted, no buffers available');
            
            const acquisitionTime = getCurrentTime() - startTime;
            this.updateAcquisitionMetrics(acquisitionTime);
            
            return null;
        }
        
        const index = this.availableIndices.pop();
        const buffer = this.buffers[index];
        
        // Track buffer usage
        this.inUseBuffers.set(buffer, index);
        
        // Update buffer lifecycle tracking
        this.bufferStates[index] = this.BUFFER_STATES.IN_FLIGHT;
        this.bufferTimestamps[index] = Date.now();
        this.bufferIds[index] = this.nextBufferId++;
        
        // Track acquisition time
        const acquisitionTime = getCurrentTime() - startTime;
        this.updateAcquisitionMetrics(acquisitionTime);
        
        return {
            buffer: buffer,
            bufferId: this.bufferIds[index]
        };
    }
    
    markTransferred(buffer) {
        this.stats.transferCount++;
        // Debug logging removed
        
        const index = this.inUseBuffers.get(buffer);
        if (index === undefined) {
            console.error('TransferableBufferPool: Attempting to mark unknown buffer as transferred');
            return;
        }
        
        // Remove from in-use tracking
        this.inUseBuffers.delete(buffer);
        
        // Update buffer lifecycle tracking
        this.bufferStates[index] = this.BUFFER_STATES.PROCESSING;
        // Keep timestamp for timeout tracking
    }
    
    release(buffer) {
        const index = this.inUseBuffers.get(buffer);
        if (index === undefined) {
            console.error('TransferableBufferPool: Attempting to release unknown buffer');
            return;
        }
        
        // Remove from in-use tracking
        this.inUseBuffers.delete(buffer);
        
        // Update buffer lifecycle tracking
        this.bufferStates[index] = this.BUFFER_STATES.AVAILABLE;
        this.bufferTimestamps[index] = 0;
        this.bufferIds[index] = 0;
        
        // Mark index as available again
        this.availableIndices.push(index);
    }
    
    returnBuffer(bufferId, buffer) {
        if (!buffer || !(buffer instanceof ArrayBuffer)) {
            console.error('TransferableBufferPool: Invalid buffer provided for return');
            this.stats.validationFailures++;
            return false;
        }
        
        // Validate buffer size matches expected
        const expectedSize = this.bufferCapacity * 4;
        if (this.options.enableValidation && buffer.byteLength !== expectedSize) {
            console.error('TransferableBufferPool: Returned buffer size mismatch. Expected:', expectedSize, 'Got:', buffer.byteLength);
            this.stats.validationFailures++;
            return false;
        }
        
        // Find buffer by ID for proper lifecycle tracking
        let targetIndex = -1;
        for (let i = 0; i < this.poolSize; i++) {
            if (this.bufferIds[i] === bufferId && this.bufferStates[i] === this.BUFFER_STATES.PROCESSING) {
                targetIndex = i;
                break;
            }
        }
        
        if (targetIndex !== -1) {
            // Found the buffer, return it to its original slot
            this.buffers[targetIndex] = buffer;
            this.bufferStates[targetIndex] = this.BUFFER_STATES.AVAILABLE;
            this.bufferTimestamps[targetIndex] = 0;
            this.bufferIds[targetIndex] = 0;
            
            if (!this.availableIndices.includes(targetIndex)) {
                this.availableIndices.push(targetIndex);
            }
            
            this.stats.returnedBuffers++;
            
            // Calculate buffer reuse rate
            if (this.stats.transferCount > 0) {
                this.stats.bufferReuseRate = (this.stats.returnedBuffers / this.stats.transferCount) * 100;
            }
            
            // Buffer returned to pool successfully
            return true;
        }
        
        console.warn('TransferableBufferPool: Could not return buffer to pool');
        return false;
    }
    
    getStats() {
        // Always return current stats, not cached values
        const stats = {
            acquireCount: this.stats.acquireCount,
            transferCount: this.stats.transferCount,
            poolExhaustedCount: this.stats.poolExhaustedCount,
            timeoutCount: this.stats.timeoutCount,
            validationFailures: this.stats.validationFailures,
            returnedBuffers: this.stats.returnedBuffers,
            bufferReuseRate: this.stats.bufferReuseRate,
            averageTurnoverTime: this.stats.averageTurnoverTime,
            availableBuffers: this.availableIndices.length,
            inUseBuffers: this.inUseBuffers.size,
            totalBuffers: this.poolSize
        };
        return stats;
    }
    
    startTimeoutChecker() {
        // Note: setInterval is not available in AudioWorklet context
        // We'll check for timeouts manually during buffer operations
        this.timeoutCheckEnabled = true;
        this.lastTimeoutCheck = Date.now();
    }
    
    stopTimeoutChecker() {
        this.timeoutCheckEnabled = false;
        this.lastTimeoutCheck = 0;
    }
    
    checkForTimedOutBuffers() {
        const now = Date.now();
        let reclaimedCount = 0;
        
        for (let i = 0; i < this.poolSize; i++) {
            const state = this.bufferStates[i];
            const timestamp = this.bufferTimestamps[i];
            
            if ((state === this.BUFFER_STATES.IN_FLIGHT || state === this.BUFFER_STATES.PROCESSING) && 
                timestamp > 0 && 
                (now - timestamp) > this.options.timeout) {
                
                this.bufferStates[i] = this.BUFFER_STATES.TIMED_OUT;
                this.bufferTimestamps[i] = 0;
                const bufferId = this.bufferIds[i];
                this.bufferIds[i] = 0;
                
                // Create a new buffer to replace the lost one
                this.buffers[i] = new ArrayBuffer(this.bufferCapacity * 4);
                this.bufferStates[i] = this.BUFFER_STATES.AVAILABLE;
                this.perfCounters.allocationCount++;
                
                if (!this.availableIndices.includes(i)) {
                    this.availableIndices.push(i);
                }
                
                reclaimedCount++;
                this.stats.timeoutCount++;
                
                console.warn(`TransferableBufferPool: Buffer ${bufferId} timed out after ${this.options.timeout}ms, reclaimed buffer at index ${i}`);
            }
        }
        
        if (reclaimedCount > 0) {
            console.log(`TransferableBufferPool: Reclaimed ${reclaimedCount} timed out buffers`);
        }
    }
    
    updateAcquisitionMetrics(acquisitionTime) {
        this.perfCounters.totalAcquisitionTime += acquisitionTime;
        this.perfCounters.fastestAcquisition = Math.min(this.perfCounters.fastestAcquisition, acquisitionTime);
        this.perfCounters.slowestAcquisition = Math.max(this.perfCounters.slowestAcquisition, acquisitionTime);
        
        // Calculate pool hit rate
        if (this.stats.acquireCount > 0) {
            this.perfCounters.poolHitRate = ((this.stats.acquireCount - this.stats.poolExhaustedCount) / this.stats.acquireCount) * 100;
        }
    }
    
    getPerformanceMetrics() {
        const avgAcquisitionTime = this.stats.acquireCount > 0 ? 
            this.perfCounters.totalAcquisitionTime / this.stats.acquireCount : 0;
            
        return {
            allocationCount: this.perfCounters.allocationCount,
            poolHitRate: this.perfCounters.poolHitRate.toFixed(2) + '%',
            acquisitionMetrics: {
                average: avgAcquisitionTime.toFixed(3) + 'ms',
                fastest: this.perfCounters.fastestAcquisition === Infinity ? 'N/A' : 
                    this.perfCounters.fastestAcquisition.toFixed(3) + 'ms',
                slowest: this.perfCounters.slowestAcquisition.toFixed(3) + 'ms',
                total: this.perfCounters.totalAcquisitionTime.toFixed(3) + 'ms'
            }
        };
    }
    
    
}

// Message Protocol (inlined for AudioWorklet compatibility)
// Message type constants matching Rust enums
const ToWorkletMessageType = {
    START_PROCESSING: 'startProcessing',
    STOP_PROCESSING: 'stopProcessing',
    UPDATE_BATCH_CONFIG: 'updateBatchConfig',
    RETURN_BUFFER: 'returnBuffer'
};

const FromWorkletMessageType = {
    PROCESSOR_READY: 'processorReady',
    PROCESSING_STARTED: 'processingStarted',
    PROCESSING_STOPPED: 'processingStopped',
    AUDIO_DATA_BATCH: 'audioDataBatch',
    PROCESSING_ERROR: 'processingError',
    STATUS_UPDATE: 'statusUpdate',
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
        return getCurrentTime();
    }

    createProcessorReadyMessage(options = {}) {
        const messageId = this.generateMessageId();
        const timestamp = this.getCurrentTimestamp();
        
        return {
            messageId: messageId,
            timestamp: timestamp,
            payload: {
                type: FromWorkletMessageType.PROCESSOR_READY,
                chunkSize: options.chunkSize || AUDIO_CHUNK_SIZE,
                batchSize: options.batchSize || BUFFER_SIZE,
                bufferPoolSize: options.bufferPoolSize || 4,
                sampleRate: options.sampleRate || 44100
            }
        };
    }

    createProcessingStartedMessage() {
        const messageId = this.generateMessageId();
        const timestamp = this.getCurrentTimestamp();
        
        return {
            messageId: messageId,
            timestamp: timestamp,
            payload: {
                type: FromWorkletMessageType.PROCESSING_STARTED
            }
        };
    }

    createProcessingStoppedMessage() {
        const messageId = this.generateMessageId();
        const timestamp = this.getCurrentTimestamp();
        
        return {
            messageId: messageId,
            timestamp: timestamp,
            payload: {
                type: FromWorkletMessageType.PROCESSING_STOPPED
            }
        };
    }

    createAudioDataBatchMessage(buffer, options = {}) {
        const messageId = this.generateMessageId();
        const timestamp = this.getCurrentTimestamp();
        
        return {
            messageId: messageId,
            timestamp: timestamp,
            payload: {
                type: FromWorkletMessageType.AUDIO_DATA_BATCH,
                data: {
                    sampleRate: options.sampleRate || 48000,
                    sampleCount: options.sampleCount || 0,
                    bufferLength: buffer ? buffer.byteLength : 0,
                    timestamp: timestamp,
                    sequenceNumber: options.chunkCounter || 0,
                    bufferId: options.bufferId || 0,
                    bufferPoolStats: options.bufferPoolStats || null
                },
                buffer: buffer
            }
        };
    }

    createProcessingErrorMessage(error, code = WorkletErrorCode.GENERIC) {
        const messageId = this.generateMessageId();
        const timestamp = this.getCurrentTimestamp();
        
        return {
            messageId: messageId,
            timestamp: timestamp,
            payload: {
                type: FromWorkletMessageType.PROCESSING_ERROR,
                error: error,
                code: code
            }
        };
    }

    createStatusUpdateMessage(status) {
        const messageId = this.generateMessageId();
        const timestamp = this.getCurrentTimestamp();
        
        return {
            messageId: messageId,
            timestamp: timestamp,
            payload: {
                type: FromWorkletMessageType.STATUS_UPDATE,
                status: {
                    active: status.isProcessing,
                    sampleRate: this.sampleRate || 44100,
                    bufferSize: this.bufferSize || 128,
                    processedBatches: status.chunkCounter,
                    avgProcessingTimeMs: parseFloat(status.performanceMetrics?.audioProcessing?.averageProcessingTime) || 0.0,
                    buffer_pool_stats: status.buffer_pool_stats
                }
            }
        };
    }


    createBatchConfigUpdatedMessage(config) {
        const messageId = this.generateMessageId();
        const timestamp = this.getCurrentTimestamp();
        
        return {
            messageId: messageId,
            timestamp: timestamp,
            payload: {
                type: FromWorkletMessageType.BATCH_CONFIG_UPDATED,
                config: { ...config }
            }
        };
    }

    createProcessorDestroyedMessage() {
        const messageId = this.generateMessageId();
        const timestamp = this.getCurrentTimestamp();
        
        return {
            messageId: messageId,
            timestamp: timestamp,
            payload: {
                type: FromWorkletMessageType.PROCESSOR_DESTROYED
            }
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
        
        // Handle envelope messages
        if (message.payload && message.payload.type === FromWorkletMessageType.AUDIO_DATA_BATCH && message.payload.buffer) {
            transferables.push(message.payload.buffer);
        }
        // Handle direct messages (for backward compatibility)
        else if (message.type === FromWorkletMessageType.AUDIO_DATA_BATCH && message.buffer) {
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
        this.chunkSize = AUDIO_CHUNK_SIZE;
        
        // Batch configuration for transferable buffers
        this.batchSize = BUFFER_SIZE; // 32 chunks of 128 samples
        this.chunksPerBatch = this.batchSize / this.chunkSize;
        
        // Initialize buffer pool for ping-pong recycling
        this.bufferPool = new TransferableBufferPool(16, this.batchSize); // 16 buffers in pool
        this.bufferPoolConfig = {
            maxConsecutiveFailures: 3, // Max consecutive pool failures before warning
            warningThreshold: 10       // Warn if pool exhausted count exceeds this
        };
        this.consecutivePoolFailures = 0;
        
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
        
        // Performance monitoring
        this.performanceMonitoring = {
            enabled: true,
            processingTimes: [],
            maxSamples: 1000,
            gcPauseThreshold: 10, // ms
            lastProcessTime: 0,
            metrics: {
                averageProcessingTime: 0,
                maxProcessingTime: 0,
                minProcessingTime: Infinity,
                gcPausesDetected: 0,
                droppedChunks: 0,
                processedChunks: 0
            }
        };
        
        // Current batch buffer tracking
        this.currentBuffer = null;
        this.currentBufferArray = null;
        this.currentBufferId = 0; // Track buffer ID for ping-pong pattern
        this.writePosition = 0;
        
        // Timeout configuration for low-latency sending
        this.bufferTimeout = 100; // 100ms timeout for partial buffers (allows natural buffer filling)
        this.lastBufferStartTime = 0;
        
        // Processing state
        this.isProcessing = false;
        this.chunkCounter = 0;
        
        
        
        // Setup message handling
        this.port.onmessage = (event) => {
            // Message logging kept for debugging
            // Received message from main thread
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
        
        // Constructor complete, ready for processing
    }
    
    /**
     * Acquire a new buffer for batching with lifecycle tracking
     */
    acquireNewBuffer() {
        this.bufferStats.acquireCount++;
        
        // Try to acquire from pool first
        const acquisition = this.bufferPool.acquire();
        
        if (acquisition) {
            // Successfully acquired from pool
            this.currentBuffer = acquisition.buffer;
            this.currentBufferId = acquisition.bufferId;
            this.consecutivePoolFailures = 0; // Reset failure counter
            this.currentBufferArray = new Float32Array(this.currentBuffer);
            this.writePosition = 0;
            this.lastBufferStartTime = this.currentTime || getCurrentTime();
            
            
            // Buffer acquired from pool successfully
        } else {
            // Pool exhausted - skip processing rather than fallback allocation
            this.bufferStats.poolExhaustedCount++;
            this.consecutivePoolFailures++;
            
            // Clear buffer references to indicate no buffer available
            this.currentBuffer = null;
            this.currentBufferArray = null;
            this.currentBufferId = 0;
            this.writePosition = 0;
            
            // Log warning based on failure frequency
            if (this.consecutivePoolFailures >= this.bufferPoolConfig.maxConsecutiveFailures) {
                console.warn('PitchDetectionProcessor: Pool exhausted for', this.consecutivePoolFailures, 'consecutive attempts, skipping analysis data');
            } else if (this.bufferStats.poolExhaustedCount >= this.bufferPoolConfig.warningThreshold) {
                console.warn('PitchDetectionProcessor: Pool exhaustion count exceeded threshold:', this.bufferStats.poolExhaustedCount);
            }
        }
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
            this.currentBufferId = 0;
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
                
                // Create buffer pool statistics to include with the audio data
                const poolStats = this.bufferPool.getStats();
                const bufferPoolStats = {
                    pool_size: this.bufferPool.poolSize,
                    available_buffers: poolStats.availableBuffers,
                    in_use_buffers: poolStats.inUseBuffers,
                    total_buffers: poolStats.totalBuffers,
                    acquire_count: poolStats.acquireCount,
                    transfer_count: poolStats.transferCount,
                    pool_exhausted_count: poolStats.poolExhaustedCount,
                    consecutive_pool_failures: this.consecutivePoolFailures,
                    pool_hit_rate: poolStats.acquireCount > 0 ? 
                        ((poolStats.acquireCount - poolStats.poolExhaustedCount) / poolStats.acquireCount) * 100 : 0.0,
                    pool_efficiency: poolStats.transferCount > 0 ? 
                        (poolStats.transferCount / (poolStats.transferCount + poolStats.poolExhaustedCount)) * 100 : 0.0,
                    buffer_utilization_percent: this.bufferStats.averageBufferUtilization * 100,
                    total_megabytes_transferred: this.bufferStats.totalBytesTransferred / 1024 / 1024
                };
                
                // Create typed message for audio data batch
                const batchMessage = this.messageProtocol.createAudioDataBatchMessage(this.currentBuffer, {
                    sampleRate: sampleRate,
                    sampleCount: metadata.sampleCount,
                    chunkCounter: metadata.chunkCounter,
                    bufferId: this.currentBufferId,
                    bufferPoolStats: bufferPoolStats
                });
                
                // Send buffer with transferable
                const transferables = this.messageProtocol.getTransferableObjects(batchMessage);
                this.port.postMessage(batchMessage, transferables);
                
                // Mark buffer as transferred in pool (this creates a replacement)
                // Only mark if buffer was acquired from pool
                if (this.consecutivePoolFailures === 0) {
                    this.bufferPool.markTransferred(this.currentBuffer);
                }
                
                
                // Track buffer transfer statistics
                this.bufferStats.transferCount++;
                this.bufferStats.bufferLifecycle.transferred++;
                this.bufferStats.totalBytesTransferred += this.writePosition * 4; // 4 bytes per float32
                
                
                // Log distinctive message for debugging
                if (this.bufferStats.transferCount % 10 === 0) {
                }
                
                // Calculate buffer utilization
                const utilization = this.writePosition / this.batchSize;
                this.bufferStats.averageBufferUtilization = 
                    (this.bufferStats.averageBufferUtilization * (this.bufferStats.transferCount - 1) + utilization) / 
                    this.bufferStats.transferCount;
                
                // Buffer transferred successfully with utilization
                
                // Clear references to transferred buffer immediately
                this.currentBuffer = null;
                this.currentBufferArray = null;
                this.currentBufferId = 0;
                this.writePosition = 0;
                
            } catch (error) {
                console.error('PitchDetectionProcessor: Error sending buffer:', error);
                // Clear buffer references on error
                this.currentBuffer = null;
                this.currentBufferArray = null;
                this.currentBufferId = 0;
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
                    
                    // Clean up any remaining buffer state
                    if (this.currentBuffer && this.currentBufferId > 0) {
                        this.bufferPool.release(this.currentBuffer);
                        // Released buffer on stop processing
                        this.currentBuffer = null;
                        this.currentBufferArray = null;
                        this.currentBufferId = 0;
                        this.writePosition = 0;
                    }
                    
                    const stoppedMessage = this.messageProtocol.createProcessingStoppedMessage();
                    this.port.postMessage(stoppedMessage);
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
                            this.currentBufferId = 0;
                            this.writePosition = 0;
                        }
                        
                        // Update timeout if provided
                        if (actualMessage.config.bufferTimeout !== undefined) {
                            this.bufferTimeout = Math.max(0, actualMessage.config.bufferTimeout);
                        }
                        
                        // Batch config updated
                        
                        const batchConfigUpdatedMessage = this.messageProtocol.createBatchConfigUpdatedMessage({
                            batchSize: this.batchSize,
                            chunksPerBatch: this.chunksPerBatch,
                            bufferTimeout: this.bufferTimeout
                        });
                        this.port.postMessage(batchConfigUpdatedMessage);
                    }
                    break;
                
                case ToWorkletMessageType.RETURN_BUFFER:
                    if (actualMessage.bufferId !== undefined) {
                        // Extract buffer from message envelope if present
                        let returnedBuffer = null;
                        if (message.payload && message.payload.buffer) {
                            returnedBuffer = message.payload.buffer;
                        } else if (actualMessage.buffer) {
                            returnedBuffer = actualMessage.buffer;
                        } else if (message.buffer) {
                            returnedBuffer = message.buffer;
                        }
                        
                        if (returnedBuffer) {
                            // Return buffer to pool for reuse
                            const success = this.bufferPool.returnBuffer(actualMessage.bufferId, returnedBuffer);
                            if (success) {
                                // Buffer successfully returned to pool
                            } else {
                                console.warn('PitchDetectionProcessor: Failed to return buffer to pool:', actualMessage.bufferId);
                            }
                        } else {
                            console.warn('PitchDetectionProcessor: ReturnBuffer message missing buffer data');
                        }
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
     * Process audio data in 128-sample chunks
     * This method is called by the Web Audio API for each processing quantum
     * 
     * @param {Float32Array[][]} inputs - Input audio data (channels x samples)
     * @param {Float32Array[][]} outputs - Output audio data (channels x samples)
     * @param {Object} parameters - Audio parameters (unused in this implementation)
     * @returns {boolean} - True to keep processor alive, false to terminate
     */
    process(inputs, outputs, parameters) {
        const processStartTime = getCurrentTime();
        
        // Debug logging removed - functionality is working
        
        const input = inputs[0];
        const output = outputs[0];
        
        // Debug logging removed
        
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
        
        // Process microphone input audio
        const processedAudio = new Float32Array(inputChannel);
        
        // Pass-through processed audio to output
        if (output && output.length > 0 && output[0]) {
            const outputChannel = output[0];
            if (outputChannel && outputChannel.length === processedAudio.length) {
                // First, copy the processed audio to the output
                outputChannel.set(processedAudio);
                
            }
        }
        
        // Accumulate processed audio data for batching
        // Debug logging removed to reduce spam
        if (this.isProcessing) {
            // Debug logging removed - functionality is working
            try {
                // Debug: Check buffer state at start of each process call
                
                // Ensure we have a buffer to write to
                if (!this.currentBuffer || !this.currentBufferArray) {
                    this.acquireNewBuffer();
                }
                
                // If we still don't have a buffer (pool exhausted), skip this chunk
                if (!this.currentBufferArray) {
                    console.warn('PitchDetectionProcessor: No buffer available, skipping chunk');
                    this.performanceMonitoring.metrics.droppedChunks++;
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
                const currentTime = this.currentTime || getCurrentTime();
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
        
        // Track performance metrics
        if (this.performanceMonitoring.enabled) {
            const processingTime = getCurrentTime() - processStartTime;
            
            // Detect potential GC pauses
            if (this.performanceMonitoring.lastProcessTime > 0) {
                const timeSinceLastProcess = processStartTime - this.performanceMonitoring.lastProcessTime;
                if (timeSinceLastProcess > this.performanceMonitoring.gcPauseThreshold) {
                    this.performanceMonitoring.metrics.gcPausesDetected++;
                    console.warn(`PitchDetectionProcessor: Potential GC pause detected (${timeSinceLastProcess.toFixed(2)}ms between process calls)`);
                }
            }
            this.performanceMonitoring.lastProcessTime = processStartTime;
            
            // Update metrics
            this.performanceMonitoring.metrics.processedChunks++;
            this.performanceMonitoring.processingTimes.push(processingTime);
            
            // Keep only recent samples
            if (this.performanceMonitoring.processingTimes.length > this.performanceMonitoring.maxSamples) {
                this.performanceMonitoring.processingTimes.shift();
            }
            
            // Update statistics
            this.performanceMonitoring.metrics.maxProcessingTime = Math.max(
                this.performanceMonitoring.metrics.maxProcessingTime,
                processingTime
            );
            this.performanceMonitoring.metrics.minProcessingTime = Math.min(
                this.performanceMonitoring.metrics.minProcessingTime,
                processingTime
            );
            
            // Calculate average
            const sum = this.performanceMonitoring.processingTimes.reduce((a, b) => a + b, 0);
            this.performanceMonitoring.metrics.averageProcessingTime = 
                sum / this.performanceMonitoring.processingTimes.length;
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
        
        // Clean up buffer pool and handle in-flight buffers
        if (this.bufferPool) {
            // Release any current buffer back to pool if not transferred
            if (this.currentBuffer && this.currentBufferId > 0) {
                this.bufferPool.release(this.currentBuffer);
                // Released current buffer on destroy
            }
            
            // Stop timeout checker to prevent further cleanup
            this.bufferPool.stopTimeoutChecker();
            
            // Log final pool statistics
            // Buffer pool cleanup complete
        }
        
        // Clear all buffer references
        this.currentBuffer = null;
        this.currentBufferArray = null;
        this.currentBufferId = 0;
        this.writePosition = 0;
        
        const destroyedMessage = this.messageProtocol.createProcessorDestroyedMessage();
        this.port.postMessage(destroyedMessage);
    }
}

// Register the processor with the AudioWorklet
registerProcessor('pitch-processor', PitchDetectionProcessor);