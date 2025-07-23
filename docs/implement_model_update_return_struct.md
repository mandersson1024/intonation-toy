# Implementation Plan: Replace Observable Data with Update Return Struct

## Overview
This plan outlines the steps to replace the current observable_data pattern for engine-to-model communication with a new pattern where model.update() returns a struct that becomes an argument for the next model.update() call.

## Current State Analysis

### Current Architecture
- **Communication Pattern**: Engine pushes data to `DataSource` objects via setters, Model pulls data via observers
- **Interface**: `EngineToModelInterface` contains three `DataSource` instances:
  - `audio_analysis_source: DataSource<Option<AudioAnalysis>>`
  - `audio_errors_source: DataSource<Vec<AudioError>>`
  - `permission_state_source: DataSource<PermissionState>`
- **Update Flow**: 
  1. Engine.update() pushes data to setters
  2. Model.update() could pull data from observers (currently placeholder)
  3. Each update() takes only a timestamp parameter

### Proposed Architecture
- **Communication Pattern**: Engine.update() returns data, which is passed to Model.update() as a parameter
- **Update Flow**:
  1. Engine.update() returns `EngineUpdateResult` struct
  2. Model.update() takes `EngineUpdateResult` as parameter and returns `ModelUpdateResult`
  3. The main loop orchestrates passing data between layers

## Implementation Plan

### Task 1: Define Update Result Structures
Define the data structures that will be returned from update methods and passed as arguments.

- [✅] 1a. Create `EngineUpdateResult` struct in `module-interfaces/engine_to_model.rs`
  - Include fields for audio_analysis, audio_errors, and permission_state
  - Use the same data types as currently defined (AudioAnalysis, AudioError, PermissionState)
  
- [✅] 1b. Create `ModelUpdateResult` struct in `module-interfaces/model_to_presentation.rs`
  - Include fields for volume, pitch, accuracy, tuning_system, errors, permission_state
  - Use existing data types from the interface

- [✅] 1c. Update module interface exports to include the new result structs
  - Export from respective interface modules
  - Ensure visibility in main module interfaces

### Task 2: Update Engine Layer
Modify the engine to return data instead of pushing to observable_data.

- [✅] 2a. Change engine update signature
  - From: `pub fn update(&mut self, _timestamp: f64)`
  - To: `pub fn update(&mut self, timestamp: f64) -> EngineUpdateResult`

- [✅] 2b. Remove observable_data dependencies from engine
  - Remove `EngineToModelInterface` from Engine struct
  - Remove setter extraction in Engine::new()
  - Remove all calls to setters in update logic

- [✅] 2c. Modify engine update implementation to collect and return data
  - Collect audio analysis data during update
  - Collect audio errors during update
  - Collect permission state
  - Return EngineUpdateResult with all collected data

- [✅] 2d. Update audio context and adapters
  - Remove adapter connections to setters
  - Modify adapters to return data instead of pushing to setters
  - Update AudioAnalysisMerger to return merged data

### Task 3: Update Model Layer
Modify the model to accept engine data as parameter and return presentation data.

- [ ] 3a. Change model update signature
  - From: `pub fn update(&mut self, _timestamp: f64)`
  - To: `pub fn update(&mut self, timestamp: f64, engine_data: EngineUpdateResult) -> ModelUpdateResult`

- [ ] 3b. Remove observable_data dependencies from model
  - Remove `EngineToModelInterface` from Model struct
  - Remove `ModelToPresentationInterface` from Model struct
  - Remove observer/setter extraction in Model::new()

- [ ] 3c. Implement model update logic
  - Process engine_data parameter
  - Transform audio analysis to presentation data
  - Calculate pitch, volume, accuracy etc.
  - Return ModelUpdateResult with calculated data

### Task 4: Update Presentation Layer
Modify the presentation to accept model data as parameter.

- [ ] 4a. Change presentation update signature
  - From: `pub fn update(&mut self, _timestamp: f64)`
  - To: `pub fn update(&mut self, timestamp: f64, model_data: ModelUpdateResult)`

- [ ] 4b. Remove observable_data dependencies from presentation
  - Remove `ModelToPresentationInterface` from Presentation struct
  - Remove observer extraction in Presentation::new()

- [ ] 4c. Update presentation to use model_data parameter
  - Use model_data instead of pulling from observers
  - Update UI elements with data from parameter

### Task 5: Update Main Loop
Modify the main render loop to pass data between layers.

- [ ] 5a. Update the render loop in lib.rs
  - Capture return value from engine.update()
  - Pass engine result to model.update()
  - Capture return value from model.update()
  - Pass model result to presentation.update()

- [ ] 5b. Handle optional model layer
  - When model is None, skip model update
  - Provide default/empty data to presentation when model is disabled

### Task 6: Update Factory and Initialization
Remove observable_data from factory methods and initialization.

- [ ] 6a. Update factory create methods
  - Remove EngineToModelInterface creation
  - Remove ModelToPresentationInterface creation
  - Remove Rc wrapping and sharing of interfaces

- [ ] 6b. Update layer constructors
  - Remove interface parameters from Engine::new()
  - Remove interface parameters from Model::new()
  - Remove interface parameters from Presentation::new()

### Task 7: Clean Up Observable Data Dependencies
Remove the observable_data crate and related code.

- [ ] 7a. Remove observable_data from Cargo.toml dependencies
  - Remove from workspace Cargo.toml
  - Remove from pitch-toy/Cargo.toml

- [ ] 7b. Delete the observable-data crate directory
  - Delete the entire observable-data directory

- [ ] 7c. Remove module interface files that are no longer needed
  - Keep data type definitions
  - Remove DataSource-related methods
  - Update imports in interface modules

### Task 8: Update Debug Layer
Adapt debug layer to work without observable_data.

- [ ] 8a. Update HybridLiveData structure
  - Change from holding DataObservers to holding actual data
  - Add update method that takes engine and model results

- [ ] 8b. Update debug overlay
  - Pass current data state instead of observers
  - Update rendering to use passed data

- [ ] 8c. Update debug panel updates
  - Modify to receive data through update calls
  - Remove observer.get() calls

### Task 9: Testing and Validation
Ensure all functionality works with the new pattern.

- [ ] 9a. Run existing tests
  - Execute ./scripts/test-all.sh
  - Fix any failing tests

- [ ] 9b. Manual testing checklist
  - Audio permission request flow
  - Audio analysis data flow (volume/pitch)
  - Error propagation
  - Debug panel data display
  - Performance monitoring

- [ ] 9c. Performance validation
  - Compare memory usage before/after
  - Check for any performance regressions
  - Verify no memory leaks from removed Rc cycles

## Dependencies and Order of Operations

1. **Tasks 1-2** can be done in parallel (define structs and update engine)
2. **Task 3** depends on Task 1 (needs EngineUpdateResult defined)
3. **Task 4** depends on Task 1 (needs ModelUpdateResult defined)
4. **Task 5** depends on Tasks 2, 3, and 4 (needs all signatures updated)
5. **Task 6** depends on Tasks 2, 3, and 4 (needs to know what parameters to remove)
6. **Task 7** depends on all previous tasks (final cleanup)
7. **Task 8** can start after Task 1 but needs Tasks 2-5 complete to finish
8. **Task 9** must be done last

## Potential Challenges and Solutions

### Challenge 1: Maintaining Data Freshness
**Problem**: With observable pattern, data is always fresh when read. With parameter passing, data might be one frame old.
**Solution**: This is acceptable for this architecture. The data passed is from the current frame's engine update.

### Challenge 2: Optional Model Layer
**Problem**: When model is disabled, presentation still needs data.
**Solution**: Provide a default/passthrough mechanism where engine data is minimally transformed for presentation.

### Challenge 3: Debug Layer Data Access
**Problem**: Debug layer currently pulls data on-demand via observers.
**Solution**: Push data to debug layer during update cycle, similar to other layers.

### Challenge 4: Testing Adapters
**Problem**: Many adapters are designed around the observable pattern.
**Solution**: Refactor adapters to be simple data transformers that return values.

### Challenge 5: Backwards Compatibility
**Problem**: This is a breaking change to the architecture.
**Solution**: As per CLAUDE.md, breaking changes are encouraged. Make the change cleanly without compatibility layers.

## Success Criteria

- [ ] All tests pass (./scripts/test-all.sh)
- [ ] No references to observable_data remain in the codebase
- [ ] Data flows from Engine → Model → Presentation via return values and parameters
- [ ] Debug panel displays all data correctly
- [ ] No memory leaks or performance regressions
- [ ] Clean, simple update loop without observer complexity