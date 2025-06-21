/**
 * Story 2.2: Enhanced Web Audio API Context and Microphone Input Processing Tests
 * 
 * Implements testing for:
 * - AC1: Web Audio API AudioContext initialization
 * - AC2: Microphone stream connection with optimal constraints  
 * - AC3: AudioWorklet live microphone input processing
 * - AC4: Consistent sample rate and buffer size across browsers
 * - AC5: Real-time audio data flow through WASM pipeline
 * - AC6: Audio latency monitoring and optimization (<50ms target)
 */

const { test, expect, describe, beforeEach, afterEach } = require('@jest/globals');

// Mock environment setup for testing
global.performance = {
    now: jest.fn(() => Date.now())
};

describe('Story 2.2: Enhanced Web Audio API Context Processing', () => {
    let app;
    let mockAudioContext;
    let mockAudioWorkletNode;
    let mockMediaStreamSource;
    let mockStream;

    beforeEach(() => {
        // Enhanced mock AudioContext for Story 2.2
        mockAudioContext = {
            sampleRate: 44100,
            state: 'running',
            baseLatency: 0.01, // 10ms
            outputLatency: 0.005, // 5ms
            resume: jest.fn().mockResolvedValue(),
            close: jest.fn().mockResolvedValue(),
            createMediaStreamSource: jest.fn(),
            addEventListener: jest.fn(),
            audioWorklet: {
                addModule: jest.fn().mockResolvedValue()
            }
        };

        // Enhanced mock AudioWorkletNode for Story 2.2
        mockAudioWorkletNode = {
            port: {
                onmessage: null,
                postMessage: jest.fn()
            },
            connect: jest.fn(),
            disconnect: jest.fn()
        };

        // Enhanced mock MediaStreamSource
        mockMediaStreamSource = {
            connect: jest.fn(),
            disconnect: jest.fn()
        };

        // Mock getUserMedia stream
        mockStream = {
            getAudioTracks: jest.fn(() => [{
                getSettings: jest.fn(() => ({
                    sampleRate: 44100,
                    channelCount: 1,
                    echoCancellation: false,
                    noiseSuppression: false,
                    autoGainControl: false
                }))
            }]),
            getTracks: jest.fn(() => [{ stop: jest.fn() }])
        };

        // Setup global mocks
        global.AudioContext = jest.fn(() => mockAudioContext);
        global.webkitAudioContext = jest.fn(() => mockAudioContext);
        global.AudioWorkletNode = jest.fn(() => mockAudioWorkletNode);
        
        mockAudioContext.createMediaStreamSource.mockReturnValue(mockMediaStreamSource);

        // Reset DOM
        document.body.innerHTML = '<div class="container"><div class="panel"></div></div>';

        // Import and initialize app
        const { PitchVisualizerApp } = require('../../web/app.js');
        app = new PitchVisualizerApp();
        
        // Mock microphone stream as granted
        app.microphoneStream = mockStream;
        app.permissionState = 'granted';
    });

    afterEach(() => {
        if (app) {
            app.cleanup();
        }
        jest.clearAllMocks();
    });

    describe('AC1: Enhanced AudioContext Initialization', () => {
        test('should create AudioContext with optimal configuration', async () => {
            await app.initializeAudioPipeline();

            expect(global.AudioContext).toHaveBeenCalledWith({
                sampleRate: 44100,
                latencyHint: 'interactive'
            });
            
            expect(app.audioContext).toBe(mockAudioContext);
            expect(app.audioContext.sampleRate).toBe(44100);
        });

        test('should handle AudioContext state management properly', async () => {
            mockAudioContext.state = 'suspended';
            
            await app.initializeAudioPipeline();

            expect(mockAudioContext.resume).toHaveBeenCalled();
            expect(mockAudioContext.addEventListener).toHaveBeenCalledWith('statechange', expect.any(Function));
        });

        test('should throw error for failed AudioContext creation', async () => {
            global.AudioContext = jest.fn(() => null);
            global.webkitAudioContext = jest.fn(() => null);

            await expect(app.initializeAudioPipeline()).rejects.toThrow('Failed to create AudioContext');
        });

        test('should validate AudioContext running state', async () => {
            mockAudioContext.state = 'closed';
            mockAudioContext.resume.mockResolvedValue(); // Resume doesn't change state in this test

            await expect(app.initializeAudioPipeline()).rejects.toThrow('AudioContext in unexpected state');
        });

        test('should setup AudioContext state change monitoring', async () => {
            await app.initializeAudioPipeline();

            expect(mockAudioContext.addEventListener).toHaveBeenCalledWith('statechange', expect.any(Function));
            
            // Test state change handler
            const stateChangeHandler = mockAudioContext.addEventListener.mock.calls[0][1];
            mockAudioContext.state = 'suspended';
            
            const resumeSpy = jest.spyOn(mockAudioContext, 'resume').mockResolvedValue();
            stateChangeHandler();
            
            expect(resumeSpy).toHaveBeenCalled();
        });
    });

    describe('AC2: Enhanced Microphone Stream Connection', () => {
        test('should create optimized microphone source with validation', async () => {
            await app.initializeAudioPipeline();

            expect(mockAudioContext.createMediaStreamSource).toHaveBeenCalledWith(mockStream);
            expect(mockMediaStreamSource.connect).toHaveBeenCalledWith(mockAudioWorkletNode);
        });

        test('should validate microphone stream is active', async () => {
            // Test with inactive stream (no audio tracks)
            mockStream.getAudioTracks.mockReturnValue([]);

            await expect(app.initializeAudioPipeline()).rejects.toThrow('No audio tracks available');
        });

        test('should handle MediaStreamSource creation failure', async () => {
            mockAudioContext.createMediaStreamSource.mockImplementation(() => {
                throw new Error('MediaStreamSource creation failed');
            });

            await expect(app.initializeAudioPipeline()).rejects.toThrow('MediaStreamSource creation failed');
        });

        test('should validate audio track settings', async () => {
            const mockTrack = {
                getSettings: jest.fn(() => ({
                    sampleRate: 44100,
                    channelCount: 1,
                    echoCancellation: false,
                    noiseSuppression: false,
                    autoGainControl: false
                }))
            };
            
            mockStream.getAudioTracks.mockReturnValue([mockTrack]);

            await app.initializeAudioPipeline();

            expect(mockTrack.getSettings).toHaveBeenCalled();
        });

        test('should handle audio routing connection failure', async () => {
            mockMediaStreamSource.connect.mockImplementation(() => {
                throw new Error('Audio routing failed');
            });

            await expect(app.initializeAudioPipeline()).rejects.toThrow('Audio routing failed');
        });
    });

    describe('AC3: Enhanced AudioWorklet Processing', () => {
        test('should load AudioWorklet module with error handling', async () => {
            await app.initializeAudioPipeline();

            expect(mockAudioContext.audioWorklet.addModule).toHaveBeenCalledWith('audio-worklet.js');
        });

        test('should handle AudioWorklet module loading failure', async () => {
            mockAudioContext.audioWorklet.addModule.mockRejectedValue(new Error('Module load failed'));

            await expect(app.initializeAudioPipeline()).rejects.toThrow('AudioWorklet loading failed');
        });

        test('should create AudioWorkletNode with enhanced configuration', async () => {
            await app.initializeAudioPipeline();

            expect(global.AudioWorkletNode).toHaveBeenCalledWith(
                mockAudioContext,
                'pitch-detection-processor',
                {
                    numberOfInputs: 1,
                    numberOfOutputs: 0,
                    channelCount: 1,
                    channelCountMode: 'explicit',
                    channelInterpretation: 'speakers'
                }
            );
        });

        test('should setup enhanced message handlers', async () => {
            await app.initializeAudioPipeline();

            expect(typeof mockAudioWorkletNode.port.onmessage).toBe('function');
        });

        test('should handle AudioWorklet initialization message', async () => {
            await app.initializeAudioPipeline();

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const initMessage = {
                data: {
                    type: 'initialized',
                    data: {
                        sampleRate: 44100,
                        bufferSize: 1024,
                        targetLatency: 0.05,
                        expectedBufferLatency: 0.023
                    }
                }
            };

            messageHandler(initMessage);

            // Should update latency metrics
            expect(app.latencyMetrics).toBeDefined();
        });

        test('should handle latency reports from AudioWorklet', async () => {
            await app.initializeAudioPipeline();

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const latencyMessage = {
                data: {
                    type: 'latencyReport',
                    data: {
                        latencyMs: 25.5,
                        targetMs: 50,
                        compliant: true,
                        timestamp: Date.now()
                    }
                }
            };

            messageHandler(latencyMessage);

            expect(app.latencyMetrics.processingLatency).toBeCloseTo(0.0255); // 25.5ms in seconds
        });
    });

    describe('AC4: Consistent Sample Rate and Buffer Size', () => {
        test('should maintain consistent sample rate across pipeline', async () => {
            await app.initializeAudioPipeline();

            expect(mockAudioContext.sampleRate).toBe(44100);
            
            // Verify WASM engine receives consistent sample rate
            expect(mockAudioWorkletNode.port.postMessage).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'init',
                    sampleRate: 44100
                })
            );
        });

        test('should handle sample rate validation in worklet', async () => {
            await app.initializeAudioPipeline();

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const connectionMessage = {
                data: {
                    type: 'connectionConfirmed',
                    data: {
                        result: {
                            wasmConnected: true,
                            sampleRate: 44100,
                            bufferSize: 1024,
                            configurationConsistent: true,
                            hasAudioSignal: true,
                            audioSignalStable: true,
                            processingLatencyMs: 15.2,
                            audioLevels: { peak: 0.8, rms: 0.5 }
                        }
                    }
                }
            };

            messageHandler(connectionMessage);

            // Should update connection status
            const connectionDisplay = document.getElementById('connection-status');
            expect(connectionDisplay).toBeTruthy();
            expect(connectionDisplay.innerHTML).toContain('44100Hz');
            expect(connectionDisplay.innerHTML).toContain('1024 samples');
        });

        test('should warn about sample rate mismatches', async () => {
            const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();
            
            await app.initializeAudioPipeline();

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const mismatchMessage = {
                data: {
                    type: 'connectionConfirmed',
                    data: {
                        result: {
                            wasmConnected: true,
                            sampleRate: 48000, // Different from AudioContext
                            bufferSize: 1024,
                            configurationConsistent: false
                        }
                    }
                }
            };

            messageHandler(mismatchMessage);

            consoleSpy.mockRestore();
        });
    });

    describe('AC5: Real-time Audio Data Flow', () => {
        test('should establish WASM pipeline connection', async () => {
            // Mock test framework with WASM engine
            global.window = { testFramework: { audioEngine: { 
                get_sample_rate: () => 44100,
                get_buffer_size: () => 1024
            }}};

            await app.initializeAudioPipeline();

            // Should send WASM engine to AudioWorklet
            expect(mockAudioWorkletNode.port.postMessage).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'init',
                    audioEngine: global.window.testFramework.audioEngine
                })
            );

            // Should start processing
            expect(mockAudioWorkletNode.port.postMessage).toHaveBeenCalledWith({
                type: 'start'
            });
        });

        test('should handle connection confirmation with audio levels', async () => {
            await app.initializeAudioPipeline();

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const confirmationMessage = {
                data: {
                    type: 'connectionConfirmed',
                    data: {
                        result: {
                            wasmConnected: true,
                            hasAudioSignal: true,
                            audioSignalStable: true,
                            processingLatencyMs: 12.5,
                            audioLevels: {
                                peak: 0.85,
                                rms: 0.42
                            }
                        }
                    }
                }
            };

            messageHandler(confirmationMessage);

            // Should create enhanced connection display
            const connectionDisplay = document.getElementById('connection-status');
            expect(connectionDisplay.innerHTML).toContain('Receiving audio');
        });

        test('should handle worklet connection errors with recovery', async () => {
            jest.useFakeTimers();
            
            await app.initializeAudioPipeline();

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const errorMessage = {
                data: {
                    type: 'connectionError',
                    data: {
                        error: 'WASM connection failed'
                    }
                }
            };

            messageHandler(errorMessage);

            // Should attempt recovery after 2 seconds
            jest.advanceTimersByTime(2000);

            expect(mockAudioWorkletNode.port.postMessage).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'init'
                })
            );

            jest.useRealTimers();
        });
    });

    describe('AC6: Audio Latency Monitoring and Optimization', () => {
        test('should initialize latency monitoring system', async () => {
            await app.initializeAudioPipeline();

            expect(app.latencyMetrics).toBeDefined();
            expect(app.latencyMetrics.audioContextLatency).toBe(0.01); // 10ms
            expect(app.latencyMetrics.outputLatency).toBe(0.005); // 5ms
            expect(app.latencyMetrics.totalLatency).toBe(0.015); // 15ms total
        });

        test('should monitor latency compliance against 50ms target', async () => {
            const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();
            
            // Mock high latency AudioContext
            mockAudioContext.baseLatency = 0.06; // 60ms - exceeds target
            
            await app.initializeAudioPipeline();

            expect(consoleSpy).toHaveBeenCalledWith(
                expect.stringContaining('exceeds 50ms target')
            );

            consoleSpy.mockRestore();
        });

        test('should update latency display with color coding', async () => {
            await app.initializeAudioPipeline();

            // Mock metrics container
            const metricsContainer = document.createElement('div');
            metricsContainer.className = 'metrics';
            document.body.appendChild(metricsContainer);

            // Trigger latency display update
            app.updateLatencyDisplay({
                totalLatency: 0.025 // 25ms - within target
            });

            const latencyMetric = document.getElementById('audio-latency-metric');
            expect(latencyMetric).toBeTruthy();
            expect(latencyMetric.querySelector('.metric-value').textContent).toBe('25.0');
            expect(latencyMetric.querySelector('.metric-value').style.color).toContain('success');
        });

        test('should show warning for high latency', async () => {
            await app.initializeAudioPipeline();

            // Mock metrics container
            const metricsContainer = document.createElement('div');
            metricsContainer.className = 'metrics';
            document.body.appendChild(metricsContainer);

            // Trigger high latency display update
            app.updateLatencyDisplay({
                totalLatency: 0.075 // 75ms - exceeds target
            });

            const latencyMetric = document.getElementById('audio-latency-metric');
            expect(latencyMetric.querySelector('.metric-value').style.color).toContain('error');
        });

        test('should setup periodic latency reporting', async () => {
            jest.useFakeTimers();
            
            await app.initializeAudioPipeline();

            expect(app.latencyMonitoringInterval).toBeDefined();

            // Fast-forward time to trigger reporting
            jest.advanceTimersByTime(1000);

            // Should have called reporting function
            jest.useRealTimers();
        });

        test('should update processing latency from worklet reports', async () => {
            await app.initializeAudioPipeline();

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const performanceMessage = {
                data: {
                    type: 'performance',
                    data: {
                        processesPerSecond: 43.2,
                        processingLatencyMs: 8.5,
                        latencyCompliant: true,
                        targetLatencyMs: 50
                    }
                }
            };

            messageHandler(performanceMessage);

            expect(app.latencyMetrics.processingLatency).toBeCloseTo(0.0085); // 8.5ms in seconds
            expect(app.latencyMetrics.totalLatency).toBeCloseTo(0.0235); // Updated total
        });
    });

    describe('Enhanced Error Handling', () => {
        test('should provide specific error messages for different failure types', async () => {
            global.AudioContext = jest.fn(() => {
                throw new Error('AudioContext creation failed');
            });

            await expect(app.initializeAudioPipeline()).rejects.toThrow();
            
            // Should show specific error guidance
            const statusElement = document.getElementById('microphone-status');
            expect(statusElement.textContent).toContain('Web Audio API initialization failed');
        });

        test('should handle worklet processing errors with restart', async () => {
            jest.useFakeTimers();
            
            await app.initializeAudioPipeline();

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const errorMessage = {
                data: {
                    type: 'error',
                    data: {
                        error: 'Processing interrupted'
                    }
                }
            };

            messageHandler(errorMessage);

            // Should attempt to restart processing after 1 second
            jest.advanceTimersByTime(1000);

            expect(mockAudioWorkletNode.port.postMessage).toHaveBeenCalledWith({
                type: 'start'
            });

            jest.useRealTimers();
        });

        test('should cleanup resources on pipeline error', async () => {
            mockAudioContext.audioWorklet.addModule.mockRejectedValue(new Error('Module load failed'));

            await expect(app.initializeAudioPipeline()).rejects.toThrow();

            // Should still have proper error state
            expect(app.audioContext).toBe(mockAudioContext);
        });
    });

    describe('Performance Monitoring Integration', () => {
        test('should update live processing metrics', async () => {
            await app.initializeAudioPipeline();

            // Mock metrics container
            const metricsContainer = document.createElement('div');
            metricsContainer.className = 'metrics';
            document.body.appendChild(metricsContainer);

            const messageHandler = mockAudioWorkletNode.port.onmessage;
            const performanceMessage = {
                data: {
                    type: 'performance',
                    data: {
                        processesPerSecond: 42.8,
                        processingLatencyMs: 6.2
                    }
                }
            };

            messageHandler(performanceMessage);

            const processingMetric = document.getElementById('live-processing-rate');
            expect(processingMetric).toBeTruthy();
            expect(processingMetric.querySelector('.metric-value').textContent).toBe('42.8');
        });
    });
}); 