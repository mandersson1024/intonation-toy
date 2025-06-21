/**
 * Browser Capability Detection System
 * Story 2.3: Enhanced Error Handling and Browser Fallbacks
 * 
 * Comprehensive detection of required browser features:
 * - Web Audio API and AudioWorklets
 * - WebAssembly support  
 * - getUserMedia API
 * - Audio device capabilities
 */

class BrowserCapabilityDetector {
    constructor() {
        this.capabilities = {
            webAudio: null,
            audioWorklets: null,
            webAssembly: null,
            getUserMedia: null,
            permissions: null,
            audioDevices: null,
            offline: null
        };
        
        this.browserInfo = this.detectBrowserInfo();
        this.isSupported = null;
        this.unsupportedFeatures = [];
        this.warnings = [];
    }

    /**
     * Perform comprehensive capability detection
     * @returns {Object} Complete capability report
     */
    async detectCapabilities() {
        console.log('🔍 Detecting browser capabilities...');
        
        // Core feature detection
        this.capabilities.webAudio = this.detectWebAudioAPI();
        this.capabilities.audioWorklets = this.detectAudioWorklets();
        this.capabilities.webAssembly = this.detectWebAssembly();
        this.capabilities.getUserMedia = this.detectGetUserMedia();
        this.capabilities.permissions = this.detectPermissionsAPI();
        
        // Advanced capabilities
        this.capabilities.audioDevices = await this.detectAudioDevices();
        this.capabilities.offline = this.detectOfflineCapabilities();
        
        // Generate overall support assessment
        this.assessOverallSupport();
        
        return {
            isSupported: this.isSupported,
            capabilities: this.capabilities,
            browserInfo: this.browserInfo,
            unsupportedFeatures: this.unsupportedFeatures,
            warnings: this.warnings,
            recommendations: this.generateRecommendations()
        };
    }

    /**
     * Detect Web Audio API support
     */
    detectWebAudioAPI() {
        const result = {
            supported: false,
            version: null,
            features: {
                audioContext: false,
                createGain: false,
                createScriptProcessor: false,
                createAnalyser: false,
                sampleRate: null,
                maxChannelCount: null
            }
        };

        try {
            // Check for AudioContext or webkitAudioContext
            const AudioContextClass = window.AudioContext || window.webkitAudioContext;
            
            if (!AudioContextClass) {
                return result;
            }

            // Create temporary context for feature testing
            const testContext = new AudioContextClass();
            
            result.supported = true;
            result.features.audioContext = true;
            result.features.createGain = typeof testContext.createGain === 'function';
            result.features.createScriptProcessor = typeof testContext.createScriptProcessor === 'function';
            result.features.createAnalyser = typeof testContext.createAnalyser === 'function';
            result.features.sampleRate = testContext.sampleRate;
            result.features.maxChannelCount = testContext.destination.maxChannelCount;
            
            // Determine version based on features
            if (testContext.audioWorklet) {
                result.version = 'modern'; // AudioWorklet support
            } else if (testContext.createScriptProcessor) {
                result.version = 'legacy'; // ScriptProcessorNode only
            }
            
            // Clean up test context
            if (testContext.state !== 'closed') {
                testContext.close();
            }
            
        } catch (error) {
            console.warn('Web Audio API detection error:', error);
        }

        if (!result.supported) {
            this.unsupportedFeatures.push('Web Audio API');
        }

        return result;
    }

    /**
     * Detect AudioWorklet support (required for real-time processing)
     */
    detectAudioWorklets() {
        const result = {
            supported: false,
            available: false,
            reason: null
        };

        try {
            const AudioContextClass = window.AudioContext || window.webkitAudioContext;
            
            if (!AudioContextClass) {
                result.reason = 'No AudioContext support';
                return result;
            }

            // Check if audioWorklet property exists
            const tempContext = new AudioContextClass();
            result.available = 'audioWorklet' in tempContext;
            result.supported = result.available && typeof tempContext.audioWorklet.addModule === 'function';
            
            if (!result.supported) {
                result.reason = result.available ? 'AudioWorklet not functional' : 'AudioWorklet not available';
                this.warnings.push('AudioWorklet not supported - will fall back to ScriptProcessorNode (higher latency)');
            }
            
            tempContext.close();
            
        } catch (error) {
            result.reason = `Detection error: ${error.message}`;
            console.warn('AudioWorklet detection error:', error);
        }

        if (!result.supported) {
            this.unsupportedFeatures.push('AudioWorklets');
        }

        return result;
    }

    /**
     * Detect WebAssembly support (critical requirement)
     */
    detectWebAssembly() {
        const result = {
            supported: false,
            version: null,
            features: {
                basic: false,
                streaming: false,
                threads: false,
                simd: false
            },
            reason: null
        };

        try {
            // Basic WASM support
            result.features.basic = typeof WebAssembly === 'object' && 
                                   typeof WebAssembly.instantiate === 'function';
            
            if (!result.features.basic) {
                result.reason = 'WebAssembly object not available';
                return result;
            }

            // Streaming compilation
            result.features.streaming = typeof WebAssembly.instantiateStreaming === 'function';
            
            // Threads support (SharedArrayBuffer)
            result.features.threads = typeof SharedArrayBuffer !== 'undefined';
            
            // SIMD support (newer feature)
            result.features.simd = 'WebAssembly' in window && 'v128' in WebAssembly;
            
            result.supported = result.features.basic;
            result.version = result.features.streaming ? 'modern' : 'basic';
            
            if (!result.features.streaming) {
                this.warnings.push('WebAssembly streaming not supported - slower WASM loading');
            }
            
        } catch (error) {
            result.reason = `Detection error: ${error.message}`;
            console.warn('WebAssembly detection error:', error);
        }

        if (!result.supported) {
            this.unsupportedFeatures.push('WebAssembly');
        }

        return result;
    }

    /**
     * Detect getUserMedia support
     */
    detectGetUserMedia() {
        const result = {
            supported: false,
            api: null,
            constraints: {
                audio: false,
                echoCancellation: false,
                noiseSuppression: false,
                autoGainControl: false,
                sampleRate: false
            }
        };

        try {
            // Check for modern mediaDevices API
            if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
                result.supported = true;
                result.api = 'mediaDevices';
                
                // Test constraint support (these are hints, not requirements)
                result.constraints.audio = true;
                result.constraints.echoCancellation = true;
                result.constraints.noiseSuppression = true;
                result.constraints.autoGainControl = true;
                result.constraints.sampleRate = true;
                
            } else if (navigator.getUserMedia || navigator.webkitGetUserMedia || navigator.mozGetUserMedia) {
                // Legacy API
                result.supported = true;
                result.api = 'legacy';
                result.constraints.audio = true;
                this.warnings.push('Using legacy getUserMedia API - limited audio constraints');
                
            } else {
                result.api = 'none';
            }
            
        } catch (error) {
            console.warn('getUserMedia detection error:', error);
        }

        if (!result.supported) {
            this.unsupportedFeatures.push('getUserMedia');
        }

        return result;
    }

    /**
     * Detect Permissions API support
     */
    detectPermissionsAPI() {
        const result = {
            supported: false,
            queries: {
                microphone: false
            }
        };

        try {
            if (navigator.permissions && navigator.permissions.query) {
                result.supported = true;
                result.queries.microphone = true;
            }
        } catch (error) {
            console.warn('Permissions API detection error:', error);
        }

        return result;
    }

    /**
     * Detect available audio devices
     */
    async detectAudioDevices() {
        const result = {
            supported: false,
            inputDevices: 0,
            outputDevices: 0,
            devices: [],
            error: null
        };

        try {
            if (!navigator.mediaDevices || !navigator.mediaDevices.enumerateDevices) {
                result.error = 'enumerateDevices not supported';
                return result;
            }

            const devices = await navigator.mediaDevices.enumerateDevices();
            
            result.supported = true;
            result.devices = devices.map(device => ({
                kind: device.kind,
                label: device.label,
                deviceId: device.deviceId ? 'present' : 'absent'
            }));
            
            result.inputDevices = devices.filter(d => d.kind === 'audioinput').length;
            result.outputDevices = devices.filter(d => d.kind === 'audiooutput').length;
            
            if (result.inputDevices === 0) {
                this.warnings.push('No audio input devices detected');
            }
            
        } catch (error) {
            result.error = error.message;
            console.warn('Audio devices detection error:', error);
        }

        return result;
    }

    /**
     * Detect offline/service worker capabilities
     */
    detectOfflineCapabilities() {
        const result = {
            serviceWorker: false,
            cacheAPI: false,
            offlineSupported: false
        };

        try {
            result.serviceWorker = 'serviceWorker' in navigator;
            result.cacheAPI = 'caches' in window;
            result.offlineSupported = result.serviceWorker && result.cacheAPI;
        } catch (error) {
            console.warn('Offline capabilities detection error:', error);
        }

        return result;
    }

    /**
     * Detect browser information
     */
    detectBrowserInfo() {
        const ua = navigator.userAgent;
        const result = {
            name: 'unknown',
            version: 'unknown',
            platform: navigator.platform || 'unknown',
            mobile: /Mobi|Android/i.test(ua)
        };

        // Browser detection
        if (ua.includes('Chrome') && !ua.includes('Edge')) {
            result.name = 'Chrome';
            const match = ua.match(/Chrome\/(\d+)/);
            result.version = match ? match[1] : 'unknown';
        } else if (ua.includes('Firefox')) {
            result.name = 'Firefox';
            const match = ua.match(/Firefox\/(\d+)/);
            result.version = match ? match[1] : 'unknown';
        } else if (ua.includes('Safari') && !ua.includes('Chrome')) {
            result.name = 'Safari';
            const match = ua.match(/Version\/(\d+)/);
            result.version = match ? match[1] : 'unknown';
        } else if (ua.includes('Edge')) {
            result.name = 'Edge';
            const match = ua.match(/Edge\/(\d+)/);
            result.version = match ? match[1] : 'unknown';
        }

        return result;
    }

    /**
     * Assess overall browser support
     */
    assessOverallSupport() {
        const critical = ['webAudio', 'webAssembly', 'getUserMedia'];
        const supported = critical.every(feature => this.capabilities[feature]?.supported);
        
        this.isSupported = supported;
        
        if (!supported) {
            console.warn('❌ Browser lacks critical features:', this.unsupportedFeatures);
        } else if (this.warnings.length > 0) {
            console.warn('⚠️ Browser has limitations:', this.warnings);
        }
    }

    /**
     * Generate browser-specific recommendations
     */
    generateRecommendations() {
        const recommendations = [];
        
        if (!this.isSupported) {
            recommendations.push({
                type: 'critical',
                message: 'Your browser doesn\'t support required features for this application.',
                action: 'Please upgrade to a modern browser'
            });
            
            // Browser-specific upgrade recommendations
            const browserRecs = this.getBrowserUpgradeRecommendations();
            recommendations.push(...browserRecs);
        }
        
        if (this.warnings.length > 0) {
            recommendations.push({
                type: 'warning',
                message: 'Your browser has limited support for some features.',
                action: 'Consider upgrading for the best experience'
            });
        }
        
        return recommendations;
    }

    /**
     * Get browser-specific upgrade recommendations
     */
    getBrowserUpgradeRecommendations() {
        const recs = [];
        
        switch (this.browserInfo.name) {
            case 'Chrome':
                if (parseInt(this.browserInfo.version) < 66) {
                    recs.push({
                        type: 'upgrade',
                        browser: 'Chrome',
                        message: 'Chrome 66+ required for AudioWorklet support',
                        downloadUrl: 'https://www.google.com/chrome/'
                    });
                }
                break;
                
            case 'Firefox':
                if (parseInt(this.browserInfo.version) < 76) {
                    recs.push({
                        type: 'upgrade',
                        browser: 'Firefox',
                        message: 'Firefox 76+ required for AudioWorklet support',
                        downloadUrl: 'https://www.mozilla.org/firefox/'
                    });
                }
                break;
                
            case 'Safari':
                if (parseInt(this.browserInfo.version) < 14) {
                    recs.push({
                        type: 'upgrade',
                        browser: 'Safari',
                        message: 'Safari 14+ required for full WebAssembly support',
                        downloadUrl: 'https://www.apple.com/safari/'
                    });
                }
                break;
                
            default:
                recs.push({
                    type: 'alternative',
                    message: 'For best experience, use Chrome 66+, Firefox 76+, or Safari 14+',
                    browsers: [
                        { name: 'Chrome', url: 'https://www.google.com/chrome/' },
                        { name: 'Firefox', url: 'https://www.mozilla.org/firefox/' },
                        { name: 'Safari', url: 'https://www.apple.com/safari/' }
                    ]
                });
        }
        
        return recs;
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = BrowserCapabilityDetector;
} else {
    window.BrowserCapabilityDetector = BrowserCapabilityDetector;
} 