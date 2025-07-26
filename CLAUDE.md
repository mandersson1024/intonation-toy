# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Coding guidelines
- Read docs/general_coding_guidelines.md
- Read docs/rust_coding_guidelines.md

## Project

### Building and testing
- Never run the server automatically. Instead tell the user to start the server using `trunk serve` and ask for manual testing. Be specific about what to test and what you expect in response.
- You are allowed to use `cargo build` and `cargo check` to look for errors and warnings.
- Browsers console logs are engouraged for tricky debugging scenarios. Add a distinct prefix to the relevant log lines so the user can filter on them.

### Engine layer
- Responsible for AudioWorklet management and pitch detection

### Model layer
- Responsible for the tuning system and root note
- The root note is always a note in standard tuning and represents the tonic pitch, from which intervals are calculated

### Presentation layer
- Responsible for user input and visualization of data processed by the model

### Debug layer
- Has priviliged access to data from all systems
- Conditionally compiled under cfg(debug_assertions)

### General
- For volume data, the internal representation is always amplitude, not dB.

