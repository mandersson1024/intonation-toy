# The Detached Buffer Problem

## Overview

This document explains the design decision to use simple buffer allocation rather than complex recycling patterns for AudioWorklet transferable buffers.

## The Core Issue

When using transferable ArrayBuffers for zero-copy message passing between the AudioWorklet thread and main thread, buffers become "detached" after transfer:

```javascript
// AudioWorklet thread
const buffer = new ArrayBuffer(4096);
const floatArray = new Float32Array(buffer);
// Fill buffer with audio data...

// Transfer buffer to main thread
this.port.postMessage({
    type: 'audioData',
    buffer: buffer
}, [buffer]); // <- This transfer list makes the buffer transferable

// After this point:
// buffer.byteLength === 0 (detached!)
// floatArray.length === 0 (unusable!)
// The buffer now belongs to the main thread
```

## Design Decision: Allocation Over Recycling

The system uses simple allocation for each audio batch rather than attempting to recycle buffers.

### Current Implementation Pattern

```javascript
// AudioWorklet: Create new buffer for each batch
process(inputs) {
    const newBuffer = new ArrayBuffer(4096);
    const samples = new Float32Array(newBuffer);
    // Fill samples with audio data...
    this.port.postMessage({buffer: newBuffer}, [newBuffer]);
    // newBuffer is now detached, we create a new one next time
}

// Main thread: Receive and process
handleAudioData(event) {
    const samples = new Float32Array(event.data.buffer);
    // Process samples...
    // Garbage collector handles cleanup when done
}
```

### Why Not Use a Ping-Pong Pattern?

A ping-pong pattern would involve the main thread sending buffers back to the AudioWorklet for reuse:

```javascript
// Theoretical ping-pong approach (NOT IMPLEMENTED)
// Main thread would send buffers back:
processedBuffer = new ArrayBuffer(4096);
audioWorklet.port.postMessage({
    type: 'returnBuffer',
    buffer: processedBuffer
}, [processedBuffer]);
```

This approach is not used because:

1. **Added Complexity** - Requires buffer management on both threads
2. **Synchronization Issues** - Must ensure buffers are available when needed
3. **Minimal Performance Benefit** - Modern JS engines efficiently handle ArrayBuffer allocation
4. **Predictable Allocation Rate** - Buffers are allocated at a known, steady rate

## Performance Considerations

### Current Performance Profile
- **Buffer Size**: 4096 bytes (1024 float32 samples)
- **Allocation Rate**: ~47 buffers/second at 48kHz with 1024-sample batches
- **Memory Pressure**: ~188 KB/second allocation rate
- **GC Impact**: Minimal due to predictable allocation pattern

### When This Might Need to Change

This allocation-based approach may need reconsideration if:

1. **Higher Sample Rates** - 96kHz or 192kHz audio would double/quadruple allocation rate
2. **Larger Batch Sizes** - Bigger buffers might benefit from pooling
3. **Memory-Constrained Environments** - Mobile devices or embedded systems
4. **GC Pressure Becomes Visible** - If profiling shows GC pauses affecting audio

## Monitoring Points

To determine if the approach needs changing, monitor:

```javascript
// Performance monitoring hooks
let allocationCount = 0;
let lastGCTime = performance.now();

function allocateBuffer() {
    allocationCount++;
    
    // Log allocation rate every 1000 buffers
    if (allocationCount % 1000 === 0) {
        console.log(`Allocation rate: ${allocationCount} buffers`);
    }
    
    return new ArrayBuffer(4096);
}
```

## Future Migration Path

If recycling becomes necessary, the migration path would be:

1. **Implement Buffer Pool** on AudioWorklet side
2. **Add Return Channel** for main thread to send buffers back
3. **Handle Pool Exhaustion** gracefully with fallback allocation
4. **Profile Performance** to validate improvements

## Conclusion

The current design prioritizes simplicity and maintainability over theoretical performance optimization. The allocation-based approach is sufficient for current requirements and can be revisited if performance profiling indicates a need for change.