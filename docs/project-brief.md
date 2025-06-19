# Project Brief: Real-time Pitch Visualizer

## Executive Summary

The Real-time Pitch Visualizer is an educational music application that provides live audio pitch detection with compelling visual feedback and musical interval analysis. Targeting musicians and music students (particularly children), the tool transforms traditional pitch practice into an engaging, playful experience while maintaining professional-grade accuracy. The Mac-native application addresses the gap in existing tuning tools by providing interval feedback and child-friendly interaction, making it both an effective learning tool and an entertaining musical toy.

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
- **Technical KPI**: Maintains <20ms audio latency with 60 FPS visual performance
- **Community KPI**: GitHub stars/forks indicating developer community interest

## MVP Scope

### Core Features (Must Have)

- **Live Audio Input**: Low-latency microphone processing with pitch detection accuracy
- **Reference Pitch Selection**: User-configurable reference (note names, frequencies)
- **Tuning System Selection**: Choice between standard 12-TET (12-tone equal temperament) and just intonation
- **Interval Calculation**: Display of musical intervals relative to reference pitch using selected tuning system
- **Basic Visual Feedback**: Numerical display showing pitch deviation in cents
- **Headphone Audio Output**: Plays reference pitch through headphones to prevent feedback
- **Mac Native GUI**: Clean, responsive interface with modular architecture

### Out of Scope for MVP

- Complex visual animations or graphics beyond basic numerical feedback
- Multiple simultaneous pitch detection (polyphonic input)
- Recording/playback functionality
- Network features or sharing capabilities
- Windows/Linux support
- Advanced DSP effects or audio processing

### MVP Success Criteria

- Accurate pitch detection within ±5 cents for single-note input
- Visual updates at 60 FPS with perceptually synchronized audio
- Stable performance on Mac hardware with standard audio interfaces
- Intuitive enough for a 7-year-old to use independently
- Developer and child use it for actual practice sessions

## Post-MVP Vision

### Phase 2 Features

- **Enhanced Visuals**: Compelling graphics, animations, and child-friendly themes

## Technical Considerations

### Platform Requirements

- **Target Platforms**: macOS native application (10.15+ minimum)
- **Audio Requirements**: Core Audio integration, support for built-in and USB audio interfaces
- **Performance Requirements**: <20ms audio latency, 60 FPS graphics rendering, real-time pitch detection

### Technology Preferences

- **Frontend**: GUI (framework to be determined with architect; open to native menu bar vs. fully custom rendered UI based on technical practicality)
- **Backend/Audio Processing**: Rust for core audio processing and DSP
- **Architecture**: Modular design with separate audio, DSP, and rendering layers
- **Development Tools**: Cursor (VSCode), Rust toolchain

### Architecture Considerations

- **Repository Structure**: Single repository with clear separation of audio/DSP/UI modules
- **Service Architecture**: Monolithic native app with modular internal architecture
- **Real-time Constraints**: Careful threading for audio processing separate from UI
- **Audio Pipeline**: Input → Analysis → Display pipeline with minimal buffering

## Constraints & Assumptions

### Constraints

- **Platform**: Mac-only to maintain manageable scope
- **Timeline**: MVP-first approach with iterative visual development
- **Resources**: Single developer project with occasional child user testing
- **Audio Environment**: Assumes relatively quiet practice environment

### Key Assumptions

- Users have access to Mac computers for music practice
- Standard audio interfaces (built-in or USB) provide sufficient quality
- Musicians understand basic interval concepts or are willing to learn
- Visual feedback can be iterated based on user testing and preferences

## Risks & Open Questions

### Key Risks

- **Latency Challenge**: Achieving <20ms total latency while maintaining accuracy may require significant optimization
- **User Adoption**: Tool might be too niche or complex for target age group
- **Audio Feedback**: Self-reference from speakers could interfere with pitch detection even with headphone design

### Open Questions

- What visual metaphors will be most effective for interval feedback?
- How to balance professional accuracy with child-friendly simplicity?
- Should the tool include any gamification elements in MVP?
- How should pitch deviation be displayed in just intonation mode? (cents from just target vs. cents from 12-TET equivalent vs. alternative units)

### Areas Needing Further Research

- Optimal pitch detection algorithms for real-time performance
- Child user interface design patterns for music education
- Audio feedback cancellation techniques if speaker output becomes necessary

## Appendices

### A. Research Summary

**Competitive Analysis**: Existing tuners focus on basic pitch accuracy without interval awareness or engaging visuals. Sonofield app approaches interval feedback but lacks child-friendly design.

**Technical Feasibility**: Real-time pitch detection is well-established, with Core Audio providing sufficient tools for Mac native development.

### B. Stakeholder Input

**Primary Stakeholder** (Developer): Wants impressive technical achievement with personal utility for family music practice.

**End User** (7-year-old): Needs fun, immediate feedback that feels like play rather than work.

### C. References

- Core Audio documentation for low-latency audio processing
- Music education research on interval training effectiveness  
- Existing pitch detection implementations and performance benchmarks 