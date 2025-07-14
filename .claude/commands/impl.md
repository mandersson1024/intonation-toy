---
allowed-tools: Read, Glob, Grep, LS, Edit, MultiEdit, Write, Bash, TodoWrite, Task, mcp__ide__getDiagnostics, mcp__ide__executeCode
description: Implement a plan systematically, marking planned items as done
---

Implement the plan for: $ARGUMENTS

This command will:
- Load and parse the existing implementation plan file
- Use TodoWrite to track progress through the planned tasks
- Implement each step systematically
- Mark items as completed as progress is made
- Run tests and linting to ensure quality
- Update the plan file with completion status

Please implement the plan for: $ARGUMENTS

Steps:
1. Find and load the relevant plan file from docs/implement_[feature-name].md
2. Convert plan items to TodoWrite format for tracking
3. Implement each task systematically
4. Update header comments with usage examples to all modules that have been touched
5. Mark tasks as completed in both TodoWrite and update the plan file with ✅ checkmarks
6. Run tests after each significant change using `wasm-pack test --node`
7. Ensure all linting passes before declaring completion
8. Save final status to the plan file in docs/