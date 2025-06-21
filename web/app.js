/**
 * Main Application Controller for Real-time Pitch Visualizer
 * Story 2.1: Microphone Permission Request Flow
 * 
 * Handles microphone permissions, Web Audio API initialization,
 * and integration with existing WASM audio processing pipeline.
 */

class PitchVisualizerApp {
    constructor() {
        this.audioContext = null;
        this.microphoneStream = null;
        this.audioEngine = null;
        this.permissionState = 'unknown'; // 'unknown', 'requesting', 'granted', 'denied', 'error'
        
        // DOM elements for permission UI
        this.permissionModal = null;
        this.permissionButton = null;
        this.statusDisplay = null;
        
        // Story 2.3: Enhanced error handling and browser compatibility
        this.browserCapabilityDetector = null;
        this.errorManager = null;

        this.compatibilityReport = null;
        this.audioDeviceManager = null;
        this.wasmConnectionLogged = false;
        this.wasmConnectionFailureLogged = false;
        
        this.wasmAvailable = false;
        this.wasmAudioEngine = null;
        
        this.init();
    }

    async init() {
        console.log('🎵 Pitch Visualizer App initializing...');
        
        // Story 2.3: Initialize error handling and browser compatibility detection
        await this.initializeErrorHandling();
        
        // Check browser compatibility first
        const compatibilityCheck = await this.checkBrowserCompatibility();
        if (!compatibilityCheck.isSupported) {
            // Browser not supported - show upgrade guidance
            await this.handleUnsupportedBrowser(compatibilityCheck);
            return;
        }
        
        // Initialize UI components
        this.createPermissionUI();
        this.updatePermissionStatus('Ready to request microphone access');
        
        // Check if microphone permissions were previously granted
        this.checkExistingPermissions();
    }

    /**
     * Create child-friendly permission request UI
     * Implements AC1: Clear, child-friendly messaging
     * Implements AC2: Graceful permission denial handling
     */
    createPermissionUI() {
        // Create permission modal
        this.permissionModal = document.createElement('div');
        this.permissionModal.id = 'permission-modal';
        this.permissionModal.className = 'permission-modal';
        this.permissionModal.innerHTML = `
            <div class="permission-content">
                <div class="permission-icon">🎤</div>
                <h2>Let's Make Music Together!</h2>
                <p class="permission-message">
                    To help you learn music, we need to listen to your instrument or voice. 
                    This lets us show you if your notes are in tune! 🎵
                </p>
                <div class="permission-benefits">
                    <div class="benefit-item">
                        <span class="benefit-icon">🎯</span>
                        <span>See if your notes are perfectly in tune</span>
                    </div>
                    <div class="benefit-item">
                        <span class="benefit-icon">📊</span>
                        <span>Watch your pitch in real-time</span>
                    </div>
                    <div class="benefit-item">
                        <span class="benefit-icon">🎓</span>
                        <span>Learn music faster and better</span>
                    </div>
                </div>
                <div class="permission-actions">
                    <button id="grant-permission-btn" class="btn-primary permission-btn">
                        🎤 Let's Start!
                    </button>
                    <button id="maybe-later-btn" class="btn-secondary permission-btn">
                        Maybe Later
                    </button>
                </div>
                <div class="permission-note">
                    <small>🔒 We only use your microphone for pitch detection. No audio is recorded or saved.</small>
                </div>
            </div>
        `;

        // Create or find existing status display to avoid duplication
        this.statusDisplay = document.getElementById('microphone-status');
        if (!this.statusDisplay) {
            this.statusDisplay = document.createElement('div');
            this.statusDisplay.id = 'microphone-status';
            this.statusDisplay.className = 'status info';
            
            // Find or create status container in existing test interface
            const existingContainer = document.querySelector('.container .panel');
            if (existingContainer) {
                // Insert at the top of the panel, but after any existing status elements
                const firstChild = existingContainer.firstElementChild;
                if (firstChild && firstChild.classList.contains('status')) {
                    existingContainer.insertBefore(this.statusDisplay, firstChild.nextSibling);
                } else {
                    existingContainer.insertBefore(this.statusDisplay, firstChild);
                }
            } else {
                document.body.appendChild(this.statusDisplay);
            }
        }
        
        // Add permission modal to page
        document.body.appendChild(this.permissionModal);

        // Bind event listeners
        this.bindPermissionEvents();
    }

    /**
     * Bind event listeners for permission UI
     */
    bindPermissionEvents() {
        const grantBtn = document.getElementById('grant-permission-btn');
        const maybeLaterBtn = document.getElementById('maybe-later-btn');

        grantBtn?.addEventListener('click', () => this.requestMicrophonePermission());
        maybeLaterBtn?.addEventListener('click', () => this.handlePermissionDelay());
        
        // Close modal on background click
        this.permissionModal.addEventListener('click', (e) => {
            if (e.target === this.permissionModal) {
                this.handlePermissionDelay();
            }
        });
        
        // Keyboard accessibility
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.permissionModal.style.display === 'flex') {
                this.handlePermissionDelay();
            }
        });
    }

    /**
     * Check for existing microphone permissions
     */
    async checkExistingPermissions() {
        try {
            if (navigator.permissions && navigator.permissions.query) {
                const permissionStatus = await navigator.permissions.query({ name: 'microphone' });
                
                if (permissionStatus.state === 'granted') {
                    this.updatePermissionStatus('🎤 Microphone access granted! Getting audio stream...');
                    this.hidePermissionModal();
                    // Even though we have permission, we still need to get the actual stream
                    await this.requestMicrophoneStreamDirectly();
                } else if (permissionStatus.state === 'denied') {
                    this.showPermissionDeniedGuidance();
                } else {
                    this.showPermissionModal();
                }
                
                // Listen for permission changes
                permissionStatus.addEventListener('change', () => {
                    this.handlePermissionChange(permissionStatus.state);
                });
            } else {
                // Fallback for browsers without Permissions API
                this.showPermissionModal();
            }
        } catch (error) {
            console.warn('Could not check microphone permissions:', error);
            this.showPermissionModal();
        }
    }

    /**
     * Request microphone stream directly when permissions are already granted
     */
    async requestMicrophoneStreamDirectly() {
        try {
            // Check browser compatibility
            if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
                throw new Error('getUserMedia not supported in this browser');
            }

            // Request microphone access with optimal constraints
            const stream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    echoCancellation: false,  // We want raw audio for pitch detection
                    noiseSuppression: false, // Preserve musical nuances
                    autoGainControl: false,  // Maintain consistent levels
                    sampleRate: { ideal: 44100 }, // CD quality
                    channelCount: { ideal: 1 }    // Mono for pitch detection
                }
            });

            this.microphoneStream = stream;
            this.permissionState = 'granted';
            
            // Story 2.3: Set current device for monitoring
            if (this.audioDeviceManager && stream.getAudioTracks().length > 0) {
                const deviceId = stream.getAudioTracks()[0].getSettings().deviceId;
                this.audioDeviceManager.setCurrentDevice(deviceId);
                this.audioDeviceManager.startMonitoring();
            }
            
            this.updatePermissionStatus('✅ Audio stream obtained! Initializing pipeline...');
            
            // Initialize Web Audio API and WASM pipeline
            await this.initializeAudioPipeline();
            
        } catch (error) {
            console.error('Error getting microphone stream:', error);
            this.permissionState = 'error';
            this.handlePermissionError(error);
        }
    }

    /**
     * Request microphone permission using getUserMedia
     * Implements AC3: Web Audio API context initialization
     * Implements AC5: Cross-browser compatibility
     * Implements AC6: Error states with actionable feedback
     */
    async requestMicrophonePermission() {
        this.permissionState = 'requesting';
        this.updatePermissionStatus('🔄 Requesting microphone access...');
        
        const grantBtn = document.getElementById('grant-permission-btn');
        if (grantBtn) {
            grantBtn.disabled = true;
            grantBtn.innerHTML = '🔄 Requesting...';
        }

        try {
            // Check browser compatibility
            if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
                throw new Error('getUserMedia not supported in this browser');
            }

            // Request microphone access with optimal constraints
            const stream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    echoCancellation: false,  // We want raw audio for pitch detection
                    noiseSuppression: false, // Preserve musical nuances
                    autoGainControl: false,  // Maintain consistent levels
                    sampleRate: { ideal: 44100 }, // CD quality
                    channelCount: { ideal: 1 }    // Mono for pitch detection
                }
            });

            this.microphoneStream = stream;
            this.permissionState = 'granted';
            
            this.updatePermissionStatus('✅ Microphone access granted! Initializing audio...');
            this.hidePermissionModal();
            
            // Initialize Web Audio API and WASM pipeline
            await this.initializeAudioPipeline();
            
        } catch (error) {
            this.permissionState = 'denied';
            this.handlePermissionError(error);
        }
    }

    /**
     * Handle permission errors with specific guidance
     * Story 2.3: Enhanced with ErrorManager integration
     * Implements AC2: Graceful permission denial handling
     * Implements AC6: Actionable feedback for errors
     */
    async handlePermissionError(error) {
        console.error('Microphone permission error:', error);
        
        // Use new error manager if available
        if (this.errorManager) {
            await this.errorManager.handleError(error, 'Microphone Permission', {
                permissionState: this.permissionState,
                userAgent: navigator.userAgent
            });
        } else {
            // Fallback to existing error handling
            let errorMessage = '';
            let actionGuidance = '';
            
            if (error.name === 'NotAllowedError' || error.name === 'PermissionDeniedError') {
                errorMessage = '🚫 Microphone access was blocked';
                actionGuidance = this.getBrowserSpecificGuidance();
            } else if (error.name === 'NotFoundError') {
                errorMessage = '🎤 No microphone found';
                actionGuidance = 'Please connect a microphone and try again.';
            } else if (error.name === 'NotSupportedError') {
                errorMessage = '❌ Your browser doesn\'t support microphone access';
                actionGuidance = 'Please try using Chrome, Firefox, Safari, or Edge (latest versions).';
            } else {
                errorMessage = '⚠️ Something went wrong';
                actionGuidance = 'Please refresh the page and try again.';
            }

            this.showPermissionDeniedGuidance(errorMessage, actionGuidance);
        }
    }

    /**
     * Show permission denied guidance with browser-specific instructions
     */
    showPermissionDeniedGuidance(errorMessage = '🚫 Microphone access was blocked', actionGuidance = '') {
        this.permissionModal.innerHTML = `
            <div class="permission-content permission-denied">
                <div class="permission-icon">🔒</div>
                <h2>Oops! We Need Microphone Access</h2>
                <div class="error-message">${errorMessage}</div>
                <div class="guidance-section">
                    <h3>How to Enable Microphone Access:</h3>
                    <div class="browser-guidance">
                        ${actionGuidance || this.getBrowserSpecificGuidance()}
                    </div>
                </div>
                <div class="permission-actions">
                    <button id="retry-permission-btn" class="btn-primary permission-btn">
                        🔄 Try Again
                    </button>

                </div>
            </div>
        `;

        this.showPermissionModal();
        
        // Rebind events for new buttons
        document.getElementById('retry-permission-btn')?.addEventListener('click', () => {
            this.requestMicrophonePermission();
        });

    }

    /**
     * Get browser-specific guidance for enabling permissions
     */
    getBrowserSpecificGuidance() {
        const userAgent = navigator.userAgent.toLowerCase();
        
        if (userAgent.includes('chrome')) {
            return `
                <div class="browser-step">
                    <strong>Chrome:</strong> Click the 🎤 icon in the address bar, then select "Always allow"
                </div>
                <div class="browser-step">
                    Or go to Settings → Privacy and Security → Site Settings → Microphone
                </div>
            `;
        } else if (userAgent.includes('firefox')) {
            return `
                <div class="browser-step">
                    <strong>Firefox:</strong> Click the shield icon in the address bar
                </div>
                <div class="browser-step">
                    Or go to Settings → Privacy & Security → Permissions → Microphone
                </div>
            `;
        } else if (userAgent.includes('safari')) {
            return `
                <div class="browser-step">
                    <strong>Safari:</strong> Go to Safari → Settings → Websites → Microphone
                </div>
                <div class="browser-step">
                    Make sure this site is set to "Allow"
                </div>
            `;
        } else {
            return `
                <div class="browser-step">
                    Look for a 🎤 icon in your browser's address bar and click "Allow"
                </div>
                <div class="browser-step">
                    Or check your browser's privacy settings to enable microphone access for this site
                </div>
            `;
        }
    }

    /**
     * Initialize Web Audio API pipeline with enhanced configuration for Story 2.2
     * Implements AC1: AudioContext initialization with proper configuration
     * Implements AC2: Microphone stream connection with optimal constraints
     * Implements AC4: Consistent sample rate and buffer size across browsers
     * Implements AC6: Latency monitoring and optimization
     */
    async initializeAudioPipeline() {
        try {
            this.updatePermissionStatus('🔧 Setting up Web Audio API context...');
            
            // Enhanced AudioContext creation with optimal configuration
            const audioContextOptions = {
                sampleRate: 44100,  // Consistent sample rate for pitch detection
                latencyHint: 'interactive'  // Optimize for low latency
            };
            
            // Cross-browser AudioContext creation with fallback
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)(audioContextOptions);
            
            // Validate AudioContext creation
            if (!this.audioContext) {
                throw new Error('Failed to create AudioContext - Web Audio API not supported');
            }
            
            // Monitor initial AudioContext state
            console.log(`AudioContext created: ${this.audioContext.state}, SR: ${this.audioContext.sampleRate}Hz`);
            
            // Enhanced AudioContext state management with recovery
            await this.handleAudioContextState();
            
            this.updatePermissionStatus('🔧 Loading real-time audio processor...');
            
            // Load and register AudioWorklet processor with error handling
            try {
                await this.audioContext.audioWorklet.addModule('audio-worklet.js');
                console.log('AudioWorklet module loaded successfully');
            } catch (workletError) {
                console.error('Failed to load AudioWorklet:', workletError);
                throw new Error(`AudioWorklet loading failed: ${workletError.message}`);
            }
            
            // Create AudioWorklet node with enhanced configuration
            this.audioWorkletNode = new AudioWorkletNode(this.audioContext, 'pitch-detection-processor', {
                numberOfInputs: 1,
                numberOfOutputs: 0,  // No output needed, just processing
                channelCount: 1,     // Mono for pitch detection
                channelCountMode: 'explicit',
                channelInterpretation: 'speakers'
            });
            
            this.updatePermissionStatus('🔧 Connecting microphone with optimal settings...');
            
            // Enhanced microphone source creation with validation
            await this.createOptimizedMicrophoneSource();
            
            // Set up enhanced message handlers for AudioWorklet
            this.setupAudioWorkletHandlers();
            
            // Initialize latency monitoring
            this.initializeLatencyMonitoring();
            
            // Initialize the worklet with WASM engine
            this.initializeWorkletWithWASM();
            
            this.updatePermissionStatus('🎵 Real-time audio pipeline active! Monitoring performance...');
            console.log('Enhanced audio pipeline initialized successfully for Story 2.2');
            
            // Hide permission UI and show main interface
            this.showMainInterface();
            
        } catch (error) {
            console.error('Failed to initialize enhanced audio pipeline:', error);
            await this.handlePipelineError(error);
        }
    }

    /**
     * Enhanced AudioContext state management with recovery mechanisms
     * Implements AC1: Proper AudioContext state handling across browsers
     */
    async handleAudioContextState() {
        console.log(`Initial AudioContext state: ${this.audioContext.state}`);
        
        // Handle suspended state (browser autoplay policy)
        if (this.audioContext.state === 'suspended') {
            this.updatePermissionStatus('🔧 Activating audio context...');
            try {
                await this.audioContext.resume();
                console.log('AudioContext resumed successfully');
            } catch (resumeError) {
                console.error('Failed to resume AudioContext:', resumeError);
                throw new Error(`AudioContext resume failed: ${resumeError.message}`);
            }
        }
        
        // Validate final state is running
        if (this.audioContext.state !== 'running') {
            throw new Error(`AudioContext in unexpected state: ${this.audioContext.state}`);
        }
        
        // Set up state change monitoring
        this.audioContext.addEventListener('statechange', () => {
            console.log(`AudioContext state changed to: ${this.audioContext.state}`);
            if (this.audioContext.state === 'suspended') {
                this.handleAudioContextSuspension();
            }
        });
        
        console.log(`AudioContext ready: ${this.audioContext.sampleRate}Hz, state: ${this.audioContext.state}`);
    }

    /**
     * Create optimized microphone source with enhanced audio constraints
     * Implements AC2: Microphone stream connection with appropriate constraints
     * Implements AC4: Consistent audio configuration across browsers
     */
    async createOptimizedMicrophoneSource() {
        if (!this.microphoneStream) {
            throw new Error('Microphone stream not available');
        }
        
        // Validate microphone stream is active
        const audioTracks = this.microphoneStream.getAudioTracks();
        if (audioTracks.length === 0) {
            throw new Error('No audio tracks available in microphone stream');
        }
        
        const audioTrack = audioTracks[0];
        console.log('Audio track settings:', audioTrack.getSettings());
        
        // Create MediaStreamSource with validation
        try {
            this.microphoneSource = this.audioContext.createMediaStreamSource(this.microphoneStream);
            console.log('MediaStreamSource created successfully');
        } catch (sourceError) {
            console.error('Failed to create MediaStreamSource:', sourceError);
            throw new Error(`MediaStreamSource creation failed: ${sourceError.message}`);
        }
        
        // Connect microphone to AudioWorklet with enhanced routing
        try {
            this.microphoneSource.connect(this.audioWorkletNode);
            console.log('Microphone connected to AudioWorklet processor');
        } catch (connectionError) {
            console.error('Failed to connect microphone to processor:', connectionError);
            throw new Error(`Audio routing failed: ${connectionError.message}`);
        }
        
        // Validate audio routing
        this.validateAudioRouting();
    }

    /**
     * Initialize latency monitoring system
     * Implements AC6: Latency monitoring and optimization
     */
    initializeLatencyMonitoring() {
        this.latencyMetrics = {
            audioContextLatency: this.audioContext.baseLatency || 0,
            outputLatency: this.audioContext.outputLatency || 0,
            totalLatency: 0,
            processingLatency: 0,
            lastMeasurement: 0
        };
        
        // Calculate initial total latency
        this.latencyMetrics.totalLatency = 
            this.latencyMetrics.audioContextLatency + 
            this.latencyMetrics.outputLatency;
        
        console.log('Latency monitoring initialized:', this.latencyMetrics);
        
        // Set up periodic latency reporting
        this.latencyMonitoringInterval = setInterval(() => {
            this.reportLatencyMetrics();
        }, 1000);
        
        // Monitor for latency target compliance (AC6: <50ms)
        if (this.latencyMetrics.totalLatency > 0.05) { // 50ms in seconds
            console.warn(`Audio latency ${(this.latencyMetrics.totalLatency * 1000).toFixed(1)}ms exceeds 50ms target`);
        }
    }

    /**
     * Validate audio routing and configuration
     */
    validateAudioRouting() {
        if (!this.microphoneSource) {
            throw new Error('Microphone source not created');
        }
        
        if (!this.audioWorkletNode) {
            throw new Error('AudioWorklet node not created');
        }
        
        // Check if nodes are properly connected (basic validation)
        console.log('Audio routing validation passed');
    }

    /**
     * Handle AudioContext suspension events
     */
    handleAudioContextSuspension() {
        console.warn('AudioContext suspended - attempting recovery');
        this.updatePermissionStatus('⚠️ Audio context suspended - click to reactivate');
        
        // Attempt automatic recovery
        this.audioContext.resume().then(() => {
            console.log('AudioContext auto-recovery successful');
            this.updatePermissionStatus('🎵 Audio pipeline reactivated');
        }).catch(error => {
            console.error('AudioContext auto-recovery failed:', error);
            this.updatePermissionStatus('❌ Audio context recovery failed - please refresh page');
        });
    }

    /**
     * Enhanced pipeline error handling
     */
    async handlePipelineError(error) {
        console.error('Pipeline error:', error);
        
        // Provide specific error guidance
        let errorMessage = '❌ Audio pipeline setup failed: ';
        let actionGuidance = '';
        
        if (error.message.includes('AudioContext')) {
            errorMessage += 'Web Audio API initialization failed';
            actionGuidance = 'Your browser may not support Web Audio API. Please update your browser.';
        } else if (error.message.includes('AudioWorklet')) {
            errorMessage += 'Audio processor loading failed';
            actionGuidance = 'Please check your internet connection and try again.';
        } else if (error.message.includes('MediaStreamSource')) {
            errorMessage += 'Microphone connection failed';
            actionGuidance = 'Please ensure microphone access is granted and try again.';
        } else {
            errorMessage += error.message;
            actionGuidance = 'Please refresh the page and try again.';
        }
        
        this.updatePermissionStatus(errorMessage);
        
        // Show error guidance
        this.showPermissionDeniedGuidance(errorMessage, actionGuidance);
    }

    /**
     * Report latency metrics to UI
     */
    reportLatencyMetrics() {
        if (!this.latencyMetrics) return;
        
        // Update latency display in UI
        this.updateLatencyDisplay(this.latencyMetrics);
        
        // Check latency compliance
        const totalLatencyMs = this.latencyMetrics.totalLatency * 1000;
        if (totalLatencyMs > 50) {
            console.warn(`Latency warning: ${totalLatencyMs.toFixed(1)}ms exceeds 50ms target`);
        }
    }

    /**
     * Update latency display in UI
     */
    updateLatencyDisplay(metrics) {
        // Find or create latency metric display
        const metricsContainer = document.querySelector('.metrics');
        if (!metricsContainer) return;
        
        let latencyMetric = document.getElementById('audio-latency-metric');
        if (!latencyMetric) {
            latencyMetric = document.createElement('div');
            latencyMetric.id = 'audio-latency-metric';
            latencyMetric.className = 'metric';
            latencyMetric.innerHTML = `
                <div class="metric-value">-</div>
                <div class="metric-label">Audio Latency (ms)</div>
            `;
            metricsContainer.appendChild(latencyMetric);
        }
        
        // Update the metric value with color coding
        const valueElement = latencyMetric.querySelector('.metric-value');
        if (valueElement) {
            const totalLatencyMs = metrics.totalLatency * 1000;
            valueElement.textContent = totalLatencyMs.toFixed(1);
            
            // Color code based on performance target
            if (totalLatencyMs <= 50) {
                valueElement.style.color = 'var(--success)';
            } else {
                valueElement.style.color = 'var(--error)';
            }
        }
    }

    /**
     * Handle "Maybe Later" button click
     */
    handlePermissionDelay() {
        this.hidePermissionModal();
        this.updatePermissionStatus('⏳ Microphone access needed for live pitch detection. Click to request when ready.');
        
        // Add a button to the status to re-open permission flow
        this.statusDisplay.innerHTML += `
            <button id="request-mic-later" class="btn-primary" style="margin-top: 10px;">
                🎤 Enable Microphone Now
            </button>
        `;
        
        document.getElementById('request-mic-later')?.addEventListener('click', () => {
            this.showPermissionModal();
        });
    }

    /**
     * Handle permission state changes
     */
    handlePermissionChange(newState) {
        console.log('Permission state changed to:', newState);
        
        if (newState === 'granted' && this.permissionState !== 'granted') {
            this.updatePermissionStatus('✅ Microphone access granted!');
            this.hidePermissionModal();
        } else if (newState === 'denied') {
            this.showPermissionDeniedGuidance();
        }
    }

    /**
     * Show permission modal
     */
    showPermissionModal() {
        this.permissionModal.style.display = 'flex';
        
        // Focus management for accessibility
        const firstButton = this.permissionModal.querySelector('button');
        if (firstButton) {
            firstButton.focus();
        }
    }

    /**
     * Hide permission modal
     */
    hidePermissionModal() {
        this.permissionModal.style.display = 'none';
    }

    /**
     * Show main interface after permissions granted
     */
    showMainInterface() {
        // The main interface is already showing (test suite)
        // Add microphone indicator to show live status
        const indicator = document.createElement('div');
        indicator.id = 'microphone-indicator';
        indicator.className = 'status success';
        indicator.innerHTML = '🎤 <strong>Live Microphone Active</strong> - Ready for pitch detection';
        
        // Insert at top of existing interface
        const firstPanel = document.querySelector('.panel');
        if (firstPanel) {
            firstPanel.insertBefore(indicator, firstPanel.firstChild);
        }
    }

    /**
     * Update permission status display
     */
    updatePermissionStatus(message, priority = 'normal') {
        if (this.statusDisplay) {
            this.statusDisplay.innerHTML = `<strong>🎤 Microphone Status:</strong> ${message}`;
        }
    }

    /**
     * Cleanup resources
     */
    cleanup() {
        console.log('🧹 Cleaning up PitchVisualizerApp resources...');
        
        // Story 2.3: Cleanup audio device manager
        if (this.audioDeviceManager) {
            this.audioDeviceManager.cleanup();
            this.audioDeviceManager = null;
        }
        
        // Stop AudioWorklet processing
        if (this.audioWorkletNode) {
            this.audioWorkletNode.port.postMessage({ type: 'stop' });
            this.audioWorkletNode.disconnect();
            this.audioWorkletNode = null;
        }
        
        if (this.microphoneStream) {
            this.microphoneStream.getTracks().forEach(track => track.stop());
            this.microphoneStream = null;
        }
        
        if (this.audioContext) {
            this.audioContext.close();
            this.audioContext = null;
        }
        
        console.log('✅ Cleanup completed');
    }

    /**
     * Setup AudioWorklet message handlers for comprehensive processing coordination
     */
    setupAudioWorkletHandlers() {
        this.audioWorkletNode.port.onmessage = (event) => {
            const { type, data, ...rest } = event.data;
            
            switch (type) {
                case 'initialized':
                    console.log('🎯 Streamlined AudioWorklet initialized:', rest);
                    this.updateWorkletLatencyMetrics(rest.sampleRate, rest.bufferSize);
                    break;
                    
                case 'audioBuffer':
                    // Process audio data with WASM in main thread
                    this.processAudioWithWASM(event.data);
                    break;
                    
                case 'started':
                    console.log('🎤 Audio processing started');
                    break;
                    
                case 'stopped':
                    console.log('⏹️ Audio processing stopped');
                    break;
                    
                case 'performance':
                    this.updatePerformanceMetrics(rest);
                    break;
                    
                case 'error':
                    this.handleWorkletError(rest.error);
                    break;
                    
                default:
                    console.log('Unknown worklet message:', type, rest);
            }
        };
    }

    /**
     * Process audio data with WASM in main thread
     */
    processAudioWithWASM(audioData) {
        if (!this.wasmAvailable || !this.wasmAudioEngine) {
            // Fallback: basic processing without WASM
            this.handleProcessingResult({
                audioProcessed: false,
                pitchDetected: false,
                detectedFrequency: 0,
                wasmAvailable: false,
                processingTimeMs: 0
            });
            return;
        }

        try {
            const startTime = performance.now();
            
            // Process audio with WASM
            const result = this.wasmAudioEngine.process_realtime_audio(audioData.audioData);
            
            const processingTime = performance.now() - startTime;
            
            // Convert WASM result to expected format
            const processedResult = {
                audioProcessed: result.audio_processed,
                pitchDetected: result.pitch_detected,
                detectedFrequency: result.detected_frequency,
                pitchConfidence: result.pitch_confidence,
                processingTimeMs: processingTime,
                bufferLatencyMs: result.buffer_latency_ms,
                wasmAvailable: true,
                bufferSize: audioData.bufferSize
            };
            
            // Handle the processed result
            this.handleProcessingResult(processedResult);
            
        } catch (error) {
            console.error('WASM processing error:', error);
            this.handleWasmProcessingError(error, 'main_thread_processing');
        }
    }

    /**
     * Enhanced AudioWorklet setup with comprehensive handlers for Story 2.2
     * AFTER:  Direct WASM engine passing to AudioWorklet
     */
    initializeWorkletWithWASM() {
        console.log('🎯 Initializing AudioWorklet with main-thread WASM processing...');
        
        // Load WASM in main thread
        this.loadWASMEngine().then(() => {
            // Initialize worklet for audio collection
            this.audioWorkletNode.port.postMessage({
                type: 'init',
                sampleRate: this.audioContext.sampleRate,
                targetLatency: 0.05
            });
            
            // Start processing after a short delay
            setTimeout(() => {
                this.audioWorkletNode.port.postMessage({
                    type: 'start'
                });
                console.log('✅ AudioWorklet initialization started (Main-thread WASM mode)');
            }, 100);
        });
    }

    /**
     * Load WASM engine in main thread
     */
    async loadWASMEngine() {
        try {
            console.log('🦀 Loading WASM engine in main thread...');
            
            // Import WASM module - check for production vs development paths
            let wasmModule;
            let wasmPath;
            
            try {
                // Try production path first (files in root)
                wasmModule = await import('./pitch_toy.js');
                wasmPath = './pitch_toy_bg.wasm';
            } catch {
                // Fallback to development path (files in pkg/)
                wasmModule = await import('/pkg/pitch_toy.js');
                wasmPath = '/pkg/pitch_toy_bg.wasm';
            }
            
            await wasmModule.default(wasmPath);
            
            // Create AudioEngine instance
            this.wasmAudioEngine = new wasmModule.AudioEngine(
                this.audioContext.sampleRate, 
                1024 // buffer size
            );
            
            // Configure engine
            this.wasmAudioEngine.set_target_latency(0.05);
            this.wasmAudioEngine.update_latency_components(0, 0);
            
            console.log('✅ WASM AudioEngine loaded in main thread');
            this.wasmAvailable = true;
            
        } catch (error) {
            console.error('Failed to load WASM engine:', error);
            this.wasmAvailable = false;
        }
    }

    /**
     * Handle connection confirmation with enhanced validation for Story 2.2
     */
    handleConnectionConfirmation(data) {
        if (data.result) {
            const { wasmConnected, hasAudioSignal, bufferSize, sampleRate, wasmProcessing } = data.result;
            
            if (wasmConnected) {
                // Only log connection success once
                if (!this.wasmConnectionLogged) {
                    console.log(`✅ WASM pipeline connected: ${sampleRate}Hz, ${bufferSize} samples`);
                    this.wasmConnectionLogged = true;
                }
                
                // Update connection status with enhanced information
                this.updateConnectionStatus(wasmConnected, hasAudioSignal, sampleRate, bufferSize, wasmProcessing);
                
                // Update latency calculations with buffer size
                this.updateBufferLatencyMetrics(bufferSize, sampleRate);
            } else {
                // Show WASM connection status in UI rather than flooding console
                if (!this.wasmConnectionFailureLogged) {
                    console.warn('⚠️ WASM pipeline connection not fully established');
                    this.wasmConnectionFailureLogged = true;
                    this.updatePermissionStatus('🔄 Audio processing pipeline connecting...', 'warning');
                    
                    // Show status in connection display
                    const connectionDisplay = document.getElementById('connection-status');
                    if (connectionDisplay) {
                        connectionDisplay.innerHTML = `
                            🔄 <strong>Initializing:</strong> WASM audio engine connecting...<br>
                            📊 <strong>Audio Config:</strong> ${sampleRate || 'N/A'}Hz, ${bufferSize || 'N/A'} samples<br>
                            🎤 <strong>Microphone:</strong> ${hasAudioSignal ? 'Receiving audio' : 'Waiting for audio'}<br>
                            ⚠️ <strong>Status:</strong> Pipeline initialization in progress
                        `;
                        connectionDisplay.className = 'status warning';
                    }
                }
            }
        }
    }

    /**
     * Update connection status display with enhanced information for Story 2.2 (throttled)
     */
    updateConnectionStatus(wasmConnected, hasAudioSignal, sampleRate, bufferSize, wasmProcessing) {
        
        // Find or create connection status display
        let connectionDisplay = document.getElementById('connection-status');
        if (!connectionDisplay) {
            connectionDisplay = document.createElement('div');
            connectionDisplay.id = 'connection-status';
            connectionDisplay.className = 'status success';
            connectionDisplay.style.marginTop = '10px';
            connectionDisplay.style.minHeight = '100px'; // Fixed height to prevent jumping
            
            // Add to existing panel
            const statusContainer = this.statusDisplay.parentNode;
            if (statusContainer) {
                statusContainer.appendChild(connectionDisplay);
            }
        }
        
        // Calculate buffer latency for display
        const bufferLatencyMs = bufferSize && sampleRate ? (bufferSize / sampleRate * 1000).toFixed(1) : 'N/A';
        
        // AC5: Enhanced connection confirmation with WASM processing status
        let wasmProcessingStatus = '';
        if (wasmProcessing) {
            const processingLatency = wasmProcessing.wasmProcessingLatency ? 
                `${wasmProcessing.wasmProcessingLatency.toFixed(2)}ms` : 'N/A';
            const pitchLatency = wasmProcessing.wasmPitchLatency ? 
                `${wasmProcessing.wasmPitchLatency.toFixed(2)}ms` : 'N/A';
            
            wasmProcessingStatus = `
                <br>🔄 <strong>WASM Processing:</strong> ${wasmProcessing.pipelineActive ? 'Active' : 'Inactive'}
                <br>🎯 <strong>Pitch Detection:</strong> ${wasmProcessing.pitchDetected ? 
                    `${wasmProcessing.detectedFrequency.toFixed(1)}Hz` : 'No pitch detected'}
                <br>⚡ <strong>WASM Latency:</strong> Process=${processingLatency}, Pitch=${pitchLatency}
            `;
        }
        
        // Show enhanced connection confirmation
        connectionDisplay.innerHTML = `
            ✅ <strong>Pipeline Connected:</strong> WASM audio engine ready<br>
            📊 <strong>Audio Config:</strong> ${sampleRate}Hz, ${bufferSize} samples<br>
            ⚡ <strong>Buffer Latency:</strong> ${bufferLatencyMs}ms<br>
            🎤 <strong>Microphone:</strong> ${hasAudioSignal ? 'Receiving audio' : 'Waiting for audio'}${wasmProcessingStatus}
        `;
        
        // Update main status
        this.updatePermissionStatus('🎵 Real-time audio pipeline successfully connected!', 'success');
    }

    /**
     * Enhanced performance metrics with latency tracking for Story 2.2
     */
    updatePerformanceMetrics(data) {
        // Update the existing performance dashboard with enhanced live metrics
        const metricsContainer = document.querySelector('.metrics');
        if (metricsContainer) {
            // Find or create live processing metric
            let liveMetric = document.getElementById('live-processing-rate');
            if (!liveMetric) {
                liveMetric = document.createElement('div');
                liveMetric.id = 'live-processing-rate';
                liveMetric.className = 'metric';
                liveMetric.innerHTML = `
                    <div class="metric-value">-</div>
                    <div class="metric-label">Processing Rate (Hz)</div>
                `;
                metricsContainer.appendChild(liveMetric);
            }
            
            // Update the metric value
            const valueElement = liveMetric.querySelector('.metric-value');
            if (valueElement) {
                valueElement.textContent = data.processesPerSecond.toFixed(1);
            }
        }
        
        // Update latency metrics if processing latency is provided
        if (data.processingLatencyMs && this.latencyMetrics) {
            this.latencyMetrics.processingLatency = data.processingLatencyMs / 1000; // Convert to seconds
            this.latencyMetrics.totalLatency = 
                this.latencyMetrics.audioContextLatency + 
                this.latencyMetrics.outputLatency + 
                this.latencyMetrics.processingLatency;
        }
    }

    /**
     * Update worklet-specific latency metrics
     */
    updateWorkletLatencyMetrics(sampleRate, bufferSize) {
        if (this.latencyMetrics) {
            // Calculate buffer-based latency contribution
            const bufferLatency = bufferSize / sampleRate;
            this.latencyMetrics.processingLatency = bufferLatency;
            this.latencyMetrics.totalLatency = 
                this.latencyMetrics.audioContextLatency + 
                this.latencyMetrics.outputLatency + 
                this.latencyMetrics.processingLatency;
            
            console.log(`Worklet latency updated: ${(bufferLatency * 1000).toFixed(1)}ms buffer latency`);
        }
    }

    /**
     * Update buffer-based latency calculations
     */
    updateBufferLatencyMetrics(bufferSize, sampleRate) {
        if (this.latencyMetrics && bufferSize && sampleRate) {
            const bufferLatency = bufferSize / sampleRate;
            this.latencyMetrics.processingLatency = bufferLatency;
            this.latencyMetrics.totalLatency = 
                this.latencyMetrics.audioContextLatency + 
                this.latencyMetrics.outputLatency + 
                this.latencyMetrics.processingLatency;
        }
    }

    /**
     * Update processing latency from real-time measurements
     */
    updateProcessingLatency(data) {
        if (this.latencyMetrics && data.latencyMs) {
            this.latencyMetrics.processingLatency = data.latencyMs / 1000;
            this.latencyMetrics.totalLatency = 
                this.latencyMetrics.audioContextLatency + 
                this.latencyMetrics.outputLatency + 
                this.latencyMetrics.processingLatency;
                
            // Update UI immediately for real-time feedback
            this.updateLatencyDisplay(this.latencyMetrics);
        }
    }

    /**
     * Handle worklet connection errors
     */
    handleWorkletConnectionError(error) {
        console.error('Worklet connection error:', error);
        // Attempt recovery if possible
        setTimeout(() => {
            console.log('Attempting worklet connection recovery...');
            this.initializeWorkletWithWASM();
        }, 2000);
    }

    /**
     * 🎯 UNIFIED Processing Result Handler - Phase 4 Refactoring
     * 
     * BEFORE: Separate handlers for different result types
     * AFTER:  Single comprehensive handler for all processing results
     */
    handleProcessingResult(result) {
        if (!result) return;
        
        // 📊 Update comprehensive processing display
        this.updateProcessingDisplay(result);
        
        // 📈 Handle performance metrics if available
        if (result.performanceMetrics) {
            this.updatePerformanceMetrics(result.performanceMetrics);
        }
        
        // 🎯 Handle pitch detection results
        if (result.pitchDetected) {
            this.handlePitchDetection(result.detectedFrequency, result.pitchConfidence);
        }
        
        // ⚡ Update latency metrics
        if (result.processingTimeMs || result.bufferLatencyMs) {
            this.updateLatencyMetrics(result);
        }
    }

    /**
     * Handle WASM processing errors (AC5)
     */
    handleWasmProcessingError(error, context) {
        console.error(`WASM processing error in ${context}:`, error);
        this.updatePermissionStatus(`❌ WASM processing error: ${error}`);
        
        // Create error display for AC5 testing visibility
        this.showWasmProcessingError(error, context);
    }

    /**
     * 🎯 UNIFIED Processing Display - Phase 4 Refactoring
     * 
     * BEFORE: Separate WASM processing display
     * AFTER:  Comprehensive processing display with all metrics
     */
    updateProcessingDisplay(result) {
        // Throttle updates to prevent UI flickering
        if (!this.lastDisplayUpdate || Date.now() - this.lastDisplayUpdate > 100) {
            this.lastDisplayUpdate = Date.now();
            
            // Find or create processing display
            let processingDisplay = document.getElementById('processing-status');
            if (!processingDisplay) {
                processingDisplay = document.createElement('div');
                processingDisplay.id = 'processing-status';
                processingDisplay.className = 'status info';
                processingDisplay.style.marginTop = '10px';
                processingDisplay.style.minHeight = '120px';
                
                const statusContainer = this.statusDisplay.parentNode;
                if (statusContainer) {
                    statusContainer.appendChild(processingDisplay);
                }
            }
            
            // Show comprehensive real-time processing results
            const timestamp = new Date().toLocaleTimeString();
            const wasmStatus = this.wasmAvailable ? '✅ Active' : '❌ Unavailable';
            
            processingDisplay.innerHTML = `
                <strong>🎯 Real-time Processing (Phase 4)</strong><br>
                ⏰ <strong>Last Update:</strong> ${timestamp}<br>
                🔧 <strong>WASM Engine:</strong> ${wasmStatus}<br>
                📊 <strong>Audio Processed:</strong> ${result.audioProcessed ? 'Yes' : 'No'}<br>
                🎯 <strong>Pitch Detection:</strong> ${result.pitchDetected ? 
                    `${result.detectedFrequency.toFixed(1)}Hz (${(result.pitchConfidence * 100).toFixed(1)}%)` : 'No pitch'}<br>
                ⚡ <strong>Processing Time:</strong> ${result.processingTimeMs?.toFixed(2) || 'N/A'}ms<br>
                📡 <strong>Buffer Latency:</strong> ${result.bufferLatencyMs?.toFixed(1) || 'N/A'}ms<br>
                📏 <strong>Buffer Size:</strong> ${result.bufferSize || 'N/A'} samples
            `;
        }
    }

    /**
     * Show WASM processing errors for AC5 testing
     */
    showWasmProcessingError(error, context) {
        // Find or create error display
        let errorDisplay = document.getElementById('wasm-error-status');
        if (!errorDisplay) {
            errorDisplay = document.createElement('div');
            errorDisplay.id = 'wasm-error-status';
            errorDisplay.className = 'status error';
            errorDisplay.style.marginTop = '10px';
            
            const statusContainer = this.statusDisplay.parentNode;
            if (statusContainer) {
                statusContainer.appendChild(errorDisplay);
            }
        }
        
        const timestamp = new Date().toLocaleTimeString();
        errorDisplay.innerHTML = `
            <strong>❌ WASM Processing Error (AC5)</strong><br>
            ⏰ <strong>Time:</strong> ${timestamp}<br>
            🔧 <strong>Context:</strong> ${context}<br>
            ⚠️ <strong>Error:</strong> ${error}
        `;
        
        // Auto-hide error after 10 seconds
        setTimeout(() => {
            if (errorDisplay.parentNode) {
                errorDisplay.parentNode.removeChild(errorDisplay);
            }
        }, 10000);
    }

    /**
     * 🎯 Handle pitch detection results
     */
    handlePitchDetection(frequency, confidence) {
        // Update pitch display if it exists
        const pitchDisplay = document.getElementById('pitch-display');
        if (pitchDisplay) {
            pitchDisplay.textContent = `${frequency.toFixed(1)}Hz (${(confidence * 100).toFixed(1)}%)`;
        }
        
        // Log significant pitch detections (throttled)
        if (!this.lastPitchLog || Date.now() - this.lastPitchLog > 1000) {
            console.log(`🎯 Pitch detected: ${frequency.toFixed(1)}Hz`);
            this.lastPitchLog = Date.now();
        }
    }

    /**
     * ⚡ Update latency metrics from processing results
     */
    updateLatencyMetrics(result) {
        if (result.processingTimeMs !== undefined) {
            this.lastProcessingLatency = result.processingTimeMs;
        }
        
        if (result.bufferLatencyMs !== undefined) {
            this.lastBufferLatency = result.bufferLatencyMs;
        }
        
        // Update latency display if it exists
        const latencyDisplay = document.getElementById('latency-display');
        if (latencyDisplay) {
            const totalLatency = (this.lastProcessingLatency || 0) + (this.lastBufferLatency || 0);
            latencyDisplay.textContent = `${totalLatency.toFixed(1)}ms`;
        }
    }

    /**
     * Handle general worklet errors
     */
    handleWorkletError(error) {
        console.error('Worklet processing error:', error);
        this.updatePermissionStatus('❌ Audio processing interrupted - attempting recovery');
        
        // Attempt to restart processing
        if (this.audioWorkletNode) {
            setTimeout(() => {
                this.audioWorkletNode.port.postMessage({
                    type: 'start'
                });
            }, 1000);
        }
    }

    /**
     * Story 2.3: Initialize error handling and browser compatibility systems
     */
    async initializeErrorHandling() {
        console.log('🔧 Starting error handling initialization...');
        
        try {
            // Initialize browser capability detector
            if (window.BrowserCapabilityDetector) {
                this.browserCapabilityDetector = new window.BrowserCapabilityDetector();
                console.log('✅ Browser capability detector initialized');
            } else {
                console.warn('⚠️ BrowserCapabilityDetector not found');
            }
            
            // Initialize error manager
            if (window.ErrorManager) {
                this.errorManager = new window.ErrorManager();
                console.log('✅ Error handling system initialized');
            } else {
                console.warn('⚠️ ErrorManager not found');
            }
            
            
            
            // Initialize audio device manager
            console.log('🔍 Checking for AudioDeviceManager...', !!window.AudioDeviceManager);
            if (window.AudioDeviceManager) {
                console.log('🎯 Creating AudioDeviceManager instance...');
                this.audioDeviceManager = new window.AudioDeviceManager(this);
                console.log('✅ Audio device manager initialized');
                console.log('🔍 Device manager available at: window.pitchApp.audioDeviceManager');
                console.log('🔍 Verification - audioDeviceManager exists:', !!this.audioDeviceManager);
            } else {
                console.warn('❌ AudioDeviceManager class not found - script may not be loaded');
            }
            
            console.log('🏁 Error handling initialization complete');
            
        } catch (error) {
            console.error('Failed to initialize error handling:', error);
            console.error('Error stack:', error.stack);
            // Continue without advanced error handling
        }
    }
    
    /**
     * Story 2.3: Check comprehensive browser compatibility
     */
    async checkBrowserCompatibility() {
        if (!this.browserCapabilityDetector) {
            console.warn('Browser capability detector not available - using basic checks');
            return this.performBasicCompatibilityCheck();
        }
        
        try {
            this.compatibilityReport = await this.browserCapabilityDetector.detectCapabilities();
            
            console.log('🔍 Browser compatibility report:', this.compatibilityReport);
            
            if (!this.compatibilityReport.isSupported) {
                console.warn('❌ Browser compatibility issues:', this.compatibilityReport.unsupportedFeatures);
            } else if (this.compatibilityReport.warnings.length > 0) {
                console.warn('⚠️ Browser compatibility warnings:', this.compatibilityReport.warnings);
            }
            
            return this.compatibilityReport;
            
        } catch (error) {
            console.error('Browser compatibility check failed:', error);
            if (this.errorManager) {
                await this.errorManager.handleError(error, 'Browser Compatibility Check');
            }
            return { isSupported: false, error: error };
        }
    }
    
    /**
     * Basic compatibility check fallback
     */
    performBasicCompatibilityCheck() {
        const hasWebAudio = !!(window.AudioContext || window.webkitAudioContext);
        const hasWebAssembly = typeof WebAssembly === 'object';
        const hasGetUserMedia = !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia);
        
        const isSupported = hasWebAudio && hasWebAssembly && hasGetUserMedia;
        
        return {
            isSupported: isSupported,
            capabilities: {
                webAudio: { supported: hasWebAudio },
                webAssembly: { supported: hasWebAssembly },
                getUserMedia: { supported: hasGetUserMedia }
            },
            unsupportedFeatures: [
                !hasWebAudio ? 'Web Audio API' : null,
                !hasWebAssembly ? 'WebAssembly' : null,
                !hasGetUserMedia ? 'getUserMedia' : null
            ].filter(Boolean)
        };
    }
    
    /**
     * Story 2.3: Handle unsupported browser scenario
     */
    async handleUnsupportedBrowser(compatibilityReport) {
        console.log('🚫 Browser not supported, activating fallback mode');
        
        if (this.errorManager) {
            // Use error manager to handle this gracefully
            const error = new Error(`Unsupported browser features: ${compatibilityReport.unsupportedFeatures?.join(', ')}`);
            error.name = 'BrowserCompatibilityError';
            await this.errorManager.handleError(error, 'Browser Compatibility', {
                compatibilityReport: compatibilityReport
            });
        } else {
            this.showBasicUnsupportedMessage(compatibilityReport);
        }
    }
    
    /**
     * Show basic unsupported browser message
     */
    showBasicUnsupportedMessage(compatibilityReport) {
        const message = document.createElement('div');
        message.className = 'unsupported-browser-message';
        message.innerHTML = `
            <div class="unsupported-content">
                <h2>🚀 Browser Upgrade Needed</h2>
                <p>This application requires modern browser features that aren't supported in your current browser.</p>
                <div class="missing-features">
                    <h3>Missing features:</h3>
                    <ul>
                        ${compatibilityReport.unsupportedFeatures?.map(feature => `<li>${feature}</li>`).join('') || '<li>Unknown compatibility issues</li>'}
                    </ul>
                </div>
                <div class="browser-recommendations">
                    <h3>Recommended browsers:</h3>
                    <div class="browser-links">
                        <a href="https://www.google.com/chrome/" target="_blank" class="browser-link">Chrome 66+</a>
                        <a href="https://www.mozilla.org/firefox/" target="_blank" class="browser-link">Firefox 76+</a>
                        <a href="https://www.apple.com/safari/" target="_blank" class="browser-link">Safari 14+</a>
                    </div>
                </div>
            </div>
        `;
        document.body.appendChild(message);
    }
}

// Initialize PitchVisualizerApp once when DOM is ready
// This ensures only one instance is created
if (!window.pitchApp) {
    document.addEventListener('DOMContentLoaded', () => {
        window.pitchApp = new PitchVisualizerApp();
        console.log('🎵 PitchVisualizerApp initialized - waiting for WASM foundation...');
    });
}

// Cleanup on page unload
window.addEventListener('beforeunload', () => {
    if (window.pitchApp) {
        window.pitchApp.cleanup();
    }
}); 