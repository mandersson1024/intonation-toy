/**
 * Audio Device Management System
 * Story 2.3: Audio Device Recovery Mechanisms
 * 
 * Handles audio device disconnection, reconnection, and provides
 * manual recovery controls for robust audio device management.
 */

class AudioDeviceManager {
    constructor(pitchApp) {
        this.pitchApp = pitchApp;
        this.isMonitoring = false;
        this.lastKnownDevices = [];
        this.currentDeviceId = null;
        this.reconnectionAttempts = 0;
        this.maxReconnectionAttempts = 3;
        this.recoveryUI = null;
        this.deviceCheckInterval = null;
        
        this.setupDeviceMonitoring();
    }

    /**
     * Initialize device monitoring and event listeners
     */
    async setupDeviceMonitoring() {
        try {
            console.log('🔍 Setting up audio device monitoring...');
            
            // Check if device enumeration is supported
            if (!navigator.mediaDevices || !navigator.mediaDevices.enumerateDevices) {
                console.warn('Device enumeration not supported');
                return;
            }

            // Get initial device list
            await this.updateDeviceList();
            console.log('📋 Initial device list obtained:', this.lastKnownDevices.length, 'devices');
            
            // Bind the event handler to preserve 'this' context
            this.boundHandleDeviceChange = this.handleDeviceChange.bind(this);
            
            // Listen for device changes
            if (navigator.mediaDevices.addEventListener) {
                navigator.mediaDevices.addEventListener('devicechange', this.boundHandleDeviceChange);
                console.log('✅ Audio device monitoring initialized - listening for device changes');
            } else {
                console.warn('⚠️ Device change events not supported - using polling fallback only');
            }
            
            // Start periodic device checking as fallback
            this.startPeriodicDeviceCheck();
            console.log('🔄 Periodic device checking started');
            
        } catch (error) {
            console.error('Failed to setup device monitoring:', error);
        }
    }

    /**
     * Update the list of available devices
     */
    async updateDeviceList() {
        try {
            const devices = await navigator.mediaDevices.enumerateDevices();
            const audioInputs = devices.filter(device => device.kind === 'audioinput');
            
            this.lastKnownDevices = audioInputs.map(device => ({
                deviceId: device.deviceId,
                label: device.label,
                groupId: device.groupId
            }));
            
            // Store device count for monitoring
            
        } catch (error) {
            console.error('Failed to enumerate devices:', error);
        }
    }

    /**
     * Handle device change events
     */
    async handleDeviceChange() {
        console.log('🔄 Audio device change detected!');
        
        try {
            const previousDevices = [...this.lastKnownDevices];
            console.log('📋 Previous devices:', previousDevices.map(d => d.label || d.deviceId));
            
            await this.updateDeviceList();
            console.log('📋 Current devices:', this.lastKnownDevices.map(d => d.label || d.deviceId));
            
            // Check if current device was disconnected
            if (this.currentDeviceId && this.isCurrentDeviceDisconnected()) {
                console.warn('🔌 Current audio device disconnected:', this.currentDeviceId);
                await this.handleDeviceDisconnection();
            }
            
            // Check for new devices
            const newDevices = this.detectNewDevices(previousDevices);
            if (newDevices.length > 0) {
                console.log('🆕 New audio devices detected:', newDevices.map(d => d.label || d.deviceId));
                await this.handleNewDevices(newDevices);
            }
            
            // Check for removed devices
            const removedDevices = previousDevices.filter(prevDevice => 
                !this.lastKnownDevices.some(currentDevice => 
                    currentDevice.deviceId === prevDevice.deviceId
                )
            );
            
            if (removedDevices.length > 0) {
                console.log('📱 Devices removed:', removedDevices.map(d => d.label || d.deviceId));
            }
            
        } catch (error) {
            console.error('Error handling device change:', error);
        }
    }

    /**
     * Check if current device is still available
     */
    isCurrentDeviceDisconnected() {
        if (!this.currentDeviceId) return false;
        
        return !this.lastKnownDevices.some(device => 
            device.deviceId === this.currentDeviceId
        );
    }

    /**
     * Detect newly connected devices
     */
    detectNewDevices(previousDevices) {
        return this.lastKnownDevices.filter(currentDevice => 
            !previousDevices.some(prevDevice => 
                prevDevice.deviceId === currentDevice.deviceId
            )
        );
    }

    /**
     * Handle device disconnection
     */
    async handleDeviceDisconnection() {
        console.warn('🚨 Handling audio device disconnection...');
        
        // Stop current audio stream
        if (this.pitchApp.microphoneStream) {
            this.pitchApp.microphoneStream.getTracks().forEach(track => track.stop());
            this.pitchApp.microphoneStream = null;
        }
        
        // Update app state
        this.pitchApp.permissionState = 'device-disconnected';
        this.pitchApp.updatePermissionStatus('🔌 Audio device disconnected - attempting recovery...', 'warning');
        
        // Show recovery UI
        this.showDeviceRecoveryUI();
        
        // Attempt automatic recovery
        await this.attemptAutomaticRecovery();
    }

    /**
     * Handle new devices being connected
     */
    async handleNewDevices(newDevices) {
        console.log('✨ New audio devices available for recovery:', newDevices);
        
        // If we're in a disconnected state, offer recovery
        if (this.pitchApp.permissionState === 'device-disconnected') {
            this.updateRecoveryUIWithNewDevices(newDevices);
        }
    }

    /**
     * Attempt automatic recovery with available devices
     */
    async attemptAutomaticRecovery() {
        if (this.reconnectionAttempts >= this.maxReconnectionAttempts) {
            console.warn('🛑 Max reconnection attempts reached');
            this.showManualRecoveryUI();
            return;
        }
        
        this.reconnectionAttempts++;
        console.log(`🔄 Attempting automatic recovery (${this.reconnectionAttempts}/${this.maxReconnectionAttempts})...`);
        
        try {
            // Wait a moment for device to stabilize
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            // Try to reconnect with default device
            await this.reconnectWithDevice();
            
            // Reset reconnection counter on success
            this.reconnectionAttempts = 0;
            this.hideRecoveryUI();
            
            console.log('✅ Automatic device recovery successful');
            
        } catch (error) {
            console.error(`❌ Automatic recovery attempt ${this.reconnectionAttempts} failed:`, error);
            
            if (this.reconnectionAttempts < this.maxReconnectionAttempts) {
                // Retry with exponential backoff
                const delay = Math.pow(2, this.reconnectionAttempts) * 1000;
                setTimeout(() => this.attemptAutomaticRecovery(), delay);
            } else {
                this.showManualRecoveryUI();
            }
        }
    }

    /**
     * Reconnect with a specific device or default
     */
    async reconnectWithDevice(deviceId = null) {
        try {
            const constraints = {
                audio: {
                    echoCancellation: false,
                    noiseSuppression: false,
                    autoGainControl: false,
                    sampleRate: { ideal: 44100 },
                    channelCount: { ideal: 1 }
                }
            };
            
            // Add device constraint if specified
            if (deviceId) {
                constraints.audio.deviceId = { exact: deviceId };
            }
            
            // Get new stream
            const stream = await navigator.mediaDevices.getUserMedia(constraints);
            
            // Update app state
            this.pitchApp.microphoneStream = stream;
            this.currentDeviceId = stream.getAudioTracks()[0].getSettings().deviceId;
            this.pitchApp.permissionState = 'granted';
            
            // Reconnect audio pipeline
            await this.reconnectAudioPipeline(stream);
            
            this.pitchApp.updatePermissionStatus('✅ Audio device reconnected successfully!', 'success');
            
        } catch (error) {
            console.error('Device reconnection failed:', error);
            throw error;
        }
    }

    /**
     * Reconnect the audio processing pipeline
     */
    async reconnectAudioPipeline(stream) {
        try {
            // Disconnect old source if it exists
            if (this.pitchApp.microphoneSource) {
                this.pitchApp.microphoneSource.disconnect();
            }
            
            // Create new microphone source
            this.pitchApp.microphoneSource = this.pitchApp.audioContext.createMediaStreamSource(stream);
            
            // Reconnect to audio worklet
            if (this.pitchApp.audioWorkletNode) {
                this.pitchApp.microphoneSource.connect(this.pitchApp.audioWorkletNode);
                console.log('🔗 Audio pipeline reconnected');
            }
            
        } catch (error) {
            console.error('Failed to reconnect audio pipeline:', error);
            throw error;
        }
    }

    /**
     * Show device recovery UI
     */
    showDeviceRecoveryUI() {
        this.hideRecoveryUI(); // Remove any existing UI
        
        this.recoveryUI = document.createElement('div');
        this.recoveryUI.id = 'device-recovery-ui';
        this.recoveryUI.className = 'device-recovery-modal';
        this.recoveryUI.innerHTML = `
            <div class="device-recovery-content">
                <div class="recovery-header">
                    <div class="recovery-icon">🔌</div>
                    <h3>Audio Device Disconnected</h3>
                    <p>Your microphone was disconnected. We're trying to reconnect automatically...</p>
                </div>
                
                <div class="recovery-status">
                    <div class="recovery-progress">
                        <div class="progress-bar">
                            <div class="progress-fill"></div>
                        </div>
                        <div class="recovery-text">Attempting automatic recovery...</div>
                    </div>
                </div>
                
                <div class="recovery-actions">
                    <button id="manual-recovery-btn" class="btn-primary recovery-btn">
                        🔄 Try Manual Recovery
                    </button>

                </div>
                
                <div class="recovery-tips">
                    <h4>💡 Recovery Tips:</h4>
                    <ul>
                        <li>Make sure your microphone is properly connected</li>
                        <li>Try unplugging and reconnecting your device</li>
                        <li>Check your computer's audio settings</li>
                        <li>Restart your browser if the problem persists</li>
                    </ul>
                </div>
            </div>
        `;
        
        document.body.appendChild(this.recoveryUI);
        this.bindRecoveryEvents();
        this.startRecoveryAnimation();
    }

    /**
     * Show manual recovery UI with device selection
     */
    showManualRecoveryUI() {
        if (this.recoveryUI) {
            const statusSection = this.recoveryUI.querySelector('.recovery-status');
            const actionsSection = this.recoveryUI.querySelector('.recovery-actions');
            
            statusSection.innerHTML = `
                <div class="manual-recovery-section">
                    <h4>🎯 Manual Device Selection</h4>
                    <p>Choose your microphone from the list below:</p>
                    <div id="device-list" class="device-list">
                        <!-- Devices will be populated here -->
                    </div>
                </div>
            `;
            
            actionsSection.innerHTML = `
                <button id="refresh-devices-btn" class="btn-info recovery-btn">
                    🔄 Refresh Device List
                </button>
                
            `;
            
            this.populateDeviceList();
            this.bindRecoveryEvents();
        }
    }

    /**
     * Update recovery UI when new devices are detected
     */
    updateRecoveryUIWithNewDevices(newDevices) {
        if (this.recoveryUI) {
            const recoveryText = this.recoveryUI.querySelector('.recovery-text');
            if (recoveryText) {
                recoveryText.textContent = `✨ New device detected! Attempting connection...`;
                recoveryText.style.color = 'var(--success)';
            }
            
            // Automatically attempt connection to the first new device
            if (newDevices.length > 0) {
                setTimeout(async () => {
                    try {
                        await this.reconnectWithDevice(newDevices[0].deviceId);
                        // Hide recovery UI on successful reconnection
                        this.hideRecoveryUI();
                        this.reconnectionAttempts = 0; // Reset counter
                        console.log('✅ Automatic device recovery successful via new device detection');
                    } catch (error) {
                        console.error('❌ New device reconnection failed:', error);
                        // Show manual recovery if automatic fails
                        this.showManualRecoveryUI();
                    }
                }, 1000);
            }
        }
    }

    /**
     * Populate device selection list
     */
    async populateDeviceList() {
        await this.updateDeviceList();
        
        const deviceList = document.getElementById('device-list');
        if (!deviceList) return;
        
        if (this.lastKnownDevices.length === 0) {
            deviceList.innerHTML = `
                <div class="no-devices">
                    <p>🚫 No audio input devices found</p>
                    <p><small>Please connect a microphone and refresh the list</small></p>
                </div>
            `;
            return;
        }
        
        deviceList.innerHTML = this.lastKnownDevices.map(device => `
            <div class="device-item" data-device-id="${device.deviceId}">
                <div class="device-info">
                    <div class="device-name">${device.label || 'Unknown Device'}</div>
                    <div class="device-id">${device.deviceId.substring(0, 20)}...</div>
                </div>
                <button class="device-select-btn btn-primary" data-device-id="${device.deviceId}">
                    Select
                </button>
            </div>
        `).join('');
        
        // Bind device selection events
        deviceList.querySelectorAll('.device-select-btn').forEach(btn => {
            btn.addEventListener('click', async (e) => {
                const deviceId = e.target.dataset.deviceId;
                try {
                    await this.reconnectWithDevice(deviceId);
                    // Hide recovery UI on successful manual reconnection
                    this.hideRecoveryUI();
                    this.reconnectionAttempts = 0; // Reset counter
                    console.log('✅ Manual device recovery successful');
                } catch (error) {
                    console.error('❌ Manual device reconnection failed:', error);
                    // Keep the UI open and show error
                    const recoveryText = this.recoveryUI?.querySelector('.recovery-text');
                    if (recoveryText) {
                        recoveryText.textContent = `❌ Connection failed. Please try another device.`;
                        recoveryText.style.color = 'var(--error)';
                    }
                }
            });
        });
    }

    /**
     * Bind recovery UI event listeners
     */
    bindRecoveryEvents() {
        const manualRecoveryBtn = document.getElementById('manual-recovery-btn');

        const refreshDevicesBtn = document.getElementById('refresh-devices-btn');
        
        manualRecoveryBtn?.addEventListener('click', () => {
            this.showManualRecoveryUI();
        });

        
        refreshDevicesBtn?.addEventListener('click', () => {
            this.populateDeviceList();
        });
    }

    /**
     * Start recovery animation
     */
    startRecoveryAnimation() {
        const progressFill = this.recoveryUI?.querySelector('.progress-fill');
        if (progressFill) {
            progressFill.style.animation = 'recoveryProgress 3s ease-in-out infinite';
        }
    }

    /**
     * Hide recovery UI
     */
    hideRecoveryUI() {
        if (this.recoveryUI) {
            this.recoveryUI.remove();
            this.recoveryUI = null;
        }
    }



    /**
     * Start periodic device checking as fallback
     */
    startPeriodicDeviceCheck() {
        this.deviceCheckInterval = setInterval(() => {
            if (this.isMonitoring) {
                this.checkDeviceStatus();
            }
        }, 2000); // Check every 2 seconds for better responsiveness
        console.log('🕐 Periodic device status checking every 2 seconds');
    }

    /**
     * Check device status periodically
     */
    async checkDeviceStatus() {
        if (!this.isMonitoring) return;
        
        try {
            // Check if we have a microphone stream
            if (!this.pitchApp.microphoneStream) {
                return;
            }
            
            const tracks = this.pitchApp.microphoneStream.getAudioTracks();
            
            // Check if tracks are still active
            const hasActiveTracks = tracks.some(track => track.readyState === 'live');
            const hasEndedTracks = tracks.some(track => track.readyState === 'ended');
            
            if (hasEndedTracks) {
                console.warn('🚨 Periodic check detected ended audio tracks');
                await this.handleDeviceDisconnection();
                return;
            }
            
            if (!hasActiveTracks && this.pitchApp.permissionState === 'granted') {
                console.warn('🔍 Periodic check detected inactive audio tracks');
                await this.handleDeviceDisconnection();
                return;
            }
            
            // Also check device list changes
            const previousDeviceCount = this.lastKnownDevices.length;
            await this.updateDeviceList();
            
            if (this.lastKnownDevices.length !== previousDeviceCount) {
                console.log(`📱 Device count changed: ${previousDeviceCount} → ${this.lastKnownDevices.length}`);
                await this.handleDeviceChange();
            }
            
        } catch (error) {
            console.error('Error during periodic device check:', error);
        }
    }

    /**
     * Set current device ID for tracking
     */
    setCurrentDevice(deviceId) {
        this.currentDeviceId = deviceId;
        console.log(`📱 Current audio device set: ${deviceId}`);
        console.log('🔍 Device monitoring status:', this.isMonitoring ? 'ACTIVE' : 'INACTIVE');
    }

    /**
     * Start monitoring
     */
    startMonitoring() {
        this.isMonitoring = true;
        console.log('👁️ Audio device monitoring started');
    }

    /**
     * Stop monitoring
     */
    stopMonitoring() {
        this.isMonitoring = false;
        this.hideRecoveryUI();
        
        if (this.deviceCheckInterval) {
            clearInterval(this.deviceCheckInterval);
            this.deviceCheckInterval = null;
        }
        
        console.log('⏹️ Audio device monitoring stopped');
    }

    /**
     * Cleanup
     */
    cleanup() {
        this.stopMonitoring();
        
        // Remove event listeners
        if (navigator.mediaDevices && navigator.mediaDevices.removeEventListener && this.boundHandleDeviceChange) {
            navigator.mediaDevices.removeEventListener('devicechange', this.boundHandleDeviceChange);
            console.log('🔌 Device change event listener removed');
        }
        
        console.log('🧹 Audio device manager cleaned up');
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = AudioDeviceManager;
} else {
    window.AudioDeviceManager = AudioDeviceManager;
    
    // Add global debug methods for testing
    window.debugDeviceManager = function() {
        console.log('🔍 Device Manager Debug Report:');
        console.log('- window.pitchApp exists:', !!window.pitchApp);
        console.log('- AudioDeviceManager class loaded:', !!window.AudioDeviceManager);
        
        if (window.pitchApp) {
            console.log('- pitchApp.audioDeviceManager exists:', !!window.pitchApp.audioDeviceManager);
            
            if (window.pitchApp.audioDeviceManager) {
                const manager = window.pitchApp.audioDeviceManager;
                console.log('✅ Device Manager Found! Status:');
                console.log('  - Is Monitoring:', manager.isMonitoring);
                console.log('  - Current Device ID:', manager.currentDeviceId);
                console.log('  - Known Devices:', manager.lastKnownDevices.length);
                console.log('  - Reconnection Attempts:', manager.reconnectionAttempts);
                console.log('  - Has Microphone Stream:', !!window.pitchApp.microphoneStream);
                console.log('  - Permission State:', window.pitchApp.permissionState);
                
                if (window.pitchApp.microphoneStream) {
                    const tracks = window.pitchApp.microphoneStream.getAudioTracks();
                    console.log('  - Audio Tracks:', tracks.map(t => `${t.label}: ${t.readyState}`));
                }
            } else {
                console.log('❌ Device manager not initialized on pitchApp');
            }
        } else {
            console.log('❌ window.pitchApp not found - app may not be initialized');
        }
    };
    
    window.testDeviceChange = function() {
        if (window.pitchApp?.audioDeviceManager) {
            console.log('🧪 Manually triggering device change...');
            window.pitchApp.audioDeviceManager.handleDeviceChange();
        }
    };
    
    window.initializeDeviceManager = function() {
        if (window.pitchApp && !window.pitchApp.audioDeviceManager) {
            console.log('🔧 Manually initializing device manager...');
            if (window.AudioDeviceManager) {
                window.pitchApp.audioDeviceManager = new window.AudioDeviceManager(window.pitchApp);
                console.log('✅ Device manager manually initialized');
                return true;
            } else {
                console.error('❌ AudioDeviceManager class not available');
                return false;
            }
        } else {
            console.log('ℹ️ Device manager already exists or pitchApp not found');
            return false;
        }
    };
} 