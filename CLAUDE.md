# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

This file is manually edited.

## House Rules
- When inserting today's date, use the `date` command in the terminal to check. Never guess or use a placeholder.
- Never implement changes unless explicitly instructed to do so. If you are uncertain, ask something like "Do you want me to implement these changes?"
- Never refer to Epics, Stories or Acceptance Criteria etc in comments or names.
- When refactoring, never refer to before/after, old/new, legacy/enhanced etc in comments or names.
- Never mention that "this was added", "this replaced that" or "that was deleted/removed" etc in comments.
- Never pretend that you can test something that requires manual testing. Always tell the user to test manually, be specific about what to test, and wait for confirmation.
- Never create unreferenced infrastructure for future tasks. It will only create compiler warnings, complicate code review. The roadmap might change before we get to use it anyway. Assume for now that you aren't gonna need it (YAGNI). Write TODO comments for expected future code. Use stubs as placeholders for incomplete but referenced implementations.
- Make placeholders obvious to the user. Always include the word "placeholder" in placeholder strings.
- Never say "You are absolutely right!" unless you have given it some thought and concluded that I in fact am. A lot of times I will be wrong, you see.
- Never document how many tests we have. This is most often not interesting and very much subject to change.
- Sometimes the developer will play devil's advocate and ask tricky questions in attempts to verify the solidity of your reasoning. Do not take this as sarcastic remarks to make you change the implementation, but take it as a challenge to think harder and find out the truth of the matter.
- Always explicitly distinguish between working functionality and placeholders. Never claim a task is "complete" or "fully functional" if it contains placeholders. Use clear ✅/❌ status indicators for each component to show what works vs what needs implementation.

## Testing
This project uses `wasm-pack test --node` for all testing. We do NOT use cargo test or any other testing approach. We do NOT test browser-specific functionality. Run `./scripts/test-all.sh` to test all packages. See TESTING.md for details.

## Project Information (obsolete, to be updated)
Refer to:
- docs/architecture.md
- docs/architecture/coding standards.md
- docs/architecture/source-tree.md
- docs/architecture/tech-stack.md
