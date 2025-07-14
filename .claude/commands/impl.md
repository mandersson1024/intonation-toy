---
allowed-tools: Read, Glob, Grep, LS, Edit, MultiEdit, Write, Bash, TodoWrite, Task, mcp__ide__getDiagnostics, mcp__ide__executeCode
description: Implement a plan systematically, marking planned items as done
---

Implement the plan for: $ARGUMENTS

This command will:
- Load and parse the existing implementation plan file
- Allow selection of specific tasks to implement
- Use TodoWrite to track progress through the selected task and subtasks
- Implement each subtask systematically
- Mark items as completed as progress is made
- Run tests and linting to ensure quality
- Update the plan file with completion status

Please implement the plan for: $ARGUMENTS

If $ARGUMENTS contains "task N" or "Task N", implement only that specific task.
Otherwise, ask user to specify which task to implement.

Steps:
1. Find and load the relevant plan file from docs/implement_[feature-name].md
2. Parse the plan to identify available tasks
3. If specific task not specified, show available tasks and ask user to choose
4. Convert selected task and its subtasks to TodoWrite format for tracking
5. Implement each subtask systematically
6. Update header comments with usage examples to all modules that have been touched
7. Mark subtasks as completed in both TodoWrite and update the plan file with ✅ checkmarks
8. Run tests after each significant change using `wasm-pack test --node`
9. Ensure all linting passes before declaring completion
10. Save final status to the plan file in docs/