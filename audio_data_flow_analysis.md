# Audio Data Flow Analysis - pitch-toy

## Overview

The pitch-toy audio system implements a real-time audio processing pipeline using WebAudio API's AudioWorklet for low-latency processing. The architecture follows a producer-consumer pattern with pre-allocated circular buffers to maintain zero-allocation during steady-state operation.

## Current Architecture Components

### 1. Audio Input Sources
- **Microphone Input**: Primary audio source via getUserMedia API
- **Test Signal Generator**: Built-in signal generator for testing (sine, square, sawtooth, triangle, white/pink noise)
- **Background Noise**: Independent noise layer that can be mixed with either source

### 2. AudioWorklet Processor (audio-processor.js)
- **Fixed Processing**: 128-sample chunks (Web Audio API standard)
- **Real-time Thread**: Runs on dedicated audio rendering thread
- **Message Passing**: Uses MessagePort for bidirectional communication
- **No SharedArrayBuffer**: Currently uses postMessage with Float32Array copies

### 3. Main Thread Components

#### AudioWorkletManager (worklet.rs)
- **Central Coordinator**: Manages AudioWorklet lifecycle and communication
- **Message Handler**: Processes incoming audio data messages
- **Event Publishing**: Publishes buffer events when buffers fill
- **Direct Integration**: Volume detector processes data directly in message handler

#### Buffer Pool (buffer_pool.rs)
- **Pre-allocated Memory**: Fixed collection of CircularBuffers
- **Memory Budget**: 50MB limit for GPU/Audio memory
- **Zero-allocation**: Buffers created upfront, reused throughout lifetime
- **Overflow Tracking**: Monitors buffer overflows across all buffers

#### Circular Buffers (buffer.rs)
- **Ring Buffer Implementation**: Uses VecDeque internally
- **Fixed Capacity**: Must be multiple of 128 samples
- **States**: Empty, Filling, Full, Overflow, Processing
- **Sliding Window Support**: Non-destructive reads for analysis
- **Overflow Handling**: Evicts oldest data when full

### 4. Audio Data Consumers

#### Volume Detector (volume_detector.rs)
- **Real-time Analysis**: Processes each 128-sample chunk
- **Direct Integration**: Runs inside AudioWorklet message handler
- **Metrics**: RMS, peak, fast/slow peak levels
- **Confidence Weighting**: Provides 0.0-1.0 confidence based on volume level
- **Observable Updates**: Uses setter pattern for UI updates

#### Pitch Analyzer (pitch_analyzer.rs)
- **Event-driven**: Subscribes to BufferFilled events
- **Buffer-based**: Waits for sufficient samples (2048 typical)
- **YIN Algorithm**: Uses pitch detector for fundamental frequency
- **Volume Integration**: Uses volume confidence for pitch confidence
- **Observable Updates**: Uses setter pattern for UI updates

#### Buffer Analyzer (buffer_analyzer.rs)
- **Sequential Processing**: Reads blocks from circular buffer
- **Window Functions**: Supports Hamming, Blackman windowing
- **Zero-copy Interface**: Can process into pre-allocated buffers
- **Pitch Detection Bridge**: Used by pitch analyzer to extract windows

## Current Data Flow Pattern

```
1. Audio Input (Mic/TestSignal)
   ↓
2. AudioWorklet Processor (128-sample chunks)
   ↓
3. postMessage to Main Thread (Float32Array copy)
   ↓
4. AudioWorkletManager::handle_worklet_message()
   ↓
5. Parallel Processing:
   a) Volume Detector → Direct analysis → Observable update
   b) Buffer Pool → CircularBuffer write → BufferFilled event
   ↓
6. Pitch Analyzer (on BufferFilled event)
   ↓
7. BufferAnalyzer → Extract window → Pitch detection
   ↓
8. Observable updates → UI components
```

## Key Observations

### Message Passing Architecture
- **No SharedArrayBuffer**: Currently uses postMessage with data copies
- **Serialization Overhead**: Each 128-sample chunk is copied from audio thread
- **Event-driven**: Uses event system for buffer-filled notifications
- **Direct Processing**: Volume detection happens immediately in message handler

### Buffer Management
- **Single Buffer Usage**: Only buffer index 0 is actively used
- **Circular Buffer**: Handles overflow by evicting old data
- **Pre-allocation**: All memory allocated upfront to avoid runtime allocations
- **Multiple Consumers**: Same buffer data consumed by multiple analyzers

### Performance Characteristics
- **Latency**: ~2.67ms per 128-sample chunk at 48kHz
- **Update Frequency**: UI updates every 16 chunks (~34ms)
- **Memory Usage**: Configurable pool size within 50MB limit
- **Zero-allocation**: No allocations during steady-state processing

### Observable Data Pattern
- **Setter-based Updates**: Direct updates via observable_data setters
- **No Event Bus**: Moving away from event-based updates for data
- **Typed Updates**: Strongly typed data structures for each update type

## Potential Improvements

1. **SharedArrayBuffer**: Could eliminate data copying between threads
2. **Ring Buffer in AudioWorklet**: Could manage buffering on audio thread
3. **Batch Processing**: Could reduce message frequency
4. **Direct Memory Access**: Could use WASM shared memory for zero-copy
5. **Buffer Pool Utilization**: Currently only using one buffer of the pool

## Conclusion

The current architecture provides a functional real-time audio processing pipeline with reasonable latency. The main overhead comes from message passing and data copying between the audio thread and main thread. The system maintains zero-allocation during processing through pre-allocated buffers and careful memory management.