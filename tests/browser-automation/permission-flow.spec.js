/**
 * Permission Flow Integration Tests
 * Microphone Permission Request Flow
 * 
 * Tests the permission request UI, getUserMedia integration,
 * and error handling scenarios with mocked browser APIs.
 */

describe('Microphone Permission Flow', () => {
    let mockNavigator;
    let mockMediaDevices;
    let app;

    beforeEach(() => {
        // Reset DOM
        document.body.innerHTML = '';
        
        // Create mock navigator.mediaDevices
        mockMediaDevices = {
            getUserMedia: jest.fn(),
            query: jest.fn()
        };
        
        mockNavigator = {
            mediaDevices: mockMediaDevices,
            permissions: {
                query: jest.fn()
            },
            userAgent: 'Mozilla/5.0 (Chrome Test)'
        };
        
        // Replace global navigator
        Object.defineProperty(global, 'navigator', {
            value: mockNavigator,
            writable: true
        });
        
        // Mock AudioContext
        global.AudioContext = jest.fn().mockImplementation(() => ({
            state: 'running',
            sampleRate: 44100,
            resume: jest.fn().mockResolvedValue(),
            close: jest.fn().mockResolvedValue(),
            createMediaStreamSource: jest.fn().mockReturnValue({
                connect: jest.fn()
            }),
            audioWorklet: {
                addModule: jest.fn().mockResolvedValue()
            }
        }));
        
        global.AudioWorkletNode = jest.fn().mockImplementation(() => ({
            port: {
                postMessage: jest.fn(),
                onmessage: null
            },
            connect: jest.fn(),
            disconnect: jest.fn()
        }));
        
        // Mock performance
        global.performance = {
            now: jest.fn().mockReturnValue(1000)
        };
    });

    afterEach(() => {
        jest.clearAllMocks();
        if (app) {
            app.cleanup();
        }
    });

    describe('Permission Request UI', () => {
        test('should display child-friendly permission modal on initialization', async () => {
            // Mock permission state as prompt
            mockNavigator.permissions.query.mockResolvedValue({
                state: 'prompt',
                addEventListener: jest.fn()
            });

            // Load the app (simulate loading app.js)
            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();

            // Wait for initialization
            await new Promise(resolve => setTimeout(resolve, 100));

            // Check if permission modal exists
            const modal = document.getElementById('permission-modal');
            expect(modal).toBeTruthy();
            expect(modal.style.display).toBe('flex');

            // Check child-friendly content
            const heading = modal.querySelector('h2');
            expect(heading.textContent).toBe("Let's Make Music Together!");

            const benefits = modal.querySelectorAll('.benefit-item');
            expect(benefits.length).toBeGreaterThan(0);

            const grantButton = document.getElementById('grant-permission-btn');
            expect(grantButton.textContent).toContain("Let's Start!");
        });

        test('should hide modal when permission already granted', async () => {
            // Mock permission state as granted
            mockNavigator.permissions.query.mockResolvedValue({
                state: 'granted',
                addEventListener: jest.fn()
            });

            // Mock successful getUserMedia (since checkExistingPermissions will call initializeAudioPipeline)
            const mockStream = {
                getTracks: () => [{ stop: jest.fn() }]
            };
            mockNavigator.mediaDevices.getUserMedia.mockResolvedValue(mockStream);

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();

            await new Promise(resolve => setTimeout(resolve, 100));

            const modal = document.getElementById('permission-modal');
            expect(modal.style.display).toBe('none');
        });

        test('should show permission denied guidance on denied state', async () => {
            mockNavigator.permissions.query.mockResolvedValue({
                state: 'denied',
                addEventListener: jest.fn()
            });

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();

            await new Promise(resolve => setTimeout(resolve, 100));

            const modal = document.getElementById('permission-modal');
            expect(modal.style.display).toBe('flex');
            
            const errorMessage = modal.querySelector('.error-message');
            expect(errorMessage).toBeTruthy();
            expect(errorMessage.textContent).toContain('blocked');
        });
    });

    describe('getUserMedia Integration', () => {
        test('should request microphone with optimal constraints', async () => {
            const mockStream = {
                getTracks: () => [{ stop: jest.fn() }]
            };
            mockNavigator.mediaDevices.getUserMedia.mockResolvedValue(mockStream);

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            await app.requestMicrophonePermission();

            expect(mockNavigator.mediaDevices.getUserMedia).toHaveBeenCalledWith({
                audio: {
                    echoCancellation: false,
                    noiseSuppression: false,
                    autoGainControl: false,
                    sampleRate: { ideal: 44100 },
                    channelCount: { ideal: 1 }
                }
            });
        });

        test('should handle permission denial gracefully', async () => {
            const permissionError = new Error('Permission denied');
            permissionError.name = 'NotAllowedError';
            mockNavigator.mediaDevices.getUserMedia.mockRejectedValue(permissionError);

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            await app.requestMicrophonePermission();

            expect(app.permissionState).toBe('denied');
            
            const modal = document.getElementById('permission-modal');
            const errorMessage = modal.querySelector('.error-message');
            expect(errorMessage.textContent).toContain('blocked');
        });

        test('should handle missing microphone error', async () => {
            const deviceError = new Error('No microphone found');
            deviceError.name = 'NotFoundError';
            mockNavigator.mediaDevices.getUserMedia.mockRejectedValue(deviceError);

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            await app.requestMicrophonePermission();

            const modal = document.getElementById('permission-modal');
            const errorMessage = modal.querySelector('.error-message');
            expect(errorMessage.textContent).toContain('No microphone found');
        });

        test('should handle unsupported browser error', async () => {
            const supportError = new Error('Not supported');
            supportError.name = 'NotSupportedError';
            mockNavigator.mediaDevices.getUserMedia.mockRejectedValue(supportError);

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            await app.requestMicrophonePermission();

            const modal = document.getElementById('permission-modal');
            const errorMessage = modal.querySelector('.error-message');
            expect(errorMessage.textContent).toContain("doesn't support microphone");
        });
    });

    describe('Browser-Specific Guidance', () => {
        test('should provide Chrome-specific guidance', () => {
            mockNavigator.userAgent = 'Mozilla/5.0 (Chrome/90.0)';
            
            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            const guidance = app.getBrowserSpecificGuidance();
            expect(guidance).toContain('Chrome');
            expect(guidance).toContain('address bar');
        });

        test('should provide Firefox-specific guidance', () => {
            mockNavigator.userAgent = 'Mozilla/5.0 (Firefox/88.0)';
            
            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            const guidance = app.getBrowserSpecificGuidance();
            expect(guidance).toContain('Firefox');
            expect(guidance).toContain('shield icon');
        });

        test('should provide Safari-specific guidance', () => {
            mockNavigator.userAgent = 'Mozilla/5.0 (Safari/14.0)';
            
            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            const guidance = app.getBrowserSpecificGuidance();
            expect(guidance).toContain('Safari');
            expect(guidance).toContain('Settings');
        });

        test('should provide generic guidance for unknown browsers', () => {
            mockNavigator.userAgent = 'Unknown Browser';
            
            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            const guidance = app.getBrowserSpecificGuidance();
            expect(guidance).toContain('privacy settings');
        });
    });

    describe('Permission State Management', () => {
        test('should track permission state changes', async () => {
            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            expect(app.permissionState).toBe('unknown');
            
            // Simulate permission request
            const mockStream = {
                getTracks: () => [{ stop: jest.fn() }]
            };
            mockNavigator.mediaDevices.getUserMedia.mockResolvedValue(mockStream);
            
            await app.requestMicrophonePermission();
            expect(app.permissionState).toBe('granted');
        });

        test('should handle permission state transitions', () => {
            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            
            app.handlePermissionChange('granted');
            expect(app.permissionState).toBe('unknown'); // State not changed unless actually granted
            
            // Mock having microphone stream
            app.microphoneStream = { getTracks: () => [] };
            app.permissionState = 'requesting';
            
            app.handlePermissionChange('granted');
            // Should hide modal and update status
        });
    });

    describe('Accessibility Features', () => {
        test('should support keyboard navigation', async () => {
            mockNavigator.permissions.query.mockResolvedValue({
                state: 'prompt',
                addEventListener: jest.fn()
            });

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();

            await new Promise(resolve => setTimeout(resolve, 100));

            // Test Escape key handling
            const escapeEvent = new KeyboardEvent('keydown', { key: 'Escape' });
            document.dispatchEvent(escapeEvent);

            // Should close modal
            const modal = document.getElementById('permission-modal');
            expect(modal.style.display).toBe('none');
        });

        test('should focus on first button when modal opens', async () => {
            mockNavigator.permissions.query.mockResolvedValue({
                state: 'prompt',
                addEventListener: jest.fn()
            });

            // Mock focus method
            HTMLElement.prototype.focus = jest.fn();

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();

            await new Promise(resolve => setTimeout(resolve, 100));

            app.showPermissionModal();

            const grantButton = document.getElementById('grant-permission-btn');
            expect(grantButton.focus).toHaveBeenCalled();
        });
    });

    describe('Resource Cleanup', () => {
        test('should cleanup microphone stream on app destruction', () => {
            const mockTrack = { stop: jest.fn() };
            const mockStream = { getTracks: () => [mockTrack] };

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            app.microphoneStream = mockStream;
            app.audioContext = { close: jest.fn() };

            app.cleanup();

            expect(mockTrack.stop).toHaveBeenCalled();
            expect(app.microphoneStream).toBeNull();
            expect(app.audioContext.close).toHaveBeenCalled();
        });

        test('should stop AudioWorklet on cleanup', () => {
            const mockWorklet = {
                port: { postMessage: jest.fn() },
                disconnect: jest.fn()
            };

            const { PitchVisualizerApp } = require('../../web/app.js');
            app = new PitchVisualizerApp();
            app.audioWorkletNode = mockWorklet;

            app.cleanup();

            expect(mockWorklet.port.postMessage).toHaveBeenCalledWith({ type: 'stop' });
            expect(mockWorklet.disconnect).toHaveBeenCalled();
            expect(app.audioWorkletNode).toBeNull();
        });
    });
}); 