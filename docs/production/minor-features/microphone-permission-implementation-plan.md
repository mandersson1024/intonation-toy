# Microphone Permission Implementation Plan

## Current State Analysis

### ✅ Existing Infrastructure:
- Error handling for microphone permissions (`MicrophonePermission` category)
- MediaStream handling in `AudioEngineComponent` 
- Browser compatibility detection (`getUserMedia` support)
- Permission error types and recovery strategies already defined

### ❌ Missing Components:
- No actual `getUserMedia()` implementation
- Missing microphone permission UI component  
- No permission state management

---

## Implementation Tasks

### 1. Create MicrophonePermissionComponent
**File:** `src/components/microphone_permission.rs`

**Responsibilities:**
1. Request microphone permission via `getUserMedia()`
2. Handle permission states (pending, granted, denied, unsupported)
3. Display appropriate UI for each state
4. Provide callback with MediaStream on success

**Key Features:**
- Permission status display
- Retry mechanism for denied permissions
- Browser compatibility messaging  
- Loading states during permission request

### 2. Create MicrophonePermissionHook
**File:** `src/hooks/use_microphone_permission.rs`

**Hook Interface:**
```rust
pub fn use_microphone_permission() -> (
    PermissionState,           // Current state
    Option<MediaStream>,       // Stream if granted
    Callback<()>,             // Request permission
    Option<ApplicationError>, // Last error
)
```

**States:**
- `NotRequested` - Initial state
- `Requesting` - Permission dialog shown
- `Granted(MediaStream)` - Success with stream
- `Denied` - User denied permission
- `Unsupported` - Browser doesn't support getUserMedia

### 3. Integrate with AudioControlPanel
**File:** `src/components/audio_control_panel.rs`

**Changes:**
- Add microphone permission button
- Show permission status
- Pass MediaStream to AudioEngine only after permission granted
- Handle permission errors gracefully

### 4. Update Main Application Flow
**File:** `src/main.rs` (if exists) or main component

**Flow:**
1. App starts → Check if permission needed
2. Show permission UI → Request on user action
3. Handle result → Pass MediaStream to AudioEngine
4. Continue with audio processing

---

## Technical Implementation Details

### getUserMedia() Integration:
```rust
// In the hook/component
let request_permission = Callback::from(move |_| {
    wasm_bindgen_futures::spawn_local(async move {
        let navigator = web_sys::window().unwrap().navigator();
        let media_devices = navigator.media_devices().unwrap();
        
        let mut constraints = MediaStreamConstraints::new();
        constraints.audio(&true.into());
        
        match media_devices.get_user_media_with_constraints(&constraints) {
            Ok(promise) => {
                // Handle promise resolution
            }
            Err(e) => {
                // Handle error with existing error manager
            }
        }
    });
});
```

### Error Integration:
- Use existing `ApplicationError::microphone_permission_denied()`
- Leverage existing error recovery strategies
- Display errors via existing error toast system

---

## Component Hierarchy:
```
App
├── MicrophonePermissionComponent
│   ├── Permission UI (request/status)
│   └── Error handling
├── AudioControlPanel (updated)
│   ├── Permission status display
│   └── Audio controls (enabled after permission)
└── AudioEngineComponent
    └── Receives MediaStream from permission
```

---

## Testing Strategy:
1. **Unit Tests:** Permission state transitions
2. **Integration Tests:** Permission + AudioEngine flow  
3. **Manual Tests:** Different browsers, permission scenarios
4. **Error Tests:** Denied permissions, unsupported browsers

---

## Priority Order:
1. **High:** `use_microphone_permission` hook (core logic)
2. **High:** `MicrophonePermissionComponent` (UI)
3. **Medium:** Update `AudioControlPanel` integration
4. **Medium:** Add tests
5. **Low:** Polish UI/UX

---

## Next Steps:
Ready to implement? Start with the `use_microphone_permission` hook as it provides the core functionality that other components will depend on.

## Development Notes:
- Existing error handling infrastructure can be leveraged
- MediaStream integration points already exist in AudioEngineComponent
- Browser compatibility detection is already implemented
- Permission error categories and recovery strategies are defined 