# Buffer Pool Configuration and Tuning Guide

## Overview

This guide provides comprehensive information on configuring and tuning the buffer pool ping-pong pattern for optimal performance in different scenarios.

## Configuration Options

### Pool Size Configuration

The pool size determines how many buffers are pre-allocated in the pool.

```javascript
// Default configuration
const pool = new TransferableBufferPool(16, 1024, {
    timeout: 5000,
    enableTimeouts: true,
    enableValidation: true,
    enableGCPauseDetection: true
});
```

**Pool Size Guidelines:**
- **Small (4-8 buffers)**: Low-latency applications, minimal memory usage
- **Medium (16 buffers)**: Balanced performance (recommended default)
- **Large (32+ buffers)**: High-throughput applications, sustained processing

### Configuration Parameters

| Parameter | Default | Range | Description |
|-----------|---------|-------|-------------|
| `poolSize` | 16 | 2-128 | Number of buffers in pool |
| `bufferCapacity` | 1024 | 128-4096 | Buffer size in samples |
| `timeout` | 5000ms | 100-30000ms | Buffer timeout before reclaim |
| `enableTimeouts` | true | boolean | Enable timeout recovery |
| `enableValidation` | true | boolean | Enable buffer size validation |
| `enableGCPauseDetection` | true | boolean | Enable GC pause detection |
| `gcPauseThreshold` | 50ms | 1-1000ms | GC pause detection threshold |

## Performance Tuning Scenarios

### 1. Low Latency Configuration

**Use Case**: Real-time audio applications, live monitoring, interactive audio
**Target**: <10ms latency, minimal buffer pools

```javascript
// Low latency setup
const pool = new TransferableBufferPool(4, 512, {
    timeout: 1000,
    enableTimeouts: true,
    enableValidation: true,
    gcPauseThreshold: 20
});
```

**Console Commands:**
```bash
pool size 4
pool timeout 1000
pool gc threshold 20
```

**Expected Performance:**
- Pool hit rate: 85-95%
- GC pauses: <5 per minute
- Memory usage: 8KB (4 × 2KB buffers)
- Latency: 5-10ms

### 2. Balanced Performance Configuration

**Use Case**: General audio processing, pitch detection, music applications
**Target**: Good balance of latency and throughput

```javascript
// Balanced setup (default)
const pool = new TransferableBufferPool(16, 1024, {
    timeout: 5000,
    enableTimeouts: true,
    enableValidation: true,
    gcPauseThreshold: 50
});
```

**Console Commands:**
```bash
pool size 16
pool timeout 5000
pool gc threshold 50
```

**Expected Performance:**
- Pool hit rate: 95-98%
- GC pauses: <2 per minute
- Memory usage: 64KB (16 × 4KB buffers)
- Latency: 10-25ms

### 3. High Throughput Configuration

**Use Case**: Batch processing, analysis, non-real-time applications
**Target**: Maximum throughput, higher latency acceptable

```javascript
// High throughput setup
const pool = new TransferableBufferPool(32, 2048, {
    timeout: 10000,
    enableTimeouts: true,
    enableValidation: true,
    gcPauseThreshold: 100
});
```

**Console Commands:**
```bash
pool size 32
pool timeout 10000
pool gc threshold 100
```

**Expected Performance:**
- Pool hit rate: 98-99%
- GC pauses: <1 per minute
- Memory usage: 256KB (32 × 8KB buffers)
- Latency: 25-50ms

## Monitoring and Optimization

### Performance Metrics

Use the `perf` command to monitor buffer pool performance:

```bash
# Show current performance metrics
perf

# Expected output:
🔬 Audio Processing Performance Metrics
Buffer Pool Performance:
  Pool Size: 16 buffers
  Available: 12 buffers
  Hit Rate: 98.5%
  Avg Acquisition: 0.05ms
  Total Allocations: 47

Audio Processing:
  Avg Process Time: 0.12ms
  Max Process Time: 0.48ms
  GC Pauses: 2
  Dropped Chunks: 0
  Processed Chunks: 45,231

Memory & Efficiency:
  Zero-Copy Transfers: ✓
  Pool Exhaustion: 0.1%
  Buffer Reuse Rate: 94.2%
```

### Key Performance Indicators

#### Pool Hit Rate
- **Target**: >95%
- **Good**: 90-95%
- **Poor**: <90%

**Optimization:**
- Increase pool size if hit rate is low
- Check for buffer leaks or timeout issues
- Monitor GC pause frequency

#### Buffer Acquisition Time
- **Target**: <0.1ms
- **Good**: 0.1-0.5ms
- **Poor**: >0.5ms

**Optimization:**
- Reduce pool size if acquisition is slow
- Enable GC pause detection
- Check for memory pressure

#### GC Pause Frequency
- **Target**: <1 per minute
- **Good**: 1-5 per minute
- **Poor**: >5 per minute

**Optimization:**
- Increase pool size to reduce allocations
- Lower GC pause threshold for better detection
- Check for memory leaks

## Tuning Workflow

### 1. Baseline Measurement

Start with default configuration and measure performance:

```bash
# Reset metrics for clean baseline
perf reset

# Let system run for 1-2 minutes
# Then check performance
perf
```

### 2. Identify Bottlenecks

Common issues and solutions:

**Low Pool Hit Rate (<90%)**
```bash
# Check pool status
pool

# Increase pool size
pool size 24

# Monitor improvement
perf
```

**High GC Pause Frequency (>5/min)**
```bash
# Check GC detection
perf gc

# Increase pool size to reduce allocations
pool size 24

# Lower GC threshold for better detection
pool gc threshold 30
```

**High Buffer Acquisition Time (>0.5ms)**
```bash
# Check pool configuration
pool

# Reduce pool size if too large
pool size 8

# Check for timeout issues
pool timeout 3000
```

### 3. Performance Optimization

**For Real-time Applications:**
```bash
# Optimize for low latency
pool size 8
pool timeout 2000
pool gc threshold 25

# Verify performance
perf
```

**For Batch Processing:**
```bash
# Optimize for throughput
pool size 32
pool timeout 10000
pool gc threshold 100

# Verify performance
perf
```

## Advanced Configuration

### Custom Buffer Pool Options

```javascript
// Advanced configuration with custom options
const customPool = new TransferableBufferPool(16, 1024, {
    // Timeout configuration
    timeout: 5000,
    enableTimeouts: true,
    
    // Validation settings
    enableValidation: true,
    
    // Performance tracking
    perfTracking: true,
    perfSampleSize: 1000,
    
    // GC pause detection
    gcPauseThreshold: 50,
    
    // Debug options
    enableDebugLogging: false,
    logLevel: 'info'
});
```

### Dynamic Configuration Updates

```javascript
// Runtime configuration changes
pool.updateConfiguration({
    timeout: 3000,
    gcPauseThreshold: 25
});

// Enable/disable features
pool.enableGCPauseDetection(30);
pool.disableValidation();
```

## Best Practices

### 1. Pool Size Selection

**Rule of Thumb**: Pool size should be 2-4x the number of buffers typically in-flight

```javascript
// For 48kHz, 1024 sample buffers, ~46 buffers/second
// If processing takes 50ms, ~2-3 buffers are in-flight
// Pool size: 8-12 buffers (choose 12 for safety margin)
```

### 2. Timeout Configuration

**Guidelines**:
- **Low latency**: 1-2 seconds
- **Normal applications**: 5 seconds
- **Batch processing**: 10-30 seconds

```javascript
// Timeout should be 5-10x expected processing time
// If processing takes 500ms, timeout should be 2500-5000ms
```

### 3. Memory Usage Optimization

**Memory Calculation**:
```javascript
// Memory = poolSize × bufferCapacity × 4 bytes
// Example: 16 × 1024 × 4 = 64KB
// For mobile devices, keep under 1MB total
```

### 4. GC Pause Detection

**Thresholds**:
- **Sensitive applications**: 20-30ms
- **Normal applications**: 50ms
- **Batch processing**: 100ms

## Configuration Examples

### Example 1: Music Production App

```javascript
// Low latency, high quality
const musicPool = new TransferableBufferPool(8, 512, {
    timeout: 2000,
    enableTimeouts: true,
    enableValidation: true,
    gcPauseThreshold: 25
});

// Console setup
// pool size 8
// pool timeout 2000
// pool gc threshold 25
```

### Example 2: Audio Analysis Tool

```javascript
// High throughput, analysis focus
const analysisPool = new TransferableBufferPool(24, 2048, {
    timeout: 8000,
    enableTimeouts: true,
    enableValidation: true,
    gcPauseThreshold: 75
});

// Console setup
// pool size 24
// pool timeout 8000
// pool gc threshold 75
```

### Example 3: Mobile Audio App

```javascript
// Memory-constrained, battery-efficient
const mobilePool = new TransferableBufferPool(6, 512, {
    timeout: 3000,
    enableTimeouts: true,
    enableValidation: false, // Reduce overhead
    gcPauseThreshold: 40
});

// Console setup
// pool size 6
// pool timeout 3000
// pool gc disable
```

## Troubleshooting Performance Issues

### Issue: Low Pool Hit Rate

**Symptoms**: Pool hit rate <90%, frequent "pool exhausted" messages

**Diagnosis**:
```bash
pool
# Check available vs total buffers
# Check timeout count
```

**Solutions**:
1. Increase pool size: `pool size 24`
2. Reduce timeout: `pool timeout 3000`
3. Check for buffer leaks
4. Monitor processing time

### Issue: High GC Pause Frequency

**Symptoms**: >5 GC pauses per minute, audio glitches

**Diagnosis**:
```bash
perf gc
# Check GC pause count and frequency
```

**Solutions**:
1. Increase pool size: `pool size 32`
2. Lower GC threshold: `pool gc threshold 30`
3. Check for memory leaks
4. Optimize processing code

### Issue: High Buffer Acquisition Time

**Symptoms**: Acquisition time >0.5ms, processing delays

**Diagnosis**:
```bash
perf
# Check avg acquisition time
# Check pool configuration
```

**Solutions**:
1. Reduce pool size if too large: `pool size 8`
2. Check GC pause interference
3. Optimize pool algorithms
4. Check system load

## Monitoring Dashboard

### Key Metrics to Track

1. **Pool Hit Rate**: Target >95%
2. **Buffer Acquisition Time**: Target <0.1ms
3. **GC Pause Frequency**: Target <1/minute
4. **Memory Usage**: Monitor total allocation
5. **Processing Time**: Track audio processing latency

### Alerting Thresholds

```javascript
// Example monitoring thresholds
const thresholds = {
    poolHitRate: 90,      // Alert if <90%
    acquisitionTime: 0.5,  // Alert if >0.5ms
    gcPauseFreq: 5,       // Alert if >5/minute
    memoryUsage: 100,     // Alert if >100MB
    processingTime: 2.0   // Alert if >2ms
};
```

## Performance Validation

### Validation Test Suite

```bash
# Performance validation workflow
perf reset
# Run application for 5 minutes
perf

# Check targets:
# - Pool hit rate >95%
# - GC pauses <5 total
# - Acquisition time <0.1ms average
# - Zero dropped chunks
```

### Load Testing

```bash
# Stress test configuration
pool size 4  # Reduce pool to stress test
# Run high-load scenario
perf
# Verify graceful degradation
```

This configuration guide provides the foundation for optimal buffer pool performance across different use cases and system constraints.