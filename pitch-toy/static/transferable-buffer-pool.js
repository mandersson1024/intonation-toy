/**
 * Transferable Buffer Pool for AudioWorklet
 * 
 * Manages a pool of reusable ArrayBuffers for efficient audio data transfer
 * from AudioWorklet to main thread using postMessage with transferables.
 * 
 * Key Features:
 * - Pre-allocated buffers to avoid GC pressure
 * - Handles buffer detachment after transfer
 * - Automatic buffer replacement when transferred
 * - Configurable pool size and buffer capacity
 * 
 * Usage:
 * ```js
 * const pool = new TransferableBufferPool(4, 1024);
 * const buffer = pool.acquire();
 * if (buffer) {
 *     // Fill buffer with audio data
 *     // Transfer it via postMessage
 *     pool.markTransferred(buffer);
 * }
 * ```
 */
class TransferableBufferPool {
    /**
     * Create a new transferable buffer pool
     * @param {number} poolSize - Number of buffers in the pool (4-8 recommended)
     * @param {number} bufferCapacity - Size of each buffer in samples (e.g., 1024 = 8 chunks)
     */
    constructor(poolSize = 4, bufferCapacity = 1024, options = {}) {
        this.poolSize = poolSize;
        this.bufferCapacity = bufferCapacity;
        this.buffers = [];
        this.availableIndices = [];
        this.inUseBuffers = new Map(); // Maps buffer to its index
        
        // Buffer lifecycle management
        this.bufferStates = []; // Track state of each buffer
        this.bufferTimestamps = []; // Track when buffer was acquired
        this.bufferIds = []; // Track buffer IDs for ping-pong
        this.nextBufferId = 1;
        
        // Configuration options
        this.options = {
            timeout: options.timeout || 5000, // 5 seconds default timeout
            enableTimeouts: options.enableTimeouts !== false,
            enableValidation: options.enableValidation !== false,
            ...options
        };
        
        // Buffer states enum
        this.BUFFER_STATES = {
            AVAILABLE: 'available',
            IN_FLIGHT: 'in_flight',
            PROCESSING: 'processing',
            TIMED_OUT: 'timed_out'
        };
        
        // Initialize pool with pre-allocated buffers
        for (let i = 0; i < poolSize; i++) {
            this.buffers.push(new ArrayBuffer(bufferCapacity * 4)); // 4 bytes per float32
            this.availableIndices.push(i);
            this.bufferStates.push(this.BUFFER_STATES.AVAILABLE);
            this.bufferTimestamps.push(0);
            this.bufferIds.push(0);
        }
        
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
        
        // Start timeout checker if enabled
        if (this.options.enableTimeouts) {
            this.startTimeoutChecker();
        }
    }
    
    /**
     * Acquire a buffer from the pool
     * @returns {Object|null} - Object with buffer and bufferId, or null if pool exhausted
     */
    acquire() {
        this.stats.acquireCount++;
        
        if (this.availableIndices.length === 0) {
            this.stats.poolExhaustedCount++;
            console.warn('TransferableBufferPool: Pool exhausted, no buffers available');
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
        
        return {
            buffer: buffer,
            bufferId: this.bufferIds[index]
        };
    }
    
    /**
     * Mark a buffer as transferred and create replacement
     * @param {ArrayBuffer} buffer - Buffer that was transferred
     */
    markTransferred(buffer) {
        this.stats.transferCount++;
        
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
        
        // Create replacement buffer
        const newBuffer = new ArrayBuffer(this.bufferCapacity * 4);
        this.buffers[index] = newBuffer;
        
        // Mark index as available again
        this.availableIndices.push(index);
        
        // Reset buffer state for new buffer
        this.bufferStates[index] = this.BUFFER_STATES.AVAILABLE;
        this.bufferTimestamps[index] = 0;
        this.bufferIds[index] = 0;
    }
    
    /**
     * Release a buffer back to the pool without transferring
     * @param {ArrayBuffer} buffer - Buffer to release
     */
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
    
    /**
     * Return a buffer to the pool by ID (for ping-pong pattern)
     * @param {number} bufferId - ID of the buffer to return
     * @param {ArrayBuffer} buffer - The actual buffer being returned
     */
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
            
            console.log('TransferableBufferPool: Buffer returned to pool, ID:', bufferId, 'Index:', targetIndex);
            return true;
        } else {
            // Fallback: use any available slot (for compatibility)
            if (this.availableIndices.length < this.poolSize) {
                const index = this.availableIndices.length > 0 ? this.availableIndices[0] : this.buffers.length;
                if (index < this.poolSize) {
                    this.buffers[index] = buffer;
                    this.bufferStates[index] = this.BUFFER_STATES.AVAILABLE;
                    this.bufferTimestamps[index] = 0;
                    this.bufferIds[index] = 0;
                    
                    if (!this.availableIndices.includes(index)) {
                        this.availableIndices.push(index);
                    }
                    
                    this.stats.returnedBuffers++;
                    console.log('TransferableBufferPool: Buffer returned to pool (fallback), ID:', bufferId, 'Index:', index);
                    return true;
                }
            }
        }
        
        console.warn('TransferableBufferPool: Could not return buffer to pool (pool full or invalid state)');
        return false;
    }
    
    /**
     * Check if a buffer is detached (transferred)
     * @param {ArrayBuffer} buffer - Buffer to check
     * @returns {boolean} - True if buffer is detached
     */
    isDetached(buffer) {
        return buffer.byteLength === 0;
    }
    
    /**
     * Get pool statistics
     * @returns {Object} - Pool usage statistics
     */
    getStats() {
        // Calculate average turnover time
        let totalTurnoverTime = 0;
        let processingBuffers = 0;
        const now = Date.now();
        
        for (let i = 0; i < this.poolSize; i++) {
            if (this.bufferStates[i] === this.BUFFER_STATES.PROCESSING && this.bufferTimestamps[i] > 0) {
                totalTurnoverTime += (now - this.bufferTimestamps[i]);
                processingBuffers++;
            }
        }
        
        this.stats.averageTurnoverTime = processingBuffers > 0 ? totalTurnoverTime / processingBuffers : 0;
        
        return {
            ...this.stats,
            availableBuffers: this.availableIndices.length,
            inUseBuffers: this.inUseBuffers.size,
            totalBuffers: this.poolSize,
            processingBuffers: processingBuffers,
            timedOutBuffers: this.bufferStates.filter(state => state === this.BUFFER_STATES.TIMED_OUT).length
        };
    }
    
    /**
     * Reset pool statistics
     */
    resetStats() {
        this.stats.acquireCount = 0;
        this.stats.transferCount = 0;
        this.stats.poolExhaustedCount = 0;
        this.stats.timeoutCount = 0;
        this.stats.validationFailures = 0;
        this.stats.returnedBuffers = 0;
        this.stats.bufferReuseRate = 0;
        this.stats.averageTurnoverTime = 0;
    }
    
    /**
     * Start the timeout checker for buffer lifecycle management
     */
    startTimeoutChecker() {
        if (this.timeoutChecker) {
            clearInterval(this.timeoutChecker);
        }
        
        // Check for timed out buffers every second
        this.timeoutChecker = setInterval(() => {
            this.checkForTimedOutBuffers();
        }, 1000);
    }
    
    /**
     * Stop the timeout checker
     */
    stopTimeoutChecker() {
        if (this.timeoutChecker) {
            clearInterval(this.timeoutChecker);
            this.timeoutChecker = null;
        }
    }
    
    /**
     * Check for buffers that have timed out and reclaim them
     */
    checkForTimedOutBuffers() {
        const now = Date.now();
        let reclaimedCount = 0;
        
        for (let i = 0; i < this.poolSize; i++) {
            const state = this.bufferStates[i];
            const timestamp = this.bufferTimestamps[i];
            
            // Check if buffer is in processing state and has timed out
            if ((state === this.BUFFER_STATES.IN_FLIGHT || state === this.BUFFER_STATES.PROCESSING) && 
                timestamp > 0 && 
                (now - timestamp) > this.options.timeout) {
                
                // Buffer has timed out, reclaim it
                this.bufferStates[i] = this.BUFFER_STATES.TIMED_OUT;
                this.bufferTimestamps[i] = 0;
                const bufferId = this.bufferIds[i];
                this.bufferIds[i] = 0;
                
                // Create a new buffer to replace the lost one
                this.buffers[i] = new ArrayBuffer(this.bufferCapacity * 4);
                this.bufferStates[i] = this.BUFFER_STATES.AVAILABLE;
                
                // Add back to available indices if not already there
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
    
    /**
     * Get buffer state information for debugging
     */
    getBufferStates() {
        return {
            states: [...this.bufferStates],
            timestamps: [...this.bufferTimestamps],
            ids: [...this.bufferIds],
            availableIndices: [...this.availableIndices]
        };
    }
    
    /**
     * Destroy the pool and clean up resources
     */
    destroy() {
        this.stopTimeoutChecker();
        this.buffers = [];
        this.availableIndices = [];
        this.inUseBuffers.clear();
        this.bufferStates = [];
        this.bufferTimestamps = [];
        this.bufferIds = [];
    }
}

// Export for use in AudioWorklet
if (typeof module !== 'undefined' && module.exports) {
    module.exports = TransferableBufferPool;
}