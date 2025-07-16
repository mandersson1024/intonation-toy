/**
 * AudioWorklet Message Protocol
 * 
 * This module provides type-safe message construction and validation for AudioWorklet communication.
 * It mirrors the Rust message protocol types to ensure consistency between main thread and worklet.
 * 
 * Features:
 * - Type-safe message construction
 * - Message validation on both send and receive
 * - Structured error reporting
 * - Message correlation for request/response patterns
 * - Consistent types matching Rust implementation
 * 
 * Usage:
 * ```js
 * import { AudioWorkletMessageProtocol } from './audio-message-protocol.js';
 * 
 * const protocol = new AudioWorkletMessageProtocol();
 * 
 * // Create a typed message
 * const message = protocol.createStartProcessingMessage();
 * 
 * // Validate an incoming message
 * if (protocol.validateMessage(receivedMessage)) {
 *     // Handle message safely
 * }
 * ```
 */

// Message type constants matching Rust enums

/**
 * Message types sent from main thread to AudioWorklet
 */
export const ToWorkletMessageType = {
    START_PROCESSING: 'startProcessing',
    STOP_PROCESSING: 'stopProcessing',
    UPDATE_TEST_SIGNAL_CONFIG: 'updateTestSignalConfig',
    UPDATE_BATCH_CONFIG: 'updateBatchConfig',
    UPDATE_BACKGROUND_NOISE_CONFIG: 'updateBackgroundNoiseConfig',
    GET_STATUS: 'getStatus'
};

/**
 * Message types sent from AudioWorklet to main thread
 */
export const FromWorkletMessageType = {
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

/**
 * Error codes for worklet errors
 */
export const WorkletErrorCode = {
    INITIALIZATION_FAILED: 'InitializationFailed',
    PROCESSING_FAILED: 'ProcessingFailed',
    BUFFER_OVERFLOW: 'BufferOverflow',
    INVALID_CONFIGURATION: 'InvalidConfiguration',
    MEMORY_ALLOCATION_FAILED: 'MemoryAllocationFailed',
    GENERIC: 'Generic'
};

/**
 * AudioWorklet Message Protocol class
 * Provides type-safe message construction and validation
 */
export class AudioWorkletMessageProtocol {
    constructor() {
        this.messageIdCounter = 0;
    }

    /**
     * Generate a unique message ID
     */
    generateMessageId() {
        return ++this.messageIdCounter;
    }

    /**
     * Get current timestamp
     */
    getCurrentTimestamp() {
        return performance.now();
    }

    // Message constructors for ToWorkletMessage types

    /**
     * Create a start processing message
     */
    createStartProcessingMessage() {
        return {
            type: ToWorkletMessageType.START_PROCESSING,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create a stop processing message
     */
    createStopProcessingMessage() {
        return {
            type: ToWorkletMessageType.STOP_PROCESSING,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create an update test signal config message
     */
    createUpdateTestSignalConfigMessage(config) {
        if (!this.validateTestSignalConfig(config)) {
            throw new Error('Invalid test signal configuration');
        }

        return {
            type: ToWorkletMessageType.UPDATE_TEST_SIGNAL_CONFIG,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create an update batch config message
     */
    createUpdateBatchConfigMessage(config) {
        if (!this.validateBatchConfig(config)) {
            throw new Error('Invalid batch configuration');
        }

        return {
            type: ToWorkletMessageType.UPDATE_BATCH_CONFIG,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create an update background noise config message
     */
    createUpdateBackgroundNoiseConfigMessage(config) {
        if (!this.validateBackgroundNoiseConfig(config)) {
            throw new Error('Invalid background noise configuration');
        }

        return {
            type: ToWorkletMessageType.UPDATE_BACKGROUND_NOISE_CONFIG,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create a get status message
     */
    createGetStatusMessage() {
        return {
            type: ToWorkletMessageType.GET_STATUS,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    // Message constructors for FromWorkletMessage types

    /**
     * Create a processor ready message
     */
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

    /**
     * Create a processing started message
     */
    createProcessingStartedMessage() {
        return {
            type: FromWorkletMessageType.PROCESSING_STARTED,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create a processing stopped message
     */
    createProcessingStoppedMessage() {
        return {
            type: FromWorkletMessageType.PROCESSING_STOPPED,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create an audio data batch message
     */
    createAudioDataBatchMessage(buffer, options = {}) {
        if (!buffer || !(buffer instanceof ArrayBuffer)) {
            throw new Error('Invalid buffer: must be ArrayBuffer');
        }

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

    /**
     * Create a processing error message
     */
    createProcessingErrorMessage(error, code = WorkletErrorCode.GENERIC) {
        return {
            type: FromWorkletMessageType.PROCESSING_ERROR,
            error: error,
            code: code,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create a status update message
     */
    createStatusUpdateMessage(status) {
        if (!this.validateStatusUpdate(status)) {
            throw new Error('Invalid status update');
        }

        return {
            type: FromWorkletMessageType.STATUS_UPDATE,
            isProcessing: status.isProcessing,
            chunkCounter: status.chunkCounter,
            bufferPoolStats: status.bufferPoolStats,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create test signal config updated message
     */
    createTestSignalConfigUpdatedMessage(config) {
        return {
            type: FromWorkletMessageType.TEST_SIGNAL_CONFIG_UPDATED,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create background noise config updated message
     */
    createBackgroundNoiseConfigUpdatedMessage(config) {
        return {
            type: FromWorkletMessageType.BACKGROUND_NOISE_CONFIG_UPDATED,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create batch config updated message
     */
    createBatchConfigUpdatedMessage(config) {
        return {
            type: FromWorkletMessageType.BATCH_CONFIG_UPDATED,
            config: { ...config },
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    /**
     * Create processor destroyed message
     */
    createProcessorDestroyedMessage() {
        return {
            type: FromWorkletMessageType.PROCESSOR_DESTROYED,
            messageId: this.generateMessageId(),
            timestamp: this.getCurrentTimestamp()
        };
    }

    // Message validation methods

    /**
     * Validate a message has required fields
     */
    validateMessage(message) {
        if (!message || typeof message !== 'object') {
            return false;
        }

        if (!message.type || typeof message.type !== 'string') {
            return false;
        }

        // Check if it's a valid message type
        const allMessageTypes = { ...ToWorkletMessageType, ...FromWorkletMessageType };
        const isValidType = Object.values(allMessageTypes).includes(message.type);
        
        if (!isValidType) {
            return false;
        }

        return true;
    }

    /**
     * Validate test signal configuration
     */
    validateTestSignalConfig(config) {
        if (!config || typeof config !== 'object') {
            return false;
        }

        // Check required fields
        if (typeof config.enabled !== 'boolean') {
            return false;
        }

        if (config.enabled) {
            if (typeof config.frequency !== 'number' || config.frequency <= 0) {
                return false;
            }
            if (typeof config.amplitude !== 'number' || config.amplitude < 0 || config.amplitude > 1) {
                return false;
            }
            if (typeof config.waveform !== 'string') {
                return false;
            }
            if (typeof config.sample_rate !== 'number' || config.sample_rate <= 0) {
                return false;
            }
        }

        return true;
    }

    /**
     * Validate batch configuration
     */
    validateBatchConfig(config) {
        if (!config || typeof config !== 'object') {
            return false;
        }

        if (config.batchSize !== undefined) {
            if (typeof config.batchSize !== 'number' || config.batchSize <= 0) {
                return false;
            }
        }

        if (config.bufferTimeout !== undefined) {
            if (typeof config.bufferTimeout !== 'number' || config.bufferTimeout < 0) {
                return false;
            }
        }

        return true;
    }

    /**
     * Validate background noise configuration
     */
    validateBackgroundNoiseConfig(config) {
        if (!config || typeof config !== 'object') {
            return false;
        }

        if (typeof config.enabled !== 'boolean') {
            return false;
        }

        if (config.enabled) {
            if (typeof config.level !== 'number' || config.level < 0 || config.level > 1) {
                return false;
            }
            if (typeof config.type !== 'string') {
                return false;
            }
        }

        return true;
    }

    /**
     * Validate status update structure
     */
    validateStatusUpdate(status) {
        if (!status || typeof status !== 'object') {
            return false;
        }

        if (typeof status.isProcessing !== 'boolean') {
            return false;
        }

        if (typeof status.chunkCounter !== 'number') {
            return false;
        }

        if (status.bufferPoolStats && typeof status.bufferPoolStats !== 'object') {
            return false;
        }

        return true;
    }

    /**
     * Get message type from message object
     */
    getMessageType(message) {
        return message?.type;
    }

    /**
     * Check if message is a ToWorklet message
     */
    isToWorkletMessage(message) {
        return Object.values(ToWorkletMessageType).includes(message?.type);
    }

    /**
     * Check if message is a FromWorklet message
     */
    isFromWorkletMessage(message) {
        return Object.values(FromWorkletMessageType).includes(message?.type);
    }

    /**
     * Validate buffer metadata
     */
    validateBufferMetadata(buffer, metadata) {
        if (!buffer || !(buffer instanceof ArrayBuffer)) {
            return { valid: false, error: 'Invalid buffer: must be ArrayBuffer' };
        }

        if (!metadata || typeof metadata !== 'object') {
            return { valid: false, error: 'Invalid metadata: must be object' };
        }

        // Validate sample count
        if (typeof metadata.sampleCount !== 'number' || metadata.sampleCount < 0) {
            return { valid: false, error: 'Invalid sampleCount: must be non-negative number' };
        }

        // Validate batch size
        if (typeof metadata.batchSize !== 'number' || metadata.batchSize <= 0) {
            return { valid: false, error: 'Invalid batchSize: must be positive number' };
        }

        // Check buffer size consistency
        const expectedBufferSize = metadata.batchSize * 4; // 4 bytes per float32
        if (buffer.byteLength < expectedBufferSize) {
            return { valid: false, error: `Buffer too small: expected at least ${expectedBufferSize} bytes, got ${buffer.byteLength}` };
        }

        // Validate sample count doesn't exceed batch size
        if (metadata.sampleCount > metadata.batchSize) {
            return { valid: false, error: `Sample count ${metadata.sampleCount} exceeds batch size ${metadata.batchSize}` };
        }

        return { valid: true };
    }

    /**
     * Extract transferable objects from message
     */
    getTransferableObjects(message) {
        const transferables = [];
        
        if (message.type === FromWorkletMessageType.AUDIO_DATA_BATCH && message.buffer) {
            transferables.push(message.buffer);
        }

        return transferables;
    }
}