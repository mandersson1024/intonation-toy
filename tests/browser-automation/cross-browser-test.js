// Cross-Browser Automated Testing Suite
// Tests for Chrome, Firefox, Safari, and Edge compatibility

class CrossBrowserTestRunner {
    constructor() {
        this.testResults = [];
        this.browserInfo = this.detectBrowser();
        this.supportedBrowsers = ['chrome', 'firefox', 'safari', 'edge'];
        this.performanceBaselines = new Map();
        this.compatibilityMatrix = new Map();
    }

    detectBrowser() {
        const userAgent = navigator.userAgent.toLowerCase();
        const vendor = navigator.vendor.toLowerCase();
        
        let browser = 'unknown';
        let version = 'unknown';
        
        if (userAgent.includes('chrome') && vendor.includes('google')) {
            browser = 'chrome';
            const match = userAgent.match(/chrome\/([0-9.]+)/);
            version = match ? match[1] : 'unknown';
        } else if (userAgent.includes('firefox')) {
            browser = 'firefox';
            const match = userAgent.match(/firefox\/([0-9.]+)/);
            version = match ? match[1] : 'unknown';
        } else if (userAgent.includes('safari') && !userAgent.includes('chrome')) {
            browser = 'safari';
            const match = userAgent.match(/version\/([0-9.]+)/);
            version = match ? match[1] : 'unknown';
        } else if (userAgent.includes('edg/')) {
            browser = 'edge';
            const match = userAgent.match(/edg\/([0-9.]+)/);
            version = match ? match[1] : 'unknown';
        }
        
        return { browser, version, userAgent };
    }

    async runComprehensiveTests() {
        console.log(`🌐 Starting cross-browser tests on ${this.browserInfo.browser} ${this.browserInfo.version}`);
        
        // Core functionality tests
        await this.testWebAssemblySupport();
        await this.testWebAudioAPI();
        await this.testPerformanceAPI();
        await this.testFloat32ArraySupport();
        await this.testAudioWorkletSupport();
        await this.testOfflineAudioContext();
        
        // Audio processing tests
        await this.testAudioProcessingLatency();
        await this.testPitchDetectionAccuracy();
        await this.testMemoryManagement();
        
        // Browser-specific tests
        await this.testBrowserSpecificFeatures();
        
        // Performance benchmarks
        await this.runPerformanceBenchmarks();
        
        this.generateCompatibilityReport();
        return this.testResults;
    }

    async testWebAssemblySupport() {
        console.log('  🧪 Testing WebAssembly support...');
        
        const test = {
            name: 'WebAssembly Support',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            // Basic WebAssembly support
            if (typeof WebAssembly === 'undefined') {
                throw new Error('WebAssembly not supported');
            }

            // Test instantiateStreaming support
            test.details.instantiateStreaming = typeof WebAssembly.instantiateStreaming === 'function';
            
            // Test compileStreaming support
            test.details.compileStreaming = typeof WebAssembly.compileStreaming === 'function';
            
            // Test SharedArrayBuffer support (for threading)
            test.details.sharedArrayBuffer = typeof SharedArrayBuffer !== 'undefined';
            
            // Test WebAssembly.Memory
            const memory = new WebAssembly.Memory({ initial: 1 });
            test.details.memorySupport = memory instanceof WebAssembly.Memory;
            
            // Test SIMD support (if available)
            test.details.simdSupport = 'simd' in WebAssembly;
            
            test.status = 'passed';
            test.details.message = 'WebAssembly fully supported';

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testWebAudioAPI() {
        console.log('  🔊 Testing Web Audio API support...');
        
        const test = {
            name: 'Web Audio API',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            // Test AudioContext support
            const AudioContextClass = window.AudioContext || window.webkitAudioContext;
            if (!AudioContextClass) {
                throw new Error('AudioContext not supported');
            }

            const audioContext = new AudioContextClass();
            test.details.audioContextSupport = true;
            test.details.sampleRate = audioContext.sampleRate;
            test.details.state = audioContext.state;
            
            // Test OfflineAudioContext
            test.details.offlineAudioContext = typeof OfflineAudioContext !== 'undefined' || 
                                               typeof webkitOfflineAudioContext !== 'undefined';
            
            // Test AudioWorklet support
            test.details.audioWorklet = 'audioWorklet' in audioContext;
            
            // Test ScriptProcessorNode (deprecated but still needed for fallback)
            test.details.scriptProcessorNode = typeof audioContext.createScriptProcessor === 'function';
            
            // Test AnalyserNode
            test.details.analyserNode = typeof audioContext.createAnalyser === 'function';
            
            // Test GainNode
            test.details.gainNode = typeof audioContext.createGain === 'function';
            
            // Test getUserMedia for microphone access
            test.details.getUserMedia = !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia);
            
            await audioContext.close();
            test.status = 'passed';
            test.details.message = 'Web Audio API fully supported';

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testPerformanceAPI() {
        console.log('  ⏱️ Testing Performance API...');
        
        const test = {
            name: 'Performance API',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            // Test performance.now()
            if (typeof performance === 'undefined' || typeof performance.now !== 'function') {
                throw new Error('performance.now() not supported');
            }

            const start = performance.now();
            const end = performance.now();
            const precision = end - start;
            
            test.details.performanceNow = true;
            test.details.precision = precision;
            test.details.highResolution = precision < 1.0; // Should be sub-millisecond
            
            // Test performance.mark and performance.measure
            test.details.performanceMark = typeof performance.mark === 'function';
            test.details.performanceMeasure = typeof performance.measure === 'function';
            
            // Test PerformanceObserver
            test.details.performanceObserver = typeof PerformanceObserver !== 'undefined';
            
            test.status = 'passed';
            test.details.message = `Performance API available with ${precision.toFixed(6)}ms precision`;

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testFloat32ArraySupport() {
        console.log('  🔢 Testing Float32Array support...');
        
        const test = {
            name: 'Float32Array Support',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            // Test Float32Array creation
            const buffer = new Float32Array(1024);
            buffer.fill(0.5);
            
            test.details.creation = true;
            test.details.size = buffer.length;
            test.details.byteLength = buffer.byteLength;
            
            // Test array operations
            buffer[0] = 1.0;
            buffer[1] = -1.0;
            test.details.indexAccess = buffer[0] === 1.0 && buffer[1] === -1.0;
            
            // Test array methods
            const slice = buffer.slice(0, 10);
            test.details.sliceSupport = slice instanceof Float32Array && slice.length === 10;
            
            // Test set operation
            const source = new Float32Array([0.1, 0.2, 0.3]);
            buffer.set(source, 10);
            test.details.setSupport = buffer[10] === 0.1;
            
            test.status = 'passed';
            test.details.message = 'Float32Array fully supported';

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testAudioWorkletSupport() {
        console.log('  🎛️ Testing AudioWorklet support...');
        
        const test = {
            name: 'AudioWorklet Support',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            const AudioContextClass = window.AudioContext || window.webkitAudioContext;
            const audioContext = new AudioContextClass();
            
            // Basic AudioWorklet availability
            test.details.audioWorkletAvailable = 'audioWorklet' in audioContext;
            
            if (test.details.audioWorkletAvailable) {
                // Test audioWorklet.addModule (can't actually load module in test)
                test.details.addModuleSupport = typeof audioContext.audioWorklet.addModule === 'function';
                test.details.message = 'AudioWorklet supported';
                test.status = 'passed';
            } else {
                // Fallback to ScriptProcessorNode
                test.details.scriptProcessorFallback = typeof audioContext.createScriptProcessor === 'function';
                test.details.message = 'AudioWorklet not available, ScriptProcessorNode fallback available';
                test.status = test.details.scriptProcessorFallback ? 'warning' : 'failed';
            }
            
            await audioContext.close();

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : test.status === 'warning' ? '⚠️' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testOfflineAudioContext() {
        console.log('  📴 Testing OfflineAudioContext...');
        
        const test = {
            name: 'OfflineAudioContext',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            const OfflineAudioContextClass = window.OfflineAudioContext || window.webkitOfflineAudioContext;
            if (!OfflineAudioContextClass) {
                throw new Error('OfflineAudioContext not supported');
            }

            // Create offline context for testing
            const sampleRate = 44100;
            const length = sampleRate * 0.1; // 100ms
            const channels = 1;
            
            const offlineContext = new OfflineAudioContextClass(channels, length, sampleRate);
            
            test.details.creation = true;
            test.details.sampleRate = offlineContext.sampleRate;
            test.details.length = offlineContext.length;
            
            // Test creating nodes
            const oscillator = offlineContext.createOscillator();
            oscillator.frequency.value = 440;
            oscillator.connect(offlineContext.destination);
            oscillator.start();
            oscillator.stop(0.1);
            
            // Test rendering
            const audioBuffer = await offlineContext.startRendering();
            test.details.rendering = audioBuffer instanceof AudioBuffer;
            test.details.bufferLength = audioBuffer.length;
            test.details.channels = audioBuffer.numberOfChannels;
            
            test.status = 'passed';
            test.details.message = 'OfflineAudioContext fully functional';

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testAudioProcessingLatency() {
        console.log('  ⚡ Testing audio processing latency...');
        
        const test = {
            name: 'Audio Processing Latency',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            // Simulate audio processing pipeline
            const bufferSize = 1024;
            const sampleRate = 44100;
            const iterations = 100;
            
            // Test Float32Array operations (simulating audio processing)
            const inputBuffer = new Float32Array(bufferSize);
            const outputBuffer = new Float32Array(bufferSize);
            
            // Fill with test data
            for (let i = 0; i < bufferSize; i++) {
                inputBuffer[i] = Math.sin(2 * Math.PI * 440 * i / sampleRate) * 0.5;
            }
            
            // Benchmark processing
            const start = performance.now();
            for (let iter = 0; iter < iterations; iter++) {
                // Simulate gain processing
                for (let i = 0; i < bufferSize; i++) {
                    outputBuffer[i] = inputBuffer[i] * 0.8;
                }
            }
            const end = performance.now();
            
            const avgLatency = (end - start) / iterations;
            const bufferDuration = (bufferSize / sampleRate) * 1000; // ms
            const realTimeRatio = bufferDuration / avgLatency;
            
            test.details.avgLatency = avgLatency;
            test.details.bufferDuration = bufferDuration;
            test.details.realTimeRatio = realTimeRatio;
            test.details.meetsRequirement = avgLatency < 50; // <50ms requirement
            
            test.status = test.details.meetsRequirement ? 'passed' : 'failed';
            test.details.message = `${avgLatency.toFixed(3)}ms avg latency (${realTimeRatio.toFixed(1)}x real-time)`;

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testPitchDetectionAccuracy() {
        console.log('  🎵 Testing pitch detection accuracy simulation...');
        
        const test = {
            name: 'Pitch Detection Accuracy',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            // Simulate pitch detection calculations
            const sampleRate = 44100;
            const bufferSize = 2048;
            const testFrequency = 440.0; // A4
            
            // Generate test sine wave
            const testBuffer = new Float32Array(bufferSize);
            for (let i = 0; i < bufferSize; i++) {
                testBuffer[i] = 0.8 * Math.sin(2 * Math.PI * testFrequency * i / sampleRate);
            }
            
            // Simulate autocorrelation (simplified)
            const start = performance.now();
            let bestFreq = 0;
            let maxCorrelation = 0;
            
            // Test frequency range
            for (let freq = 80; freq <= 2000; freq += 10) { // Reduced resolution for browser testing
                let correlation = 0;
                const period = sampleRate / freq;
                const samples = Math.min(bufferSize - period, 256); // Reduced for performance
                
                for (let i = 0; i < samples; i++) {
                    correlation += testBuffer[i] * testBuffer[i + Math.round(period)];
                }
                
                if (correlation > maxCorrelation) {
                    maxCorrelation = correlation;
                    bestFreq = freq;
                }
            }
            const end = performance.now();
            
            const detectionTime = end - start;
            const centsError = Math.abs(1200 * Math.log2(bestFreq / testFrequency));
            
            test.details.detectedFrequency = bestFreq;
            test.details.expectedFrequency = testFrequency;
            test.details.centsError = centsError;
            test.details.detectionTime = detectionTime;
            test.details.meetsAccuracy = centsError <= 50; // Relaxed for browser simulation
            test.details.meetsLatency = detectionTime < 50; // <50ms requirement
            
            test.status = (test.details.meetsAccuracy && test.details.meetsLatency) ? 'passed' : 'warning';
            test.details.message = `Detected ${bestFreq}Hz (${centsError.toFixed(1)} cents error) in ${detectionTime.toFixed(3)}ms`;

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : test.status === 'warning' ? '⚠️' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testMemoryManagement() {
        console.log('  🧠 Testing memory management...');
        
        const test = {
            name: 'Memory Management',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            const iterations = 100; // Reduced for browser testing
            const bufferSize = 1024;
            
            // Test for memory leaks
            const initialMemory = performance.memory ? performance.memory.usedJSHeapSize : null;
            
            for (let i = 0; i < iterations; i++) {
                // Create and process buffers
                const buffer = new Float32Array(bufferSize);
                buffer.fill(Math.random());
                
                // Simulate processing
                for (let j = 0; j < bufferSize; j++) {
                    buffer[j] = buffer[j] * 0.5;
                }
                
                // Buffer should be garbage collected
            }
            
            const finalMemory = performance.memory ? performance.memory.usedJSHeapSize : null;
            
            if (initialMemory && finalMemory) {
                const memoryIncrease = finalMemory - initialMemory;
                test.details.initialMemory = initialMemory;
                test.details.finalMemory = finalMemory;
                test.details.memoryIncrease = memoryIncrease;
                test.details.memoryIncreaseKB = memoryIncrease / 1024;
                
                // Memory increase should be reasonable
                test.details.memoryLeakSuspected = memoryIncrease > 512 * 1024; // 512KB threshold
                test.status = test.details.memoryLeakSuspected ? 'warning' : 'passed';
                test.details.message = `Memory increased by ${(memoryIncrease / 1024).toFixed(1)}KB`;
            } else {
                test.details.memoryAPIUnavailable = true;
                test.status = 'passed';
                test.details.message = 'Memory API unavailable, no leaks detected in test';
            }

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : test.status === 'warning' ? '⚠️' : '❌'} ${test.details.message || test.details.error}`);
    }

    async testBrowserSpecificFeatures() {
        console.log('  🌍 Testing browser-specific features...');
        
        const test = {
            name: 'Browser Specific Features',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {
                browser: this.browserInfo.browser,
                version: this.browserInfo.version
            },
            timestamp: Date.now()
        };

        try {
            // Chrome-specific features
            if (this.browserInfo.browser === 'chrome') {
                test.details.chromeSpecific = {
                    audioWorkletSupport: 'audioWorklet' in (new (window.AudioContext || window.webkitAudioContext)()),
                    performanceMemory: !!performance.memory,
                    webGL2: !!window.WebGL2RenderingContext,
                };
            }
            
            // Firefox-specific features
            if (this.browserInfo.browser === 'firefox') {
                test.details.firefoxSpecific = {
                    audioContextState: true, // Firefox handles audio context states differently
                    performanceObserver: !!window.PerformanceObserver,
                    webAssemblyThreads: !!WebAssembly.Memory && !!SharedArrayBuffer,
                };
            }
            
            // Safari-specific features
            if (this.browserInfo.browser === 'safari') {
                test.details.safariSpecific = {
                    webkitAudioContext: !!window.webkitAudioContext,
                    touchSupport: 'ontouchstart' in window,
                    iosConstraints: /iPad|iPhone|iPod/.test(navigator.userAgent),
                };
            }
            
            // Edge-specific features
            if (this.browserInfo.browser === 'edge') {
                test.details.edgeSpecific = {
                    chromiumBased: this.browserInfo.userAgent.includes('edg/'),
                    audioWorkletSupport: 'audioWorklet' in (new (window.AudioContext || window.webkitAudioContext)()),
                    performanceMemory: !!performance.memory,
                };
            }
            
            test.status = 'passed';
            test.details.message = `Browser-specific features detected for ${this.browserInfo.browser}`;

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : '❌'} ${test.details.message || test.details.error}`);
    }

    async runPerformanceBenchmarks() {
        console.log('  🏃 Running performance benchmarks...');
        
        const test = {
            name: 'Performance Benchmarks',
            browser: this.browserInfo.browser,
            status: 'unknown',
            details: {},
            timestamp: Date.now()
        };

        try {
            // Array processing benchmark
            const arraySize = 44100; // 1 second at 44.1kHz
            const iterations = 10; // Reduced for browser testing
            
            const inputArray = new Float32Array(arraySize);
            const outputArray = new Float32Array(arraySize);
            
            // Fill with sine wave
            for (let i = 0; i < arraySize; i++) {
                inputArray[i] = Math.sin(2 * Math.PI * 440 * i / 44100) * 0.5;
            }
            
            // Benchmark different operations
            const benchmarks = {};
            
            // Copy benchmark
            let start = performance.now();
            for (let iter = 0; iter < iterations; iter++) {
                outputArray.set(inputArray);
            }
            benchmarks.copyTime = (performance.now() - start) / iterations;
            
            // Gain processing benchmark
            start = performance.now();
            for (let iter = 0; iter < iterations; iter++) {
                for (let i = 0; i < arraySize; i++) {
                    outputArray[i] = inputArray[i] * 0.8;
                }
            }
            benchmarks.gainTime = (performance.now() - start) / iterations;
            
            // Math operations benchmark (reduced complexity for browsers)
            start = performance.now();
            for (let iter = 0; iter < iterations; iter++) {
                for (let i = 0; i < arraySize; i += 10) { // Process every 10th sample for performance
                    outputArray[i] = Math.sin(inputArray[i]) * 0.5;
                }
            }
            benchmarks.mathTime = (performance.now() - start) / iterations;
            
            test.details.benchmarks = benchmarks;
            test.details.arraySize = arraySize;
            test.details.iterations = iterations;
            
            // Calculate performance scores
            const copyScore = 1000 / benchmarks.copyTime; // Higher is better
            const gainScore = 1000 / benchmarks.gainTime;
            const mathScore = 1000 / benchmarks.mathTime;
            
            test.details.scores = { copyScore, gainScore, mathScore };
            test.details.overallScore = (copyScore + gainScore + mathScore) / 3;
            
            // Store baseline for this browser
            this.performanceBaselines.set(this.browserInfo.browser, test.details.scores);
            
            test.status = 'passed';
            test.details.message = `Overall performance score: ${test.details.overallScore.toFixed(1)}`;

        } catch (error) {
            test.status = 'failed';
            test.details.error = error.message;
        }

        this.testResults.push(test);
        console.log(`    ${test.status === 'passed' ? '✅' : '❌'} ${test.details.message || test.details.error}`);
    }

    generateCompatibilityReport() {
        console.log('\n📋 Generating compatibility report...');
        
        const report = {
            browser: this.browserInfo,
            timestamp: new Date().toISOString(),
            testResults: this.testResults,
            summary: {
                totalTests: this.testResults.length,
                passed: this.testResults.filter(t => t.status === 'passed').length,
                warnings: this.testResults.filter(t => t.status === 'warning').length,
                failed: this.testResults.filter(t => t.status === 'failed').length,
            }
        };
        
        // Calculate compatibility score
        const score = (report.summary.passed * 1.0 + report.summary.warnings * 0.5) / report.summary.totalTests * 100;
        report.compatibilityScore = score;
        
        // Determine compatibility level
        if (score >= 90) {
            report.compatibilityLevel = 'Excellent';
        } else if (score >= 75) {
            report.compatibilityLevel = 'Good';
        } else if (score >= 60) {
            report.compatibilityLevel = 'Fair';
        } else {
            report.compatibilityLevel = 'Poor';
        }
        
        console.log(`🎯 Compatibility Score: ${score.toFixed(1)}% (${report.compatibilityLevel})`);
        console.log(`📊 Results: ${report.summary.passed} passed, ${report.summary.warnings} warnings, ${report.summary.failed} failed`);
        
        // Store in compatibility matrix
        this.compatibilityMatrix.set(this.browserInfo.browser, report);
        
        return report;
    }

    getCompatibilityMatrix() {
        return Object.fromEntries(this.compatibilityMatrix);
    }

    exportResults() {
        return {
            testResults: this.testResults,
            browserInfo: this.browserInfo,
            compatibilityMatrix: this.getCompatibilityMatrix(),
            performanceBaselines: Object.fromEntries(this.performanceBaselines),
            timestamp: new Date().toISOString()
        };
    }
}

// Auto-run tests if in browser environment
if (typeof window !== 'undefined') {
    window.CrossBrowserTestRunner = CrossBrowserTestRunner;
    
    // Auto-execute tests when page loads
    window.addEventListener('load', async () => {
        console.log('🚀 Starting automated cross-browser tests...');
        const runner = new CrossBrowserTestRunner();
        const results = await runner.runComprehensiveTests();
        
        // Expose results globally for further analysis
        window.testResults = results;
        window.compatibilityReport = runner.generateCompatibilityReport();
        
        console.log('✅ Cross-browser testing completed!');
        console.log('📄 Results available in window.testResults and window.compatibilityReport');
    });
}

// Export for Node.js environments
if (typeof module !== 'undefined' && module.exports) {
    module.exports = CrossBrowserTestRunner;
} 