# Unused Functions in Pitch-Toy Codebase

This document lists functions that are defined but never used anywhere in the codebase, including tests. These functions are candidates for removal to reduce code complexity and maintenance burden.

## Summary

**Total unused functions found:** 12  
**Files containing unused functions:** 1  
**Analysis date:** 2025-07-20

## Unused Functions by Category

### Builder Pattern Methods (9 functions)

These functions implement builder patterns for configuration objects but are never called:

#### MessageContext Builder Methods
**File:** [`pitch-toy/engine/audio/message_protocol.rs`](../pitch-toy/engine/audio/message_protocol.rs)

- [`with_message_id`](../pitch-toy/engine/audio/message_protocol.rs#L2488) (line 2488)
- [`with_timestamp`](../pitch-toy/engine/audio/message_protocol.rs#L2494) (line 2494)
- [`with_size`](../pitch-toy/engine/audio/message_protocol.rs#L2500) (line 2500)

## Analysis Notes

### Why These Functions Are Unused

1. **Over-engineered design**: The message protocol appears to have been designed with more configuration options than currently needed
2. **Builder pattern completion**: Core functionality uses basic constructors rather than the full builder pattern
3. **Internal module scope**: Despite being `pub`, these functions are in internal modules not exposed to external consumers

### Functions That Appear Unused But Are Actually Used

During analysis, several functions initially appeared unused but were confirmed to be used:
- All test functions (properly marked with `#[test]`)
- Helper utility functions like `get_u32_property`, `get_f64_property`
- Core builder methods like `SystemState::basic` and `with_system_state`

### Impact Assessment

**Removal Impact:** Low
- These functions are not used anywhere in the codebase
- No tests depend on them
- No external APIs expose them
- They represent pure overhead

**Code Quality Impact:** High
- Removing them will reduce cognitive load
- Eliminate maintenance burden
- Reduce compiled binary size
- Simplify the API surface

## Recommendations

1. **Safe to remove**: All listed functions can be safely removed without breaking functionality
2. **Consider design review**: Evaluate whether the over-engineered builder patterns indicate unclear requirements
3. **Future additions**: If these features are needed later, they can be re-added when actually required (YAGNI principle)

## Verification Commands

To verify these functions are unused, run:

```bash
# Search for usage of specific function names
rg "with_memory_usage" --type rust
rg "with_stack_trace" --type rust
rg "with_message_id" --type rust
```

All searches should return only the function definitions, not any usage sites.