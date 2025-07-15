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
    constructor(poolSize = 4, bufferCapacity = 1024) {
        this.poolSize = poolSize;
        this.bufferCapacity = bufferCapacity;
        this.buffers = [];
        this.availableIndices = [];
        this.inUseBuffers = new Map(); // Maps buffer to its index
        
        // Initialize pool with pre-allocated buffers
        for (let i = 0; i < poolSize; i++) {
            this.buffers.push(new ArrayBuffer(bufferCapacity * 4)); // 4 bytes per float32
            this.availableIndices.push(i);
        }
        
        // Statistics
        this.stats = {
            acquireCount: 0,
            transferCount: 0,
            poolExhaustedCount: 0
        };
    }
    
    /**
     * Acquire a buffer from the pool
     * @returns {ArrayBuffer|null} - Available buffer or null if pool exhausted
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
        
        return buffer;
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
        
        // Create replacement buffer
        const newBuffer = new ArrayBuffer(this.bufferCapacity * 4);
        this.buffers[index] = newBuffer;
        
        // Mark index as available again
        this.availableIndices.push(index);
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
        
        // Mark index as available again
        this.availableIndices.push(index);
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
        return {
            ...this.stats,
            availableBuffers: this.availableIndices.length,
            inUseBuffers: this.inUseBuffers.size,
            totalBuffers: this.poolSize
        };
    }
    
    /**
     * Reset pool statistics
     */
    resetStats() {
        this.stats.acquireCount = 0;
        this.stats.transferCount = 0;
        this.stats.poolExhaustedCount = 0;
    }
}

// Export for use in AudioWorklet
if (typeof module !== 'undefined' && module.exports) {
    module.exports = TransferableBufferPool;
}