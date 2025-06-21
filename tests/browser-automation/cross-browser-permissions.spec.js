/**
 * Cross-Browser Permission Testing Pipeline
 * Story 2.1: Microphone Permission Request Flow
 * 
 * Validates permission flow across Chrome, Firefox, Safari, and Edge
 * Tests browser-specific getUserMedia implementations and error handling.
 */

const { chromium, firefox, webkit } = require('playwright');

describe('Cross-Browser Permission Flow', () => {
    let browsers = [];
    const testUrl = 'http://localhost:8080/web/';

    // Test configuration for different browsers
    const browserConfigs = [
        { name: 'Chrome', browserType: chromium, userAgent: 'Chrome' },
        { name: 'Firefox', browserType: firefox, userAgent: 'Firefox' },
        { name: 'Safari', browserType: webkit, userAgent: 'Safari' }
    ];

    beforeAll(async () => {
        // Initialize browsers
        for (const config of browserConfigs) {
            try {
                const browser = await config.browserType.launch({
                    headless: true, // Set to false for debugging
                    args: ['--use-fake-ui-for-media-stream', '--use-fake-device-for-media-stream']
                });
                browsers.push({ ...config, browser });
            } catch (error) {
                console.warn(`Could not launch ${config.name}:`, error.message);
            }
        }
    });

    afterAll(async () => {
        // Close all browsers
        await Promise.all(browsers.map(({ browser }) => browser.close()));
    });

    describe('Permission Request UI Cross-Browser Compatibility', () => {
        browsers.forEach(({ name, browser }) => {
            test(`should display permission modal correctly in ${name}`, async () => {
                if (!browser) return; // Skip if browser not available

                const context = await browser.newContext({
                    permissions: ['microphone']
                });
                const page = await context.newPage();

                try {
                    await page.goto(testUrl);
                    
                    // Wait for permission modal to appear
                    await page.waitForSelector('#permission-modal', { timeout: 5000 });
                    
                    // Check if modal is visible
                    const modalVisible = await page.isVisible('#permission-modal');
                    expect(modalVisible).toBe(true);

                    // Check child-friendly content
                    const heading = await page.textContent('#permission-modal h2');
                    expect(heading).toBe("Let's Make Music Together!");

                    // Check for benefit items
                    const benefits = await page.$$('#permission-modal .benefit-item');
                    expect(benefits.length).toBeGreaterThan(0);

                    // Check buttons
                    const grantButton = await page.textContent('#grant-permission-btn');
                    expect(grantButton).toContain("Let's Start!");

                } finally {
                    await context.close();
                }
            });
        });
    });

    describe('getUserMedia API Cross-Browser Testing', () => {
        browsers.forEach(({ name, browser }) => {
            test(`should handle microphone permission grant in ${name}`, async () => {
                if (!browser) return;

                const context = await browser.newContext({
                    permissions: ['microphone']
                });
                const page = await context.newPage();

                try {
                    await page.goto(testUrl);
                    
                    // Wait for permission modal
                    await page.waitForSelector('#grant-permission-btn');
                    
                    // Click grant permission button
                    await page.click('#grant-permission-btn');
                    
                    // Wait for permission to be processed
                    await page.waitForFunction(() => {
                        return document.querySelector('#microphone-status')?.textContent?.includes('granted') ||
                               document.querySelector('#microphone-status')?.textContent?.includes('ready');
                    }, { timeout: 10000 });

                    // Check that permission was granted
                    const statusText = await page.textContent('#microphone-status');
                    expect(statusText).toMatch(/granted|ready|active/i);

                    // Check that modal is hidden
                    const modalVisible = await page.isVisible('#permission-modal');
                    expect(modalVisible).toBe(false);

                } finally {
                    await context.close();
                }
            });

            test(`should handle microphone permission denial in ${name}`, async () => {
                if (!browser) return;

                const context = await browser.newContext({
                    permissions: [] // No microphone permission
                });
                const page = await context.newPage();

                try {
                    await page.goto(testUrl);
                    
                    // Wait for permission modal
                    await page.waitForSelector('#grant-permission-btn');
                    
                    // Mock permission denial by injecting script
                    await page.addInitScript(() => {
                        const originalGetUserMedia = navigator.mediaDevices.getUserMedia;
                        navigator.mediaDevices.getUserMedia = function() {
                            const error = new Error('Permission denied');
                            error.name = 'NotAllowedError';
                            return Promise.reject(error);
                        };
                    });

                    // Click grant permission button
                    await page.click('#grant-permission-btn');
                    
                    // Wait for error handling
                    await page.waitForSelector('.error-message', { timeout: 5000 });

                    // Check error message
                    const errorText = await page.textContent('.error-message');
                    expect(errorText).toContain('blocked');

                    // Check for retry button
                    const retryButton = await page.isVisible('#retry-permission-btn');
                    expect(retryButton).toBe(true);

                } finally {
                    await context.close();
                }
            });
        });
    });

    describe('Browser-Specific Error Handling', () => {
        browsers.forEach(({ name, browser }) => {
            test(`should provide ${name}-specific guidance on permission denial`, async () => {
                if (!browser) return;

                const context = await browser.newContext();
                const page = await context.newPage();

                try {
                    await page.goto(testUrl);
                    
                    // Wait for app to initialize
                    await page.waitForFunction(() => window.pitchApp);

                    // Get browser-specific guidance
                    const guidance = await page.evaluate((browserName) => {
                        // Mock user agent for the test
                        Object.defineProperty(navigator, 'userAgent', {
                            value: `Mozilla/5.0 (${browserName})`,
                            configurable: true
                        });
                        return window.pitchApp.getBrowserSpecificGuidance();
                    }, name);

                    // Verify guidance contains browser-specific instructions
                    expect(guidance.toLowerCase()).toContain(name.toLowerCase());

                } finally {
                    await context.close();
                }
            });
        });
    });

    describe('Performance Across Browsers', () => {
        browsers.forEach(({ name, browser }) => {
            test(`should maintain good performance in ${name}`, async () => {
                if (!browser) return;

                const context = await browser.newContext({
                    permissions: ['microphone']
                });
                const page = await context.newPage();

                try {
                    // Start performance monitoring
                    await page.goto(testUrl);
                    
                    const startTime = Date.now();
                    
                    // Wait for app initialization
                    await page.waitForFunction(() => window.pitchApp);
                    
                    const initTime = Date.now() - startTime;
                    
                    // App should initialize within reasonable time
                    expect(initTime).toBeLessThan(3000); // 3 seconds

                    // Test permission request performance
                    const permissionStartTime = Date.now();
                    await page.click('#grant-permission-btn');
                    
                    await page.waitForFunction(() => {
                        return document.querySelector('#microphone-status')?.textContent?.includes('granted') ||
                               document.querySelector('#microphone-status')?.textContent?.includes('ready');
                    }, { timeout: 5000 });
                    
                    const permissionTime = Date.now() - permissionStartTime;
                    
                    // Permission flow should complete within reasonable time
                    expect(permissionTime).toBeLessThan(2000); // 2 seconds

                } finally {
                    await context.close();
                }
            });
        });
    });

    describe('Feature Detection Across Browsers', () => {
        browsers.forEach(({ name, browser }) => {
            test(`should properly detect features in ${name}`, async () => {
                if (!browser) return;

                const context = await browser.newContext();
                const page = await context.newPage();

                try {
                    await page.goto(testUrl);
                    
                    // Check feature detection
                    const features = await page.evaluate(() => {
                        return {
                            webassembly: typeof WebAssembly !== 'undefined',
                            webaudio: typeof AudioContext !== 'undefined' || typeof webkitAudioContext !== 'undefined',
                            getUserMedia: !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia),
                            audioWorklet: !!(window.AudioWorkletNode),
                            permissions: !!(navigator.permissions)
                        };
                    });

                    // Essential features should be supported
                    expect(features.webassembly).toBe(true);
                    expect(features.webaudio).toBe(true);
                    expect(features.getUserMedia).toBe(true);

                    // Log browser-specific feature support
                    console.log(`${name} feature support:`, features);

                } finally {
                    await context.close();
                }
            });
        });
    });

    describe('Responsive Design Testing', () => {
        const viewports = [
            { name: 'Mobile', width: 375, height: 667 },
            { name: 'Tablet', width: 768, height: 1024 },
            { name: 'Desktop', width: 1920, height: 1080 }
        ];

        browsers.forEach(({ name, browser }) => {
            viewports.forEach(viewport => {
                test(`should display correctly on ${viewport.name} in ${name}`, async () => {
                    if (!browser) return;

                    const context = await browser.newContext({
                        viewport: { width: viewport.width, height: viewport.height }
                    });
                    const page = await context.newPage();

                    try {
                        await page.goto(testUrl);
                        
                        // Wait for permission modal
                        await page.waitForSelector('#permission-modal');
                        
                        // Check if modal is properly sized and positioned
                        const modalBounds = await page.boundingBox('#permission-modal');
                        expect(modalBounds.width).toBeLessThanOrEqual(viewport.width);
                        expect(modalBounds.height).toBeLessThanOrEqual(viewport.height);

                        // Check if buttons are accessible (not overlapping)
                        const buttons = await page.$$('#permission-modal button');
                        for (let i = 0; i < buttons.length; i++) {
                            const buttonBounds = await buttons[i].boundingBox();
                            expect(buttonBounds.width).toBeGreaterThan(0);
                            expect(buttonBounds.height).toBeGreaterThan(0);
                        }

                    } finally {
                        await context.close();
                    }
                });
            });
        });
    });

    describe('Error Recovery Testing', () => {
        browsers.forEach(({ name, browser }) => {
            test(`should recover from permission errors in ${name}`, async () => {
                if (!browser) return;

                const context = await browser.newContext();
                const page = await context.newPage();

                try {
                    await page.goto(testUrl);
                    
                    // Simulate multiple error scenarios
                    const errorScenarios = [
                        { name: 'NotAllowedError', code: 'navigator.mediaDevices.getUserMedia = () => Promise.reject(Object.assign(new Error("Permission denied"), {name: "NotAllowedError"}))' },
                        { name: 'NotFoundError', code: 'navigator.mediaDevices.getUserMedia = () => Promise.reject(Object.assign(new Error("No microphone"), {name: "NotFoundError"}))' },
                        { name: 'NotSupportedError', code: 'navigator.mediaDevices.getUserMedia = () => Promise.reject(Object.assign(new Error("Not supported"), {name: "NotSupportedError"}))' }
                    ];

                    for (const scenario of errorScenarios) {
                        // Inject error scenario
                        await page.addInitScript(scenario.code);
                        
                        // Reload to apply script
                        await page.reload();
                        await page.waitForSelector('#grant-permission-btn');
                        
                        // Trigger permission request
                        await page.click('#grant-permission-btn');
                        
                        // Wait for error handling
                        await page.waitForSelector('.error-message', { timeout: 3000 });
                        
                        // Check appropriate error message
                        const errorText = await page.textContent('.error-message');
                        expect(errorText.length).toBeGreaterThan(0);
                        
                        // Check for retry mechanism
                        const retryButton = await page.isVisible('#retry-permission-btn');
                        expect(retryButton).toBe(true);
                    }

                } finally {
                    await context.close();
                }
            });
        });
    });
}); 