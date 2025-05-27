import { test, expect } from 'vitest';
import { chromium } from 'playwright';

interface LoadingMetrics {
  startTime: number;
  totalLoadTime: number;
  hadFeaturedContent: boolean;
  hadNoFeaturedMessage: boolean;
  hadLoadingSpinner: boolean;
  stuckInLoading: boolean;
  errorCount: number;
  peerbitConnected: boolean;
}

const BENCHMARK_RUNS = 3;
const DEV_SERVER_URL = 'http://localhost:5175'; // Default Vite dev server port

test('Dev Server Loading Benchmark - REAL CONDITIONS', async () => {
  console.log('🚀 Starting REAL Loading Benchmark');
  console.log(`Testing against: ${DEV_SERVER_URL}`);
  console.log(`Running ${BENCHMARK_RUNS} iterations to measure ACTUAL load performance`);
  console.log('⚠️  Make sure dev server is running: pnpm watch:web');

  const results: LoadingMetrics[] = [];

  // Test if dev server is available
  try {
    const testBrowser = await chromium.launch({ headless: true });
    const testPage = await testBrowser.newPage();
    await testPage.goto(DEV_SERVER_URL, { timeout: 5000 });
    await testBrowser.close();
    console.log('✅ Dev server is accessible');
  } catch (_error) {
    console.error('❌ Dev server not accessible. Run: pnpm watch:web');
    throw new Error('Dev server not running. Please start with: pnpm watch:web');
  }

  for (let i = 1; i <= BENCHMARK_RUNS; i++) {
    const startTime = Date.now();
    const metrics: LoadingMetrics = {
      startTime,
      totalLoadTime: 0,
      hadFeaturedContent: false,
      hadNoFeaturedMessage: false,
      hadLoadingSpinner: false,
      stuckInLoading: false,
      errorCount: 0,
      peerbitConnected: false,
    };

    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();

    // Track console logs for Peerbit connection status
    page.on('console', msg => {
      const text = msg.text();
      if (msg.type() === 'error') {
        // BLOCKLIST: Ignore expected P2P/networking errors that are normal
        const ignoredErrors = [
          'ws://127.0.0.1:', 'ERR_CONNECTION_REFUSED',  // Peer advertising local addresses
          'mdi-', 'Expected number',                     // MDI icon loading issues
          'lazystream', 'UnexpectedEOFError',           // P2P stream errors
          'unexpected end of input',                     // Network/stream termination
          'outbound inflight queue error',              // P2P queue errors
          'WebSocket connection',                        // WebSocket connection issues
          'net::ERR_', 'Failed to load resource',       // General network errors
        ];
        
        if (ignoredErrors.some(ignored => text.includes(ignored))) {
          return; // These are expected in P2P environment
        }
        
        metrics.errorCount++;
        console.log(`Run ${i} Console Error: ${text}`);
      }
      
      // Check for Peerbit connection indicators
      if (text.includes('peer') || text.includes('Peerbit') || text.includes('connected') || text.includes('node')) {
        console.log(`Run ${i} Peerbit Activity: ${text}`);
        if (text.includes('connected') || text.includes('ready')) {
          metrics.peerbitConnected = true;
        }
      }
    });

    try {
      console.log(`Run ${i}: Starting...`);
      
      // Navigate to the app
      await page.goto(DEV_SERVER_URL, { waitUntil: 'domcontentloaded' });
      
      // Wait for #app to be visible - this should be immediate now
      const appStart = Date.now();
      await page.waitForSelector('#app', { state: 'visible', timeout: 8000 });
      const appLoadTime = Date.now() - appStart;
      console.log(`Run ${i}: #app visible in ${appLoadTime}ms`);
      
      // Check for loading spinner at multiple points
      let foundLoadingSpinner = false;
      
      // Check immediately after app loads
      const initialLoadingSpinner = await page.$('[data-testid="loading"], .v-progress-circular');
      if (initialLoadingSpinner) {
        foundLoadingSpinner = true;
        metrics.hadLoadingSpinner = true;
        console.log(`Run ${i}: Loading spinner detected initially`);
      }
      
      // Also check after a short delay (in case it appears after Vue renders)
      await page.waitForTimeout(100);
      const delayedLoadingSpinner = await page.$('[data-testid="loading"], .v-progress-circular');
      if (delayedLoadingSpinner && !foundLoadingSpinner) {
        foundLoadingSpinner = true;
        metrics.hadLoadingSpinner = true;
        console.log(`Run ${i}: Loading spinner detected after delay`);
      }
      
      if (foundLoadingSpinner) {
        // Wait up to 15 seconds to see if it resolves
        try {
          await page.waitForSelector('[data-testid="loading"], .v-progress-circular', { state: 'hidden', timeout: 15000 });
          console.log(`Run ${i}: Loading spinner resolved`);
        } catch {
          metrics.stuckInLoading = true;
          console.log(`Run ${i}: STUCK IN LOADING STATE`);
        }
      } else {
        console.log(`Run ${i}: No loading spinner detected`);
      }
      
      // Wait longer for content to potentially appear (Peerbit can be slow)
      await page.waitForTimeout(5000);
      
      // Check for featured content with much broader selectors
      const featuredContentSelectors = [
        '[data-testid="featured-content"]',
        '.featured-content', 
        '.release-card', 
        '.content-card',
        '.v-card',  // Vuetify cards
        '.featured-slider',
        '.content-section',
        'img[src*="mock"]', // Mock images from static data
        'img[src*="bafkrei"]', // IPFS images 
        '.v-container .v-sheet:not([data-testid="loading"])', // Content containers
        'h1, h2, h3', // Any headings that might indicate content
      ];
      
      let foundContent = false;
      for (const selector of featuredContentSelectors) {
        const elements = await page.$$(selector);
        if (elements.length > 0) {
          foundContent = true;
          console.log(`Run ${i}: ✅ Found content with selector "${selector}" (${elements.length} items)`);
          break;
        }
      }
      
      if (foundContent) {
        metrics.hadFeaturedContent = true;
      }

      // Check for "No featured content" message with broader detection
      const noContentSelectors = [
        '[data-testid="no-featured-content"]',
        'text="No featured content yet"',
        'text="No content here"',
        ':text("No featured content")',
        ':text("No content")',
      ];
      
      for (const selector of noContentSelectors) {
        try {
          const element = await page.$(selector);
          if (element) {
            metrics.hadNoFeaturedMessage = true;
            console.log(`Run ${i}: ⚠️  "No featured content" message found with selector "${selector}"`);
            break;
          }
        } catch (_e) {
          // Ignore errors from text selectors
        }
      }

      // Debug: Log page content to understand what's actually rendered
      const appContent = await page.textContent('#app');
      const hasLoadingText = appContent?.includes('Loading') || false;
      const hasContentText = appContent?.includes('Featured') || appContent?.includes('Music') || appContent?.includes('Movies') || false;
      console.log(`Run ${i}: Page text includes Loading: ${hasLoadingText}, Content keywords: ${hasContentText}`);
      
      // Take screenshots for all runs to debug
      await page.screenshot({ path: `/tmp/dev-benchmark-run-${i}.png`, fullPage: true });

    } catch (_error) {
      console.log(`Run ${i} ❌ FAILED:`, _error.message);
      metrics.errorCount++;
    } finally {
      metrics.totalLoadTime = Date.now() - startTime;
      await browser.close();
    }

    results.push(metrics);
    
    const status = metrics.hadFeaturedContent ? '✅ SUCCESS' : 
                  metrics.stuckInLoading ? '🔄 STUCK' :
                  metrics.hadNoFeaturedMessage ? '⚠️  NO CONTENT' : '❌ FAILED';
                  
    console.log(`Run ${i}: ${metrics.totalLoadTime}ms ${status} (errors: ${metrics.errorCount})`);
    
    // Small delay between runs
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  // Calculate statistics
  const loadTimes = results.map(r => r.totalLoadTime);
  const successfulRuns = results.filter(r => r.hadFeaturedContent);
  const stuckRuns = results.filter(r => r.stuckInLoading);
  const noContentRuns = results.filter(r => r.hadNoFeaturedMessage);
  const peerbitConnectedRuns = results.filter(r => r.peerbitConnected);

  console.log('\n📊 REAL PERFORMANCE RESULTS:');
  console.log(`Total runs: ${results.length}`);
  console.log(`Min load time: ${Math.min(...loadTimes)}ms`);
  console.log(`Max load time: ${Math.max(...loadTimes)}ms`);
  console.log(`Avg load time: ${(loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length).toFixed(1)}ms`);
  
  console.log('\n🎯 SUCCESS RATES:');
  console.log(`✅ Featured content loaded: ${successfulRuns.length}/${results.length} (${((successfulRuns.length / results.length) * 100).toFixed(1)}%)`);
  console.log(`🔄 Stuck in loading: ${stuckRuns.length}/${results.length} (${((stuckRuns.length / results.length) * 100).toFixed(1)}%)`);
  console.log(`⚠️  "No content" message: ${noContentRuns.length}/${results.length} (${((noContentRuns.length / results.length) * 100).toFixed(1)}%)`);
  console.log(`🌐 Peerbit connected: ${peerbitConnectedRuns.length}/${results.length} (${((peerbitConnectedRuns.length / results.length) * 100).toFixed(1)}%)`);

  const totalErrors = results.reduce((sum, r) => sum + r.errorCount, 0);
  console.log(`❌ Total console errors: ${totalErrors}`);

  // Race condition analysis
  const raceConditionIndicators = {
    inconsistentResults: new Set([successfulRuns.length, stuckRuns.length, noContentRuns.length]).size > 1,
    lowSuccessRate: (successfulRuns.length / results.length) < 0.8,
    highStuckRate: (stuckRuns.length / results.length) > 0.2,
    peerbitConnectionIssues: (peerbitConnectedRuns.length / results.length) < 0.8,
  };

  console.log('\n🐛 RACE CONDITION ANALYSIS:');
  Object.entries(raceConditionIndicators).forEach(([key, value]) => {
    const icon = value ? '❌' : '✅';
    console.log(`${icon} ${key}: ${value}`);
  });

  // Log baseline results for comparison
  const baselineResults = {
    timestamp: new Date().toISOString(),
    version: 'baseline-before-optimization',
    testType: 'dev-server-real-conditions',
    summary: {
      totalRuns: results.length,
      successRate: (successfulRuns.length / results.length) * 100,
      stuckInLoadingRate: (stuckRuns.length / results.length) * 100,
      noContentMessageRate: (noContentRuns.length / results.length) * 100,
      peerbitConnectionRate: (peerbitConnectedRuns.length / results.length) * 100,
      minLoadTime: Math.min(...loadTimes),
      maxLoadTime: Math.max(...loadTimes),
      avgLoadTime: loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length,
      totalErrors: totalErrors,
      raceConditionDetected: Object.values(raceConditionIndicators).some(Boolean),
    },
    runs: results,
  };

  console.log('\n💾 BASELINE RESULTS:');
  console.log(JSON.stringify(baselineResults, null, 2));

  // Validate that we got real data
  expect(results.length).toBe(BENCHMARK_RUNS);
  
  // This test should demonstrate the race condition - if success rate is too low, we have a problem
  if ((successfulRuns.length / results.length) < 0.5) {
    console.log('\n🚨 RACE CONDITION CONFIRMED: Success rate below 50%');
  }
  
  // Don't fail the test - we want to see the real numbers
  console.log('\n✅ Benchmark completed - data collected for optimization');
  
}, 600000); // 10 minute timeout