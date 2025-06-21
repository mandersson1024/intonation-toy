# Epic Backlog
## Real-time Pitch Visualizer

**Document Version**: 1.0  
**Created By**: Product Owner (Sarah)  
**Date**: June 2025  
**Project Phase**: Development Planning

---

## Project Context

**Project**: Web-based Real-time Pitch Visualizer  
**Target Users**: Children (6-16) learning instruments, music educators  
**Technology Stack**: Rust + WebAssembly, Web Audio API, JavaScript/HTML/CSS  
**Key Performance Requirements**: <50ms audio latency, 60 FPS visual updates  

**Strategic Alignment**: All epics align with project brief, technical architecture, and MVP prioritization (P0-P3).

---

## Epic Overview

| Epic ID | Epic Name | Priority | Stories | Status | Dependencies |
|---------|-----------|----------|---------|--------|--------------|
| EP-001 | WASM Audio Processing Foundation | P0 | 3 | **IN PROGRESS** (1/3 complete) | None |
| EP-002 | Browser Audio Integration & Permissions | P0 | 3 | Not Started | None |
| EP-003 | Educational Interval Analysis | P1 | 3 | Not Started | EP-001, EP-002 |
| EP-004 | Web Interface & Visualization | P1-P2 | 4 | Not Started | EP-001, EP-002 |

---

## Epic Details

### **EP-001: WASM Audio Processing Foundation**

**Priority**: P0 (Critical Foundation)  
**Estimated Stories**: 3  
**MVP Phase**: Phase 1 (Foundation)

#### Epic Goal
Establish the core WebAssembly-based audio processing pipeline that can perform real-time pitch detection in web browsers with sub-50ms latency.

#### Epic Description

**Technical Context:**
- New web-based pitch visualizer for educational music training
- Core requirement: Real-time audio processing in browser environment
- Technology choice: Rust compiled to WASM for performance-critical DSP
- Integration: WASM engine interfacing with Web Audio API via AudioWorklets

**What's Being Built:**
- Rust-based pitch detection algorithms compiled to WebAssembly
- WASM/JavaScript bridge using wasm-bindgen
- AudioWorklet processor for real-time audio handling
- Core audio engine with configurable parameters

**Success Criteria:**
- ✅ Pitch detection accuracy within ±5 cents
- ✅ Processing latency <50ms (web platform constraint)
- ✅ WASM module loads and executes in Chrome, Firefox, Safari
- ✅ Memory usage remains stable during continuous operation

#### Story Breakdown
1. **EP-001-S01**: ✅ **COMPLETE** - Set up WASM compilation pipeline with wasm-pack and basic audio processing structure
   - **Enhanced Delivery**: Professional testing infrastructure with performance monitoring established
   - **Quality Standards**: Browser test suite, automated development workflow, comprehensive documentation
2. **EP-001-S02**: Implement pitch detection algorithms in Rust and compile to WASM with proper JS bindings
3. **EP-001-S03**: Create AudioWorklet integration that processes microphone input through WASM engine

#### Dependencies
- **Upstream**: None (foundation epic)
- **Downstream**: EP-003, EP-004 depend on this epic

#### Risk Assessment
- **High Risk**: WASM performance in real-time audio context
- **Mitigation**: Prototype core algorithms early, benchmark across browsers
- **Fallback**: Reduce buffer sizes if latency targets not met

---

### **EP-002: Browser Audio Integration & Permissions**

**Priority**: P0 (Critical Foundation)  
**Estimated Stories**: 3  
**MVP Phase**: Phase 1 (Foundation)

#### Epic Goal
Enable secure microphone access and Web Audio API integration across major browsers with user-friendly permission handling.

#### Epic Description

**Technical Context:**
- Browser security requires explicit user permission for microphone access
- Web Audio API varies slightly across browser implementations
- Target browsers: Chrome, Firefox, Safari, Edge
- User experience must be child-friendly and error-tolerant

**What's Being Built:**
- getUserMedia permission request flow with clear user feedback
- Web Audio Context initialization and configuration
- Microphone input stream processing setup
- Cross-browser compatibility handling and error management

**Success Criteria:**
- ✅ Microphone access works reliably across target browsers
- ✅ Permission denial handled gracefully with user guidance
- ✅ Audio input stream connects successfully to processing pipeline
- ✅ Error states provide actionable feedback to users

#### Story Breakdown
1. **EP-002-S01**: Implement microphone permission request flow with user-friendly UI
2. **EP-002-S02**: Set up Web Audio API context and microphone input processing
3. **EP-002-S03**: Add error handling and fallbacks for unsupported browsers/permissions

#### Dependencies
- **Upstream**: None (foundation epic)
- **Downstream**: EP-001 (WASM processing), EP-004 (UI integration)

#### Risk Assessment
- **Medium Risk**: Browser permission denial blocking core functionality
- **Mitigation**: Clear permission UX, educational messaging
- **Fallback**: Demo mode with synthetic audio if permissions denied

---

### **EP-003: Educational Interval Analysis**

**Priority**: P1 (MVP Viability)  
**Estimated Stories**: 3  
**MVP Phase**: Phase 2 (Educational Value)

#### Epic Goal
Provide real-time musical interval analysis and feedback that helps users understand pitch relationships relative to configurable reference notes.

#### Epic Description

**Educational Context:**
- Core differentiator from basic tuners - educational interval awareness
- Must support both 12-Tone Equal Temperament and Just Intonation
- Reference pitch selection enables personalized learning
- Interval feedback helps users understand musical relationships

**What's Being Built:**
- Reference pitch selection interface (note names, frequencies)
- Interval calculation algorithms for multiple tuning systems
- Real-time interval analysis relative to selected reference
- Educational interval display with musical terminology

**Success Criteria:**
- ✅ Accurate interval identification within 5 cents
- ✅ Support for 12-TET and Just Intonation tuning systems
- ✅ User-configurable reference pitch (note or frequency)
- ✅ Real-time interval feedback with musical interval names

#### Story Breakdown
1. **EP-003-S01**: Implement reference pitch selection and management
2. **EP-003-S02**: Build interval calculation algorithms for 12-TET and Just Intonation
3. **EP-003-S03**: Create interval display and feedback system

#### Dependencies
- **Upstream**: EP-001 (audio processing), EP-002 (audio input)
- **Downstream**: EP-004 (UI display of interval information)

#### Risk Assessment
- **Low Risk**: Well-established musical mathematics
- **Mitigation**: Validate algorithms against known musical intervals
- **Fallback**: Start with 12-TET only if Just Intonation proves complex

---

### **EP-004: Web Interface & Visualization**

**Priority**: P1-P2 (Usability)  
**Estimated Stories**: 4  
**MVP Phase**: Phase 2-3 (User Experience)

#### Epic Goal
Create an engaging, child-friendly web interface that displays real-time pitch and interval information with compelling visual feedback.

#### Epic Description

**User Experience Context:**
- Primary users: 7-year-old children (must be intuitive and engaging)
- Secondary users: Music educators and adult learners
- Responsive design: Works on tablets, laptops, desktops
- Performance requirement: 60 FPS visual updates for smooth experience

**What's Being Built:**
- HTML/CSS interface with child-friendly design
- Real-time pitch and interval display components
- Canvas-based visualization for pitch feedback
- Control interfaces for reference pitch and tuning system selection
- Responsive design for cross-device compatibility

**Success Criteria:**
- ✅ 60 FPS visual updates with smooth animations
- ✅ Child-friendly interface (7-year-old can use independently)
- ✅ Responsive design works on tablets and desktop
- ✅ Real-time display updates synchronized with audio processing

#### Story Breakdown
1. **EP-004-S01**: Build basic HTML/CSS interface with pitch display components
2. **EP-004-S02**: Implement Canvas-based real-time pitch visualization
3. **EP-004-S03**: Add control interfaces for reference pitch and tuning system selection
4. **EP-004-S04**: Optimize for responsive design across devices

#### Dependencies
- **Upstream**: EP-001 (audio data), EP-002 (audio input), EP-003 (interval data)
- **Downstream**: None (presentation layer)

#### Risk Assessment
- **Medium Risk**: Performance of real-time Canvas updates
- **Mitigation**: Optimize Canvas rendering, use requestAnimationFrame
- **Fallback**: Reduce visual complexity if performance targets not met

---

## Epic Sequencing & Dependencies

```
Phase 1 (P0 - Foundation): Weeks 1-3
┌─────────────────┐    ┌─────────────────────────────┐
│   EP-002        │    │         EP-001              │
│ Browser Audio   │◄───┤   WASM Audio Processing     │
│ & Permissions   │    │      Foundation             │
└─────────────────┘    └─────────────────────────────┘
         │                           │
         └───────────┐     ┌─────────┘
                     ▼     ▼
Phase 2 (P1 - Viability): Weeks 4-5
┌─────────────────────────────────────────────────────┐
│              EP-003                                 │
│        Educational Interval Analysis               │
└─────────────────────────────────────────────────────┘
                           │
                           ▼
Phase 2-3 (P1-P2 - Usability): Weeks 4-7
┌─────────────────────────────────────────────────────┐
│              EP-004                                 │
│        Web Interface & Visualization               │
└─────────────────────────────────────────────────────┘
```

## Next Steps

1. **Immediate**: Create Story 1.2 (EP-001-S02) for pitch detection implementation
2. **Sprint Planning**: Continue EP-001 with enhanced testing standards established in Story 1.1
3. **Quality Standards**: All future stories must meet Definition of Done requirements including browser test suite validation
4. **Velocity Assessment**: Story 1.1 delivered significant value beyond scope - adjust planning for comprehensive delivery approach

## Sprint Planning Updates (Post Story 1.1)

### **Enhanced Testing Standards** 
All future stories must include:
- Professional browser test suite validation (http://localhost:8080/web/)
- Performance benchmarking with established thresholds (<100μs excellent, <500μs acceptable)
- Cross-browser testing automation
- Real-time performance monitoring
- Comprehensive documentation updates

### **Development Workflow Standards**
- Consistent development environment (Ruby server, port 8080)
- Automated build and serve pipeline (`./dev.sh`)
- Enhanced Definition of Done compliance
- Architecture documentation maintenance

### **Velocity Insights**
- Story 1.1 delivered substantial value enhancement beyond original scope
- Testing infrastructure significantly reduces risk for future stories  
- Quality standards elevated - may impact story estimation
- Foundation established enables faster subsequent development

---

## Notes

- All epics align with MVP prioritization framework
- Story estimates will be refined during story creation phase
- Technical risks front-loaded to P0 phase per architecture recommendations
- Educational value (interval analysis) prioritized in P1 phase
- User experience enhancements deferred to P2 for iterative improvement

**Last Updated**: June 2025 (Post Story 1.1 completion)  
**Next Review**: After Story 1.2 planning and creation  
**Status**: EP-001 Story 1.1 complete with enhanced testing standards established 