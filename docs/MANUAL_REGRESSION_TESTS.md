# Manual Regression Test Protocol

## Overview
This document provides a structured approach for manually testing the intonation-toy application to ensure core functionality works correctly after changes.

## Prerequisites
- Start the development server: `trunk serve`
- Use a modern browser (Chrome/Firefox)
- Have a microphone available for audio input testing
- Test in a quiet environment for accurate pitch detection

## Test Categories

### 1. Application Startup & Platform Validation
**Goal**: Verify the application initializes correctly across different environments.

#### Basic Startup
**Steps:**
1. Navigate to application URL
2. Wait for loading to complete

**Expected Result:**
- Application loads without errors
- UI elements are visible
- No error messages in console

#### Browser Compatibility
**Steps:**
1. Test in Chrome
2. Test in Firefox
3. Check developer console for errors

**Expected Result:**
- Application works in both browsers
- No critical API missing errors
- Platform validation passes

#### Microphone Permission
**Steps:**
1. Click anywhere on the application (first user gesture)
2. Accept microphone permission when prompted

**Expected Result:**
- Permission dialog appears
- Permission granted successfully
- Audio input indicator shows activity

### 2. Tuning System Functionality
**Goal**: Verify both Equal Temperament and Just Intonation work correctly.

#### Equal Temperament Mode
**Steps:**
1. Ensure ET mode is selected
2. Play/sing a known pitch (e.g., A4=440Hz)
3. Observe pitch detection and visualization

**Expected Result:**
- Pitch detected accurately
- Visualization updates in real-time
- Cents offset shown relative to ET intervals

#### Just Intonation Mode
**Steps:**
1. Switch to Just Intonation mode
2. Play/sing the same pitch
3. Compare results with ET mode

**Expected Result:**
- Mode switch successful
- Different intonation analysis than ET
- Visualization reflects JI intervals

#### Tuning System Toggle
**Steps:**
1. Switch between ET and JI multiple times
2. Verify each switch takes effect immediately

**Expected Result:**
- No lag in switching
- Analysis updates immediately
- UI reflects current mode

### 3. Tonal Center Configuration
**Goal**: Verify tonal center changes affect pitch analysis correctly.

#### Tonal Center Selection
**Steps:**
1. Change tonal center (e.g., from C to D)
2. Play a consistent pitch
3. Observe interval analysis changes

**Expected Result:**
- Tonal center updates immediately
- Interval calculations adjust to new tonal center
- Visual representation updates

#### Standard Tuning Reference
**Steps:**
1. Set tonal center to A (MIDI 69)
2. Play A4=440Hz
3. Verify it shows as tonal center/unison

**Expected Result:**
- Shows 0 semitones, 0 cents
- Pitch detection shows ~440Hz
- Visualization centers on root

### 4. Scale Awareness
**Goal**: Verify scale-aware features work correctly.

#### Major Scale
**Steps:**
1. Set scale to Major
2. Play notes in and out of the major scale
3. Observe how non-scale notes are handled

**Expected Result:**
- Scale notes show accurate intervals
- Non-scale notes map to nearest scale notes
- Cents offset shows deviation from scale note

#### Chromatic Scale
**Steps:**
1. Switch to Chromatic scale
2. Play various chromatic pitches
3. Compare with Major scale behavior

**Expected Result:**
- All 12 semitones treated equally
- No note mapping occurs
- Direct interval analysis for all notes

### 5. Audio Input & Pitch Detection
**Goal**: Verify pitch detection accuracy and responsiveness.

#### Pitch Detection Accuracy
**Steps:**
1. Use a tuner app or known audio source
2. Play sustained notes at different pitches
3. Compare detected pitch with reference

**Expected Result:**
- Pitch detection within ±5 cents of reference
- Stable readings for sustained notes
- Quick response to pitch changes

#### Volume Sensitivity
**Steps:**
1. Play notes at different volumes
2. Test very quiet and loud inputs
3. Verify detection threshold

**Expected Result:**
- Detects reasonable volume range
- No false positives from noise
- Graceful handling of too-quiet input

#### Clarity/Confidence
**Steps:**
1. Play pure tones vs. complex timbres
2. Observe clarity measurements
3. Test with noisy environment

**Expected Result:**
- Pure tones show high clarity
- Complex timbres show appropriate clarity
- Noise reduces clarity appropriately

### 6. Visual Representation
**Goal**: Verify real-time visualization works correctly.

#### Real-time Updates
**Steps:**
1. Play sustained notes
2. Observe visualization updates
3. Change pitch gradually

**Expected Result:**
- Smooth, real-time updates
- No lag or stuttering
- Visual elements track pitch changes

#### Cents Visualization
**Steps:**
1. Play slightly sharp/flat notes
2. Observe cents offset display
3. Test both positive and negative offsets

**Expected Result:**
- Accurate cents display
- Visual indication of sharp/flat
- Smooth transitions between values

### 7. Debug Features (Debug Builds Only)
**Goal**: Verify debug panel functionality in development builds.

#### Debug Panel
**Steps:**
1. Verify debug panel is visible
2. Check performance metrics display
3. Verify audio system information

**Expected Result:**
- Panel shows relevant debug info
- FPS counter updates
- Memory usage displayed
- Audio device info shown

#### Test Signal
**Steps:**
1. Enable test signal generation
2. Configure different waveforms
3. Verify signal replaces microphone input

**Expected Result:**
- Test signal generates correctly
- Signal parameters adjustable
- Can switch back to microphone

### 8. Performance & Stability
**Goal**: Verify application maintains performance under extended use.

#### Extended Use
**Steps:**
1. Run application for 10+ minutes
2. Continuously provide audio input
3. Monitor performance and memory

**Expected Result:**
- No performance degradation
- Memory usage remains stable
- No crashes or errors

#### Rapid Input Changes
**Steps:**
1. Rapidly change between different pitches
2. Switch settings frequently
3. Observe system response

**Expected Result:**
- Handles rapid changes smoothly
- No lag accumulation
- System remains responsive

## Pass/Fail Criteria

### Critical Issues (Must Fix Before Release)
- Application fails to start
- Microphone permission fails
- Pitch detection completely inaccurate (>20 cents error)
- Application crashes during normal use
- Major visual rendering issues

### Minor Issues (Should Fix)
- Small pitch detection inaccuracies (5-20 cents)
- UI responsiveness issues
- Debug panel display problems
- Non-critical console warnings

### Enhancement Opportunities
- Minor visual improvements
- Performance optimizations
- Additional debug information

## Test Environment Notes
- Test on both localhost development and production builds
- Verify different browser versions when possible
- Test with various microphone types/qualities
- Consider testing on different devices if web-deployed

## Regression Test Execution
1. **Pre-testing**: Ensure `trunk serve` is running and application loads
2. **Execute tests**: Go through each category systematically
3. **Document issues**: Note any failures with steps to reproduce
4. **Re-test fixes**: Verify fixes don't break other functionality
5. **Sign-off**: Confirm all critical issues resolved before deployment

---
*This protocol should be executed after any significant code changes, especially those affecting the engine, model, or presentation layers.*