# Implementation Plan: Remove Global Audio Context Manager

## Current State Analysis

The codebase has a global `AUDIO_CONTEXT_MANAGER` in `pitch-toy/audio/mod.rs` that provides application-wide access to the AudioContextManager. This completely violates the principle of dependency injection that was the whole point of removing the global AudioWorkletManager.

### Critical Design Constraint
**We must NOT introduce any other variable with global access in place of the removed global. ALL access must be through proper dependency injection.**

### Current Global Usage Points
1. **Console Commands** (`audio/commands.rs:143`): Audio status command
2. **Console Service** (`audio/mod.rs:161`): Setting manager on console service  
3. **Audio System Ready Check** (`audio/mod.rs:146-152`): Checking if audio is initialized
4. **Initialization** (`pitch-toy/lib.rs:366-371`): Setting the global manager

## Implementation Plan

### Task 1: Remove All Global Infrastructure Immediately
Delete all global access mechanisms without replacement.

- [ ] 1a. Delete global AUDIO_CONTEXT_MANAGER declaration in `mod.rs`
- [ ] 1b. Delete `set_global_audio_context_manager()` function
- [ ] 1c. Delete `get_audio_context_manager()` function
- [ ] 1d. Delete global-based `is_audio_system_ready()` function
- [ ] 1e. Remove global setter call in `initialize_audio_systems_new()`

### Task 2: Fix Console Commands Through Proper Injection
Pass AudioSystemContext to console commands properly.

- [ ] 2a. Modify console command registration to accept context
  - Update command handler signatures to receive AudioSystemContext
  - Pass context from initialization point
- [ ] 2b. Update audio status command implementation
  - Accept AudioSystemContext parameter
  - Query status directly from context
- [ ] 2c. Update all console command call sites
  - Pass context reference where commands are invoked

### Task 3: Fix Console Service Creation
Pass AudioContextManager directly to console service.

- [ ] 3a. Change `create_console_audio_service()` signature
  - Add AudioContextManager parameter
  - Remove any global access attempts
- [ ] 3b. Update all console service creation sites
  - Pass manager from AudioSystemContext
  - Ensure proper ownership/borrowing

### Task 4: Replace Ready Check with Direct Context Query
Remove standalone ready check function.

- [ ] 4a. Delete standalone `is_audio_system_ready()` function
- [ ] 4b. Add ready check method to AudioSystemContext
  - `context.is_ready()` or similar
- [ ] 4c. Update all ready check sites
  - Use context instance method instead

### Task 5: Fix Thread Safety Issues Without Globals
Handle Send+Sync requirements properly.

- [ ] 5a. Identify all cross-thread communication points
  - Action listeners that need audio access
  - Background tasks requiring audio state
- [ ] 5b. Implement message passing for cross-thread needs
  - Use channels for thread-safe communication
  - No shared state, only messages
- [ ] 5c. Refactor action listeners
  - Pass message senders instead of direct references
  - Handle audio operations through messages

### Task 6: Ensure AudioSystemContext is Passed Everywhere
Make sure context is available wherever needed.

- [ ] 6a. Trace all audio operation entry points
  - Find where audio operations originate
  - Ensure context is available
- [ ] 6b. Update function signatures throughout
  - Add context parameters where missing
  - Remove any assumptions of global access
- [ ] 6c. Fix initialization flow
  - Ensure context is created early
  - Pass to all components that need it

### Task 7: Verify and Test
Ensure everything works without any global access.

- [ ] 7a. Compile and fix all errors
  - No code should reference removed globals
- [ ] 7b. Run `./scripts/test-all.sh`
  - All tests must pass
- [ ] 7c. Add tests for proper injection
  - Test that context is required
  - Test that no global access exists

## Dependencies and Order

1. **Task 1** - Do this FIRST. Break everything immediately.
2. **Tasks 2-4** - Fix the broken pieces with proper injection
3. **Task 5** - Handle special cases (threading)
4. **Task 6** - Ensure complete coverage
5. **Task 7** - Verify everything works

## Testing Considerations

- Every change must pass `./scripts/test-all.sh`
- Manual testing of console commands
- Verify audio still initializes and works
- Check that no global access remains

## Potential Challenges and Solutions

### Challenge 1: Thread Safety
**Problem**: Rc<RefCell<>> isn't Send+Sync but action listeners need it.
**Solution**: Use channels and message passing. No shared state across threads.

### Challenge 2: Console Architecture
**Problem**: Console expects direct access to audio state.
**Solution**: Pass context/manager at registration time.

### Challenge 3: Finding All Usage Sites
**Problem**: Global access might be used in unexpected places.
**Solution**: Delete globals first, let compiler find all usage sites.

## Success Criteria

- **ZERO** global variables or thread-local storage for audio access
- All audio operations require explicit context parameter
- No compatibility layers or backward compatibility code
- All tests pass
- Clean, dependency-injected architecture throughout