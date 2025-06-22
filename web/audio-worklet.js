/**
 * AudioWorklet Processor for Pitch Detection
 * Provides low-latency audio processing with WASM integration
 */

class PitchProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();
        
        // Configuration
        this.bufferSize = 1024;
        this.sampleRate = sampleRate;
        this.targetLatencyMs = 10.0;
        
        // Processing state
        this.processingEnabled = true;
        this.audioBuffer = new Float32Array(this.bufferSize);
        this.bufferIndex = 0;
        
        // Performance monitoring
        this.processingTimes = [];
        this.lastPerformanceReport = 0;
        this.performanceReportInterval = 1000; // 1 second
        
        // WASM interface (will be initialized when available)
        this.wasmAudioEngine = null;
        this.wasmInitialized = false;
        
        // Setup message handling
        this.port.onmessage = this.handleMessage.bind(this);
        
        console.log('PitchProcessor initialized', {
            bufferSize: this.bufferSize,
            sampleRate: this.sampleRate,
            targetLatency: this.targetLatencyMs
        });
        
        // Request WASM initialization
        this.port.postMessage({
            type: 'request-wasm-init',
            sampleRate: this.sampleRate,
            bufferSize: this.bufferSize
        });
    }
    
    /**
     * Main audio processing callback
     * Called by the browser for each audio quantum (128 samples)
     */
    process(inputs, outputs, parameters) {
        if (!this.processingEnabled) {
            return true;
        }
        
        const startTime = performance.now();
        
        // Get input audio data
        const input = inputs[0];
        const output = outputs[0];
        
        if (input.length > 0 && input[0].length > 0) {
            const inputChannel = input[0];
            const outputChannel = output[0];
            
            // Process audio samples
            this.processAudioSamples(inputChannel, outputChannel);
            
            // Record processing time
            const processingTime = performance.now() - startTime;
            this.recordProcessingTime(processingTime);
        }
        
        return true; // Keep processor alive
    }
    
    /**
     * Process audio samples with buffering for pitch detection
     */
    processAudioSamples(inputSamples, outputSamples) {
        // Copy input to output (passthrough)
        for (let i = 0; i < inputSamples.length; i++) {
            outputSamples[i] = inputSamples[i];
            
            // Accumulate samples in buffer for processing
            this.audioBuffer[this.bufferIndex] = inputSamples[i];
            this.bufferIndex++;
            
            // Process when buffer is full
            if (this.bufferIndex >= this.bufferSize) {
                this.processAudioBuffer();
                this.bufferIndex = 0;
            }
        }
    }
    
    /**
     * Process full audio buffer for pitch detection
     */
    processAudioBuffer() {
        if (!this.wasmInitialized || !this.wasmAudioEngine) {
            // Send buffer to main thread for processing if WASM not available
            this.port.postMessage({
                type: 'audio-buffer',
                buffer: Array.from(this.audioBuffer),
                timestamp: performance.now()
            });
            return;
        }
        
        try {
            // Process with WASM engine (when available)
            const result = this.wasmAudioEngine.process_realtime_audio(this.audioBuffer);
            
            // Send results to main thread
            this.port.postMessage({
                type: 'audio-data',
                data: {
                    pitchFrequency: result.pitch_frequency || -1.0,
                    confidence: result.confidence || 0.0,
                    processingTimeMs: result.processing_time_ms || 0.0,
                    audioLevel: this.calculateAudioLevel(this.audioBuffer),
                    timestamp: performance.now()
                }
            });
            
        } catch (error) {
            console.error('WASM processing error:', error);
            
            // Fallback to main thread processing
            this.port.postMessage({
                type: 'audio-buffer',
                buffer: Array.from(this.audioBuffer),
                timestamp: performance.now(),
                error: error.message
            });
        }
    }
    
    /**
     * Calculate audio level (RMS) for visualization
     */
    calculateAudioLevel(buffer) {
        let sum = 0;
        for (let i = 0; i < buffer.length; i++) {
            sum += buffer[i] * buffer[i];
        }
        return Math.sqrt(sum / buffer.length);
    }
    
    /**
     * Record processing time for performance monitoring
     */
    recordProcessingTime(timeMs) {
        this.processingTimes.push(timeMs);
        
        // Limit history size
        if (this.processingTimes.length > 100) {
            this.processingTimes.shift();
        }
        
        // Report performance periodically
        const now = performance.now();
        if (now - this.lastPerformanceReport > this.performanceReportInterval) {
            this.reportPerformance();
            this.lastPerformanceReport = now;
        }
    }
    
    /**
     * Report performance metrics to main thread
     */
    reportPerformance() {
        if (this.processingTimes.length === 0) return;
        
        const avgTime = this.processingTimes.reduce((a, b) => a + b, 0) / this.processingTimes.length;
        const maxTime = Math.max(...this.processingTimes);
        const minTime = Math.min(...this.processingTimes);
        
        // Calculate quantum utilization (128 samples at sample rate)
        const quantumDurationMs = (128 / this.sampleRate) * 1000;
        const utilization = (avgTime / quantumDurationMs) * 100;
        
        this.port.postMessage({
            type: 'performance-metrics',
            metrics: {
                averageProcessingTimeMs: avgTime,
                maxProcessingTimeMs: maxTime,
                minProcessingTimeMs: minTime,
                quantumUtilization: utilization,
                targetLatencyMs: this.targetLatencyMs,
                sampleCount: this.processingTimes.length,
                isWithinTarget: avgTime < this.targetLatencyMs
            }
        });
        
        // Clear old data
        this.processingTimes = [];
    }
    
    /**
     * Handle messages from main thread
     */
    handleMessage(event) {
        const { type, data } = event.data;
        
        switch (type) {
            case 'wasm-init':
                this.initializeWasm(data);
                break;
                
            case 'set-enabled':
                this.processingEnabled = data.enabled;
                console.log('Processing enabled:', this.processingEnabled);
                break;
                
            case 'set-target-latency':
                this.targetLatencyMs = data.latencyMs;
                console.log('Target latency updated:', this.targetLatencyMs);
                break;
                
            case 'configure':
                this.configure(data);
                break;
                
            default:
                console.warn('Unknown message type:', type);
        }
    }
    
    /**
     * Initialize WASM audio engine
     */
    initializeWasm(wasmData) {
        try {
            // This would be the actual WASM engine instance
            // For now, we'll simulate the interface
            this.wasmAudioEngine = wasmData.engine;
            this.wasmInitialized = true;
            
            console.log('WASM AudioEngine initialized in worklet');
            
            this.port.postMessage({
                type: 'wasm-initialized',
                success: true
            });
            
        } catch (error) {
            console.error('Failed to initialize WASM in worklet:', error);
            
            this.port.postMessage({
                type: 'wasm-initialized',
                success: false,
                error: error.message
            });
        }
    }
    
    /**
     * Configure processor parameters
     */
    configure(config) {
        if (config.bufferSize && config.bufferSize !== this.bufferSize) {
            this.bufferSize = config.bufferSize;
            this.audioBuffer = new Float32Array(this.bufferSize);
            this.bufferIndex = 0;
            console.log('Buffer size updated:', this.bufferSize);
        }
        
        if (config.targetLatencyMs) {
            this.targetLatencyMs = config.targetLatencyMs;
        }
        
        // Configure WASM engine if available
        if (this.wasmInitialized && this.wasmAudioEngine && config.wasmConfig) {
            try {
                // Apply WASM-specific configuration
                if (config.wasmConfig.pitchAlgorithm) {
                    this.wasmAudioEngine.set_pitch_algorithm(config.wasmConfig.pitchAlgorithm);
                }
                
                if (config.wasmConfig.frequencyRange) {
                    this.wasmAudioEngine.set_pitch_frequency_range(
                        config.wasmConfig.frequencyRange.min,
                        config.wasmConfig.frequencyRange.max
                    );
                }
                
            } catch (error) {
                console.error('WASM configuration error:', error);
            }
        }
    }
    
    /**
     * Get processor status
     */
    getStatus() {
        return {
            processingEnabled: this.processingEnabled,
            wasmInitialized: this.wasmInitialized,
            bufferSize: this.bufferSize,
            targetLatencyMs: this.targetLatencyMs,
            sampleRate: this.sampleRate
        };
    }
}

// Register the processor
registerProcessor('pitch-processor', PitchProcessor);

console.log('PitchProcessor registered successfully'); 