# Profiling Setup

This document describes how to profile the Intonation Toy application using the Chrome DevTools Performance panel. The profiling setup follows the Rust/WASM profiling manual and provides detailed performance insights for the real-time audio processing pipeline.

## Quick Start

To build and run the application with profiling enabled:

```bash
# Build and serve with profiling instrumentation using the profiling HTML file
trunk serve --release --features profiling --index intonation-toy/index.profiling.html -- --profile profiling
```

Then open Chrome DevTools Performance panel and record while using the application.

## Configuration

The profiling setup includes:

### Build Configuration

- **Profiling build profile** in `Cargo.toml` that inherits from release but preserves debug information
- **Feature flag** (`profiling`) to conditionally compile profiling instrumentation
- **Trunk configuration** with `data-wasm-opt="0"` to preserve symbols (configured in `index.profiling.html`)

### Instrumented Code Sections

The following performance-critical areas are instrumented with User Timing marks and console.time measurements:

1. **Engine Layer Update** (`engine_update`) - Audio processing and pitch detection pipeline
2. **Model Layer Update** (`model_update`) - Data processing and tuning system calculations  
3. **User Action Processing** (`user_action_processing`) - UI interaction handling
4. **Pitch Analysis** (`pitch_analyze_samples`, `pitch_analyze_from_buffer`) - Core pitch detection algorithms

## Using Chrome DevTools

1. **Start profiling build**: `trunk serve --release --features profiling --index intonation-toy/index.profiling.html -- --profile profiling`
2. **Open Chrome DevTools** (F12)
3. **Go to Performance panel**
4. **Click Record** button
5. **Interact with the application** (speak/sing into microphone, change settings)
6. **Stop recording** after 10-30 seconds
7. **Analyze the results**:
   - Look for the User Timing marks in the timeline
   - Check function names in the flame chart (they should be preserved)
   - Review console.time measurements in the Console panel

## Interpreting Results

### User Timing Marks

The profiled sections will appear as colored marks in the Performance timeline:
- `engine_update` - Shows audio processing frequency and duration
- `model_update` - Shows model computation overhead
- `user_action_processing` - Shows UI interaction costs
- `pitch_analyze_samples` - Shows pitch detection algorithm performance

### Function Names

With debug symbols preserved, you should see meaningful function names in the flame chart instead of just memory addresses, making it easier to identify performance bottlenecks.

### Performance Metrics

The application already includes detailed performance metrics for:
- Audio latency measurements
- Memory usage tracking  
- Pitch detection timing
- Frame rate monitoring

These metrics complement the profiling data and can be viewed in the debug console (debug builds only).

## Build Profiles

### Development
```bash
trunk serve
```
Standard development build with full debug info.

### Profiling  
```bash
trunk serve --release --features profiling --index intonation-toy/index.profiling.html -- --profile profiling
```
Optimized build with debug symbols preserved for profiling.

### Production
```bash
trunk build --release
```
Fully optimized production build without profiling instrumentation.

## Troubleshooting

**No function names in profiler**: Ensure you're using the profiling build profile with `--index intonation-toy/index.profiling.html` which has `data-wasm-opt="0"` configured.

**No User Timing marks**: Check that the `profiling` feature is enabled and the application is running the instrumented code paths.

**Poor performance during profiling**: This is expected - the profiling instrumentation adds overhead. Use the production build for actual performance testing.

## References

- [Chrome DevTools Performance Panel](https://developers.google.com/web/tools/chrome-devtools/evaluate-performance)
- [User Timing API](https://developer.mozilla.org/en-US/docs/Web/API/User_Timing_API)
- [Rust WASM Profiling Manual](./rust_wasm_profiling_manual.md)