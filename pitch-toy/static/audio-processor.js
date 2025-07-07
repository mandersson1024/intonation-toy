/**
 * AudioWorklet Processor for Real-Time Pitch Detection
 * 
 * This processor handles real-time audio processing in the dedicated audio thread.
 * It processes audio in fixed 128-sample chunks as required by the Web Audio API
 * and forwards audio data to the main thread for pitch detection processing.
 * 
 * Key Features:
 * - Fixed 128-sample chunk processing (Web Audio API standard)
 * - Real-time audio data forwarding via MessagePort
 * - Low-latency processing with minimal overhead
 * - Error handling and processor lifecycle management
 * 
 * Communication:
 * - Receives: Configuration and control messages from main thread
 * - Sends: Audio data chunks and processing status to main thread
 */

class PitchDetectionProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
        
        // Constructor logging kept for debugging
        console.log('PitchDetectionProcessor: Constructor called - processor instance created');
        
        // Fixed chunk size as per Web Audio API specification
        this.chunkSize = 128;
        
        // Processing state
        this.isProcessing = false;
        this.chunkCounter = 0;
        
        // Test signal configuration
        this.testSignalConfig = {
            enabled: false,
            frequency: 440.0,
            amplitude: 0.3,
            waveform: 'sine',
            sample_rate: 48000.0
        };
        
        // Background noise configuration (independent of test signal)
        this.backgroundNoiseConfig = {
            enabled: false,
            level: 0.0,
            type: 'white_noise'  // white_noise, pink_noise
        };
        
        // Test signal generation state
        this.testSignalPhase = 0.0;
        
        // Setup message handling
        this.port.onmessage = (event) => {
            // Message logging kept for debugging
            console.log('PitchDetectionProcessor: Received message:', event.data);
            this.handleMessage(event.data);
        };
        
        // Initialize processor
        this.port.postMessage({
            type: 'processorReady',
            chunkSize: this.chunkSize,
            timestamp: this.currentTime || 0
        });
        
        console.log('PitchDetectionProcessor: Constructor complete, ready for processing');
    }
    
    /**
     * Handle messages from main thread
     * @param {Object} message - Message from main thread
     */
    handleMessage(message) {
        switch (message.type) {
            case 'startProcessing':
                this.isProcessing = true;
                this.port.postMessage({
                    type: 'processingStarted',
                    timestamp: this.currentTime || 0
                });
                break;
                
            case 'stopProcessing':
                this.isProcessing = false;
                this.port.postMessage({
                    type: 'processingStopped',
                    timestamp: this.currentTime || 0
                });
                break;
                
            case 'getStatus':
                this.port.postMessage({
                    type: 'status',
                    isProcessing: this.isProcessing,
                    chunkCounter: this.chunkCounter,
                    timestamp: this.currentTime || 0
                });
                break;
                
            case 'updateTestSignalConfig':
                if (message.config) {
                    this.testSignalConfig = { ...this.testSignalConfig, ...message.config };
                    // Reset phase when configuration changes
                    this.testSignalPhase = 0.0;
                    console.log('PitchDetectionProcessor: Test signal config updated:', this.testSignalConfig);
                    this.port.postMessage({
                        type: 'testSignalConfigUpdated',
                        config: this.testSignalConfig,
                        timestamp: this.currentTime || 0
                    });
                }
                break;
                
            case 'updateBackgroundNoiseConfig':
                if (message.config) {
                    this.backgroundNoiseConfig = { ...this.backgroundNoiseConfig, ...message.config };
                    console.log('PitchDetectionProcessor: Background noise config updated:', this.backgroundNoiseConfig);
                    this.port.postMessage({
                        type: 'backgroundNoiseConfigUpdated',
                        config: this.backgroundNoiseConfig,
                        timestamp: this.currentTime || 0
                    });
                }
                break;
                
            default:
                console.warn('PitchDetectionProcessor: Unknown message type:', message.type);
        }
    }
    
    /**
     * Generate test signal samples
     * @param {number} numSamples - Number of samples to generate
     * @returns {Float32Array} - Generated test signal samples
     */
    generateTestSignal(numSamples) {
        const samples = new Float32Array(numSamples);
        const config = this.testSignalConfig;
        
        if (!config.enabled) {
            return samples; // Return silence if disabled
        }
        
        const phaseIncrement = (2 * Math.PI * config.frequency) / config.sample_rate;
        
        for (let i = 0; i < numSamples; i++) {
            let sample = 0.0;
            
            // Generate waveform
            switch (config.waveform) {
                case 'sine':
                    sample = Math.sin(this.testSignalPhase);
                    break;
                case 'square':
                    sample = Math.sin(this.testSignalPhase) >= 0 ? 1.0 : -1.0;
                    break;
                case 'sawtooth':
                    sample = 2.0 * (this.testSignalPhase / (2 * Math.PI) - Math.floor(this.testSignalPhase / (2 * Math.PI) + 0.5));
                    break;
                case 'triangle':
                    const t = this.testSignalPhase / (2 * Math.PI) - Math.floor(this.testSignalPhase / (2 * Math.PI));
                    sample = t < 0.5 ? 4.0 * t - 1.0 : 3.0 - 4.0 * t;
                    break;
                case 'white_noise':
                    sample = (Math.random() * 2.0 - 1.0);
                    break;
                case 'pink_noise':
                    // Simplified pink noise approximation
                    sample = (Math.random() * 2.0 - 1.0) * 0.5;
                    break;
                default:
                    sample = Math.sin(this.testSignalPhase);
            }
            
            // Apply amplitude scaling
            sample *= config.amplitude;
            
            // Clamp to valid range
            sample = Math.max(-1.0, Math.min(1.0, sample));
            
            samples[i] = sample;
            
            // Update phase for next sample
            this.testSignalPhase += phaseIncrement;
            
            // Keep phase in [0, 2π] range to prevent precision issues
            if (this.testSignalPhase >= 2 * Math.PI) {
                this.testSignalPhase -= 2 * Math.PI;
            }
        }
        
        return samples;
    }
    
    /**
     * Generate background noise samples
     * @param {number} numSamples - Number of samples to generate
     * @returns {Float32Array} - Generated background noise samples
     */
    generateBackgroundNoise(numSamples) {
        const samples = new Float32Array(numSamples);
        const config = this.backgroundNoiseConfig;
        
        if (!config.enabled || config.level <= 0.0) {
            return samples; // Return silence if disabled or level is 0
        }
        
        for (let i = 0; i < numSamples; i++) {
            let sample = 0.0;
            
            // Generate noise based on type
            switch (config.type) {
                case 'white_noise':
                    sample = (Math.random() * 2.0 - 1.0);
                    break;
                case 'pink_noise':
                    // Simplified pink noise approximation
                    sample = (Math.random() * 2.0 - 1.0) * 0.5;
                    break;
                default:
                    sample = (Math.random() * 2.0 - 1.0); // Default to white noise
            }
            
            // Apply level scaling
            sample *= config.level;
            
            // Clamp to valid range
            sample = Math.max(-1.0, Math.min(1.0, sample));
            
            samples[i] = sample;
        }
        
        return samples;
    }
    
    /**
     * Process audio data in 128-sample chunks
     * This method is called by the Web Audio API for each processing quantum
     * 
     * @param {Float32Array[][]} inputs - Input audio data (channels x samples)
     * @param {Float32Array[][]} outputs - Output audio data (channels x samples)
     * @param {Object} parameters - Audio parameters (unused in this implementation)
     * @returns {boolean} - True to keep processor alive, false to terminate
     */
    process(inputs, outputs, parameters) {
        // Debug logging disabled - verification complete
        // if (this.chunkCounter < 5) {
        //     console.log(`PitchDetectionProcessor: process() called - chunk ${this.chunkCounter}, inputs: ${inputs.length}, outputs: ${outputs.length}`);
        // }
        
        const input = inputs[0];
        const output = outputs[0];
        
        // Debug logging disabled - remove spam
        // if (this.chunkCounter % 100 === 0) {
        //     console.log(`AudioWorklet: Processing chunk ${this.chunkCounter}, input channels: ${input ? input.length : 0}, processing: ${this.isProcessing}`);
        // }
        
        // Check if we have valid input
        if (!input || input.length === 0) {
            // No input available, pass through silence
            // Debug logging disabled
            // if (this.chunkCounter % 100 === 0) {
            //     console.log(`AudioWorklet: No input available - inputs array length: ${inputs.length}`);
            // }
            if (output && output.length > 0) {
                const outputChannel = output[0];
                if (outputChannel) {
                    outputChannel.fill(0);
                }
            }
            this.chunkCounter++;
            return true;
        }
        
        // Get first channel (mono processing for pitch detection)
        const inputChannel = input[0];
        
        if (!inputChannel || inputChannel.length !== this.chunkSize) {
            // Invalid input chunk size, pass through silence
            // Debug logging disabled
            // if (this.chunkCounter % 100 === 0) {
            //     console.log(`AudioWorklet: Invalid input - channel: ${!!inputChannel}, length: ${inputChannel ? inputChannel.length : 0}, expected: ${this.chunkSize}`);
            // }
            if (output && output.length > 0 && output[0]) {
                output[0].fill(0);
            }
            this.chunkCounter++;
            return true;
        }
        
        // Generate processed audio: test signal OR microphone input
        let processedAudio;
        if (this.testSignalConfig.enabled) {
            // Test signal is enabled - replace mic input with test signal
            processedAudio = this.generateTestSignal(this.chunkSize);
        } else {
            // Use microphone input
            processedAudio = new Float32Array(inputChannel);
        }
        
        // Generate and mix background noise (independent of test signal/mic)
        if (this.backgroundNoiseConfig.enabled) {
            const backgroundNoise = this.generateBackgroundNoise(this.chunkSize);
            
            // Mix background noise with the processed audio
            for (let i = 0; i < this.chunkSize; i++) {
                processedAudio[i] += backgroundNoise[i];
                
                // Clamp to valid range to prevent clipping
                processedAudio[i] = Math.max(-1.0, Math.min(1.0, processedAudio[i]));
            }
        }
        
        // Pass-through processed audio to output
        if (output && output.length > 0 && output[0]) {
            const outputChannel = output[0];
            if (outputChannel && outputChannel.length === processedAudio.length) {
                outputChannel.set(processedAudio);
            }
        }
        
        // Forward processed audio data to main thread for analysis
        if (this.isProcessing) {
            try {
                // Send the processed audio (test signal or mic input) for analysis
                const audioData = new Float32Array(processedAudio);
                
                this.port.postMessage({
                    type: 'audioData',
                    samples: audioData,
                    chunkSize: this.chunkSize,
                    chunkCounter: this.chunkCounter,
                    timestamp: this.currentTime || 0
                });
                
                this.chunkCounter++;
            } catch (error) {
                console.error('PitchDetectionProcessor: Error sending audio data:', error);
                
                // Send error notification to main thread
                this.port.postMessage({
                    type: 'processingError',
                    error: error.message,
                    timestamp: this.currentTime || 0
                });
            }
        }
        
        // Keep processor alive
        return true;
    }
    
    /**
     * Called when processor is being terminated
     */
    destroy() {
        this.isProcessing = false;
        this.port.postMessage({
            type: 'processorDestroyed',
            timestamp: this.currentTime || 0
        });
    }
}

// Register the processor with the AudioWorklet
registerProcessor('pitch-processor', PitchDetectionProcessor);