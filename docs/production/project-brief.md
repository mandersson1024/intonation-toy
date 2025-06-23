# Project Brief: Real-time Pitch Visualizer

## Executive Summary

The Real-time Pitch Visualizer is an educational music application that provides live audio pitch detection with compelling visual feedback and musical interval analysis. Targeting musicians and music students (particularly children), the tool transforms traditional pitch practice into an engaging, playful experience while maintaining professional-grade accuracy. The web-based application addresses the gap in existing tuning tools by providing interval feedback and child-friendly interaction, making it both an effective learning tool and an entertaining musical toy.

## Problem Statement

Current pitch training tools fall into two categories: technical tuners that show only basic pitch accuracy without educational context, or complex music software that overwhelms beginners. Musicians and students, especially children, struggle with:

- **Limited interval awareness**: Most tuners only show whether you're sharp or flat, not how your pitch relates to musical intervals
- **Boring, clinical interfaces**: Existing tools lack engaging visual feedback that would motivate practice
- **Educational disconnect**: Tools focus on technical accuracy without connecting to musical understanding
- **Child accessibility**: Few tools are designed to be intuitive and fun for young learners

The impact is that students avoid pitch practice, develop poor intonation habits, and miss opportunities to build fundamental musical skills through engaging play. With no compelling solution that combines accuracy, education, and entertainment, there's a clear market gap for an interval-aware, visually engaging pitch training tool.

## Proposed Solution

The Real-time Pitch Visualizer combines high-accuracy pitch detection with educational interval feedback and engaging visual design. Key differentiators include:

- **Interval Intelligence**: Beyond basic tuning, shows musical relationships (major thirds, fifths, etc.) relative to user-selected reference pitches
- **Dual UX Themes**: "Children" mode for playful learning and "Nerds" mode for detailed technical feedback
- **Real-time Performance**: 60 FPS graphics with perceptually tight audio-visual synchronization
- **Educational Focus**: Designed specifically for learning, not just measurement
- **Child-Friendly Design**: Built to be fun enough that a 7-year-old wants to play with it

This solution succeeds where others fail by treating pitch training as both skill development and play, making it sustainable for long-term learning.

## Target Users

### Primary User Segment: Young Musicians & Students

**Demographics**: Children ages 6-16 learning instruments requiring intonation (voice, strings, winds), their parents/teachers seeking engaging practice tools

**Current Behaviors**: Practice with traditional tuners or teacher feedback, often avoiding pitch work due to boredom or frustration

**Specific Needs**: 
- Immediate, clear feedback on pitch accuracy
- Understanding of musical context (intervals, not just "right/wrong")
- Engaging interface that makes practice enjoyable
- Progress visibility to maintain motivation

**Goals**: Develop good intonation habits, understand musical relationships, enjoy practice time

### Secondary User Segment: Adult Musicians & Educators

**Demographics**: Music teachers, adult learners, casual musicians wanting to improve intonation

**Current Behaviors**: Use professional tuners, metronomes, or DAW tools for pitch reference

**Specific Needs**:
- Professional-grade accuracy with educational value
- Tools that enhance teaching effectiveness
- Flexibility for different musical contexts and references

**Goals**: Improve personal playing, enhance student instruction, streamline practice routines

## Goals & Success Metrics

### Business Objectives

- **Personal Achievement**: Create a tool that the developer and their 7-year-old actively choose to use for musical practice
- **Technical Portfolio**: Build impressive open-source project demonstrating advanced real-time audio processing capabilities
- **Community Impact**: Provide valuable educational tool to music learning community

### User Success Metrics

- **Engagement**: Users return to the tool regularly for practice (not just one-time use)
- **Learning Outcomes**: Users demonstrate improved pitch accuracy over time
- **Enjoyment Factor**: Children find the tool fun and choose to use it voluntarily

### Key Performance Indicators (KPIs)

- **Primary KPI**: Personal usage by developer + child for actual music practice (sustained over 3+ months)
- **Technical KPI**: Maintains <50ms audio latency with 60 FPS visual performance
- **Community KPI**: GitHub stars/forks indicating developer community interest

## MVP Scope

### Core Features (Must Have)

- **Live Audio Input**: Low-latency microphone processing with pitch detection accuracy
- **Reference Pitch Selection**: User-configurable reference (note names, frequencies)
- **Tuning System Selection**: Choice between standard 12-TET (12-tone equal temperament) and just intonation
- **Interval Calculation**: Display of musical intervals relative to reference pitch using selected tuning system
- **Basic Visual Feedback**: Numerical display showing pitch deviation in cents
- **Headphone Audio Output**: Plays reference pitch through headphones to prevent feedback
- **Web Interface**: Clean, responsive web-based interface accessible across devices

### Out of Scope for MVP

- Complex visual animations or graphics beyond basic numerical feedback
- Multiple simultaneous pitch detection (polyphonic input)
- Recording/playback functionality
- Network features or sharing capabilities
- Mobile app versions (iOS/Android native apps)
- Advanced DSP effects or audio processing

### MVP Success Criteria

- Accurate pitch detection within ±5 cents for single-note input
- Visual updates at 60 FPS with perceptually synchronized audio
- Stable performance across modern web browsers with Web Audio API support
- Intuitive enough for a 7-year-old to use independently
- Developer and child use it for actual practice sessions

## Post-MVP Vision

### Phase 2 Features

- **Enhanced Visuals**: Compelling graphics, animations, and child-friendly themes

## Technical Considerations

### Platform Requirements

- **Target Platforms**: Modern web browsers supporting Web Audio API (Chrome, Firefox, Safari, Edge)
- **Audio Requirements**: Web Audio API integration, support for microphone input via getUserMedia
- **Performance Requirements**: <50ms audio latency (web constraints), 60 FPS graphics rendering, real-time pitch detection

### Technology Preferences

- **Frontend**: Rust compiled to WebAssembly (WASM) with web framework integration
- **Audio Processing**: Rust DSP algorithms compiled to WASM, interfacing with Web Audio API
- **Graphics**: Canvas API or WebGL for visual rendering, potentially driven by Rust/WASM
- **Architecture**: Modular design with separate audio, DSP, and rendering modules
- **Development Tools**: Cursor (VSCode), Rust toolchain, wasm-pack, web dev tools

### Architecture Considerations

- **Repository Structure**: Single repository with clear separation of audio processing/DSP/UI modules
- **Service Architecture**: Client-side web application with modular internal architecture
- **Real-time Constraints**: Web Audio API scheduling and worklet-based processing for optimal performance
- **Audio Pipeline**: Microphone → Web Audio API → Analysis → Visual Display pipeline with minimal buffering

## Constraints & Assumptions

### Constraints

- **Platform**: Modern web browsers with WebAssembly support only (Chrome 69+, Firefox 76+, Safari 14.1+, Edge 79+)
- **WASM Requirement**: WebAssembly support is mandatory - no JavaScript fallbacks provided
- **Timeline**: MVP-first approach with iterative visual development
- **Resources**: Single developer project with occasional child user testing
- **Audio Environment**: Assumes relatively quiet practice environment
- **Browser Security**: Requires HTTPS for microphone access, user permission prompts

### Key Assumptions

- Users have access to modern web browsers with microphone permissions
- Built-in microphones or USB audio interfaces provide sufficient quality for web audio
- Musicians understand basic interval concepts or are willing to learn
- Visual feedback can be iterated based on user testing and preferences
- Users will accept browser permission prompts for microphone access

## Risks & Open Questions

### Key Risks

- **Browser Latency**: Web Audio API may introduce higher latency than native applications (~50ms vs ~20ms)
- **User Adoption**: Tool might be too niche or complex for target age group
- **Audio Feedback**: Self-reference from speakers could interfere with pitch detection even with headphone design
- **Browser Compatibility**: Different browsers may have varying Web Audio API performance and support
- **Microphone Permissions**: Users may deny microphone access, blocking core functionality

### Open Questions

- What visual metaphors will be most effective for interval feedback?
- How to balance professional accuracy with child-friendly simplicity?
- Should the tool include any gamification elements in MVP?
- How should pitch deviation be displayed in just intonation mode? (cents from just target vs. cents from 12-TET equivalent vs. alternative units)

### Areas Needing Further Research

- Web Audio API pitch detection performance across different browsers
- Child user interface design patterns for web-based music education
- Browser microphone access UX patterns for educational applications
- Web-based audio feedback cancellation techniques

## Appendices

### A. Research Summary

**Competitive Analysis**: Existing tuners focus on basic pitch accuracy without interval awareness or engaging visuals. Sonofield app approaches interval feedback but lacks child-friendly design.

**Technical Feasibility**: Real-time pitch detection is well-established, with Web Audio API providing sufficient tools for browser-based development with acceptable latency constraints.

### B. Stakeholder Input

**Primary Stakeholder** (Developer): Wants impressive technical achievement with personal utility for family music practice.

**End User** (7-year-old): Needs fun, immediate feedback that feels like play rather than work.

### C. References

- Web Audio API documentation for browser-based audio processing
- Music education research on interval training effectiveness  
- Existing web-based pitch detection implementations and browser performance benchmarks 