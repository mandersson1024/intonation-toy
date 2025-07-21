# Implementation Plan: Three-Layer Architecture Refactoring

## Current State Analysis

The pitch-toy codebase currently has a **hybrid architecture** that mixes three-layer concepts with direct integration patterns:

### ✅ **Well-Implemented Components**
- **Engine Layer**: Complete audio processing system with dependency injection
- **Module Interfaces**: Defined but unused interface structs
- **Communication Systems**: Observable data and action patterns
- **Debug GUI**: Comprehensive egui-based debugging tools
- **Testing Infrastructure**: Comprehensive wasm-pack test suite

### ❌ **Problems to Address**
- **Empty Model Layer**: Only TODO comments, no implementation
- **Direct Coupling**: lib.rs directly manages all systems, bypassing architecture
- **Unused Interfaces**: Module interfaces defined but not integrated
- **Mixed Privileges**: Debug GUI has same access as main application
- **No Layer Update Sequence**: No organized three-layer update flow

### 🎯 **Target Architecture**
Refactor to match the implementation example in `docs/high-level-architecture.md`, with proper:
- Layer separation and interface usage
- Organized update sequence (engine → model → presenter)
- Debug GUI with read-only observation privileges
- Placeholder model and presenter implementations

---

## Implementation Tasks

### Task 1: Foundation - Interface Integration
**Priority**: Critical - Required for all subsequent tasks

- [x] **1a**: Create interface factory system
  - [x] Add methods to module interface structs to expose data sources and actions
  - [x] **Note**: These are Rust structs containing `DataSource<T>` and `Action<T>` objects, not abstract interfaces
  - [x] Add getter methods for extracting data setters (e.g., `audio_analysis_setter()` → `DataSetter`)
  - [x] Add getter methods for extracting data observers (e.g., `audio_analysis_observer()` → `DataObserver`)
  - [x] Add getter methods for extracting action triggers and listeners

- [x] **1b**: Update engine layer to use interfaces
  - [x] Modify `AudioEngine` to accept `EngineToModelInterface` and `ModelToEngineInterface`
  - [x] **Important**: Extract data setters from `EngineToModelInterface` (not Java-style interfaces - these are Rust structs containing `DataSource` objects)
  - [x] Route audio analysis data through extracted setters from `EngineToModelInterface`
  - [x] Listen for microphone permission requests using action listeners from `ModelToEngineInterface`
  - [x] Remove direct data setter dependencies from engine

- [x] **1c**: Test interface integration
  - [x] Create unit tests for interface data flow
  - [x] Test engine → model data propagation
  - [x] Test model → engine action handling
  - [x] Verify no breaking changes to existing audio functionality

**Dependencies**: None  
**Testing**: Run `./scripts/test-all.sh` after each subtask  
**Risk**: Low - Interfaces are already defined, just need integration

---

### Task 2: Model Layer Empty Shell Implementation
**Priority**: High - Core architecture component

- [x] **2a**: Create minimal DataModel struct
  - [x] Implement `DataModel::create()` method accepting required interfaces
  - [x] Store interfaces but don't use them (pure placeholder)
  - [x] Add `update(timestamp)` method that does nothing
  - [x] Return `Ok(())` for successful creation

- [x] **2b**: Add minimal compilation requirements
  - [x] Ensure struct compiles with interface parameters
  - [x] Add minimal error handling (return success always)
  - [x] Add placeholder documentation comments
  - [x] Ensure no runtime panics or crashes

- [x] **2c**: Basic model layer tests
  - [x] Test that DataModel::create() succeeds
  - [x] Test that update() method can be called without panicking
  - [x] Verify struct accepts required interfaces
  - [x] No functional testing - just compilation and basic runtime safety

**Dependencies**: Task 1 (Interface Integration)  
**Testing**: Basic compilation and runtime safety tests  
**Risk**: Low - Empty shell with no functionality

---

### Task 3: Presentation Layer Empty Shell Implementation
**Priority**: High - Core architecture component

- [x] **3a**: Create minimal Presenter struct
  - [x] Implement `Presenter::create()` method accepting required interfaces
  - [x] Store interfaces but don't use them (pure placeholder)
  - [x] Add `update(timestamp)` method that does nothing
  - [x] Add `render(&mut screen)` method that does nothing
  - [x] Return `Ok(())` for successful creation

- [x] **3b**: Add minimal compilation requirements
  - [x] Ensure struct compiles with interface parameters
  - [x] Add minimal error handling (return success always)
  - [x] Add placeholder documentation comments
  - [x] Ensure no runtime panics or crashes

- [x] **3c**: Basic presentation layer tests
  - [x] Test that Presenter::create() succeeds
  - [x] Test that update() and render() methods can be called without panicking
  - [x] Verify struct accepts required interfaces
  - [x] No functional testing - just compilation and basic runtime safety

**Dependencies**: Task 2 (Model Layer)  
**Testing**: Basic compilation and runtime safety tests  
**Risk**: Low - Empty shell with no functionality

---

### Task 4: Main Application Refactoring
**Priority**: Critical - Core architecture implementation

- [x] **4a**: Refactor lib.rs initialization
  - [x] Remove direct audio system initialization from `start()`
  - [x] Implement interface creation using factory functions
  - [x] Create engine, model, and presenter using new pattern
  - [x] Move audio system initialization into AudioEngine::create()

- [ ] **4b**: Implement new render loop structure
  - [ ] Refactor `run_three_d()` to accept layer instances
  - [ ] Implement three-layer update sequence (engine → model → presenter)
  - [ ] Keep existing SpriteScene rendering in render loop (presenter does nothing)
  - [ ] Maintain timestamp synchronization across layers

- [ ] **4c**: Update debug GUI integration
  - [ ] Modify debug GUI to accept interface observers only
  - [ ] Remove direct access to internal layer state
  - [ ] Ensure debug GUI works with new data flow
  - [ ] Maintain existing debug functionality

**Dependencies**: Tasks 1, 2, 3 (All layer implementations)  
**Testing**: Full integration testing + manual testing required  
**Risk**: High - Core application changes, potential breaking changes

---

### Task 5: Testing and Validation
**Priority**: High - Ensure refactoring success

- [ ] **5a**: Integration testing
  - [ ] Test that all layers can be created and called without panicking
  - [ ] Test that render loop calls all layer update methods
  - [ ] Test debug GUI observational access
  - [ ] Verify no functionality regression (existing functionality still works)

- [ ] **5b**: Performance validation
  - [ ] Test render loop performance (should match current)
  - [ ] Test audio processing latency (should match current)
  - [ ] Test memory usage (should not increase significantly)
  - [ ] Profile interface overhead

- [ ] **5c**: Manual testing requirements
  - [ ] Start development server manually and test basic functionality
  - [ ] Test microphone permission flow
  - [ ] Test debug console functionality
  - [ ] Test live data display in debug GUI
  - [ ] Test audio processing with real microphone input

**Dependencies**: Task 4 (Main Application Refactoring)  
**Testing**: Full test suite + manual testing checklist  
**Risk**: Low - Validation only

---

### Task 6: Cleanup and Documentation
**Priority**: Medium - Polish and maintainability

- [ ] **6a**: Remove obsolete code
  - [ ] Remove direct audio initialization from lib.rs
  - [ ] Remove bypass patterns and direct coupling
  - [ ] Clean up unused imports and dependencies
  - [ ] Remove global variable dependencies where possible

- [ ] **6b**: Update documentation
  - [ ] Update architecture documentation with actual implementation
  - [ ] Add code comments explaining interface usage
  - [ ] Update testing documentation if needed
  - [ ] Document placeholder implementations for future development

- [ ] **6c**: Code quality improvements
  - [ ] Run linting and fix any issues
  - [ ] Ensure consistent error handling patterns
  - [ ] Verify all tests pass
  - [ ] Check for dead code warnings

**Dependencies**: Task 5 (Testing and Validation)  
**Testing**: Lint checks + full test suite  
**Risk**: Low - Cleanup only

---

## Implementation Order and Dependencies

```
Task 1 (Interface Integration)
    ↓
Task 2 (Model Layer) + Task 3 (Presentation Layer) [Parallel]
    ↓
Task 4 (Main Application Refactoring)
    ↓
Task 5 (Testing and Validation)
    ↓
Task 6 (Cleanup and Documentation)
```

## Testing Strategy

1. **Unit Tests**: Test each layer in isolation using existing patterns
2. **Integration Tests**: Test interface data flow between layers
3. **Manual Testing**: Required for audio functionality and user interaction
4. **Regression Testing**: Ensure no existing functionality is broken
5. **Performance Testing**: Verify no performance degradation

## Potential Challenges and Solutions

### Challenge 1: Interface Data Access
**Problem**: Module interfaces may need getter methods for data sources and actions  
**Solution**: Add methods to interface structs to expose observable data and action triggers

### Challenge 2: Audio Engine Integration
**Problem**: Engine currently uses direct setters, needs to work with interfaces  
**Solution**: Modify engine to accept interface and route data through it instead of direct setters

### Challenge 3: Debug GUI Access
**Problem**: Debug GUI currently has privileged access to all systems  
**Solution**: Limit debug GUI to interface observers only, remove direct state access

### Challenge 4: Placeholder Implementations
**Problem**: Model and presenter layers need to compile and not crash but do nothing  
**Solution**: Implement empty shell structs that accept interfaces but don't use them, with clear TODO markers

### Challenge 5: Testing Audio Functionality
**Problem**: Audio processing requires manual testing with real microphone  
**Solution**: Provide specific manual testing checklist and preserve existing automated tests

## Success Criteria

✅ **Architecture Compliance**: Code matches the implementation example in docs/high-level-architecture.md  
✅ **No Functionality Loss**: All existing features continue to work  
✅ **Clean Interfaces**: All communication goes through defined interfaces  
✅ **Debug GUI Isolation**: Debug GUI has read-only observational access only  
✅ **Test Coverage**: All tests pass, no decrease in test coverage  
✅ **Placeholder Clarity**: Model and presenter placeholders are clearly marked as empty shells

## Estimated Implementation Time

- **Task 1**: 4-6 hours (interface integration)
- **Task 2**: 2-3 hours (model layer empty shell)
- **Task 3**: 2-3 hours (presentation layer empty shell)
- **Task 4**: 8-10 hours (main application refactoring)
- **Task 5**: 4-6 hours (testing and validation)
- **Task 6**: 2-4 hours (cleanup and documentation)

**Total**: 22-32 hours of development work

**Critical Path**: Interface Integration → Layer Implementation → Main Application Refactoring

This plan provides a systematic approach to refactoring the codebase into the three-layer architecture while preserving existing functionality and maintaining code quality.