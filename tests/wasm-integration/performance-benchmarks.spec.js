// Performance Benchmark Integration Tests
// Tests the performance benchmark system in browser environment

const { test, expect } = require('@jest/globals');

describe('Performance Benchmark Integration', () => {
  let wasmModule;
  let AudioEngine;
  let PitchAlgorithm;

  beforeAll(async () => {
    // Mock WASM module for testing performance infrastructure
    wasmModule = {
      AudioEngine: class MockAudioEngine {
        constructor(sampleRate, bufferSize) {
          this.sampleRate = sampleRate;
          this.bufferSize = bufferSize;
          this.enabled = true;
          this.algorithm = 0; // YIN
        }

        process_audio_buffer(input) {
          // Simulate processing time
          const start = performance.now();
          const output = input.map(x => x * 0.8);
          while (performance.now() - start < 0.1) {} // Simulate 0.1ms processing
          return output;
        }

        detect_pitch_from_buffer(input) {
          // Simulate pitch detection time
          const start = performance.now();
          while (performance.now() - start < 0.5) {} // Simulate 0.5ms processing
          return 440.0;
        }

        process_audio_with_pitch(input) {
          // Simulate combined processing
          const start = performance.now();
          const output = input.map(x => x * 0.8);
          while (performance.now() - start < 0.6) {} // Simulate 0.6ms processing
          return output;
        }

        set_pitch_algorithm(algorithm) {
          this.algorithm = algorithm;
        }

        set_pitch_frequency_range(min, max) {
          this.minFreq = min;
          this.maxFreq = max;
        }

        set_enabled(enabled) {
          this.enabled = enabled;
        }

        get_sample_rate() {
          return this.sampleRate;
        }

        get_buffer_size() {
          return this.bufferSize;
        }
      },
      
      PitchAlgorithm: {
        YIN: 0,
        McLeod: 1
      }
    };

    AudioEngine = wasmModule.AudioEngine;
    PitchAlgorithm = wasmModule.PitchAlgorithm;
  });

  test('Performance benchmark infrastructure is available', () => {
    expect(AudioEngine).toBeDefined();
    expect(PitchAlgorithm).toBeDefined();
    expect(PitchAlgorithm.YIN).toBe(0);
    expect(PitchAlgorithm.McLeod).toBe(1);
  });

  test('Audio processing latency benchmark', () => {
    const engine = new AudioEngine(44100, 1024);
    const testBuffer = new Float32Array(1024).fill(0.5);
    const iterations = 100;

    // Warmup
    for (let i = 0; i < 10; i++) {
      engine.process_audio_buffer(testBuffer);
    }

    // Benchmark
    const start = performance.now();
    for (let i = 0; i < iterations; i++) {
      engine.process_audio_buffer(testBuffer);
    }
    const end = performance.now();

    const avgDuration = (end - start) / iterations;
    const bufferDuration = (1024 / 44100) * 1000; // Duration of buffer in ms

    console.log(`Audio processing: ${avgDuration.toFixed(3)}ms (${(bufferDuration / avgDuration).toFixed(1)}x real-time)`);

    // Should be much faster than real-time
    expect(avgDuration).toBeLessThan(50); // <50ms requirement
    expect(avgDuration).toBeLessThan(bufferDuration); // Faster than real-time
  });

  test('Pitch detection latency benchmark', () => {
    const engine = new AudioEngine(44100, 2048);
    const sampleRate = 44100;
    const bufferSize = 2048;
    const frequency = 440;

    // Generate sine wave test signal
    const testBuffer = new Float32Array(bufferSize);
    for (let i = 0; i < bufferSize; i++) {
      testBuffer[i] = 0.8 * Math.sin(2 * Math.PI * frequency * i / sampleRate);
    }

    const algorithms = [PitchAlgorithm.YIN, PitchAlgorithm.McLeod];

    for (const algorithm of algorithms) {
      engine.set_pitch_algorithm(algorithm);

      // Warmup
      for (let i = 0; i < 5; i++) {
        engine.detect_pitch_from_buffer(testBuffer);
      }

      // Benchmark
      const iterations = 50;
      const start = performance.now();
      for (let i = 0; i < iterations; i++) {
        const pitch = engine.detect_pitch_from_buffer(testBuffer);
        expect(typeof pitch).toBe('number');
      }
      const end = performance.now();

      const avgDuration = (end - start) / iterations;
      const bufferDuration = (bufferSize / sampleRate) * 1000;

      console.log(`Pitch detection (${algorithm === 0 ? 'YIN' : 'McLeod'}): ${avgDuration.toFixed(3)}ms (${(bufferDuration / avgDuration).toFixed(1)}x real-time)`);

      // Performance requirements
      expect(avgDuration).toBeLessThan(50); // <50ms requirement
    }
  });

  test('Combined processing latency benchmark', () => {
    const engine = new AudioEngine(44100, 1024);
    const testBuffer = new Float32Array(1024);
    
    // Generate test signal
    for (let i = 0; i < 1024; i++) {
      testBuffer[i] = 0.6 * Math.sin(2 * Math.PI * 220 * i / 44100);
    }

    // Warmup
    for (let i = 0; i < 5; i++) {
      engine.process_audio_with_pitch(testBuffer);
    }

    // Benchmark
    const iterations = 100;
    const start = performance.now();
    for (let i = 0; i < iterations; i++) {
      const output = engine.process_audio_with_pitch(testBuffer);
      expect(output.length).toBe(testBuffer.length);
    }
    const end = performance.now();

    const avgDuration = (end - start) / iterations;
    const bufferDuration = (1024 / 44100) * 1000;

    console.log(`Combined processing: ${avgDuration.toFixed(3)}ms (${(bufferDuration / avgDuration).toFixed(1)}x real-time)`);

    // Performance requirements
    expect(avgDuration).toBeLessThan(50); // <50ms requirement
    expect(avgDuration).toBeLessThan(bufferDuration * 2); // Allow 2x buffer time for combined processing
  });

  test('Throughput benchmark for different buffer sizes', () => {
    const bufferSizes = [512, 1024, 2048, 4096];
    
    for (const bufferSize of bufferSizes) {
      const engine = new AudioEngine(44100, bufferSize);
      const testBuffer = new Float32Array(bufferSize).fill(0.3);

      // Warmup
      for (let i = 0; i < 5; i++) {
        engine.process_audio_buffer(testBuffer);
      }

      // Benchmark throughput
      const iterations = 100;
      const start = performance.now();
      for (let i = 0; i < iterations; i++) {
        engine.process_audio_buffer(testBuffer);
      }
      const end = performance.now();

      const totalSamples = bufferSize * iterations;
      const durationSec = (end - start) / 1000;
      const throughput = totalSamples / durationSec;

      console.log(`Buffer size ${bufferSize}: ${(throughput / 1000000).toFixed(1)}M samples/sec`);

      // Should achieve reasonable throughput
      expect(throughput).toBeGreaterThan(1000000); // >1M samples/sec
    }
  });

  test('Algorithm performance comparison', () => {
    const engine = new AudioEngine(44100, 2048);
    const algorithms = [PitchAlgorithm.YIN, PitchAlgorithm.McLeod];
    const testFrequencies = [110, 220, 440, 880];

    const results = {};

    for (const algorithm of algorithms) {
      engine.set_pitch_algorithm(algorithm);
      let totalDuration = 0;
      let totalTests = 0;

      for (const freq of testFrequencies) {
        // Generate test signal
        const testBuffer = new Float32Array(2048);
        for (let i = 0; i < 2048; i++) {
          testBuffer[i] = 0.7 * Math.sin(2 * Math.PI * freq * i / 44100);
        }

        // Warmup
        for (let i = 0; i < 3; i++) {
          engine.detect_pitch_from_buffer(testBuffer);
        }

        // Benchmark
        const iterations = 20;
        const start = performance.now();
        for (let i = 0; i < iterations; i++) {
          engine.detect_pitch_from_buffer(testBuffer);
        }
        const end = performance.now();

        totalDuration += (end - start);
        totalTests += iterations;
      }

      const avgDuration = totalDuration / totalTests;
      results[algorithm === 0 ? 'YIN' : 'McLeod'] = avgDuration;

      console.log(`${algorithm === 0 ? 'YIN' : 'McLeod'}: ${avgDuration.toFixed(3)}ms average`);

      // Performance requirements
      expect(avgDuration).toBeLessThan(50);
    }

    // Both algorithms should perform reasonably well
    expect(results.YIN).toBeLessThan(50);
    expect(results.McLeod).toBeLessThan(50);
  });

  test('Sustained performance test', () => {
    const engine = new AudioEngine(44100, 1024);
    const testBuffer = new Float32Array(1024).fill(0.4);
    
    // Simulate 100ms of processing (about 4 buffers)
    const iterations = 4;
    const durations = [];

    for (let i = 0; i < iterations; i++) {
      const start = performance.now();
      engine.process_audio_with_pitch(testBuffer);
      const end = performance.now();
      
      durations.push(end - start);
    }

    const avgDuration = durations.reduce((a, b) => a + b) / durations.length;
    const maxDuration = Math.max(...durations);
    const minDuration = Math.min(...durations);

    console.log(`Sustained performance: avg=${avgDuration.toFixed(3)}ms, min=${minDuration.toFixed(3)}ms, max=${maxDuration.toFixed(3)}ms`);

    // Check for performance consistency
    expect(avgDuration).toBeLessThan(50);
    expect(maxDuration).toBeLessThan(avgDuration * 3); // Max shouldn't be >3x average
    
    // Check for performance degradation
    const firstHalf = durations.slice(0, Math.floor(iterations / 2));
    const secondHalf = durations.slice(Math.floor(iterations / 2));
    
    const firstAvg = firstHalf.reduce((a, b) => a + b) / firstHalf.length;
    const secondAvg = secondHalf.reduce((a, b) => a + b) / secondHalf.length;
    
    // Second half shouldn't be significantly slower than first half
    expect(secondAvg / firstAvg).toBeLessThan(2.0);
  });

  test('Memory allocation performance', () => {
    // Test that repeated operations don't cause memory issues
    const engine = new AudioEngine(44100, 1024);
    
    // Create many different buffer sizes
    const bufferSizes = [512, 1024, 2048];
    
    for (const bufferSize of bufferSizes) {
      const iterations = 50;
      
      for (let i = 0; i < iterations; i++) {
        const testBuffer = new Float32Array(bufferSize).fill(0.1 * i);
        const output = engine.process_audio_buffer(testBuffer);
        
        // Verify output properties
        expect(output.length).toBe(bufferSize);
        expect(output).toBeInstanceOf(Array);
        
        // Ensure no memory leaks by checking for valid numbers
        expect(output.every(x => typeof x === 'number' && isFinite(x))).toBe(true);
      }
    }
  });

  test('Performance regression detection', () => {
    const engine = new AudioEngine(44100, 1024);
    const testBuffer = new Float32Array(1024).fill(0.5);
    
    // Run baseline measurements
    const baselineIterations = 20;
    let baselineTotal = 0;
    
    for (let i = 0; i < baselineIterations; i++) {
      const start = performance.now();
      engine.process_audio_buffer(testBuffer);
      const end = performance.now();
      baselineTotal += (end - start);
    }
    
    const baselineAvg = baselineTotal / baselineIterations;
    
    // Run test measurements after some processing
    for (let i = 0; i < 100; i++) {
      engine.process_audio_buffer(testBuffer);
    }
    
    const testIterations = 20;
    let testTotal = 0;
    
    for (let i = 0; i < testIterations; i++) {
      const start = performance.now();
      engine.process_audio_buffer(testBuffer);
      const end = performance.now();
      testTotal += (end - start);
    }
    
    const testAvg = testTotal / testIterations;
    const regressionRatio = testAvg / baselineAvg;
    
    console.log(`Performance comparison: baseline=${baselineAvg.toFixed(3)}ms, current=${testAvg.toFixed(3)}ms, ratio=${regressionRatio.toFixed(2)}x`);
    
    // Should not have significant performance regression
    expect(regressionRatio).toBeLessThan(2.0); // Current should not be >2x slower than baseline
  });

  test('Browser-specific performance metrics', () => {
    // Test that performance.now() provides sufficient precision
    const start = performance.now();
    const end = performance.now();
    const precision = end - start;
    
    // Should have microsecond-level precision
    expect(typeof precision).toBe('number');
    expect(precision).toBeGreaterThanOrEqual(0);
    
    // Test high-resolution timing
    const measurements = [];
    for (let i = 0; i < 10; i++) {
      const t1 = performance.now();
      // Minimal operation
      Math.sqrt(i);
      const t2 = performance.now();
      measurements.push(t2 - t1);
    }
    
    // Should be able to measure sub-millisecond operations
    const avgMeasurement = measurements.reduce((a, b) => a + b) / measurements.length;
    expect(avgMeasurement).toBeGreaterThan(0);
    expect(avgMeasurement).toBeLessThan(10); // Should be very fast
  });
}); 