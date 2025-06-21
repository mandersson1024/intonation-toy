/**
 * Demo Mode System
 * Story 2.3: Unsupported Browser Fallback
 * 
 * Provides educational experience with pre-recorded audio samples
 * when real-time microphone access is not available.
 */

class DemoMode {
    constructor() {
        this.isActive = false;
        this.currentSample = null;
        this.samples = this.initializeSamples();
        this.demoUI = null;
        this.audioContext = null;
        this.gainNode = null;
        this.analyserNode = null;
        this.isPlaying = false;
        this.animationId = null;
    }

    /**
     * Initialize pre-recorded audio samples for demonstration
     */
    initializeSamples() {
        return [
            {
                id: 'perfect-a4',
                name: 'Perfect A4 (440 Hz)',
                description: 'A perfectly tuned A4 note - the standard tuning reference',
                frequency: 440,
                category: 'reference',
                icon: '🎯',
                educational: 'This is the standard tuning pitch used by orchestras worldwide!'
            },
            {
                id: 'major-third',
                name: 'Major Third Interval',
                description: 'C4 to E4 - A bright, happy-sounding interval',
                frequencies: [261.63, 329.63], // C4 to E4
                category: 'intervals',
                icon: '🎵',
                educational: 'Major thirds sound happy and bright - like the start of "Happy Birthday"!'
            },
            {
                id: 'perfect-fifth',
                name: 'Perfect Fifth Interval',
                description: 'C4 to G4 - A stable, strong interval',
                frequencies: [261.63, 392.00], // C4 to G4
                category: 'intervals',
                icon: '🎼',
                educational: 'Perfect fifths are super stable - like the start of "Star Wars" theme!'
            },
            {
                id: 'octave',
                name: 'Octave Interval',
                description: 'C4 to C5 - Same note, different register',
                frequencies: [261.63, 523.25], // C4 to C5
                category: 'intervals',
                icon: '🎭',
                educational: 'Octaves are the same note - they sound "the same but different"!'
            },
            {
                id: 'slightly-flat',
                name: 'Slightly Flat Note',
                description: 'A4 at 435 Hz - Notice how it sounds "off"',
                frequency: 435,
                category: 'tuning',
                icon: '📉',
                educational: 'This note is flat - it sounds a bit "saggy" compared to perfect pitch!'
            },
            {
                id: 'slightly-sharp',
                name: 'Slightly Sharp Note',
                description: 'A4 at 445 Hz - Sounds tense and high',
                frequency: 445,
                category: 'tuning',
                icon: '📈',
                educational: 'This note is sharp - it sounds "tense" and reaches upward!'
            }
        ];
    }

    /**
     * Activate demo mode
     */
    async activate(reason = 'Browser compatibility') {
        console.log('🎮 Activating demo mode:', reason);
        this.isActive = true;
        
        try {
            await this.setupAudioContext();
            this.createDemoUI();
            this.showDemoMode();
            return true;
        } catch (error) {
            console.error('Failed to activate demo mode:', error);
            this.showSimpleFallback();
            return false;
        }
    }

    /**
     * Setup audio context for demo playback
     */
    async setupAudioContext() {
        if (!window.AudioContext && !window.webkitAudioContext) {
            throw new Error('No audio context available');
        }

        const AudioContextClass = window.AudioContext || window.webkitAudioContext;
        this.audioContext = new AudioContextClass();
        
        // Create audio processing nodes
        this.gainNode = this.audioContext.createGain();
        this.analyserNode = this.audioContext.createAnalyser();
        this.analyserNode.fftSize = 2048;
        
        // Connect nodes
        this.gainNode.connect(this.analyserNode);
        this.analyserNode.connect(this.audioContext.destination);
        
        // Set initial volume
        this.gainNode.gain.value = 0.3;
    }

    /**
     * Create demo mode user interface
     */
    createDemoUI() {
        this.demoUI = document.createElement('div');
        this.demoUI.id = 'demo-mode';
        this.demoUI.className = 'demo-mode-container';
        this.demoUI.innerHTML = `
            <div class="demo-header">
                <div class="demo-icon">🎮</div>
                <h2>Demo Mode</h2>
                <p class="demo-subtitle">Explore pitch detection with pre-recorded samples</p>
            </div>
            
            <div class="demo-explanation">
                <div class="demo-info-box">
                    <h3>🎵 Learn About Pitch!</h3>
                    <p>Even though we can't use your microphone, you can still learn about pitch and intervals with these demo sounds!</p>
                </div>
            </div>
            
            <div class="demo-controls">
                <div class="demo-categories">
                    <button class="demo-category-btn active" data-category="all">
                        🎯 All Samples
                    </button>
                    <button class="demo-category-btn" data-category="reference">
                        📐 Reference Notes
                    </button>
                    <button class="demo-category-btn" data-category="intervals">
                        🎵 Intervals
                    </button>
                    <button class="demo-category-btn" data-category="tuning">
                        🎚️ Tuning Examples
                    </button>
                </div>
                
                <div class="demo-samples" id="demo-samples">
                    <!-- Samples will be populated here -->
                </div>
                
                <div class="demo-player">
                    <div class="demo-player-info">
                        <div class="demo-current-sample">
                            <div class="demo-sample-name">Select a sample to play</div>
                            <div class="demo-sample-description"></div>
                        </div>
                    </div>
                    
                    <div class="demo-player-controls">
                        <button id="demo-play-btn" class="demo-play-btn" disabled>
                            <span class="play-icon">▶️</span>
                            <span class="play-text">Play Sample</span>
                        </button>
                        <button id="demo-stop-btn" class="demo-stop-btn" disabled>
                            <span class="stop-icon">⏹️</span>
                            <span class="stop-text">Stop</span>
                        </button>
                    </div>
                    
                    <div class="demo-volume-control">
                        <label for="demo-volume">🔊 Volume:</label>
                        <input type="range" id="demo-volume" min="0" max="100" value="30" class="demo-volume-slider">
                        <span class="demo-volume-value">30%</span>
                    </div>
                </div>
                
                <div class="demo-visualization">
                    <div class="demo-pitch-display">
                        <div class="demo-pitch-value">Select a sample</div>
                        <div class="demo-pitch-label">Detected Pitch</div>
                    </div>
                    <canvas id="demo-waveform" class="demo-waveform-canvas" width="400" height="100"></canvas>
                </div>
                
                <div class="demo-educational-content">
                    <div class="demo-learning-box">
                        <h4>🎓 What you're learning:</h4>
                        <div class="demo-educational-text">
                            Choose a sample above to learn about different pitches and intervals!
                        </div>
                    </div>
                </div>
                
                <div class="demo-upgrade-suggestion">
                    <div class="demo-upgrade-box">
                        <h4>🚀 Want the full experience?</h4>
                        <p>Upgrade your browser to use your own voice or instrument!</p>
                        <div class="demo-browser-links">
                            <a href="https://www.google.com/chrome/" target="_blank" class="demo-browser-link">
                                📱 Chrome 66+
                            </a>
                            <a href="https://www.mozilla.org/firefox/" target="_blank" class="demo-browser-link">
                                🦊 Firefox 76+
                            </a>
                            <a href="https://www.apple.com/safari/" target="_blank" class="demo-browser-link">
                                🧭 Safari 14+
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        `;
        
        document.body.appendChild(this.demoUI);
        this.populateSamples();
        this.bindDemoEvents();
    }

    /**
     * Populate sample buttons
     */
    populateSamples() {
        const samplesContainer = document.getElementById('demo-samples');
        
        this.samples.forEach(sample => {
            const sampleBtn = document.createElement('button');
            sampleBtn.className = 'demo-sample-btn';
            sampleBtn.dataset.sampleId = sample.id;
            sampleBtn.dataset.category = sample.category;
            sampleBtn.innerHTML = `
                <div class="demo-sample-icon">${sample.icon}</div>
                <div class="demo-sample-info">
                    <div class="demo-sample-title">${sample.name}</div>
                    <div class="demo-sample-desc">${sample.description}</div>
                </div>
            `;
            
            samplesContainer.appendChild(sampleBtn);
        });
    }

    /**
     * Bind demo mode event listeners
     */
    bindDemoEvents() {
        // Category filtering
        document.querySelectorAll('.demo-category-btn').forEach(btn => {
            btn.addEventListener('click', (e) => this.filterSamples(e.target.dataset.category));
        });
        
        // Sample selection
        document.querySelectorAll('.demo-sample-btn').forEach(btn => {
            btn.addEventListener('click', (e) => this.selectSample(e.currentTarget.dataset.sampleId));
        });
        
        // Playback controls
        document.getElementById('demo-play-btn').addEventListener('click', () => this.playCurrentSample());
        document.getElementById('demo-stop-btn').addEventListener('click', () => this.stopPlayback());
        
        // Volume control
        const volumeSlider = document.getElementById('demo-volume');
        volumeSlider.addEventListener('input', (e) => this.setVolume(e.target.value));
    }

    /**
     * Filter samples by category
     */
    filterSamples(category) {
        // Update active category button
        document.querySelectorAll('.demo-category-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.category === category);
        });
        
        // Show/hide samples
        document.querySelectorAll('.demo-sample-btn').forEach(btn => {
            const shouldShow = category === 'all' || btn.dataset.category === category;
            btn.style.display = shouldShow ? 'flex' : 'none';
        });
    }

    /**
     * Select a sample for playback
     */
    selectSample(sampleId) {
        this.currentSample = this.samples.find(s => s.id === sampleId);
        
        if (!this.currentSample) return;
        
        // Update UI
        document.querySelectorAll('.demo-sample-btn').forEach(btn => {
            btn.classList.toggle('selected', btn.dataset.sampleId === sampleId);
        });
        
        // Update player info
        document.querySelector('.demo-sample-name').textContent = this.currentSample.name;
        document.querySelector('.demo-sample-description').textContent = this.currentSample.description;
        
        // Update educational content
        document.querySelector('.demo-educational-text').textContent = this.currentSample.educational;
        
        // Enable play button
        document.getElementById('demo-play-btn').disabled = false;
        
        // Update pitch display
        if (this.currentSample.frequency) {
            document.querySelector('.demo-pitch-value').textContent = `${this.currentSample.frequency} Hz`;
        } else if (this.currentSample.frequencies) {
            document.querySelector('.demo-pitch-value').textContent = 
                `${this.currentSample.frequencies.join(' Hz, ')} Hz`;
        }
    }

    /**
     * Play the currently selected sample
     */
    async playCurrentSample() {
        if (!this.currentSample || this.isPlaying) return;
        
        try {
            // Resume audio context if needed
            if (this.audioContext.state === 'suspended') {
                await this.audioContext.resume();
            }
            
            this.isPlaying = true;
            
            // Update UI
            document.getElementById('demo-play-btn').disabled = true;
            document.getElementById('demo-stop-btn').disabled = false;
            
            // Generate and play audio
            if (this.currentSample.frequency) {
                this.playSingleTone(this.currentSample.frequency);
            } else if (this.currentSample.frequencies) {
                this.playInterval(this.currentSample.frequencies);
            }
            
            // Start visualization
            this.startVisualization();
            
        } catch (error) {
            console.error('Error playing sample:', error);
            this.stopPlayback();
        }
    }

    /**
     * Play a single tone
     */
    playSingleTone(frequency, duration = 3000) {
        const oscillator = this.audioContext.createOscillator();
        const envelope = this.audioContext.createGain();
        
        oscillator.connect(envelope);
        envelope.connect(this.gainNode);
        
        oscillator.frequency.value = frequency;
        oscillator.type = 'sine';
        
        // Envelope for smooth attack/release
        const now = this.audioContext.currentTime;
        envelope.gain.setValueAtTime(0, now);
        envelope.gain.linearRampToValueAtTime(1, now + 0.1);
        envelope.gain.linearRampToValueAtTime(0.8, now + duration/1000 - 0.2);
        envelope.gain.linearRampToValueAtTime(0, now + duration/1000);
        
        oscillator.start(now);
        oscillator.stop(now + duration/1000);
        
        oscillator.onended = () => {
            if (this.isPlaying) {
                this.stopPlayback();
            }
        };
        
        this.currentOscillator = oscillator;
    }

    /**
     * Play an interval (two frequencies)
     */
    playInterval(frequencies, duration = 4000) {
        const oscillators = frequencies.map(freq => {
            const oscillator = this.audioContext.createOscillator();
            const envelope = this.audioContext.createGain();
            
            oscillator.connect(envelope);
            envelope.connect(this.gainNode);
            
            oscillator.frequency.value = freq;
            oscillator.type = 'sine';
            
            return { oscillator, envelope };
        });
        
        const now = this.audioContext.currentTime;
        
        oscillators.forEach(({ oscillator, envelope }) => {
            // Envelope
            envelope.gain.setValueAtTime(0, now);
            envelope.gain.linearRampToValueAtTime(0.5, now + 0.1);
            envelope.gain.linearRampToValueAtTime(0.4, now + duration/1000 - 0.2);
            envelope.gain.linearRampToValueAtTime(0, now + duration/1000);
            
            oscillator.start(now);
            oscillator.stop(now + duration/1000);
        });
        
        // Stop when the first oscillator ends
        oscillators[0].oscillator.onended = () => {
            if (this.isPlaying) {
                this.stopPlayback();
            }
        };
        
        this.currentOscillators = oscillators.map(o => o.oscillator);
    }

    /**
     * Stop playback
     */
    stopPlayback() {
        this.isPlaying = false;
        
        // Stop oscillators
        if (this.currentOscillator) {
            try {
                this.currentOscillator.stop();
            } catch (e) {
                // Oscillator might already be stopped
            }
            this.currentOscillator = null;
        }
        
        if (this.currentOscillators) {
            this.currentOscillators.forEach(osc => {
                try {
                    osc.stop();
                } catch (e) {
                    // Oscillator might already be stopped
                }
            });
            this.currentOscillators = null;
        }
        
        // Stop visualization
        this.stopVisualization();
        
        // Update UI
        document.getElementById('demo-play-btn').disabled = false;
        document.getElementById('demo-stop-btn').disabled = true;
    }

    /**
     * Set playback volume
     */
    setVolume(value) {
        const volume = value / 100;
        if (this.gainNode) {
            this.gainNode.gain.value = volume;
        }
        document.querySelector('.demo-volume-value').textContent = `${value}%`;
    }

    /**
     * Start waveform visualization
     */
    startVisualization() {
        const canvas = document.getElementById('demo-waveform');
        const ctx = canvas.getContext('2d');
        const dataArray = new Uint8Array(this.analyserNode.frequencyBinCount);
        
        const draw = () => {
            if (!this.isPlaying) return;
            
            this.analyserNode.getByteTimeDomainData(dataArray);
            
            ctx.fillStyle = '#f8f9fa';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            
            ctx.lineWidth = 2;
            ctx.strokeStyle = '#007bff';
            ctx.beginPath();
            
            const sliceWidth = canvas.width / dataArray.length;
            let x = 0;
            
            for (let i = 0; i < dataArray.length; i++) {
                const v = dataArray[i] / 128.0;
                const y = v * canvas.height / 2;
                
                if (i === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
                
                x += sliceWidth;
            }
            
            ctx.stroke();
            this.animationId = requestAnimationFrame(draw);
        };
        
        draw();
    }

    /**
     * Stop visualization
     */
    stopVisualization() {
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
            this.animationId = null;
        }
        
        // Clear canvas
        const canvas = document.getElementById('demo-waveform');
        if (canvas) {
            const ctx = canvas.getContext('2d');
            ctx.fillStyle = '#f8f9fa';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
        }
    }

    /**
     * Show demo mode interface
     */
    showDemoMode() {
        // Hide any existing permission modal
        const permissionModal = document.getElementById('permission-modal');
        if (permissionModal) {
            permissionModal.style.display = 'none';
        }
        
        // Show demo mode
        this.demoUI.style.display = 'block';
        
        // Update status
        const statusDisplay = document.getElementById('microphone-status');
        if (statusDisplay) {
            statusDisplay.className = 'status info';
            statusDisplay.textContent = '🎮 Demo Mode Active - Explore pitch detection with samples!';
        }
    }

    /**
     * Show simple fallback when even demo mode fails
     */
    showSimpleFallback() {
        const fallback = document.createElement('div');
        fallback.className = 'simple-fallback';
        fallback.innerHTML = `
            <div class="fallback-content">
                <h2>🎵 Pitch Learning Mode</h2>
                <p>Your browser doesn't support the interactive features, but you can still learn!</p>
                
                <div class="fallback-lessons">
                    <div class="fallback-lesson">
                        <h3>🎯 What is Pitch?</h3>
                        <p>Pitch is how high or low a sound is. A bird chirps at a high pitch, while a lion roars at a low pitch!</p>
                    </div>
                    
                    <div class="fallback-lesson">
                        <h3>🎼 Musical Notes</h3>
                        <p>Musical notes are specific pitches. The note A4 vibrates at exactly 440 times per second (440 Hz)!</p>
                    </div>
                    
                    <div class="fallback-lesson">
                        <h3>🎵 Intervals</h3>
                        <p>Intervals are the distance between two notes. An octave is when two notes sound "the same but different"!</p>
                    </div>
                </div>
                
                <div class="fallback-upgrade">
                    <h3>🚀 Get the Full Experience</h3>
                    <p>Upgrade to a modern browser to hear examples and test your own voice!</p>
                    <div class="browser-links">
                        <a href="https://www.google.com/chrome/" target="_blank">Chrome</a>
                        <a href="https://www.mozilla.org/firefox/" target="_blank">Firefox</a>
                        <a href="https://www.apple.com/safari/" target="_blank">Safari</a>
                    </div>
                </div>
            </div>
        `;
        
        document.body.appendChild(fallback);
    }

    /**
     * Deactivate demo mode
     */
    deactivate() {
        this.isActive = false;
        this.stopPlayback();
        
        if (this.audioContext && this.audioContext.state !== 'closed') {
            this.audioContext.close();
        }
        
        if (this.demoUI) {
            this.demoUI.remove();
            this.demoUI = null;
        }
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = DemoMode;
} else {
    window.DemoMode = DemoMode;
} 