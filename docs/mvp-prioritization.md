# MVP Prioritization Framework
## Real-time Pitch Visualizer

### Overview

This document outlines the priority framework for the Real-time Pitch Visualizer MVP, organized into tiers based on **user value**, **technical risk**, and **validation potential**.

---

## Priority Tiers

### **P0 (Core Foundation) - Week 1-3**
*Must work perfectly or the product fails*

1. **WASM Audio Processing Pipeline** ✅ **COMPLETE (Enhanced)**
   - Rust pitch detection algorithms compiled to WebAssembly
   - Web Audio API integration via AudioWorklet
   - Basic frequency detection (±5 cent accuracy)
   - **Enhanced Delivery**: Professional testing infrastructure with performance monitoring
   - **Rationale**: Core web architecture must work before anything else

2. **Browser Microphone Access**
   - getUserMedia API integration
   - Microphone permission handling and error states
   - **Rationale**: Without browser audio access, nothing else matters

3. **Minimal Web Interface**
   - Simple HTML/CSS display (frequency + note name + cent deviation)
   - No fancy graphics, just functional feedback
   - **Rationale**: Users need to see if it's working across browsers

4. **Cross-Browser Compatibility Foundation** ✅ **COMPLETE (Enhanced)**
   - Chrome, Firefox, Safari basic functionality
   - WASM loading and execution
   - **Enhanced Delivery**: Automated cross-browser testing with real-time performance metrics
   - **Rationale**: Web platform requires broader compatibility testing

### **P1 (MVP Viability) - Week 4-5**
*Makes the product actually useful*

1. **Reference Pitch Selection**
   - HTML controls for setting reference note (A440, etc.)
   - WASM/JS bridge for real-time parameter updates
   - **Rationale**: Essential for educational value

2. **Interval Calculation & Display**
   - WASM-based interval calculation for performance
   - Show musical interval from reference (major 3rd, perfect 5th, etc.)
   - **Rationale**: Core differentiator from basic tuners

3. **Web Performance Optimization**
   - Target <50ms total latency (web platform constraints)
   - Optimize WASM/JS boundary crossings
   - 60 FPS visual updates across browsers
   - **Rationale**: Web platform requires different optimization approach

### **P2 (Enhanced Usability) - Week 6-7**
*Makes it pleasant to use*

1. **Web Audio Output**
   - Reference pitch playback through Web Audio API
   - Headphone/speaker output controls
   - **Rationale**: Prevents feedback, enables better practice

2. **Tuning System Selection**
   - 12-TET vs Just Intonation toggle in web interface
   - **Rationale**: Advanced feature, but educationally valuable

3. **Responsive Web Design**
   - Clean, responsive interface across devices
   - Mobile/tablet optimization
   - **Rationale**: Web platform enables broader device support

4. **Progressive Web App Features**
   - Offline capability foundation
   - App-like experience
   - **Rationale**: Bridge between web and native app experience

### **P3 (Nice to Have)**
*If time permits*

1. **Enhanced Browser Error Handling**
   - Better microphone permission UX
   - Fallback for unsupported browsers
2. **Browser Storage**
   - Settings persistence via localStorage
   - User preferences across sessions
3. **Advanced Web Audio Features**
   - Audio device selection (when supported)
   - Advanced WebGL visualizations

---

## Strategic Rationale

### **Risk-First Approach**
Start with highest technical risks:
- Web Audio API latency optimization (<50ms is challenging in browsers)
- WASM performance for real-time audio processing
- Cross-browser compatibility (different WASM and Web Audio support)
- Microphone permissions and browser security constraints
- Real-time pitch detection accuracy in web environment

### **User Validation Strategy**
- **P0**: Working web prototype accessible via browser for initial testing
- **P1**: Educationally useful (key differentiator) with cross-browser validation
- **P2**: Pleasant to use across devices (retention factor)

### **Iteration Checkpoints**

**After P0** ✅ **ACHIEVED**: Child can access via browser (http://localhost:8080/web/). Basic WASM foundation works across Chrome, Firefox, Safari with professional testing infrastructure in place.

**After P1**: Does it actually help with musical learning? Are the intervals meaningful? Does it work on their preferred devices?

**After P2**: Would they choose to use this over other tools? Is the web experience compelling enough?

---

## Implementation Recommendations

### **Start Simple**
- Begin with P0 + simplest P1 features
- Don't try to build everything at once
- Get WASM compilation + basic web audio working first
- Test on Chrome first, expand to other browsers incrementally

### **Defer Complex Decisions**
- Visual design (keep it HTML/CSS initially, defer WebGL)
- Just intonation vs 12-TET (start with 12-TET only)
- Advanced web features (defer PWA capabilities)
- Mobile optimization (desktop browsers first)

### **Early User Testing**
- After P0 is working, test with target user immediately on their preferred devices
- Test browser permission flow and user experience
- User feedback will inform P1-P2 priorities and device optimization
- Validate web platform assumptions early and often

### **Success Criteria by Phase**
- **P0 Complete**: Basic WASM pitch detection working with web interface across major browsers
- **P1 Complete**: Educational value demonstrated through interval feedback with web-optimized performance
- **P2 Complete**: Polished web experience compelling enough for sustained use

---

## Notes

- This prioritization assumes single developer working iteratively with web technologies
- Timeline estimates are rough and should be adjusted based on WASM/web development learning curve
- Each phase should include cross-browser testing before proceeding to next phase
- Web-specific technical risks (WASM performance, browser compatibility) are front-loaded to reduce project risk overall
- Consider using wasm-pack for streamlined Rust → WASM workflow
- Progressive enhancement approach: start with core functionality, add advanced web features incrementally 