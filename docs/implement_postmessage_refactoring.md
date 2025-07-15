# Implementation Plan: PostMessage Refactoring for Audio Data Transfer

## Overview
This plan details the refactoring of audio data transfer from SharedArrayBuffer-based circular buffers to efficient postMessage with transferable ArrayBuffers, as described in `docs/reports/audio_postmessage_transferable.md`.

## Current State Analysis

### Existing Architecture
- AudioWorklet processes 128-sample chunks
- Data is sent via postMessage (already) but not using transferables
- Main thread writes data to circular buffers in a buffer pool
- Pitch analyzer is notified via buffer_filled events
- Volume detection runs directly on received data

### Key Changes Required
1. Implement transferable ArrayBuffer usage in postMessage
2. Add batching to reduce message frequency (from 344/sec to ~43/sec)
3. Remove circular buffer dependency for audio data flow
4. Migrate pitch analysis to direct data processing
5. Maintain zero-allocation principle with buffer reuse

---

## Implementation Tasks

### Task 1: Implement Transferable Buffer Pool in AudioWorklet ✅ COMPLETED
Create a pool of reusable ArrayBuffers in the AudioWorklet processor to avoid allocations.

- [x] 1a. Create a buffer pool class for AudioWorklet processor
  - Define pool size (e.g., 4-8 buffers)
  - Each buffer sized for batching (e.g., 1024 samples = 8 chunks)
  - Implement acquire/release mechanism

- [x] 1b. Add buffer pool to AudioWorkletProcessor
  - Initialize pool in constructor
  - Track current buffer and write position
  - Handle buffer rotation when full

- [x] 1c. Implement transferable-safe buffer management
  - Ensure buffers are properly detached after transfer
  - Create new buffers to replace transferred ones
  - Add safeguards against reusing detached buffers

**Testing**: Unit tests for buffer pool allocation, rotation, and detachment handling ✅

---

### Task 2: Implement Batched Audio Data Accumulation ✅ COMPLETED
Modify AudioWorklet to batch multiple 128-sample chunks before sending.

- [x] 2a. Add accumulation logic to process method
  - Accumulate incoming chunks into current buffer
  - Track samples written to current buffer
  - Determine optimal batch size (1024 samples = ~23ms at 44.1kHz)

- [x] 2b. Implement threshold-based sending
  - Send when buffer reaches capacity
  - Add timeout mechanism for low-latency scenarios
  - Include metadata (timestamp, sample count)

- [x] 2c. Handle edge cases
  - Partial buffers on stop/pause
  - Buffer size configuration from main thread
  - Sample rate changes

**Testing**: Verify correct accumulation, timing, and edge case handling ✅

---

### Task 3: Update Main Thread Message Handling
Modify AudioWorkletManager to handle transferable buffers.

- [ ] 3a. Update message handler for transferable buffers
  - Extract Float32Array from transferred ArrayBuffer
  - Process full batch instead of single chunk
  - Handle new message format with metadata

- [ ] 3b. Create buffer recycling mechanism
  - Pool for received buffers on main thread
  - Reuse buffers for processing to maintain zero-allocation
  - Clear buffers after processing

- [ ] 3c. Update data flow to consumers
  - Direct processing for volume detection (no buffer needed)
  - Direct processing for pitch analysis (no circular buffer)
  - Maintain existing observable data updates

**Testing**: Integration tests for message handling and buffer lifecycle

---

### Task 4: Migrate Pitch Analysis to Direct Processing
Remove dependency on circular buffers and buffer_filled events.

- [ ] 4a. Create direct pitch analysis interface
  - Accept Float32Array batches directly
  - Remove BufferAnalyzer dependency
  - Maintain sliding window internally if needed

- [ ] 4b. Update pitch analyzer initialization
  - Remove event subscription code
  - Add direct processing method
  - Configure for batch-based processing

- [ ] 4c. Integrate with new message flow
  - Call pitch analyzer directly from message handler
  - Handle batched data appropriately
  - Maintain existing pitch detection logic

**Testing**: Verify pitch detection accuracy with new data flow

---

### Task 5: Remove Circular Buffer Dependencies
Clean up code that's no longer needed after migration.

- [ ] 5a. Remove buffer pool usage from audio pipeline
  - Remove buffer pool creation in audio initialization
  - Remove buffer write operations in message handler
  - Clean up buffer pool global state

- [ ] 5b. Remove buffer_filled event usage
  - Remove event dispatch from worklet
  - Remove event subscription from pitch analyzer
  - Keep event system for now (separate refactoring)

- [ ] 5c. Update or remove buffer-dependent tests
  - Identify tests using circular buffers
  - Migrate to new direct processing approach
  - Remove obsolete buffer tests

**Testing**: Ensure all tests pass after removal

---

### Task 6: Performance Optimization and Tuning
Fine-tune the implementation for optimal performance.

- [ ] 6a. Profile and optimize batch sizes
  - Measure latency vs efficiency tradeoffs
  - Find optimal batch size for different use cases
  - Make batch size configurable

- [ ] 6b. Optimize buffer pool sizes
  - Minimize memory usage while avoiding stalls
  - Tune for typical processing speeds
  - Add monitoring for pool exhaustion

- [ ] 6c. Add performance metrics
  - Message frequency monitoring
  - Transfer overhead measurement
  - Processing latency tracking

**Testing**: Performance benchmarks and stress tests

---

### Task 7: Documentation and Migration Guide
Update documentation for the new architecture.

- [ ] 7a. Update architecture documentation
  - New data flow diagrams
  - Buffer management explanation
  - Performance characteristics

- [ ] 7b. Update API documentation
  - New message formats
  - Configuration options
  - Migration notes

- [ ] 7c. Create examples
  - Basic usage example
  - Performance tuning guide
  - Troubleshooting guide

---

## Dependencies and Order of Operations

1. **Phase 1**: Tasks 1-2 (AudioWorklet changes)
   - Can be developed independently
   - Must be complete before Phase 2

2. **Phase 2**: Tasks 3-4 (Main thread integration)
   - Depends on Phase 1
   - Can be developed in parallel

3. **Phase 3**: Task 5 (Cleanup)
   - Depends on Phase 2
   - Should be done after validation

4. **Phase 4**: Tasks 6-7 (Optimization and documentation)
   - Can start after Phase 2
   - Should be complete before release

---

## Potential Challenges and Solutions

### Challenge 1: Buffer Detachment Coordination
**Problem**: Ensuring buffers aren't used after transfer
**Solution**: Strict buffer lifecycle management with state tracking

### Challenge 2: Latency vs Efficiency Tradeoff
**Problem**: Larger batches increase latency
**Solution**: Configurable batch sizes with sensible defaults

### Challenge 3: Backward Compatibility
**Problem**: Existing code expects circular buffer behavior
**Solution**: Phased migration with compatibility layer if needed

### Challenge 4: Memory Management
**Problem**: Avoiding GC pressure with buffer allocation
**Solution**: Pre-allocated buffer pools on both threads

### Challenge 5: Error Handling
**Problem**: Transfer failures or buffer exhaustion
**Solution**: Fallback mechanisms and proper error reporting

---

## Success Criteria

1. Audio data transfer uses transferable ArrayBuffers
2. Message frequency reduced by ~8x (from 344 to ~43 messages/sec)
3. Zero allocations during steady-state operation
4. Pitch detection maintains current accuracy
5. Volume detection continues working in real-time
6. No increase in audio processing latency
7. Memory usage remains within current bounds
8. All existing tests pass with new implementation

---

## Notes

- This refactoring enables future removal of the event system
- Circular buffers can be retained for other use cases if needed
- Consider keeping BufferPool for non-audio use cases
- Monitor for Web Audio API updates that might affect this approach