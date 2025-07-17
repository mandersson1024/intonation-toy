# Buffer Pool Troubleshooting Guide

## Overview

This guide provides diagnostic procedures and solutions for common buffer pool ping-pong pattern issues in the AudioWorklet implementation.

## Quick Diagnostic Commands

### Check Current Status
```bash
# Show overall performance metrics
perf

# Check buffer pool configuration
pool

# Show detailed buffer status
buffer status

# Reset performance counters for clean measurement
perf reset
```

### Common Status Indicators

**Healthy System:**
```
🔬 Audio Processing Performance Metrics
Buffer Pool Performance:
  Pool Size: 16 buffers
  Available: 12 buffers
  Hit Rate: 98.5%
  Avg Acquisition: 0.05ms
  
Audio Processing:
  GC Pauses: 0
  Dropped Chunks: 0
  Processed Chunks: 1,247
```

**Problematic System:**
```
🔬 Audio Processing Performance Metrics
Buffer Pool Performance:
  Pool Size: 16 buffers
  Available: 0 buffers
  Hit Rate: 67.2%
  Avg Acquisition: 2.3ms
  
Audio Processing:
  GC Pauses: 23
  Dropped Chunks: 156
  Processed Chunks: 891
```

## Common Issues and Solutions

### 1. Pool Exhaustion (Hit Rate < 90%)

**Symptoms:**
- Pool hit rate below 90%
- Frequent "Pool exhausted" messages
- Available buffers consistently at 0
- Dropped chunks increasing

**Diagnosis:**
```bash
# Check pool status
pool
# Look for: Available: 0 buffers, Hit Rate: <90%

# Check processing delays
perf
# Look for: High avg acquisition time, many dropped chunks
```

**Root Causes:**
- Pool size too small for processing load
- Buffers stuck in processing state
- Timeout too long for current processing speed
- Main thread processing taking too long

**Solutions:**

**Immediate Fix:**
```bash
# Increase pool size
pool size 24

# Reduce timeout for faster buffer recovery
pool timeout 3000

# Check if improvement helps
perf
```

**Long-term Solutions:**
1. **Optimize Processing Speed:**
   - Profile main thread processing time
   - Reduce complexity in volume/pitch analysis
   - Consider processing on separate threads

2. **Tune Pool Configuration:**
   - Calculate optimal pool size: `(processing_time_ms / chunk_interval_ms) * 2`
   - Set timeout to 3-5x average processing time
   - Monitor hit rate after changes

3. **Check for Buffer Leaks:**
   - Monitor buffer return messages
   - Verify timeout recovery is working
   - Look for stuck buffers in processing state

### 2. High GC Pause Frequency (> 5 per minute)

**Symptoms:**
- GC pauses > 5 per minute
- Audio glitches or interruptions
- Inconsistent processing times
- Pool acquisition time spikes

**Diagnosis:**
```bash
# Check GC pause detection
perf gc
# Look for: High pause count, frequent detections

# Check GC detection threshold
pool gc status
# Look for: Current threshold setting
```

**Root Causes:**
- Pool size too small causing allocation fallbacks
- GC detection threshold too high
- Memory pressure from other sources
- Inefficient buffer recycling

**Solutions:**

**Immediate Fix:**
```bash
# Increase pool size to reduce allocations
pool size 32

# Lower GC detection threshold
pool gc threshold 30

# Enable GC detection if disabled
pool gc enable

# Monitor improvement
perf
```

**Advanced Solutions:**
1. **Memory Optimization:**
   - Increase pool size to cover worst-case scenarios
   - Pre-allocate buffers during initialization
   - Avoid allocation in hot paths

2. **GC Tuning:**
   - Set threshold based on audio chunk interval (e.g., 50% of chunk time)
   - Monitor GC patterns during different workloads
   - Consider browser-specific optimizations

### 3. High Buffer Acquisition Time (> 0.5ms)

**Symptoms:**
- Average acquisition time > 0.5ms
- Processing delays
- Inconsistent latency
- Audio stuttering

**Diagnosis:**
```bash
# Check acquisition metrics
perf
# Look for: Avg Acquisition > 0.5ms

# Check pool configuration
pool
# Look for: Pool size, available buffers
```

**Root Causes:**
- Pool size too large causing search overhead
- Memory fragmentation
- GC pauses during acquisition
- System under high load

**Solutions:**

**Pool Size Optimization:**
```bash
# Try smaller pool size
pool size 8

# Monitor acquisition time
perf

# If still high, check for GC interference
pool gc threshold 25
```

**System-level Solutions:**
1. **Reduce Pool Size:**
   - Start with minimum viable pool size
   - Increase gradually until hit rate stabilizes
   - Monitor acquisition time vs hit rate tradeoff

2. **Check System Resources:**
   - Monitor CPU usage during audio processing
   - Check memory pressure indicators
   - Verify browser isn't throttling audio thread

### 4. Buffer Timeout Issues

**Symptoms:**
- Frequent buffer timeout messages
- Pool size decreasing over time
- Inconsistent available buffer count
- "Buffer reclaimed" messages

**Diagnosis:**
```bash
# Check timeout configuration
pool timeout status
# Look for: Current timeout setting

# Check buffer states
buffer status
# Look for: Buffers in timed-out state
```

**Root Causes:**
- Timeout too short for processing workload
- Main thread processing hanging
- Buffer return mechanism failing
- Error in buffer ID tracking

**Solutions:**

**Timeout Adjustment:**
```bash
# Increase timeout for heavy processing
pool timeout 8000

# Monitor timeout occurrences
perf

# Check buffer return success rate
buffer stats
```

**Debugging Steps:**
1. **Verify Processing Times:**
   - Measure actual processing duration
   - Set timeout to 3-5x processing time
   - Monitor for processing hangs

2. **Check Return Mechanism:**
   - Verify buffer return messages are sent
   - Check for buffer ID mismatches
   - Ensure transferable array is properly formed

### 5. Memory Leaks

**Symptoms:**
- Pool size decreasing over time
- Available buffers trending down
- Browser memory usage increasing
- Eventual pool exhaustion

**Diagnosis:**
```bash
# Monitor pool size over time
pool
# Check: Total buffers count

# Check for stuck buffers
buffer status
# Look for: Buffers in processing state for too long

# Monitor memory usage in browser dev tools
```

**Root Causes:**
- Buffers not being returned properly
- Timeout mechanism not working
- References held to transferred buffers
- Error in buffer lifecycle management

**Solutions:**

**Immediate Recovery:**
```bash
# Reset pool if possible
pool reset

# Reduce timeout for faster recovery
pool timeout 2000

# Enable aggressive timeout recovery
pool timeout enable
```

**Long-term Fixes:**
1. **Audit Buffer References:**
   - Ensure no references held after transfer
   - Verify buffer cleanup on errors
   - Check timeout recovery mechanism

2. **Improve Error Handling:**
   - Add buffer return on processing errors
   - Implement graceful shutdown cleanup
   - Add buffer leak detection

### 6. Configuration Issues

**Symptoms:**
- Pool settings not taking effect
- Unexpected pool behavior
- Configuration commands failing
- Settings reverting to defaults

**Diagnosis:**
```bash
# Check current configuration
pool config

# Try setting values and verify
pool size 20
pool
# Verify: Pool Size shows 20

# Check for configuration errors
pool validate
```

**Common Configuration Problems:**

**Invalid Pool Size:**
```bash
# Pool size too small
pool size 2
# Error: Pool size must be >= 4

# Pool size too large
pool size 200
# Error: Pool size must be <= 64
```

**Invalid Timeout:**
```bash
# Timeout too short
pool timeout 50
# Error: Timeout must be >= 100ms

# Timeout too long
pool timeout 60000
# Error: Timeout must be <= 30000ms
```

**Solutions:**
1. **Use Valid Ranges:**
   - Pool size: 4-64 buffers
   - Timeout: 100-30000ms
   - GC threshold: 1-1000ms

2. **Validate Before Apply:**
   - Check configuration bounds
   - Test with safe values first
   - Monitor system after changes

## Diagnostic Workflows

### Performance Issue Workflow

1. **Initial Assessment:**
   ```bash
   perf reset
   # Wait 2-3 minutes for data
   perf
   ```

2. **Identify Primary Issue:**
   - Hit rate < 90% → Pool exhaustion
   - GC pauses > 5 → Memory pressure
   - Acquisition time > 0.5ms → Pool size issue
   - Dropped chunks > 0 → Processing delays

3. **Apply Targeted Fix:**
   - Pool exhaustion → Increase pool size
   - Memory pressure → Reduce allocations
   - Pool size issue → Optimize pool configuration
   - Processing delays → Optimize processing code

4. **Verify Improvement:**
   ```bash
   perf reset
   # Wait 2-3 minutes
   perf
   # Check if metrics improved
   ```

### Memory Issue Workflow

1. **Check Pool Status:**
   ```bash
   pool
   buffer status
   ```

2. **Look for Patterns:**
   - Available buffers decreasing?
   - Buffers stuck in processing?
   - High timeout count?

3. **Test Recovery:**
   ```bash
   pool timeout 1000
   # Wait for timeout recovery
   pool
   # Check if buffers recovered
   ```

4. **Investigate Root Cause:**
   - Check buffer return mechanism
   - Verify timeout recovery
   - Monitor processing completion

### Configuration Issue Workflow

1. **Verify Current State:**
   ```bash
   pool config
   ```

2. **Test Configuration Changes:**
   ```bash
   pool size 16
   pool timeout 5000
   pool gc threshold 50
   ```

3. **Validate Changes:**
   ```bash
   pool
   # Verify settings applied
   ```

4. **Monitor Impact:**
   ```bash
   perf reset
   # Wait for data
   perf
   # Check performance impact
   ```

## Monitoring and Alerting

### Key Metrics to Watch

**Critical Metrics:**
- Pool hit rate < 90%
- Available buffers = 0
- GC pauses > 5/minute
- Dropped chunks > 0

**Warning Metrics:**
- Pool hit rate < 95%
- Acquisition time > 0.1ms
- Buffer timeout count increasing
- Processing time > 1.0ms

**Monitoring Script:**
```bash
#!/bin/bash
# Simple monitoring loop
while true; do
    echo "$(date): $(perf | grep 'Hit Rate\|GC Pauses\|Dropped Chunks')"
    sleep 60
done
```

### Emergency Procedures

**Pool Exhaustion Emergency:**
```bash
# Immediate relief
pool size 32
pool timeout 2000

# Monitor recovery
perf
```

**Memory Leak Emergency:**
```bash
# Force timeout recovery
pool timeout 500
# Wait 2-3 seconds
pool timeout 5000

# Check recovery
pool
```

**Performance Emergency:**
```bash
# Optimize for immediate relief
pool size 8
pool timeout 1000
pool gc threshold 25

# Monitor improvement
perf
```

## Prevention Best Practices

### 1. Proper Pool Sizing

**Calculation Method:**
```javascript
// Calculate optimal pool size
const processingTimeMs = 2.0; // Measured processing time
const chunkIntervalMs = 2.67; // 128 samples / 48kHz * 1000
const minPoolSize = Math.ceil(processingTimeMs / chunkIntervalMs) * 2;
const recommendedPoolSize = Math.max(minPoolSize, 8);
```

**Recommended Sizes:**
- **Low latency:** 4-8 buffers
- **Normal processing:** 16 buffers
- **Heavy processing:** 24-32 buffers

### 2. Timeout Management

**Timeout Guidelines:**
```javascript
// Set timeout based on processing characteristics
const avgProcessingTime = 2.0; // ms
const timeout = Math.max(avgProcessingTime * 5, 1000); // 5x processing time, minimum 1s
```

### 3. Performance Monitoring

**Regular Checks:**
```bash
# Daily health check
perf | grep "Hit Rate\|GC Pauses\|Dropped Chunks"

# Weekly deep analysis
perf
pool
buffer status
```

### 4. Configuration Management

**Safe Configuration Changes:**
```bash
# Always check current state first
pool

# Make incremental changes
pool size 18  # Small increase

# Monitor impact
perf reset
# Wait for data
perf
```

## Advanced Debugging

### Buffer Lifecycle Tracing

**Enable Debug Logging:**
```bash
# Enable detailed buffer logging
pool debug enable

# Check specific buffer states
buffer trace <buffer_id>
```

**Log Analysis:**
```bash
# Filter for buffer-related messages
grep "Buffer" browser_console.log | tail -20

# Look for patterns:
# - Buffer acquired → transferred → returned cycle
# - Timeout recovery events
# - Pool exhaustion events
```

### Performance Profiling

**Browser Developer Tools:**
1. Open Performance tab
2. Start recording
3. Run audio processing for 30 seconds
4. Stop recording and analyze:
   - Look for GC spikes
   - Check allocation patterns
   - Monitor main thread vs audio thread activity

**Memory Profiling:**
1. Open Memory tab
2. Take heap snapshot before starting
3. Run audio processing
4. Take heap snapshot after
5. Compare for memory leaks

### Cross-Browser Testing

**Test Matrix:**
```bash
# Test on different browsers
# Chrome: Check V8 GC behavior
# Firefox: Check SpiderMonkey differences
# Safari: Check WebKit compatibility

# Test different sample rates
# 44.1kHz, 48kHz, 96kHz

# Test different buffer sizes
# 512, 1024, 2048 samples
```

## Recovery Procedures

### Pool Recovery

**Soft Reset:**
```bash
pool timeout 1000
# Wait 2 seconds
pool timeout 5000
perf reset
```

**Hard Reset:**
```bash
# Stop processing
stop

# Reset pool
pool reset

# Restart processing
start
```

### System Recovery

**Browser Refresh:**
- Save current configuration
- Refresh page
- Restore configuration
- Monitor for recurring issues

**Complete Restart:**
- Document current issue
- Close browser completely
- Restart browser
- Test with default configuration

This troubleshooting guide provides comprehensive diagnostic procedures for maintaining optimal buffer pool performance in production environments.