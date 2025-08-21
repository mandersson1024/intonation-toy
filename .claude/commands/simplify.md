---
  description: Reduce complexity and cognitive load in Rust codebases
  argument-hint: <file_path>
  allowed-tools: Read, Edit, MultiEdit, Glob, Grep, Bash(cargo:*), Bash(rustfmt:*), Bash(clippy:*)
---

# Rust Code Simplification

Systematically simplify the Rust codebase by eliminating unnecessary patterns, reducing lines of code, and making logic more direct.

Target file: $ARGUMENTS

## Analysis Phase
First analyze the target file to identify simplification opportunities:
- Find verbose error handling patterns (unnecessary match blocks, verbose unwraps)
- Identify deeply nested conditional logic
- Locate trivial single-use functions that should be inlined
- Find over-engineered generics that can be concrete types
- Detect duplicate code patterns that can be consolidated
- Identify unnecessary module hierarchies

## Simplification Transformations

Apply these transformations systematically:

### 1. Boilerplate Elimination
- Replace `match Ok(v) => v, Err(e) => return Err(e)` with `?` operator
- Convert verbose error handling to concise operators
- Simplify iterator chains and method calls

### 2. Nesting Reduction  
- Transform deeply nested if-let chains to early returns
- Flatten nested match expressions
- Convert complex conditionals to guard clauses

### 3. Function Inlining
- Inline functions that are only called once
- Remove single-line wrapper functions
- Eliminate trivial helper functions

### 4. Code Consolidation
  - Merge similar match arms using `|` patterns
  - Group duplicate implementations using macros
  - Extract shared logic from similar code paths

### 5. Over-Engineering Removal
- Replace unnecessary generics with concrete types
- Flatten module hierarchies that add no value
- Remove unused features and dead code paths
- Simplify ownership patterns (String to &str conversions)

## Quality Assurance
- Run `cargo check` to ensure compilation
- Run `cargo clippy` to verify no new warnings
- Preserve all existing functionality
- Maintain public API compatibility

## Metrics Reporting
Track and report:
- Lines of code reduced
- Functions inlined count
- Nested conditions simplified
- Modules flattened
- Overall complexity reduction percentage

**Core Principle**: Every change must reduce either lines of code, cognitive complexity, or unnecessary abstractions. Make no additions, only subtractions and simplifications.
