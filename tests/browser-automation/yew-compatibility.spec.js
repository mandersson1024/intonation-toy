/**
 * Yew Browser Compatibility Test Suite
 * Tests WebAssembly and Yew support across different browsers
 */

const { test, expect } = require('@playwright/test');

// Test configuration for different browsers
const browserConfigs = [
  {
    name: 'Chrome',
    userAgent: 'Chrome/120.0.0.0',
    minVersion: 69,
    expected: true
  },
  {
    name: 'Firefox', 
    userAgent: 'Firefox/118.0',
    minVersion: 76,
    expected: true
  },
  {
    name: 'Safari',
    userAgent: 'Safari/605.1.15',
    minVersion: 14,
    expected: true
  },
  {
    name: 'Edge',
    userAgent: 'Edg/120.0.0.0',
    minVersion: 79,
    expected: true
  }
];

test.describe('Yew Browser Compatibility', () => {
  
  test.beforeEach(async ({ page }) => {
    // Navigate to the Yew app
    await page.goto('http://localhost:8080');
    
    // Wait for potential WebAssembly loading
    await page.waitForTimeout(2000);
  });

  test('WebAssembly support detection', async ({ page }) => {
    // Check if WebAssembly is supported
    const wasmSupported = await page.evaluate(() => {
      return typeof WebAssembly === 'object' && typeof WebAssembly.instantiate === 'function';
    });
    
    expect(wasmSupported).toBe(true);
  });

  test('Web Audio API support detection', async ({ page }) => {
    // Check if Web Audio API is supported
    const audioSupported = await page.evaluate(() => {
      return typeof (window.AudioContext || window.webkitAudioContext) === 'function';
    });
    
    expect(audioSupported).toBe(true);
  });

  test('Yew app renders successfully', async ({ page }) => {
    // Check if the main Yew app content is rendered
    await expect(page.locator('h1')).toContainText('Hello World from Yew!');
    await expect(page.locator('p')).toContainText('Yew project setup is working correctly.');
  });

  test('Browser compatibility status displayed', async ({ page }) => {
    // Wait for browser detection to complete
    await page.waitForSelector('.browser-info', { timeout: 5000 });
    
    // Check if browser compatibility info is displayed
    const browserInfo = page.locator('.browser-info');
    await expect(browserInfo).toBeVisible();
    
    // Check for browser name display
    const browserName = page.locator('.browser-info p strong:has-text("Browser:")');
    await expect(browserName).toBeVisible();
    
    // Check for WebAssembly status
    const wasmStatus = page.locator('.browser-info p:has-text("WebAssembly:")');
    await expect(wasmStatus).toContainText('✅ Supported');
    
    // Check for Web Audio API status  
    const audioStatus = page.locator('.browser-info p:has-text("Web Audio API:")');
    await expect(audioStatus).toContainText('✅ Supported');
  });

  test('WASM files load with correct MIME type', async ({ page }) => {
    // Intercept network requests to check WASM MIME type
    const wasmRequests = [];
    
    page.on('response', response => {
      if (response.url().includes('.wasm')) {
        wasmRequests.push({
          url: response.url(),
          contentType: response.headers()['content-type'],
          status: response.status()
        });
      }
    });
    
    // Reload page to trigger WASM loading
    await page.reload();
    await page.waitForTimeout(3000);
    
    // Check that WASM files were loaded
    expect(wasmRequests.length).toBeGreaterThan(0);
    
    // Check WASM MIME type (should be application/wasm)
    for (const request of wasmRequests) {
      expect(request.status).toBe(200);
      // Note: MIME type might vary by server configuration
      expect(request.contentType).toMatch(/application\/(wasm|octet-stream)/);
    }
  });

  test('Hot reload functionality (development mode)', async ({ page }) => {
    // This test should only run in development mode
    const isDevelopment = await page.evaluate(() => {
      return !document.querySelector('script[src*=".js"]')?.src.includes('release');
    });
    
    if (isDevelopment) {
      // Check for hot reload WebSocket connection
      const wsConnected = await page.evaluate(() => {
        return new Promise((resolve) => {
          try {
            const ws = new WebSocket('ws://localhost:8081');
            ws.onopen = () => {
              ws.close();
              resolve(true);
            };
            ws.onerror = () => resolve(false);
            setTimeout(() => resolve(false), 2000);
          } catch (e) {
            resolve(false);
          }
        });
      });
      
      // Hot reload WebSocket should be available in development
      expect(wsConnected).toBe(true);
    }
  });

  test('Graceful error handling for unsupported browsers', async ({ page }) => {
    // Simulate unsupported browser by disabling WebAssembly
    await page.addInitScript(() => {
      delete window.WebAssembly;
    });
    
    await page.reload();
    await page.waitForTimeout(2000);
    
    // Check if error handling message is displayed
    const errorMessage = page.locator('.browser-info p');
    await expect(errorMessage).toContainText('WebAssembly is not supported');
  });

});

// Performance benchmarks
test.describe('Yew Performance Benchmarks', () => {
  
  test('WASM bundle size is optimized', async ({ page }) => {
    const wasmSizes = [];
    
    page.on('response', response => {
      if (response.url().includes('.wasm')) {
        wasmSizes.push({
          url: response.url(),
          size: parseInt(response.headers()['content-length'] || '0')
        });
      }
    });
    
    await page.goto('http://localhost:8080');
    await page.waitForTimeout(3000);
    
    // Check that WASM bundle exists and is reasonably sized
    expect(wasmSizes.length).toBeGreaterThan(0);
    
    for (const wasm of wasmSizes) {
      // WASM bundle should be less than 500KB for this simple app
      expect(wasm.size).toBeLessThan(500 * 1024);
      expect(wasm.size).toBeGreaterThan(1024); // Should be more than 1KB
    }
  });

  test('App initialization time', async ({ page }) => {
    const startTime = Date.now();
    
    await page.goto('http://localhost:8080');
    
    // Wait for app to be fully rendered
    await expect(page.locator('h1')).toContainText('Hello World from Yew!');
    await expect(page.locator('.browser-info')).toBeVisible();
    
    const loadTime = Date.now() - startTime;
    
    // App should load within 5 seconds
    expect(loadTime).toBeLessThan(5000);
    console.log(`Yew app loaded in ${loadTime}ms`);
  });
}); 