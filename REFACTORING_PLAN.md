# Audio Architecture Refactoring Plan

## Current Problem
The `AudioWorkletManager` has grown beyond its name and manages far more than just the AudioWorklet:
- AudioWorklet communication and control
- Audio routing/mixing (gain nodes, microphone, speakers)
- Audio analysis (volume detector, pitch analyzer)
- Audio generation (test signals, tuning fork)
- State management for various audio settings

## Proposed Architecture

### Core Components

1. **`AudioSignalFlow`** (already exists in `signal_flow.rs`)
   - Defines and creates the complete Web Audio API node graph
   - Handles all node connections declaratively
   - Provides clean separation between graph structure and management logic

2. **`AudioPipeline`** (new - replaces current `AudioWorkletManager` role)
   - Orchestrates the entire audio system
   - Creates `AudioSignalFlow` with all connected nodes
   - Creates and coordinates specialized managers
   - Provides high-level API for the engine layer

3. **`AudioWorkletManager`** (refactored - focused scope)
   - **Only** manages AudioWorklet-specific operations:
     - Worklet message handling and communication
     - Processing control (start/stop)
     - Worklet state management
     - Message protocol and buffer management

4. **`AudioAnalyzer`** (new)
   - Manages audio analysis using the analyser node from signal flow
   - Handles volume detection
   - Handles pitch detection
   - Provides analysis results to the pipeline

5. **`AudioGenerator`** (new)
   - Manages audio generation nodes (test signals, tuning fork)
   - Controls oscillator parameters and state
   - Handles enabling/disabling of generated audio

6. **`AudioRouter`** (new)
   - Manages audio routing and mixing
   - Controls gain levels for different sources
   - Handles speaker output routing
   - Manages microphone input processing

## Implementation Plan

### Phase 1: Create AudioPipeline ✅ COMPLETED
- [x] Create new `AudioPipeline` struct
- [x] Integrate existing `AudioSignalFlow` 
- [x] Move high-level orchestration from current `AudioWorkletManager`
- [x] Update `engine/mod.rs` to use `AudioPipeline` instead of `AudioWorkletManager`

### Phase 2: Extract AudioWorkletManager ✅ COMPLETED  
- [x] Create focused `AudioWorkletManager` with only worklet-specific functionality:
  - Worklet node reference (from signal flow)
  - Message handling and communication
  - Processing control
  - Worklet state management
- [x] Remove all non-worklet functionality from current implementation

### Phase 3: Create AudioAnalyzer
- [ ] Extract volume detection logic
- [ ] Extract pitch detection logic  
- [ ] Use analyser node from `AudioSignalFlow`
- [ ] Provide clean interface for analysis results

### Phase 4: Create AudioGenerator
- [ ] Extract test signal management
- [ ] Extract tuning fork management
- [ ] Use oscillator nodes from `AudioSignalFlow`
- [ ] Handle generation state and parameters

### Phase 5: Create AudioRouter
- [ ] Extract gain control logic
- [ ] Extract speaker routing logic
- [ ] Extract microphone processing
- [ ] Use gain nodes from `AudioSignalFlow`

### Phase 6: Integration and Testing
- [ ] Ensure all components work together through `AudioPipeline`
- [ ] Update all calling code to use new architecture
- [ ] Test that all functionality still works
- [ ] Clean up any remaining coupling

## Benefits of This Architecture

1. **Single Responsibility**: Each component has a clear, focused purpose
2. **Declarative Signal Flow**: Audio graph structure is clearly defined in one place
3. **Testability**: Each component can be tested independently
4. **Maintainability**: Changes to one concern don't affect others
5. **Extensibility**: New audio features can be added as focused components
6. **Clear Dependencies**: Signal flow provides nodes, managers handle behavior

## File Structure
```
audio/
├── signal_flow.rs          (existing - audio graph definition)
├── pipeline.rs             (new - orchestration)
├── worklet_manager.rs      (refactored - worklet only)
├── analyzer.rs             (new - audio analysis)  
├── generator.rs            (new - audio generation)
├── router.rs               (new - routing/mixing)
└── mod.rs                  (updated exports)
```

## Migration Strategy
- Refactor incrementally to avoid breaking changes
- Keep tests passing at each phase
- Use feature flags if needed for gradual rollout
- Maintain backward compatibility in `engine/mod.rs` during transition