/**
 * 🎯 STREAMLINED AudioWorklet Processor - Phase 4 Refactoring
 * 
 * SIMPLIFIED APPROACH: AudioWorklet collects audio, main thread processes with WASM
 * 
 * ✅ AudioWorklet: Lightweight audio data collection and buffering
 * ✅ Main Thread: WASM loading and audio processing
 * ✅ Clean separation: Worklet for real-time audio, main thread for computation
 */

class PitchDetectionProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
        
        // 🏗️ Minimal configuration for audio collection
        this.bufferSize = 1024;
        this.sampleRate = 44100;
        this.inputBuffer = new Float32Array(this.bufferSize);
        this.bufferIndex = 0;
        
        // 🎯 Processing state
        this.isProcessing = false;
        this.isInitialized = false;
        
        // 📊 Performance tracking
        this.processCount = 0;
        this.lastReportTime = 0;
        
        // 🔄 Message handling from main thread
        this.port.onmessage = (event) => {
            this.handleMessage(event.data);
        };
        
        console.log('🎵 Streamlined AudioWorklet Processor initialized (Audio Collection Mode)');
    }

    /**
     * 📨 Message handling
     */
    handleMessage(data) {
        switch (data.type) {
            case 'init':
                this.initializeProcessor(data.sampleRate, data.targetLatency);
                break;
            case 'start':
                this.startProcessing();
                break;
            case 'stop':
                this.stopProcessing();
                break;
            case 'setBufferSize':
                this.setBufferSize(data.bufferSize);
                break;
            default:
                console.warn('Unknown message type:', data.type);
        }
    }

    /**
     * 🏭 Processor initialization - no WASM loading needed
     */
    initializeProcessor(sampleRate, targetLatency) {
        try {
            this.sampleRate = sampleRate || 44100;
            this.targetLatency = targetLatency || 0.05;
            this.isInitialized = true;
            
            this.port.postMessage({
                type: 'initialized',
                sampleRate: this.sampleRate,
                bufferSize: this.bufferSize,
                wasmAvailable: false, // WASM will be handled in main thread
                mode: 'audio_collection'
            });
            
            console.log(`🎯 Audio processor initialized: SR=${this.sampleRate}Hz, Buffer=${this.bufferSize}`);
        } catch (error) {
            console.error('Processor initialization failed:', error);
            this.port.postMessage({ type: 'error', error: error.message });
        }
    }

    /**
     * ▶️ Start processing
     */
    startProcessing() {
        if (!this.isProcessing) {
            this.isProcessing = true;
            this.bufferIndex = 0;
            this.processCount = 0;
            this.lastReportTime = currentTime;
            
            this.port.postMessage({ type: 'started' });
            console.log('🎤 Audio collection started');
        }
    }

    /**
     * ⏹️ Stop processing
     */
    stopProcessing() {
        this.isProcessing = false;
        this.port.postMessage({ type: 'stopped' });
        console.log('⏹️ Audio collection stopped');
    }

    /**
     * 🔧 Update buffer size
     */
    setBufferSize(newBufferSize) {
        this.bufferSize = newBufferSize;
        this.inputBuffer = new Float32Array(this.bufferSize);
        this.bufferIndex = 0;
        
        this.port.postMessage({
            type: 'bufferSizeChanged',
            bufferSize: this.bufferSize
        });
    }

    /**
     * 🎯 MAIN PROCESSING LOOP - Audio collection and forwarding
     */
    process(inputs, outputs, parameters) {
        if (!this.isProcessing) return true;

        const input = inputs[0];
        if (!input || input.length === 0) return true;

        const inputChannel = input[0];
        
        try {
            // 📥 Buffer audio data
            for (let i = 0; i < inputChannel.length; i++) {
                this.inputBuffer[this.bufferIndex] = inputChannel[i];
                this.bufferIndex++;
                
                // 🎯 When buffer full, send to main thread for processing
                if (this.bufferIndex >= this.bufferSize) {
                    this.sendAudioBufferToMainThread();
                    this.bufferIndex = 0;
                }
            }
            
            this.processCount++;
            
            // 📊 Report performance periodically
            if (this.processCount % 1000 === 0) {
                this.reportPerformance();
            }
            
        } catch (error) {
            console.error('Audio processing error:', error);
            this.port.postMessage({
                type: 'error',
                error: error.message,
                context: 'audio_collection'
            });
        }

        return true;
    }

    /**
     * 📤 Send audio buffer to main thread for WASM processing
     */
    sendAudioBufferToMainThread() {
        // Copy buffer to avoid issues with shared memory
        const bufferCopy = new Float32Array(this.inputBuffer);
        
        this.port.postMessage({
            type: 'audioBuffer',
            audioData: bufferCopy,
            bufferSize: this.bufferSize,
            sampleRate: this.sampleRate,
            timestamp: currentTime
        });
    }

    /**
     * 📊 Performance reporting
     */
    reportPerformance() {
        const processesPerSecond = this.processCount;
        this.processCount = 0;
        
        this.port.postMessage({
            type: 'performance',
            processesPerSecond: processesPerSecond,
            bufferSize: this.bufferSize,
            sampleRate: this.sampleRate,
            timestamp: currentTime
        });
    }

    /**
     * 🏷️ Parameter descriptors
     */
    static get parameterDescriptors() {
        return [];
    }
}

// 🚀 Register the processor
registerProcessor('pitch-detection-processor', PitchDetectionProcessor); 