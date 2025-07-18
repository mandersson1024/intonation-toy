# Global Variables Refactoring Guide

This document analyzes all global variables in the codebase, their current usage, and provides guidance for eventual refactoring while preventing increased dependency on them during ongoing development.

## Overview of Global Variables

The codebase currently has 4 global state variables that violate dependency injection principles:

1. **AUDIO_CONTEXT_MANAGER** - Thread-local global for audio context access
2. **PITCH_ANALYZER_GLOBAL** - Thread-local global for pitch analyzer access  
3. **MESSAGE_ID_GENERATOR** - Thread-local for generating unique message IDs
4. **COUNTER** - Unsafe static mut for message ID generation (appears to be dead code)

## Detailed Analysis

### 1. AUDIO_CONTEXT_MANAGER

**Location**: `pitch-toy/audio/mod.rs`

**Declaration**:
```rust
thread_local! {
    static AUDIO_CONTEXT_MANAGER: RefCell<Option<Rc<RefCell<context::AudioContextManager>>>> = RefCell::new(None);
}
```

**Current Usage**:
- Set during initialization in `lib.rs`
- Used by console commands to query audio status
- Used by `is_audio_system_ready()` function
- Used by `create_console_audio_service()`

**Why It Exists**:
- Provides backward compatibility during migration from global state
- Console commands need to access audio state without direct context reference
- Thread safety constraints with action listeners

**Refactoring Strategy**:
- Already documented in `docs/implement_remove_global_audio_context_manager.md`
- Replace with dependency injection through AudioSystemContext
- Use message passing for cross-thread communication

**How to Avoid Increasing Dependency**:
- ❌ NEVER call `get_audio_context_manager()` in new code
- ✅ Always pass AudioSystemContext as a parameter
- ✅ Use the existing AudioSystemContext methods instead
- ✅ For console commands, work toward passing context at registration time

### 2. PITCH_ANALYZER_GLOBAL

**Location**: `pitch-toy/audio/commands.rs`

**Declaration**:
```rust
thread_local! {
    static PITCH_ANALYZER_GLOBAL: RefCell<Option<Rc<RefCell<PitchAnalyzer>>>> = RefCell::new(None);
}
```

**Current Usage**:
- Set during initialization for backward compatibility
- Used exclusively by console commands:
  - `tuning` - Show/change tuning system
  - `pitch status` - Show pitch detection config
  - `pitch range` - Set frequency detection range  
  - `pitch benchmarks` - Run performance benchmarks

**Why It Exists**:
- Console commands need runtime access to pitch analyzer
- Allows configuration changes through console
- Performance monitoring and debugging

**Refactoring Strategy**:
- Pass PitchAnalyzer reference through console command registration
- Store reference in console command handler context
- Remove global setter/getter functions

**How to Avoid Increasing Dependency**:
- ❌ NEVER access PITCH_ANALYZER_GLOBAL from new code
- ✅ Access pitch analyzer through AudioSystemContext
- ✅ Pass analyzer reference explicitly to functions that need it
- ✅ For new console commands, plan for receiving context as parameter

### 3. MESSAGE_ID_GENERATOR

**Location**: `pitch-toy/audio/message_protocol.rs`

**Declaration**:
```rust
thread_local! {
    static MESSAGE_ID_GENERATOR: MessageIdGenerator = MessageIdGenerator::new();
}
```

**Current Usage**:
- Used by `generate_unique_message_id()` function
- Called when creating message envelopes for audio worklet communication
- Relatively isolated within message protocol module

**Why It Exists**:
- Ensures globally unique message IDs
- Thread-local to avoid synchronization overhead
- Simple, stateless counter

**Refactoring Strategy**:
- Move to AudioWorkletMessageFactory instance
- Pass ID generator as part of factory construction
- Remove global function, use factory methods

**How to Avoid Increasing Dependency**:
- ❌ Don't add new global ID generation functions
- ✅ Use AudioWorkletMessageFactory for message creation
- ✅ Pass ID generator through dependency injection
- ✅ Consider using UUID library for truly unique IDs

### 4. COUNTER (static mut)

**Location**: `pitch-toy/audio/message_protocol.rs` (inside `generate_message_id()` function)

**Declaration**:
```rust
static mut COUNTER: u32 = 0;
```

**Current Usage**:
- Inside `generate_message_id()` function
- Appears to be dead code (MESSAGE_ID_GENERATOR is used instead)
- Has comment indicating it should use atomics

**Why It Exists**:
- Quick implementation for unique IDs
- Likely leftover from earlier implementation

**Refactoring Strategy**:
- Delete immediately - it's unsafe and unused
- If needed, use atomic operations instead

**How to Avoid Increasing Dependency**:
- ❌ NEVER use static mut
- ❌ Don't resurrect this pattern
- ✅ Use atomics or thread-local storage if needed

## General Guidelines for Development

### DO NOT:
1. Add new global variables of any kind
2. Add new accessor functions for existing globals
3. Increase usage of existing global accessors
4. Use `static mut` under any circumstances
5. Create new thread_local storage for shared state

### DO:
1. Pass dependencies through function parameters
2. Use AudioSystemContext for audio-related needs
3. Design new features with dependency injection from the start
4. Use message passing for cross-thread communication
5. Question any code that reaches for global state

### When Working on Features:

**If you need audio context access**:
```rust
// ❌ BAD
let manager = get_audio_context_manager();

// ✅ GOOD  
fn my_function(context: &AudioSystemContext) {
    let manager = context.get_audio_context_manager();
}
```

**If you need unique IDs**:
```rust
// ❌ BAD
let id = generate_unique_message_id();

// ✅ GOOD
fn my_function(message_factory: &AudioWorkletMessageFactory) {
    let envelope = message_factory.create_envelope(...);
}
```

**If adding console commands**:
```rust
// ❌ BAD - Don't use globals
let analyzer = get_pitch_analyzer_global();

// ✅ GOOD - Plan for context parameter
// (Even if current architecture doesn't support it yet)
fn my_command(args: Vec<String>, context: &AudioSystemContext) {
    let analyzer = context.get_pitch_analyzer();
}
```

## Migration Priority

1. **HIGH**: Remove AUDIO_CONTEXT_MANAGER (plan exists)
2. **HIGH**: Delete unsafe COUNTER immediately
3. **MEDIUM**: Remove PITCH_ANALYZER_GLOBAL 
4. **LOW**: Refactor MESSAGE_ID_GENERATOR (least harmful)

## Conclusion

The goal is zero global state. Every global variable represents technical debt that makes the code harder to test, harder to reason about, and harder to refactor. By following these guidelines, we can prevent the problem from getting worse while working toward a clean, dependency-injected architecture.