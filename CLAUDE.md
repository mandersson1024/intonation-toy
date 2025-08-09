# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Coding guidelines
- IMPORTANT: Read docs/general_coding_guidelines.md
- IMPORTANT: Read docs/rust_coding_guidelines.md

## Building and testing
- The main language is Rust compiled to WASM, with some JavaScript for the audio worklet
- We only support moden browsers, with no fallback code for older browsers.
- Never run the server automatically. Instead tell the user to start the server using `trunk serve` and ask for manual testing. Be specific about what to test and what you expect in response.
- You are allowed to use `cargo build`, `cargo check` and `cargo clippy` to look for errors and warnings.
- Use console logs for tricky debugging scenarios. Add a distinct prefix to the relevant log lines so the user can filter on them. Remove those logs after the bug is fixed.
- For logging function, see intonation-toy/common/mod.rs

## The three-layer architecture

### Engine layer
- Responsible for AudioWorklet management and pitch detection

### Model layer
- Processes data coming from the engine that will later passed to the presentation layer
- Responsible for the tuning system and root note
- Responsible for pitch analysis, meaning the relationship of the detected pitch to the root note and the selected tuning system
- The pitch_detection crate returns a field called `clarity` in the `Pitch` struct. According to documentation "clarity is a measure of confidence in the pitch detection". We also use the term "clarity" in our app when refering to both clarity and confidence, which are interchangable terms when we speak informally about our app.
- `pitch_detection` external documentation: https://docs.rs/pitch-detection/latest/pitch_detection/

### Presentation layer
- Responsible for user input and visualization of data processed by the model
- The visualization is rendered using the three_d crate
- `three_d` external documentation: https://docs.rs/three-d/latest/three_d/index.html
- UI action from the presentation layers include:
  - set root note
  - set tuning system

### Debug layer
- Has priviliged access to data from all systems
- Conditionally compiled under cfg(debug_assertions)

## Stateless Design Principle

The application follows a stateless architecture where higher layers should receive fresh data each frame rather than caching data from lower layers:

- **Model layer**: Maintains only configuration state (tuning_system, root_note, current_scale). Receives fresh EngineUpdateResult data each frame and returns processed ModelUpdateResult data.
- **Presentation layer**: Should avoid caching model data. Methods should receive required configuration data as parameters rather than storing cached copies of model state. This ensures single source of truth and eliminates synchronization complexity.
- **Data flow**: Each layer processes fresh data from the layer below and passes results upward without caching intermediate state from other layers.

When modifying the presentation layer, prefer passing data as method parameters over storing cached state from the model layer. Note that HTML elements and UI components may need to maintain their own display state separately from model data.

## Functional requirements and general information
- The application is all about analyzing the relation of audio input to the root note; in musical terms - intonation
- We visualize the intonation by realtime graphic rendering to the screen
- The tuning systems we use are equal temperament and just intonation
- The root note is always a note in standard tuning and represents the tonic pitch, from which intervals are calculated
- Standard tuning means equal temperament where A4=440Hz
- The notes of just intonation are not fixed, but relative to the selected root note.
- For volume data, the internal representation is always amplitude, not dB.
- We don't adapt algorithms on the fly to adapt for performace. We always hardcode the parameters affecting performance
- The default sample rate should be 44100Hz. This is the most common standard on consumer devices

## Icon styling
- Icons should use CSS mask and background-color for proper theming
- Use var(--color-primary) or other CSS variables for icon colors
- Example CSS pattern for icon coloring:
  ```css
  .icon-class {
      -webkit-mask-repeat: no-repeat;
      -webkit-mask-position: center;
      -webkit-mask-size: contain;
      mask-repeat: no-repeat;
      mask-position: center;
      mask-size: contain;
      mask-type: alpha;
      mask: url("icons/icon-name.png") no-repeat center / contain;
      -webkit-mask: url("icons/icon-name.png") no-repeat center / contain;
      background-color: var(--color-primary);
  }
  ```
