# Background Shader Proof-of-Concept Implementation Plan

## Goal
Create a minimal working implementation of a custom shader background with texture buffer support.

## Phase 1: Basic Custom Shader (No Texture Buffer)

### Step 1: Create shader module
- Add new file: `intonation-toy/src/presentation/background_shader.rs`
- Define vertex and fragment shader strings
- Create a struct to hold shader program and state

### Step 2: Simple animated shader
- Fragment shader: Simple gradient that changes with time
- Vertex shader: Fullscreen quad vertices
- Pass time uniform to animate colors

### Step 3: Integration
- Replace current background quad rendering in `renderer.rs`
- Test that custom shader renders and animates

## Phase 2: Add Texture Buffer

### Step 1: Create texture buffer
- 512x1 texture (512 frames of history)
- RGBA format for storing 4 values per frame

### Step 2: Update texture per frame
- Store current pitch and clarity in R and G channels
- Use frame counter modulo 512 for pixel position
- Update single pixel per frame

### Step 3: Use texture in shader
- Sample texture in fragment shader
- Visualize historical data (e.g., trailing pitch visualization)

## Phase 3: Visual Effect

### Step 1: Implement simple visualization
- Read texture buffer in fragment shader
- Display last 512 pixels as horizontal gradient
- Map stored pitch value to color hue
- Map stored clarity to brightness

## Success Criteria
- [ ] Custom shader renders fullscreen quad
- [ ] Time-based animation works
- [ ] Texture buffer updates per frame
- [ ] Historical pitch data visible in background
- [ ] No performance regression

## Files to Modify
1. Create: `intonation-toy/src/presentation/background_shader.rs`
2. Modify: `intonation-toy/src/presentation/renderer.rs`
3. Modify: `intonation-toy/src/presentation/mod.rs` (add module)

## Test Approach
Manual testing with console logs to verify:
- Shader compilation success
- Uniform updates per frame
- Texture pixel updates
- Visual output matches expectations