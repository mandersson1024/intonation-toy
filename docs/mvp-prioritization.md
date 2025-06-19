# MVP Prioritization Framework
## Real-time Pitch Visualizer

### Overview

This document outlines the priority framework for the Real-time Pitch Visualizer MVP, organized into tiers based on **user value**, **technical risk**, and **validation potential**.

---

## Priority Tiers

### **P0 (Core Foundation) - Week 1-2**
*Must work perfectly or the product fails*

1. **Live Audio Input + Basic Pitch Detection**
   - Single microphone input processing
   - Basic frequency detection (±5 cent accuracy)
   - **Rationale**: Without this, nothing else matters

2. **Minimal Visual Feedback**
   - Simple numerical display (frequency + note name + cent deviation)
   - No fancy graphics, just functional feedback
   - **Rationale**: Users need to see if it's working

3. **Audio Pipeline Stability**
   - Core Audio integration
   - Basic latency optimization (<50ms to start)
   - **Rationale**: Technical foundation must be solid

### **P1 (MVP Viability) - Week 3-4**
*Makes the product actually useful*

1. **Reference Pitch Selection**
   - User can set reference note (A440, etc.)
   - **Rationale**: Essential for educational value

2. **Interval Calculation & Display**
   - Show musical interval from reference (major 3rd, perfect 5th, etc.)
   - **Rationale**: Core differentiator from basic tuners

3. **Performance Optimization**
   - Target <20ms latency
   - 60 FPS visual updates
   - **Rationale**: User experience requirement

### **P2 (Enhanced Usability) - Week 5-6**
*Makes it pleasant to use*

1. **Headphone Audio Output**
   - Reference pitch playback through headphones
   - **Rationale**: Prevents feedback, enables better practice

2. **Tuning System Selection**
   - 12-TET vs Just Intonation toggle
   - **Rationale**: Advanced feature, but educationally valuable

3. **Basic Mac Native GUI**
   - Clean, responsive interface
   - **Rationale**: Professional presentation matters

### **P3 (Nice to Have)**
*If time permits*

1. **Enhanced Error Handling**
2. **Basic Settings Persistence**
3. **Audio Device Selection**

---

## Strategic Rationale

### **Risk-First Approach**
Start with highest technical risks:
- Audio latency (<20ms is aggressive)
- Real-time pitch detection accuracy
- Audio feedback prevention

### **User Validation Strategy**
- **P0**: Working prototype for initial testing
- **P1**: Educationally useful (key differentiator)
- **P2**: Pleasant to use (retention factor)

### **Iteration Checkpoints**

**After P0**: Can your child interact with it at all? Does the basic pitch detection work?

**After P1**: Does it actually help with musical learning? Are the intervals meaningful?

**After P2**: Would they choose to use this over other tools?

---

## Implementation Recommendations

### **Start Simple**
- Begin with P0 + simplest P1 features
- Don't try to build everything at once
- Get pitch detection + basic interval display working first

### **Defer Complex Decisions**
- Visual design (keep it numerical initially)
- Just intonation vs 12-TET (start with 12-TET only)
- Advanced GUI (use basic native controls)

### **Early User Testing**
- After P0 is working, test with target user immediately
- User feedback will inform P1-P2 priorities
- Validate assumptions early and often

### **Success Criteria by Phase**
- **P0 Complete**: Basic pitch detection working with numerical feedback
- **P1 Complete**: Educational value demonstrated through interval feedback
- **P2 Complete**: Polished enough for sustained use

---

## Notes

- This prioritization assumes single developer working iteratively
- Timeline estimates are rough and should be adjusted based on actual progress
- Each phase should include user testing before proceeding to next phase
- Technical risks are front-loaded to reduce project risk overall 