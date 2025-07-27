# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Coding guidelines
- Read docs/general_coding_guidelines.md
- Read docs/rust_coding_guidelines.md

## Building and testing
- The main language is Rust compiled to WASM, with some JavaScript for the audio worklet
- We only support moden browsers, with no fallback code for older browsers.
- Never run the server automatically. Instead tell the user to start the server using `trunk serve` and ask for manual testing. Be specific about what to test and what you expect in response.
- You are allowed to use `cargo build`, `cargo check` and `cargo clippy` to look for errors and warnings.
- Use console logs for tricky debugging scenarios. Add a distinct prefix to the relevant log lines so the user can filter on them. Remove those logs after the bug is fixed.

## The three-layer architecture

### Engine layer
- Responsible for AudioWorklet management and pitch detection

### Model layer
- Processes data coming from the engine that will later passed to the presentation layer
- Responsible for the tuning system and root note
- Responsible for pitch analysis, meaning the relationship of the detected pitch to the root note and the selected tuning system

### Presentation layer
- Responsible for user input and visualization of data processed by the model
- The visualization is rendered using the three_d crate

### Debug layer
- Has priviliged access to data from all systems
- Conditionally compiled under cfg(debug_assertions)

## Functional requirements and general information
- The application is all about analyzing the relation of audio input to the root note; in musical terms - intonation
- We visualize the intonation by realtime graphic rendering to the screen
- The tuning systems we use are equal temperament and just intonation
- The root note is always a note in standard tuning and represents the tonic pitch, from which intervals are calculated
- Standard tuning means equal temperament where A4=440Hz
- The notes of just intonation are not fixed, but relative to the selected root note.
- For volume data, the internal representation is always amplitude, not dB.
- We don't adapt algorithms on the fly to adapt for performace. We always hardcode the parameters affecting performance
