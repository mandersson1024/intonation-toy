# Implementation Plan: Remove Global AudioWorklet Manager

## Overview

This document outlines the implementation plan for removing the global `get_global_audioworklet_manager()` function and replacing it with an instance-based approach using dependency injection within the audio module.

**Goal:** Improve encapsulation, testability, and maintainability while maintaining current functionality and performance.

## Current State Analysis

### Usage Distribution
- **Total usages:** 8 occurrences across 2 files, all within audio module
- **mod.rs:** 7 usages (1 function definition + 3 configuration setters + 3 action listeners)
- **microphone.rs:** 1 usage (critical initialization with error handling)

### Key Dependencies
- AudioWorkletManager depends on AudioContextManager
- Microphone functionality requires both managers
- UI integration through action pattern and observable data setters
- Console commands access global instances
- **Data setters are created before any audio initialization** - available at construction time

### Current Architecture Strengths
- Modular design with clear separation of concerns
- Generic interfaces that don't create tight coupling
- Progressive initialization allowing partial functionality
- Comprehensive testing with wasm-pack test framework
- **Data setters are created at startup before any audio components** - perfect for constructor injection

### Areas for Improvement
- Hidden dependencies through global access make testing difficult
- No explicit dependency injection makes unit testing complex
- Global state management complicates lifecycle management
- Tight coupling between initialization order and functionality

## Implementation Plan

### Task 1: Create AudioSystem Context Structure ✅
**Goal:** Create a central structure that owns and manages all audio-related instances

- [x] 1a. Create `AudioSystemContext` struct in `audio/context.rs`
  - [x] 1a.1. Define struct with owned AudioContextManager and AudioWorkletManager
  - [x] 1a.2. Add PitchAnalyzer and other component instances
  - [x] 1a.3. Implement constructor accepting data setters as mandatory parameters
  - [x] 1a.4. Add methods for accessing components safely

- [x] 1b. Implement lifecycle management methods
  - [x] 1b.1. `async fn initialize()` method combining all initialization steps
  - [x] 1b.2. `fn shutdown()` method for cleanup
  - [x] 1b.3. `fn is_ready()` method for status checking
  - [x] 1b.4. Error handling for initialization failures

- [x] 1c. Add constructor parameters for data setters
  - [x] 1c.1. Add `volume_level_setter` parameter to constructor
  - [x] 1c.2. Add `pitch_data_setter` parameter to constructor
  - [x] 1c.3. Add `audioworklet_status_setter` parameter to constructor
  - [x] 1c.4. Pass setters during construction rather than after initialization

- [x] 1d. Write comprehensive tests
  - [x] 1d.1. Test AudioSystemContext creation and initialization
  - [x] 1d.2. Test component access methods
  - [x] 1d.3. Test lifecycle management
  - [x] 1d.4. Test error handling scenarios

### Task 2: Update Function Signatures for Dependency Injection ✅
**Goal:** Modify audio module functions to accept AudioSystemContext instead of using global access

- [x] 2a. Update initialization functions to pass setters at construction
  - [x] 2a.1. Update `initialize_audio_systems()` to pass setters to context constructor
  - [x] 2a.2. Remove separate setter configuration calls
  - [x] 2a.3. Ensure setters are available during construction phase
  - [x] 2a.4. All setters are mandatory parameters

- [x] 2b. Update action listener setup
  - [x] 2b.1. Modify `setup_ui_action_listeners()` to accept context parameter
  - [x] 2b.2. Update action listener closures to use context instead of global access
  - [x] 2b.3. Ensure proper lifetime management for context references
  - [x] 2b.4. Test action listener functionality with new approach

- [x] 2c. Update microphone integration
  - [x] 2c.1. Modify microphone connection functions to accept context
  - [x] 2c.2. Update error handling to work with context-based approach
  - [x] 2c.3. Ensure proper AudioWorkletManager access in microphone.rs
  - [x] 2c.4. Test microphone functionality with dependency injection

- [x] 2d. Update pitch analyzer initialization
  - [x] 2d.1. Modify `initialize_pitch_analyzer()` to accept context parameter
  - [x] 2d.2. Update pitch analyzer configuration to use context
  - [x] 2d.3. Ensure proper integration with AudioWorkletManager
  - [x] 2d.4. Test pitch analyzer functionality

### Task 3: Implement Gradual Migration Strategy ✅
**Goal:** Implement changes incrementally to maintain functionality during transition

- [x] 3a. Create bridge pattern for compatibility
  - [x] 3a.1. Add temporary global AudioSystemContext instance
  - [x] 3a.2. Implement wrapper functions that delegate to context methods
  - [x] 3a.3. Add deprecation warnings for global access functions
  - [x] 3a.4. Ensure all tests pass with bridge pattern

- [x] 3b. Migrate module initialization
  - [x] 3b.1. Update `initialize_audio_system()` to return AudioSystemContext
  - [x] 3b.2. Update main initialization sequence in lib.rs
  - [x] 3b.3. Ensure proper context lifecycle management
  - [x] 3b.4. Test complete initialization flow

- [x] 3c. Update calling sites incrementally
  - [x] 3c.1. Start with configuration setter calls
  - [x] 3c.2. Update action listener setup
  - [x] 3c.3. Update microphone integration
  - [x] 3c.4. Update pitch analyzer initialization

- [x] 3d. Test each migration step
  - [x] 3d.1. Run full test suite after each change
  - [x] 3d.2. Verify no functionality regression
  - [x] 3d.3. Check performance impact
  - [x] 3d.4. Validate error handling still works

### Task 4: Remove Global State Infrastructure
**Goal:** Remove thread-local storage and global access functions

- [ ] 4a. Remove global access functions
  - [ ] 4a.1. Remove `get_global_audioworklet_manager()` function
  - [ ] 4a.2. Remove `set_global_audioworklet_manager()` function
  - [ ] 4a.3. Remove `AUDIOWORKLET_MANAGER_GLOBAL` thread-local variable
  - [ ] 4a.4. Update module exports to remove global functions

- [ ] 4b. Remove bridge pattern infrastructure
  - [ ] 4b.1. Remove temporary global AudioSystemContext instance
  - [ ] 4b.2. Remove wrapper functions
  - [ ] 4b.3. Remove deprecation warnings
  - [ ] 4b.4. Clean up any remaining compatibility code

- [ ] 4c. Update documentation
  - [ ] 4c.1. Update function documentation to reflect new parameters
  - [ ] 4c.2. Update module-level documentation
  - [ ] 4c.3. Update usage examples in comments
  - [ ] 4c.4. Update architectural analysis document

- [ ] 4d. Final testing and validation
  - [ ] 4d.1. Run complete test suite
  - [ ] 4d.2. Verify all functionality works correctly
  - [ ] 4d.3. Check for any remaining global state dependencies
  - [ ] 4d.4. Validate performance hasn't degraded


## Dependencies and Order of Operations

### Critical Dependencies
1. **Task 1** must be completed before **Task 2** (context structure needed for function updates)
2. **Task 2** must be completed before **Task 3** (updated functions needed for migration)
3. **Task 3** must be completed before **Task 4** (migration needed before removal)

### Initialization Order Requirements
1. AudioContextManager must be initialized before AudioWorkletManager
2. AudioWorkletManager must be initialized before PitchAnalyzer
3. Data setters must be configured after component initialization
4. Action listeners must be set up after all components are ready

## Testing Considerations

### Test Strategy
- **Unit tests** for each component in isolation
- **Integration tests** for component interactions
- **Performance tests** to validate no regression
- **End-to-end tests** for complete audio functionality

### Test Requirements
- All tests must use `wasm-pack test --node` as specified in CLAUDE.md
- Tests must pass before any task can be marked complete
- Each subtask should have corresponding tests
- Performance tests should validate no regression

### Test Coverage Goals
- 100% coverage of AudioSystemContext functionality
- 100% coverage of migration paths
- 100% coverage of error scenarios
- Maintain existing test coverage levels

## Potential Challenges and Solutions

### Challenge 1: Circular Dependencies
**Problem:** Components might have circular dependencies when using dependency injection

**Solution:**
- Use dependency injection for runtime dependencies only
- Keep initialization dependencies one-way
- Use event/observer pattern for loose coupling where needed

### Challenge 2: Performance Impact
**Problem:** Parameter passing might add overhead to performance-critical paths

**Solution:**
- Use references instead of owned instances where possible
- Profile critical paths and optimize as needed
- Consider using context caching for frequently accessed components

### Challenge 3: Complex Initialization Order
**Problem:** Proper initialization order becomes more complex with dependency injection

**Solution:**
- Implement initialization in AudioSystemContext constructor
- Use builder pattern if initialization becomes too complex
- Add validation to ensure proper initialization order

### Challenge 4: Testing Complexity
**Problem:** Testing might become more complex with dependency injection

**Solution:**
- Create comprehensive test utilities and mocks
- Use dependency injection to make testing easier, not harder
- Implement test-specific context configurations

### Challenge 5: Backwards Compatibility
**Problem:** Breaking changes might affect other parts of the system

**Solution:**
- Use gradual migration strategy with bridge pattern
- Maintain API compatibility where possible
- Add deprecation warnings before removal

### Challenge 6: Optional vs Mandatory Setters
**Problem:** Current code treats setters as optional, but making them mandatory might break things

**Solution:**
- Analysis shows setters are always available at construction time
- Provide no-op setters for testing scenarios where data publishing isn't needed
- Make setters mandatory to ensure components are always properly configured

## Success Criteria

### Functional Requirements
- [x] All current functionality maintained
- [x] No regression in audio processing capabilities
- [x] All tests pass (including new tests)
- [x] Action pattern integration continues to work
- [x] UI controls continue to function properly

### Non-Functional Requirements
- [x] No performance regression in audio processing
- [x] Improved testability with dependency injection
- [x] Better encapsulation within audio module
- [x] Clearer component dependencies
- [x] Maintained real-time audio processing capabilities

### Code Quality Requirements
- [x] No global state in audio module (bridge pattern in place during migration)
- [x] Explicit dependency declarations
- [x] Improved unit test coverage
- [x] Better separation of concerns
- [x] Cleaner initialization patterns

## Implementation Timeline

### Phase 1: Foundation (Tasks 1-2)
- Estimated effort: 3-4 days
- Create AudioSystemContext and update function signatures
- Focus on maintaining functionality during transition

### Phase 2: Migration (Task 3)
- Estimated effort: 2-3 days
- Implement gradual migration strategy
- Ensure no functionality regression

### Phase 3: Cleanup (Task 4)
- Estimated effort: 1-2 days
- Remove global state infrastructure
- Clean up temporary migration code

### Total Estimated Effort: 6-9 days

## Post-Implementation Benefits

### Immediate Benefits
- Improved testability through dependency injection
- Better encapsulation within audio module
- Clearer component dependencies
- Reduced hidden dependencies

### Long-term Benefits
- Easier to add new audio components
- Better maintainability and debugging
- Improved code organization
- Foundation for future architectural improvements

### Performance Benefits
- Potential for better optimization through explicit dependencies
- More predictable initialization and lifecycle management
- Reduced global state complexity
- Better memory management patterns