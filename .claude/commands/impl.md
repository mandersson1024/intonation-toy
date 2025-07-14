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
1. Find and load the relevant plan file
2. Convert plan items to TodoWrite format for tracking
3. Implement each task systematically
4. Mark tasks as completed in both TodoWrite and the plan file
5. Run tests after each significant change
6. Ensure all linting passes before declaring completion