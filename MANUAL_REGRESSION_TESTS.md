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

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| **Basic Startup** | 1. Navigate to application URL<br>2. Wait for loading to complete | - Application loads without errors<br>- UI elements are visible<br>- No error messages in console |
| **Browser Compatibility** | 1. Test in Chrome<br>2. Test in Firefox<br>3. Check developer console for errors | - Application works in both browsers<br>- No critical API missing errors<br>- Platform validation passes |
| **Microphone Permission** | 1. Click anywhere on the application (first user gesture)<br>2. Accept microphone permission when prompted | - Permission dialog appears<br>- Permission granted successfully<br>- Audio input indicator shows activity |

### 2. Tuning System Functionality
**Goal**: Verify both Equal Temperament and Just Intonation work correctly.

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| **Equal Temperament Mode** | 1. Ensure ET mode is selected<br>2. Play/sing a known pitch (e.g., A4=440Hz)<br>3. Observe pitch detection and visualization | - Pitch detected accurately<br>- Visualization updates in real-time<br>- Cents offset shown relative to ET intervals |
| **Just Intonation Mode** | 1. Switch to Just Intonation mode<br>2. Play/sing the same pitch<br>3. Compare results with ET mode | - Mode switch successful<br>- Different intonation analysis than ET<br>- Visualization reflects JI intervals |
| **Tuning System Toggle** | 1. Switch between ET and JI multiple times<br>2. Verify each switch takes effect immediately | - No lag in switching<br>- Analysis updates immediately<br>- UI reflects current mode |

### 3. Root Note Configuration
**Goal**: Verify root note changes affect pitch analysis correctly.

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| **Root Note Selection** | 1. Change root note (e.g., from C to D)<br>2. Play a consistent pitch<br>3. Observe interval analysis changes | - Root note updates immediately<br>- Interval calculations adjust to new root<br>- Visual representation updates |
| **Standard Tuning Reference** | 1. Set root note to A (MIDI 69)<br>2. Play A4=440Hz<br>3. Verify it shows as root/unison | - Shows 0 semitones, 0 cents<br>- Pitch detection shows ~440Hz<br>- Visualization centers on root |

### 4. Scale Awareness
**Goal**: Verify scale-aware features work correctly.

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| **Major Scale** | 1. Set scale to Major<br>2. Play notes in and out of the major scale<br>3. Observe how non-scale notes are handled | - Scale notes show accurate intervals<br>- Non-scale notes map to nearest scale notes<br>- Cents offset shows deviation from scale note |
| **Chromatic Scale** | 1. Switch to Chromatic scale<br>2. Play various chromatic pitches<br>3. Compare with Major scale behavior | - All 12 semitones treated equally<br>- No note mapping occurs<br>- Direct interval analysis for all notes |

### 5. Audio Input & Pitch Detection
**Goal**: Verify pitch detection accuracy and responsiveness.

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| **Pitch Detection Accuracy** | 1. Use a tuner app or known audio source<br>2. Play sustained notes at different pitches<br>3. Compare detected pitch with reference | - Pitch detection within ±5 cents of reference<br>- Stable readings for sustained notes<br>- Quick response to pitch changes |
| **Volume Sensitivity** | 1. Play notes at different volumes<br>2. Test very quiet and loud inputs<br>3. Verify detection threshold | - Detects reasonable volume range<br>- No false positives from noise<br>- Graceful handling of too-quiet input |
| **Clarity/Confidence** | 1. Play pure tones vs. complex timbres<br>2. Observe clarity measurements<br>3. Test with noisy environment | - Pure tones show high clarity<br>- Complex timbres show appropriate clarity<br>- Noise reduces clarity appropriately |

### 6. Visual Representation
**Goal**: Verify real-time visualization works correctly.

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| **Real-time Updates** | 1. Play sustained notes<br>2. Observe visualization updates<br>3. Change pitch gradually | - Smooth, real-time updates<br>- No lag or stuttering<br>- Visual elements track pitch changes |
| **Cents Visualization** | 1. Play slightly sharp/flat notes<br>2. Observe cents offset display<br>3. Test both positive and negative offsets | - Accurate cents display<br>- Visual indication of sharp/flat<br>- Smooth transitions between values |

### 7. Debug Features (Debug Builds Only)
**Goal**: Verify debug panel functionality in development builds.

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| **Debug Panel** | 1. Verify debug panel is visible<br>2. Check performance metrics display<br>3. Verify audio system information | - Panel shows relevant debug info<br>- FPS counter updates<br>- Memory usage displayed<br>- Audio device info shown |
| **Test Signal** | 1. Enable test signal generation<br>2. Configure different waveforms<br>3. Verify signal replaces microphone input | - Test signal generates correctly<br>- Signal parameters adjustable<br>- Can switch back to microphone |

### 8. Performance & Stability
**Goal**: Verify application maintains performance under extended use.

| Test Case | Steps | Expected Result |
|-----------|-------|-----------------|
| **Extended Use** | 1. Run application for 10+ minutes<br>2. Continuously provide audio input<br>3. Monitor performance and memory | - No performance degradation<br>- Memory usage remains stable<br>- No crashes or errors |
| **Rapid Input Changes** | 1. Rapidly change between different pitches<br>2. Switch settings frequently<br>3. Observe system response | - Handles rapid changes smoothly<br>- No lag accumulation<br>- System remains responsive |

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