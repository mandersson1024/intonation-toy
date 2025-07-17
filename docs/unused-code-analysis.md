# Unused Code Analysis Report

## Overview
This document contains the results of an unused code analysis performed on the pitch-toy codebase.

## Key Findings

### 1. Unused Rust Functions (audio/mod.rs)
- `create_console_audio_service_with_events()` (line 108)
- `create_console_audio_service_with_setter()` (line 124)  
- `create_console_audio_service_with_audioworklet_setter()` (line 144)
- `enable_test_signal_440hz()` (line 259)

These appear to be legacy functions that are no longer needed.

### 2. Unused Error Reporting System (audio/message_protocol.rs)
The entire error reporting infrastructure (lines 2337-2365) including:
- `ErrorReportingSystem` struct
- `initialize_error_reporting()`
- `with_error_reporter()`
- `report_global_error()`
- `report_protocol_error_global()`

### 3. Unused JavaScript Code (pitch-toy/static/audio-processor.js)
In the `TransferableBufferPool` class:
- `release()` method (lines 199-216)
- `enableGCPauseDetection()` method (lines 369-374)
- `destroy()` method (lines 376-384)
- GC pause detection code that's implemented but never enabled

### 4. Unused Types
- `BufferState` enum in audio/buffer.rs (line 51)
- `TestSignalConfig` struct in debug/egui/live_data_panel.rs (line 119)

### 5. Unused Constants
- `PRODUCTION_BUFFER_SIZE` in audio/buffer.rs (line 7) - Used only in conditional compilation

### 6. Test Code
Several test functions in audio/mod.rs (lines 366-518) are marked with `#[allow(dead_code)]`

### 7. Dead Code Paths
- GC pause detection code in audio-processor.js (lines 141-148, 361-367) - implemented but never enabled
- The `destroy()` method on line 376 of the buffer pool is never called

## Recommendations

1. **Remove legacy functions**: The `create_console_audio_service_with_*` variants that are unused should be removed to reduce code complexity.

2. **Remove unused error reporting system**: The entire error reporting infrastructure in `message_protocol.rs` appears to be unused and adds significant complexity.

3. **Clean up JavaScript buffer pool**: Remove unused methods like `enableGCPauseDetection()` and `destroy()` if they're not needed.

4. **Review test code**: Either enable the tests marked as dead code or remove them if they're no longer relevant.

5. **Consider removing unused types**: `BufferState` enum and `TestSignalConfig` struct could be removed if confirmed unused.

6. **Audit imports**: Run `cargo clippy` with appropriate flags to identify and remove unused imports.

## Benefits of Cleanup
- Reduced code complexity
- Smaller bundle sizes
- Easier maintenance
- Clearer codebase for new developers