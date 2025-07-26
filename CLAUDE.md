# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Coding guidelines
- Read docs/general_coding_guidelines.md
- Read docs/rust_coding_guidelines.md

## Project
- Never run the server automatically. Instead tell the user to start the server using `trunk serve` and ask for manual testing. Be specific about what to test and what you expect in response.
- Brower console logs are engouraged for tricky debugging scenarios. Add a distinct prefix to the relevant log lines so the user can filter on them
- For volume data, the internal representation is always amplitude, not dB

