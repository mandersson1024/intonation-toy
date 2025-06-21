/**
 * AudioWorklet Processor for Real-time Pitch Detection
 * Story 2.1: Connect microphone input to WASM audio processing pipeline
 * 
 * This worklet runs on the audio thread and processes microphone input
 * using the WASM audio engine established in Story 1.2
 */

class PitchDetectionProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
        
        this.bufferSize = 1024;
        this.sampleRate = 44100;
        this.inputBuffer = new Float32Array(this.bufferSize);
        this.bufferIndex = 0;
        
        // WASM engine will be initialized from main thread
        this.audioEngine = null;
        this.isProcessing = false;
        
        // Performance monitoring
        this.processCount = 0;
        this.lastReportTime = 0;
        
        // Listen for messages from main thread
        this.port.onmessage = (event) => {
            this.handleMessage(event.data);
        };
        
        console.log('🎵 PitchDetectionProcessor initialized');
    }

    /**
     * Handle messages from main thread
     */
    handleMessage(data) {
        switch (data.type) {
            case 'init':
                this.initializeAudioEngine(data.audioEngine);
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
     * Initialize with WASM audio engine from main thread
     */
    initializeAudioEngine(audioEngine) {
        try {
            this.audioEngine = audioEngine;
            this.sampleRate = this.audioEngine?.get_sample_rate() || 44100;
            this.bufferSize = this.audioEngine?.get_buffer_size() || 1024;
            this.inputBuffer = new Float32Array(this.bufferSize);
            
            this.port.postMessage({
                type: 'initialized',
                sampleRate: this.sampleRate,
                bufferSize: this.bufferSize
            });
            
            console.log(`AudioEngine initialized in worklet: SR=${this.sampleRate}, BS=${this.bufferSize}`);
        } catch (error) {
            console.error('Failed to initialize AudioEngine in worklet:', error);
            this.port.postMessage({
                type: 'error',
                error: error.message
            });
        }
    }

    /**
     * Start audio processing
     */
    startProcessing() {
        this.isProcessing = true;
        this.bufferIndex = 0;
        this.processCount = 0;
        this.lastReportTime = currentTime;
        
        this.port.postMessage({
            type: 'started'
        });
        
        console.log('🎤 Audio processing started');
    }

    /**
     * Stop audio processing
     */
    stopProcessing() {
        this.isProcessing = false;
        
        this.port.postMessage({
            type: 'stopped'
        });
        
        console.log('⏹️ Audio processing stopped');
    }

    /**
     * Set buffer size for processing
     */
    setBufferSize(newBufferSize) {
        this.bufferSize = newBufferSize;
        this.inputBuffer = new Float32Array(this.bufferSize);
        this.bufferIndex = 0;
        
        console.log(`Buffer size updated to ${newBufferSize}`);
    }

    /**
     * Main audio processing function
     * This runs on every audio quantum (128 samples by default)
     */
    process(inputs, outputs, parameters) {
        if (!this.isProcessing || !this.audioEngine) {
            return true; // Keep processor alive
        }

        const input = inputs[0];
        if (!input || input.length === 0) {
            return true; // No input available
        }

        // Get the first channel (mono input)
        const inputChannel = input[0];
        
        try {
            // Fill our buffer with incoming audio data
            for (let i = 0; i < inputChannel.length; i++) {
                this.inputBuffer[this.bufferIndex] = inputChannel[i];
                this.bufferIndex++;
                
                // When buffer is full, process it
                if (this.bufferIndex >= this.bufferSize) {
                    this.processAudioBuffer();
                    this.bufferIndex = 0;
                }
            }
            
            // Performance monitoring
            this.processCount++;
            const currentTime = performance.now();
            
            // Report performance every second
            if (currentTime - this.lastReportTime > 1000) {
                this.reportPerformance(currentTime);
                this.lastReportTime = currentTime;
            }
            
        } catch (error) {
            console.error('Audio processing error:', error);
            this.port.postMessage({
                type: 'error',
                error: error.message
            });
        }

        return true; // Keep processor alive
    }

    /**
     * Process a full buffer of audio data - Story 2.1 scope: just establish connection
     */
    processAudioBuffer() {
        if (!this.audioEngine) {
            return;
        }

        try {
            // Story 2.1: Just confirm connection to WASM pipeline
            // No actual pitch processing - that belongs in later stories (EP-003)
            
            // Check if we have valid audio data
            const hasAudioSignal = this.inputBuffer.some(sample => Math.abs(sample) > 0.001);
            
            // Confirm WASM engine is accessible (basic connection test)
            const wasmConnected = this.audioEngine && 
                                 typeof this.audioEngine.get_sample_rate === 'function';
            
            // Send connection confirmation back to main thread
            this.port.postMessage({
                type: 'connectionConfirmed',
                result: {
                    wasmConnected: wasmConnected,
                    hasAudioSignal: hasAudioSignal,
                    bufferSize: this.bufferSize,
                    sampleRate: wasmConnected ? this.audioEngine.get_sample_rate() : null
                }
            });
            
        } catch (error) {
            console.error('Connection test error:', error);
            this.port.postMessage({
                type: 'connectionError',
                error: error.message
            });
        }
    }

    /**
     * Report performance metrics to main thread
     */
    reportPerformance(currentTime) {
        const processesPerSecond = this.processCount;
        this.processCount = 0;
        
        this.port.postMessage({
            type: 'performance',
            processesPerSecond: processesPerSecond,
            bufferSize: this.bufferSize,
            timestamp: currentTime
        });
    }

    /**
     * Static method to define processor parameters
     */
    static get parameterDescriptors() {
        return [];
    }
}

// Register the processor
registerProcessor('pitch-detection-processor', PitchDetectionProcessor); 