# Implementation Plan: Audio Console Decoupling

## Overview
This document outlines the implementation strategy for decoupling audio dependencies from the console module while respecting browser constraints and maintaining real-time functionality.

## Current Architecture Problems
- Console module directly imports audio module internals
- Audio permissions, device lists, and state managed within console component
- Tight coupling through global audio context manager access
- Commands hard-coded to specific audio implementation

## Target Architecture
Hybrid approach combining:
1. **Interface-based delegation** for permission handling (respects browser gesture requirements)
2. **Event-driven updates** for real-time data (device lists, status)
3. **Command registry pattern** for extensible audio commands (already partially implemented)

## Implementation Plan

### Phase 1: Create Audio Service Interface

**Goal**: Establish clean contract between console and audio modules

**Tasks**:
1. Create `src/audio/console_service.rs` with `ConsoleAudioService` trait
2. Define minimal interface required by console:
   ```rust
   pub trait ConsoleAudioService {
       fn request_permissions(&self) -> Result<()>;
       fn subscribe_device_changes(&self, callback: Box<dyn Fn(Vec<AudioDevice>)>);
       fn subscribe_permission_changes(&self, callback: Box<dyn Fn(AudioPermission)>);
       fn get_current_status(&self) -> AudioStatus;
   }
   ```
3. Implement trait in audio module: `ConsoleAudioServiceImpl`
4. Register service instance with console module

**Files to modify**:
- `src/audio/mod.rs` - Export new service interface
- `src/audio/console_service.rs` - New file with trait and implementation
- `src/console/mod.rs` - Accept service during initialization

### Phase 2: Implement Event System

**Goal**: Enable real-time updates without tight coupling

**Tasks**:
1. Create event system in `src/events/` directory:
   - `audio_events.rs` - Define `AudioEvent` enum
   - `event_dispatcher.rs` - Generic event dispatcher
   - `mod.rs` - Export event system
2. Implement audio event publishing:
   - Device list changes
   - Permission state changes
   - Context state changes
3. Add subscription mechanism to console component

**New files**:
- `src/events/mod.rs`
- `src/events/audio_events.rs`
- `src/events/event_dispatcher.rs`

**Files to modify**:
- `src/audio/permission.rs` - Emit permission events
- `src/audio/context.rs` - Emit context state events
- `src/console/component.rs` - Subscribe to events

### Phase 3: Refactor Console Component

**Goal**: Remove direct audio imports and use service interface

**Tasks**:
1. Remove direct audio imports from `src/console/component.rs`
2. Accept `ConsoleAudioService` as constructor parameter
3. Replace direct audio calls with service interface calls
4. Replace local audio state with event-driven state updates
5. Keep permission button but delegate to service

**Files to modify**:
- `src/console/component.rs` - Major refactor to use service interface
- `src/main.rs` - Wire up service dependency injection

### Phase 4: Refactor Audio Commands

**Goal**: Decouple console commands from audio internals

**Tasks**:
1. Create `src/audio/commands.rs` with audio command implementations
2. Move audio-specific commands out of `src/console_commands.rs`
3. Register audio commands externally using existing command registry
4. Commands use service interface instead of direct audio access

**Files to modify**:
- `src/console_commands.rs` - Remove audio commands
- `src/audio/commands.rs` - New file with audio commands
- `src/audio/mod.rs` - Export and register commands

### Phase 5: Testing and Validation

**Goal**: Ensure functionality maintained and architecture improved

**Tasks**:
1. Create unit tests for service interface
2. Create integration tests for event system
3. Test permission flow with browser gesture requirements
4. Validate real-time updates still work correctly
5. Performance testing for event system overhead

**New files**:
- `src/audio/console_service_tests.rs`
- `src/events/event_dispatcher_tests.rs`
- `tests/integration/audio_console_integration.rs`

## Implementation Details

### Service Interface Implementation

```rust
// src/audio/console_service.rs
pub trait ConsoleAudioService {
    fn request_permissions(&self) -> Result<()>;
    fn subscribe_device_changes(&self, callback: Box<dyn Fn(Vec<AudioDevice>)>);
    fn subscribe_permission_changes(&self, callback: Box<dyn Fn(AudioPermission)>);
    fn get_current_status(&self) -> AudioStatus;
}

pub struct ConsoleAudioServiceImpl {
    permission_manager: Arc<PermissionManager>,
    audio_context_manager: Arc<AudioContextManager>,
    event_dispatcher: Arc<EventDispatcher>,
}

impl ConsoleAudioService for ConsoleAudioServiceImpl {
    fn request_permissions(&self) -> Result<()> {
        // Delegate to permission manager, which emits events
        self.permission_manager.request_permissions()
    }
    
    fn subscribe_device_changes(&self, callback: Box<dyn Fn(Vec<AudioDevice>)>) {
        self.event_dispatcher.subscribe(AudioEvent::DeviceListChanged, callback);
    }
    
    // ... other methods
}
```

### Event System Implementation

```rust
// src/events/audio_events.rs
#[derive(Debug, Clone)]
pub enum AudioEvent {
    DeviceListChanged(Vec<AudioDevice>),
    PermissionChanged(AudioPermission),
    ContextStateChanged(AudioContextState),
}

// src/events/event_dispatcher.rs
pub struct EventDispatcher {
    subscribers: HashMap<String, Vec<Box<dyn Fn(AudioEvent)>>>,
}

impl EventDispatcher {
    pub fn subscribe<F>(&mut self, event_type: &str, callback: F)
    where
        F: Fn(AudioEvent) + 'static,
    {
        self.subscribers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }
    
    pub fn publish(&self, event: AudioEvent) {
        // Notify all subscribers
    }
}
```

### Console Component Refactor

```rust
// src/console/component.rs
pub struct DevConsole {
    // Remove direct audio fields
    // audio_permission: AudioPermission,
    // device_list: Vec<AudioDevice>,
    
    // Add service interface
    audio_service: Box<dyn ConsoleAudioService>,
    
    // Add event-driven state
    current_devices: Vec<AudioDevice>,
    current_permission: AudioPermission,
}

impl DevConsole {
    pub fn new(audio_service: Box<dyn ConsoleAudioService>) -> Self {
        let mut console = Self {
            audio_service,
            current_devices: Vec::new(),
            current_permission: AudioPermission::NotRequested,
        };
        
        // Subscribe to events
        console.setup_event_subscriptions();
        console
    }
    
    fn setup_event_subscriptions(&mut self) {
        // Subscribe to device changes
        self.audio_service.subscribe_device_changes(Box::new(|devices| {
            // Update UI state
        }));
        
        // Subscribe to permission changes
        self.audio_service.subscribe_permission_changes(Box::new(|permission| {
            // Update UI state
        }));
    }
    
    fn on_permission_button_click(&self) {
        // Delegate to service (maintains user gesture context)
        self.audio_service.request_permissions();
    }
}
```

## Migration Strategy

1. **Phase 1 & 2**: Can be implemented in parallel (service interface + event system)
2. **Phase 3**: Requires Phase 1 completion
3. **Phase 4**: Requires Phase 1 completion  
4. **Phase 5**: Continuous throughout other phases

## Risk Mitigation

- **Browser Gesture Requirements**: Maintain button click delegation pattern
- **Real-time Performance**: Benchmark event system overhead
- **State Synchronization**: Ensure single source of truth for audio state
- **Testing Complexity**: Mock service interface for isolated console tests

## Success Criteria

- [ ] Console module has no direct imports from audio module
- [ ] Permission requests still work with browser gesture requirements
- [ ] Real-time device list updates maintain current responsiveness
- [ ] Audio commands registered externally through existing command registry
- [ ] All existing functionality preserved
- [ ] Unit tests pass for both modules independently
- [ ] Integration tests validate cross-module communication

## Future Enhancements

- Extend event system to other modules (video, network)
- Plugin architecture for console commands
- WebSocket/external event integration
- Performance monitoring for event system