/**
 * Comprehensive Error Management System
 * Story 2.3: Error Handling and Browser Fallbacks
 * 
 * Centralized error handling with user-friendly messaging,
 * recovery mechanisms, and fallback strategies.
 */

class ErrorManager {
    constructor() {
        this.errors = new Map();
        this.recoveryAttempts = new Map();
        this.maxRetries = 3;
        this.retryDelay = 1000; // Start with 1 second
        this.isRecovering = false;
        this.errorHandlers = new Map();
        this.userGuidance = null;
        
        // Initialize error categories
        this.errorCategories = {
            BROWSER_COMPATIBILITY: 'browser-compatibility',
            MICROPHONE_PERMISSION: 'microphone-permission', 
            AUDIO_CONTEXT: 'audio-context',
            WASM_LOADING: 'wasm-loading',
            WASM_RUNTIME: 'wasm-runtime',
            NETWORK_CONNECTIVITY: 'network-connectivity',
            AUDIO_DEVICE: 'audio-device',
            UNKNOWN: 'unknown'
        };
        
        this.setupErrorHandlers();
        this.setupGlobalErrorHandling();
    }

    /**
     * Setup category-specific error handlers
     */
    setupErrorHandlers() {
        // Browser compatibility errors
        this.errorHandlers.set(this.errorCategories.BROWSER_COMPATIBILITY, {
            handler: this.handleBrowserCompatibilityError.bind(this),
            recoverable: false,
            userMessage: 'Your browser doesn\'t support required features',
            childMessage: 'Oops! This app needs a newer browser to work properly.',
            recovery: this.activateDemoMode.bind(this)
        });

        // Microphone permission errors
        this.errorHandlers.set(this.errorCategories.MICROPHONE_PERMISSION, {
            handler: this.handleMicrophonePermissionError.bind(this),
            recoverable: true,
            userMessage: 'Microphone access is required for pitch detection',
            childMessage: 'We need to hear your voice to help you learn music!',
            recovery: this.retryMicrophonePermission.bind(this)
        });

        // Audio context errors
        this.errorHandlers.set(this.errorCategories.AUDIO_CONTEXT, {
            handler: this.handleAudioContextError.bind(this),
            recoverable: true,
            userMessage: 'Audio system initialization failed',
            childMessage: 'The audio part isn\'t working right now.',
            recovery: this.recoverAudioContext.bind(this)
        });

        // WASM loading errors
        this.errorHandlers.set(this.errorCategories.WASM_LOADING, {
            handler: this.handleWasmLoadingError.bind(this),
            recoverable: false,
            userMessage: 'WebAssembly module failed to load',
            childMessage: 'The special music-detection code couldn\'t load.',
            recovery: this.showWasmUpgradeGuidance.bind(this)
        });

        // WASM runtime errors
        this.errorHandlers.set(this.errorCategories.WASM_RUNTIME, {
            handler: this.handleWasmRuntimeError.bind(this),
            recoverable: true,
            userMessage: 'Audio processing error occurred',
            childMessage: 'The music detector had a problem.',
            recovery: this.recoverWasmRuntime.bind(this)
        });

        // Network connectivity errors
        this.errorHandlers.set(this.errorCategories.NETWORK_CONNECTIVITY, {
            handler: this.handleNetworkError.bind(this),
            recoverable: true,
            userMessage: 'Network connection issues detected',
            childMessage: 'Can\'t connect to the internet right now.',
            recovery: this.recoverNetworkConnection.bind(this)
        });

        // Audio device errors
        this.errorHandlers.set(this.errorCategories.AUDIO_DEVICE, {
            handler: this.handleAudioDeviceError.bind(this),
            recoverable: true,
            userMessage: 'Audio device error occurred',
            childMessage: 'Your microphone isn\'t working right now.',
            recovery: this.recoverAudioDevice.bind(this)
        });
    }

    /**
     * Setup global error handling
     */
    setupGlobalErrorHandling() {
        // Catch unhandled promise rejections
        window.addEventListener('unhandledrejection', (event) => {
            console.error('Unhandled promise rejection:', event.reason);
            this.handleError(event.reason, 'Unhandled Promise Rejection');
        });

        // Catch JavaScript errors
        window.addEventListener('error', (event) => {
            console.error('JavaScript error:', event.error);
            this.handleError(event.error, 'JavaScript Error');
        });

        // Catch network errors
        window.addEventListener('online', () => {
            this.handleNetworkRecovery();
        });

        window.addEventListener('offline', () => {
            this.handleError(new Error('Network connection lost'), 'Network');
        });
    }

    /**
     * Main error handling entry point
     */
    async handleError(error, context = 'Unknown', additionalInfo = {}) {
        const errorId = this.generateErrorId();
        const category = this.categorizeError(error, context);
        
        const errorInfo = {
            id: errorId,
            error: error,
            context: context,
            category: category,
            timestamp: new Date(),
            additionalInfo: additionalInfo,
            userAgent: navigator.userAgent,
            url: window.location.href
        };

        // Store error for tracking
        this.errors.set(errorId, errorInfo);

        console.error(`[ErrorManager] ${category}:`, error, { context, additionalInfo });

        // Get appropriate handler
        const handler = this.errorHandlers.get(category);
        if (handler) {
            await handler.handler(errorInfo);
        } else {
            await this.handleUnknownError(errorInfo);
        }

        return errorId;
    }

    /**
     * Categorize errors based on type and context
     */
    categorizeError(error, context) {
        const errorMessage = error.message || error.toString();
        const errorName = error.name || '';

        // Browser compatibility
        if (errorMessage.includes('WebAssembly') || errorMessage.includes('AudioContext') || 
            errorMessage.includes('getUserMedia') || context.includes('compatibility')) {
            return this.errorCategories.BROWSER_COMPATIBILITY;
        }

        // Microphone permission errors
        if (errorName === 'NotAllowedError' || errorName === 'PermissionDeniedError' ||
            errorMessage.includes('permission') || context.includes('microphone')) {
            return this.errorCategories.MICROPHONE_PERMISSION;
        }

        // Audio context errors
        if (errorMessage.includes('AudioContext') || errorMessage.includes('audio context') ||
            context.includes('audio') && !context.includes('microphone')) {
            return this.errorCategories.AUDIO_CONTEXT;
        }

        // WASM loading errors
        if (errorMessage.includes('WASM') || errorMessage.includes('WebAssembly') ||
            context.includes('wasm') || context.includes('WebAssembly')) {
            if (errorMessage.includes('runtime') || context.includes('runtime')) {
                return this.errorCategories.WASM_RUNTIME;
            }
            return this.errorCategories.WASM_LOADING;
        }

        // Network errors
        if (errorMessage.includes('network') || errorMessage.includes('fetch') ||
            errorName === 'NetworkError' || context.includes('network')) {
            return this.errorCategories.NETWORK_CONNECTIVITY;
        }

        // Audio device errors
        if (errorName === 'NotFoundError' || errorName === 'NotReadableError' ||
            errorName === 'OverconstrainedError' || errorMessage.includes('device')) {
            return this.errorCategories.AUDIO_DEVICE;
        }

        return this.errorCategories.UNKNOWN;
    }

    /**
     * Handle browser compatibility errors
     */
    async handleBrowserCompatibilityError(errorInfo) {
        const isChildUser = this.detectChildUser();
        
        this.showErrorMessage({
            type: 'critical',
            title: isChildUser ? '🚀 Need a Better Browser!' : 'Browser Compatibility Issue',
            message: isChildUser ? 
                'Your browser is too old for this cool music app! Ask a grown-up to help you update it.' :
                'Your browser doesn\'t support the required features for this application.',
            actions: [
                {
                    text: 'Try Demo Mode',
                    handler: () => this.activateDemoMode(errorInfo),
                    style: 'primary'
                },
                {
                    text: 'Upgrade Browser',
                    handler: () => this.showBrowserUpgradeGuidance(),
                    style: 'secondary'
                }
            ]
        });
    }

    /**
     * Handle microphone permission errors
     */
    async handleMicrophonePermissionError(errorInfo) {
        const error = errorInfo.error;
        const isChildUser = this.detectChildUser();
        let specificGuidance = '';
        let recoveryAction = null;

        // Determine specific error type
        if (error.name === 'NotAllowedError') {
            specificGuidance = isChildUser ? 
                'You said "no" to using the microphone. We need it to help you learn music!' :
                'Microphone access was denied. Please allow microphone access to use this app.';
            recoveryAction = () => this.retryMicrophonePermission();
        } else if (error.name === 'NotFoundError') {
            specificGuidance = isChildUser ?
                'No microphone found! Do you have one connected?' :
                'No microphone device found. Please connect a microphone and try again.';
            recoveryAction = () => this.showMicrophoneSetupGuidance();
        } else if (error.name === 'NotReadableError') {
            specificGuidance = isChildUser ?
                'Your microphone is busy with another app. Close other apps and try again!' :
                'Microphone is being used by another application. Please close other apps and retry.';
            recoveryAction = () => this.retryMicrophonePermission();
        }

        this.showErrorMessage({
            type: 'warning',
            title: isChildUser ? '🎤 Microphone Problem!' : 'Microphone Access Issue',
            message: specificGuidance,
            actions: [
                {
                    text: 'Try Again',
                    handler: recoveryAction,
                    style: 'primary'
                },
                {
                    text: 'Use Demo Mode',
                    handler: () => this.activateDemoMode(errorInfo),
                    style: 'secondary'
                },
                {
                    text: 'Help Me Fix This',
                    handler: () => this.showTroubleshootingGuide('microphone'),
                    style: 'info'
                }
            ]
        });
    }

    /**
     * Handle audio context errors
     */
    async handleAudioContextError(errorInfo) {
        const isChildUser = this.detectChildUser();
        
        this.showErrorMessage({
            type: 'error',
            title: isChildUser ? '🔊 Audio Problem!' : 'Audio System Error',
            message: isChildUser ?
                'The audio part isn\'t working. Let\'s try to fix it!' :
                'The audio system failed to initialize. This may be a temporary issue.',
            actions: [
                {
                    text: 'Try Again',
                    handler: () => this.recoverAudioContext(errorInfo),
                    style: 'primary'
                },
                {
                    text: 'Use Demo Mode',
                    handler: () => this.activateDemoMode(errorInfo),
                    style: 'secondary'
                }
            ]
        });
    }

    /**
     * Handle WASM loading errors
     */
    async handleWasmLoadingError(errorInfo) {
        const isChildUser = this.detectChildUser();
        
        this.showErrorMessage({
            type: 'critical',
            title: isChildUser ? '⚙️ Missing Important Code!' : 'WebAssembly Loading Error',
            message: isChildUser ?
                'This app needs special code that your browser doesn\'t have. You need a newer browser!' :
                'WebAssembly is required for this application but is not supported in your browser.',
            actions: [
                {
                    text: 'Upgrade Browser',
                    handler: () => this.showBrowserUpgradeGuidance(),
                    style: 'primary'
                },
                {
                    text: 'Learn More',
                    handler: () => this.showWasmInfo(),
                    style: 'info'
                }
            ]
        });
    }

    /**
     * Handle WASM runtime errors
     */
    async handleWasmRuntimeError(errorInfo) {
        const isChildUser = this.detectChildUser();
        
        this.showErrorMessage({
            type: 'error',
            title: isChildUser ? '🔧 Music Detector Problem!' : 'Audio Processing Error',
            message: isChildUser ?
                'The music detector had a problem. Let\'s try to restart it!' :
                'The audio processing system encountered an error. Attempting recovery...',
            actions: [
                {
                    text: 'Restart Audio System',
                    handler: () => this.recoverWasmRuntime(errorInfo),
                    style: 'primary'
                },
                {
                    text: 'Use Demo Mode',
                    handler: () => this.activateDemoMode(errorInfo),
                    style: 'secondary'
                }
            ]
        });
    }

    /**
     * Handle network errors
     */
    async handleNetworkError(errorInfo) {
        const isChildUser = this.detectChildUser();
        
        this.showErrorMessage({
            type: 'warning',
            title: isChildUser ? '📡 Connection Problem!' : 'Network Error',
            message: isChildUser ?
                'Can\'t connect to the internet. Check your WiFi!' :
                'Network connection issues detected. Some features may not work properly.',
            actions: [
                {
                    text: 'Try Again',
                    handler: () => this.recoverNetworkConnection(errorInfo),
                    style: 'primary'
                },
                {
                    text: 'Use Offline Mode',
                    handler: () => this.activateOfflineMode(),
                    style: 'secondary'
                }
            ]
        });
    }

    /**
     * Handle audio device errors
     */
    async handleAudioDeviceError(errorInfo) {
        const isChildUser = this.detectChildUser();
        
        this.showErrorMessage({
            type: 'warning',
            title: isChildUser ? '🎤 Microphone Trouble!' : 'Audio Device Error',
            message: isChildUser ?
                'Your microphone isn\'t working right. Let\'s try to fix it!' :
                'There was a problem with your audio device. Please check your microphone connection.',
            actions: [
                {
                    text: 'Try Different Settings',
                    handler: () => this.recoverAudioDevice(errorInfo),
                    style: 'primary'
                },
                {
                    text: 'Use Demo Mode',
                    handler: () => this.activateDemoMode(errorInfo),
                    style: 'secondary'
                },
                {
                    text: 'Device Help',
                    handler: () => this.showTroubleshootingGuide('audio-device'),
                    style: 'info'
                }
            ]
        });
    }

    /**
     * Handle unknown errors
     */
    async handleUnknownError(errorInfo) {
        const isChildUser = this.detectChildUser();
        
        this.showErrorMessage({
            type: 'error',
            title: isChildUser ? '❓ Something Went Wrong!' : 'Unexpected Error',
            message: isChildUser ?
                'Oops! Something unexpected happened. Don\'t worry, we can try to fix it!' :
                'An unexpected error occurred. Please try refreshing the page.',
            actions: [
                {
                    text: 'Refresh Page',
                    handler: () => window.location.reload(),
                    style: 'primary'
                },
                {
                    text: 'Use Demo Mode',
                    handler: () => this.activateDemoMode(errorInfo),
                    style: 'secondary'
                }
            ]
        });
    }

    /**
     * Detect if user is likely a child (simplified heuristic)
     */
    detectChildUser() {
        // Simple heuristics - in a real app, this might be more sophisticated
        const userAgent = navigator.userAgent.toLowerCase();
        const isTablet = /tablet|ipad/.test(userAgent);
        const isMobile = /mobile|phone/.test(userAgent);
        const hasParentalControls = window.location.href.includes('kids') || 
                                   document.title.includes('Kids') ||
                                   localStorage.getItem('user-age-group') === 'child';
        
        return isTablet || hasParentalControls;
    }

    /**
     * Show error message with appropriate UI
     */
    showErrorMessage(config) {
        // Remove any existing error message
        const existingError = document.getElementById('error-message-modal');
        if (existingError) {
            existingError.remove();
        }

        const modal = document.createElement('div');
        modal.id = 'error-message-modal';
        modal.className = 'error-modal';
        modal.innerHTML = `
            <div class="error-modal-content ${config.type}">
                <div class="error-modal-header">
                    <h3 class="error-modal-title">${config.title}</h3>
                </div>
                <div class="error-modal-body">
                    <p class="error-modal-message">${config.message}</p>
                </div>
                <div class="error-modal-actions">
                    ${config.actions.map(action => `
                        <button class="error-action-btn btn-${action.style}" data-action="${action.text}">
                            ${action.text}
                        </button>
                    `).join('')}
                </div>
            </div>
        `;

        // Bind action handlers
        config.actions.forEach(action => {
            const btn = modal.querySelector(`[data-action="${action.text}"]`);
            if (btn) {
                btn.addEventListener('click', () => {
                    modal.remove();
                    if (action.handler) {
                        action.handler();
                    }
                });
            }
        });

        document.body.appendChild(modal);
    }

    /**
     * Generate unique error ID
     */
    generateErrorId() {
        return 'error_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
    }

    // Recovery methods will be implemented here
    async activateDemoMode(errorInfo) {
        console.log('🎮 Activating demo mode due to error:', errorInfo.category);
        
        if (window.DemoMode) {
            const demoMode = new window.DemoMode();
            await demoMode.activate(`Error recovery: ${errorInfo.category}`);
        } else {
            console.warn('Demo mode not available');
        }
    }

    async retryMicrophonePermission() {
        // Implementation for microphone retry
        console.log('🔄 Retrying microphone permission...');
        if (window.pitchApp && window.pitchApp.requestMicrophonePermission) {
            try {
                await window.pitchApp.requestMicrophonePermission();
            } catch (error) {
                console.error('Retry failed:', error);
            }
        }
    }

    async recoverAudioContext(errorInfo) {
        console.log('🔄 Attempting audio context recovery...');
        // Implementation for audio context recovery
    }

    async recoverWasmRuntime(errorInfo) {
        console.log('🔄 Attempting WASM runtime recovery...');
        // Implementation for WASM runtime recovery
    }

    async recoverNetworkConnection(errorInfo) {
        console.log('🔄 Attempting network recovery...');
        // Implementation for network recovery
    }

    async recoverAudioDevice(errorInfo) {
        console.log('🔄 Attempting audio device recovery...');
        // Implementation for audio device recovery
    }

    showBrowserUpgradeGuidance() {
        console.log('📱 Showing browser upgrade guidance...');
        // Implementation for browser upgrade guidance
    }

    showWasmInfo() {
        console.log('ℹ️ Showing WASM information...');
        // Implementation for WASM info
    }

    showWasmUpgradeGuidance() {
        console.log('🔧 Showing WASM upgrade guidance...');
        // Implementation for WASM upgrade guidance - same as browser upgrade for now
        this.showBrowserUpgradeGuidance();
    }

    showTroubleshootingGuide(type) {
        console.log(`🔧 Showing troubleshooting guide for: ${type}`);
        // Implementation for troubleshooting guides
    }

    showMicrophoneSetupGuidance() {
        console.log('🎤 Showing microphone setup guidance...');
        // Implementation for microphone setup guidance
    }

    activateOfflineMode() {
        console.log('📴 Activating offline mode...');
        // Implementation for offline mode
    }

    handleNetworkRecovery() {
        console.log('🌐 Network connection restored');
        // Implementation for network recovery
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ErrorManager;
} else {
    window.ErrorManager = ErrorManager;
} 