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
        
        this.init();
    }

    async init() {
        console.log('🎵 Pitch Visualizer App initializing...');
        
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

        // Create status display
        this.statusDisplay = document.createElement('div');
        this.statusDisplay.id = 'microphone-status';
        this.statusDisplay.className = 'status info';
        
        // Add to page
        document.body.appendChild(this.permissionModal);
        
        // Find or create status container in existing test interface
        const existingContainer = document.querySelector('.container .panel');
        if (existingContainer) {
            existingContainer.insertBefore(this.statusDisplay, existingContainer.firstChild);
        } else {
            document.body.appendChild(this.statusDisplay);
        }

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
                    this.updatePermissionStatus('🎤 Microphone access already granted!');
                    this.hidePermissionModal();
                    await this.initializeAudioPipeline();
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
     * Implements AC2: Graceful permission denial handling
     * Implements AC6: Actionable feedback for errors
     */
    handlePermissionError(error) {
        console.error('Microphone permission error:', error);
        
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
                    <button id="demo-mode-btn" class="btn-secondary permission-btn">
                        🎮 Demo Mode
                    </button>
                </div>
            </div>
        `;

        this.showPermissionModal();
        
        // Rebind events for new buttons
        document.getElementById('retry-permission-btn')?.addEventListener('click', () => {
            this.requestMicrophonePermission();
        });
        
        document.getElementById('demo-mode-btn')?.addEventListener('click', () => {
            this.startDemoMode();
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
     * Initialize Web Audio API context and connect to WASM pipeline
     * Implements AC4: Connect to audio processing pipeline from EP-001
     */
    async initializeAudioPipeline() {
        try {
            this.updatePermissionStatus('🔧 Setting up audio processing...');
            
            // Create AudioContext
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)({
                sampleRate: 44100,
                latencyHint: 'interactive' // Optimize for low latency
            });

            // Resume context if needed (browser autoplay policy)
            if (this.audioContext.state === 'suspended') {
                await this.audioContext.resume();
            }

            // Load and register AudioWorklet processor
            this.updatePermissionStatus('🔧 Loading audio processor...');
            await this.audioContext.audioWorklet.addModule('audio-worklet.js');
            
            // Create AudioWorklet node
            this.audioWorkletNode = new AudioWorkletNode(this.audioContext, 'pitch-detection-processor');
            
            // Create MediaStreamSource from microphone
            const microphoneSource = this.audioContext.createMediaStreamSource(this.microphoneStream);
            
            // Connect microphone to AudioWorklet processor
            microphoneSource.connect(this.audioWorkletNode);
            
            // Set up message handling for AudioWorklet
            this.setupAudioWorkletHandlers();
            
            // Initialize the worklet with WASM engine (will be created in existing test framework)
            this.initializeWorkletWithWASM();
            
            this.updatePermissionStatus('🎵 Audio pipeline ready! You can now use live microphone input.');
            console.log('Audio pipeline initialized successfully');
            
            // Hide permission UI and show main interface
            this.showMainInterface();
            
        } catch (error) {
            console.error('Failed to initialize audio pipeline:', error);
            this.updatePermissionStatus('❌ Failed to set up audio processing. Please try again.');
        }
    }

    /**
     * Set up message handlers for AudioWorklet communication
     */
    setupAudioWorkletHandlers() {
        this.audioWorkletNode.port.onmessage = (event) => {
            const { type, data } = event.data;
            
            switch (type) {
                case 'initialized':
                    console.log('AudioWorklet initialized:', data);
                    this.updatePermissionStatus('🎵 Real-time audio processing active!');
                    break;
                    
                case 'connectionConfirmed':
                    // Handle connection confirmation (Story 2.1 scope)
                    this.handleConnectionConfirmation(data);
                    break;
                    
                case 'connectionError':
                    // Handle connection errors (Story 2.1 scope)
                    console.error('Connection error:', data.error);
                    this.updatePermissionStatus('❌ Pipeline connection error: ' + data.error);
                    break;
                    
                case 'performance':
                    // Update performance metrics
                    this.updatePerformanceMetrics(data);
                    break;
                    
                case 'error':
                    console.error('AudioWorklet error:', data.error);
                    this.updatePermissionStatus('❌ Audio processing error: ' + data.error);
                    break;
                    
                default:
                    console.log('AudioWorklet message:', event.data);
            }
        };
    }

    /**
     * Initialize AudioWorklet with WASM engine
     * Integrates with existing test framework's AudioEngine
     */
    initializeWorkletWithWASM() {
        // Wait for existing test framework to initialize WASM
        // This integrates with the existing Story 1.2 implementation
        const checkForWASM = () => {
            if (window.testFramework?.audioEngine) {
                // Send WASM engine to AudioWorklet
                this.audioWorkletNode.port.postMessage({
                    type: 'init',
                    audioEngine: window.testFramework.audioEngine
                });
                
                // Start processing
                this.audioWorkletNode.port.postMessage({
                    type: 'start'
                });
                
                console.log('🎵 WASM AudioEngine connected to live microphone input');
            } else {
                // Retry after 100ms
                setTimeout(checkForWASM, 100);
            }
        };
        
        checkForWASM();
    }

    /**
     * Handle connection confirmation from AudioWorklet (Story 2.1 scope)
     */
    handleConnectionConfirmation(data) {
        // Story 2.1: Just confirm connection is established
        if (data.result) {
            const { wasmConnected, hasAudioSignal, bufferSize, sampleRate } = data.result;
            
            if (wasmConnected) {
                console.log(`✅ WASM pipeline connected: ${sampleRate}Hz, ${bufferSize} samples`);
                
                // Update UI to show successful connection
                this.updateConnectionStatus(wasmConnected, hasAudioSignal, sampleRate, bufferSize);
            } else {
                console.log('❌ WASM pipeline connection failed');
                this.updatePermissionStatus('❌ Failed to connect to audio processing pipeline');
            }
        }
    }

    /**
     * Update connection status display (Story 2.1 scope)
     */
    updateConnectionStatus(wasmConnected, hasAudioSignal, sampleRate, bufferSize) {
        // Find or create connection status display
        let connectionDisplay = document.getElementById('connection-status');
        if (!connectionDisplay) {
            connectionDisplay = document.createElement('div');
            connectionDisplay.id = 'connection-status';
            connectionDisplay.className = 'status success';
            connectionDisplay.style.marginTop = '10px';
            
            // Add to existing panel
            const statusContainer = this.statusDisplay.parentNode;
            if (statusContainer) {
                statusContainer.appendChild(connectionDisplay);
            }
        }
        
        // Show connection confirmation
        connectionDisplay.innerHTML = `
            ✅ <strong>Pipeline Connected:</strong> WASM audio engine ready<br>
            📊 <strong>Audio Config:</strong> ${sampleRate}Hz, ${bufferSize} samples<br>
            🎤 <strong>Microphone:</strong> ${hasAudioSignal ? 'Receiving audio' : 'Waiting for audio'}
        `;
        
        // Update main status
        this.updatePermissionStatus('🎵 Audio pipeline successfully connected! Ready for processing.');
    }

    /**
     * Update performance metrics from AudioWorklet
     */
    updatePerformanceMetrics(data) {
        // Update the existing performance dashboard with live metrics
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
                    <div class="metric-label">Live Processing (Hz)</div>
                `;
                metricsContainer.appendChild(liveMetric);
            }
            
            // Update the metric value
            const valueElement = liveMetric.querySelector('.metric-value');
            if (valueElement) {
                valueElement.textContent = data.processesPerSecond.toFixed(1);
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
     * Start demo mode with synthetic audio (fallback)
     */
    startDemoMode() {
        this.hidePermissionModal();
        this.updatePermissionStatus('🎮 Demo mode: Using synthetic audio for testing (no microphone needed)');
        
        // TODO: Implement demo mode with synthetic audio signals
        console.log('Demo mode activated');
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
    updatePermissionStatus(message) {
        if (this.statusDisplay) {
            this.statusDisplay.innerHTML = `<strong>🎤 Microphone Status:</strong> ${message}`;
        }
    }

    /**
     * Cleanup resources
     */
    cleanup() {
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
    }
}

// Auto-initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.pitchApp = new PitchVisualizerApp();
});

// Cleanup on page unload
window.addEventListener('beforeunload', () => {
    if (window.pitchApp) {
        window.pitchApp.cleanup();
    }
}); 