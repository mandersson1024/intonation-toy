---
  allowed-tools: Read, Glob, Grep, LS, WebFetch, WebSearch, Task, Write
  description: Create an implementation plan with TODO list saved as a file - DO NOT IMPLEMENT
  ---

  Create a detailed implementation plan for: $ARGUMENTS

  This command will:
  - Research the current codebase to understand existing patterns
  - Analyze the requirements and constraints
  - Create a step-by-step implementation plan
  - Save the plan as a TODO list file in the project
  - STOP - Do NOT implement any code

  Please create an implementation plan for: $ARGUMENTS

  Research the codebase first, then create a detailed plan with:
  1. Current state analysis
  2. Implementation broken down into major tasks with subtasks
  3. Dependencies and order of operations
  4. Testing considerations for each task
  5. Potential challenges and solutions

  Structure the plan with:
  - Main tasks (numbered: Task 1, Task 2, etc.)
  - Subtasks under each main task (lettered: 1a, 1b, 1c, etc.)
  - Each subtask should be independently implementable
  - Use TODO checklist format with `- [ ]` for all tasks and subtasks

  Save the plan as a markdown file in the docs/ directory.
  Use filename format: docs/implement_[feature-name].md

  IMPORTANT: This is a planning-only command. Do not implement any code or use tools like Edit, MultiEdit, or exit_plan_mode.
