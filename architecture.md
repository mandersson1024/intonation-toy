# Pitch-Toy Architecture Documentation

## Overview

Pitch-toy is built on a three-layer architecture designed for separation of concerns in real-time audio processing and visualization. The application analyzes microphone input to provide musical intonation feedback, supporting multiple tuning systems and scales.

## Three-Layer Architecture

### Engine Layer (`pitch-toy/engine/`)
**Responsibility**: Raw audio processing and hardware interface

The Engine layer handles low-level audio operations without any musical interpretation:

- **Audio Hardware Interface**: Manages microphone/speaker access and browser permissions
- **Raw Audio Processing**: Performs frequency analysis and pitch detection (returns Hz values only)
- **Browser API Integration**: Handles Web Audio API, MediaStream, and AudioWorklet operations
- **Data Provider**: Returns structured raw data via `EngineUpdateResult` for Model layer processing

**Key Components:**
- `AudioEngine`: Main engine orchestrator
- `AudioSystemContext`: Web Audio API management
- `PitchDetector`: Raw frequency detection using pitch-detection crate
- `VolumeDetector`: Audio amplitude analysis
- `AudioWorklet`: Real-time audio processing pipeline

**Data Flow Out**: `EngineUpdateResult` containing:
- Raw audio analysis (frequency in Hz, volume amplitude)
- Audio system errors and status
- Microphone permission state

### Model Layer (`pitch-toy/model/`)
**Responsibility**: Data transformation, business logic, and musical interpretation

The Model layer sits between raw audio data and visual presentation:

- **Musical Theory Processing**: Converts raw frequencies to musical notes and intervals
- **Tuning System Implementation**: Applies Equal Temperament, Just Intonation calculations
- **Business Logic Validation**: Validates user actions before execution
- **Configuration State**: Manages root note, tuning system, and scale settings (but **not** audio data)
- **Accuracy Calculations**: Computes intonation accuracy in cents

**Important**: The Model layer **does not cache audio data** from the Engine. It receives fresh `EngineUpdateResult` data each frame and processes it immediately, maintaining only configuration state (tuning system, root note, etc.).

**Key Components:**
- `DataModel`: Main model orchestrator
- `ProcessedActions`: User action validation system
- `ModelLayerActions`: Validated operations for engine execution
- Tuning system calculations and frequency-to-note conversion

**Data Flow**: 
- **In**: `EngineUpdateResult` from engine + `PresentationLayerActions` from presentation
- **Out**: `ModelUpdateResult` for presentation + `ModelLayerActions` for engine

### Presentation Layer (`pitch-toy/presentation/`)
**Responsibility**: Visualization, user interface, and interaction handling

The Presentation layer manages all visual and user interaction aspects:

- **Graphics Rendering**: 3D visualization using three_d crate
- **User Interface**: Both HTML DOM elements and EGUI-based debug panels
- **User Action Collection**: Gathers user inputs without direct system modification
- **Visual Feedback**: Real-time pitch visualization and tuning indicators
- **Scene Management**: Handles startup screen and main visualization scenes

**Important**: The Presentation layer **does not cache processed data** from the Model. It receives fresh `ModelUpdateResult` data each frame for rendering, maintaining only UI state (current scene, smoothing buffers, etc.).

**Key Components:**
- `Presenter`: Main presentation orchestrator
- `MainScene`: 3D graphics rendering with tuning lines
- `StartupScene`: Initial permission request interface
- HTML UI components for controls
- Action collection system (`PresentationLayerActions`)

## Communication Architecture

### Data Flow Pattern

The architecture uses a **return-based data flow** pattern rather than observer/callback interfaces, with a crucial **stateless principle**:

```
Engine Layer    Model Layer     Presentation Layer
     |              |                   |
     v              v                   v
[Raw Audio] -> [Musical Data] -> [Visual Display]
     |              |                   |
     |              |                   |
[Actions] <--- [Validation] <--- [User Input]
```

**Critical Design Principle**: Higher layers (Model and Presentation) **do not duplicate or cache data from lower layers**. Instead, they receive fresh, updated data every frame and use this as their primary data source. This ensures:

- **Single Source of Truth**: Each layer owns its data domain exclusively
- **Consistency**: No stale cached data or synchronization issues
- **Simplicity**: No complex state management between layers
- **Real-time Accuracy**: Every frame uses the most current data available

### Main Render Loop Communication

Located in `lib.rs`, the main render loop orchestrates all three layers:

1. **Three-Layer Update Sequence**:
   ```rust
   // Engine update (returns raw audio data)
   let engine_data = engine.update(timestamp);
   
   // Model update (processes engine data, returns processed data)
   let model_data = model.update(timestamp, engine_data);
   
   // Presentation update (renders using model data)
   presenter.process_data(timestamp, model_data);
   presenter.update_graphics(viewport);
   ```

2. **User Action Processing Flow**:
   ```rust
   // Collect actions from presentation layer
   let user_actions = presenter.get_user_actions();
   
   // Validate and process in model layer
   let processed_actions = model.process_user_actions(user_actions);
   
   // Execute validated actions in engine layer
   engine.execute_actions(processed_actions.actions);
   ```

3. **Debug Action Processing** (debug builds only):
   ```rust
   // Collect privileged debug actions
   let debug_actions = presenter.get_debug_actions();
   
   // Execute directly in engine (bypasses validation)
   engine.execute_debug_actions_sync(debug_actions);
   ```

### Inter-Layer Data Structures

**Shared Types** (`shared_types.rs`):
- `EngineUpdateResult`: Engine → Model data transfer
- `ModelUpdateResult`: Model → Presentation data transfer
- `PresentationLayerActions`: User action collection
- `ModelLayerActions`: Validated engine operations

**Action Processing System**:
- **User Actions**: Tuning system changes, root note adjustments, scale changes
- **Model Validation**: Business logic validation with detailed error reporting
- **Engine Execution**: Validated operations applied to audio system

## User Interface Integration

### HTML UI Components
- **Location**: `pitch-toy/web/main_scene_ui.rs`
- **Integration**: Direct DOM manipulation for control elements
- **Communication**: Event listeners trigger presentation layer action collection
- **Lifecycle**: Created/destroyed during scene transitions

**Components:**
- Root note increment/decrement buttons
- Tuning system dropdown selector
- Scale selection controls
- Real-time state synchronization with presenter

### Debug UI System
- **Location**: `pitch-toy/debug/debug_panel/`
- **Technology**: EGUI-based immediate mode interface
- **Availability**: Debug builds only (`#[cfg(debug_assertions)]`)
- **Privileges**: Direct engine access for testing purposes

**Debug Capabilities:**
- Audio device monitoring
- Test signal generation
- Speaker output control
- Performance metrics display
- Root note audio reference

### UI State Management

**Synchronization Flow**:
1. User interacts with HTML/EGUI controls
2. Event handlers call presenter action methods
3. Actions queued in `pending_user_actions` / `pending_debug_actions`
4. Main loop retrieves actions via `get_user_actions()` / `get_debug_actions()`
5. Model validates user actions; engine executes all actions
6. State changes flow back through update cycle
7. UI elements sync with new state

## Audio Processing Pipeline

### Browser Integration
- **Microphone Access**: Requires user gesture for permission
- **Web Audio API**: AudioContext, MediaStream, AudioWorklet integration
- **Real-time Processing**: AudioWorklet handles sample-rate audio processing
- **Performance**: Optimized for 44.1kHz standard sample rate

### Permission Handling
- **Startup Flow**: Permission request → scene transition
- **Error Handling**: Graceful degradation for denied/unavailable microphone
- **State Tracking**: Permission state propagated through all layers

### Audio Analysis Chain
1. **Raw Audio Capture**: Microphone → MediaStream → AudioWorklet
2. **Frequency Detection**: Pitch detection using autocorrelation/FFT
3. **Volume Analysis**: Peak and RMS amplitude calculation
4. **Musical Conversion**: Frequency → note identification → interval calculation
5. **Visualization**: Real-time tuning line display and accuracy feedback

## Architectural Strengths

### Separation of Concerns
- **Engine**: Hardware/browser interface, no musical knowledge
- **Model**: Musical theory and business logic, no UI concerns
- **Presentation**: Visualization and interaction, no audio processing

### Return-Based Data Flow
- **Predictable**: Explicit data parameters and return values
- **Testable**: Each layer can be tested independently
- **Maintainable**: Clear data dependencies and transformation points
- **Stateless**: Higher layers receive fresh data each frame rather than caching lower layer data

### Error Handling
- **Propagation**: Errors flow upward through layers for user feedback
- **Graceful Degradation**: Application continues with limited functionality
- **Debug Support**: Comprehensive error reporting in debug builds

### Performance Considerations
- **Real-time Processing**: AudioWorklet ensures consistent audio latency
- **EMA Smoothing**: Visual smoothing for stable pitch display
- **Efficient Updates**: Return-based pattern minimizes callback overhead

## Potential Architecture Issues

### State Synchronization Complexity
- **Issue**: Multiple UI systems (HTML + EGUI) require careful state management
- **Mitigation**: Centralized state in presenter with explicit sync points

### Debug vs Release Feature Divergence
- **Issue**: Debug-only features create different code paths
- **Current Status**: Well-isolated with `#[cfg(debug_assertions)]` guards

### Browser API Dependencies
- **Issue**: Heavy reliance on Web Audio API and browser-specific features
- **Mitigation**: Platform abstraction layer and graceful fallbacks

### Memory Management
- **Issue**: Real-time audio processing requires careful memory handling
- **Current Status**: Uses buffer pooling and efficient data structures

## Future Architecture Considerations

### Extensibility Points
- **Additional Tuning Systems**: Just Intonation framework ready for expansion
- **New Scales**: Scale system supports arbitrary interval patterns
- **Enhanced Visualizations**: Three_d rendering system supports complex graphics

### Performance Optimizations
- **Worker Threads**: Consider moving more processing to AudioWorklet
- **Buffering Strategies**: Optimize for different audio device configurations
- **GPU Acceleration**: Potential for three_d graphics optimizations

### Platform Expansion
- **Desktop Support**: Architecture supports non-browser environments
- **Mobile Optimization**: Touch-friendly UI adaptations
- **Audio Plugin**: Potential for VST/AU plugin architecture

## Conclusion

The three-layer architecture successfully separates audio processing, musical logic, and visualization concerns while maintaining efficient real-time performance. The return-based data flow pattern provides clear, testable interfaces between layers. The dual UI system (HTML + EGUI) effectively serves both end-user and developer needs, though it requires careful state management. Overall, the architecture provides a solid foundation for real-time musical analysis applications.