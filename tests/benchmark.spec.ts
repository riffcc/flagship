import type {Browser, Page, ElectronApplication} from 'playwright';
import {afterAll, beforeAll, describe, expect, test} from 'vitest';
import {onBrowser} from './utils';

interface LoadingMetrics {
  startTime: number;
  firstContentfulPaint?: number;
  domContentLoaded?: number;
  featuredContentAppeared?: number;
  totalLoadTime: number;
  hadFeaturedContent: boolean;
  errorCount: number;
}

interface BenchmarkResults {
  runs: LoadingMetrics[];
  minLoadTime: number;
  maxLoadTime: number;
  avgLoadTime: number;
  successRate: number;
  avgFeaturedContentTime?: number;
}

const BENCHMARK_RUNS = 5;
const MAX_WAIT_TIME = 30000; // 30 seconds max wait

describe('Loading Benchmark Suite', function () {
  const browser: Browser | undefined = undefined;
  const electronApp: ElectronApplication | undefined = undefined;

  beforeAll(async () => {
    console.log('🚀 Starting Loading Benchmark Suite');
    console.log(`Running ${BENCHMARK_RUNS} iterations to measure load performance`);
  });

  afterAll(async () => {
    if (electronApp) {
      await electronApp.close();
    } else if (browser) {
      await browser.close();
    }
  });

  async function measureSingleLoad(runNumber: number): Promise<LoadingMetrics> {
    const metrics: LoadingMetrics = {
      startTime: Date.now(),
      totalLoadTime: 0,
      hadFeaturedContent: false,
      errorCount: 0,
    };

    let page: Page;
    let currentElectronApp: ElectronApplication | undefined;
    let currentBrowser: Browser | undefined;

    try {
      // Launch fresh instance for each run
      const testEnvironment = 'chromium'; // Can be made configurable
      
      if (testEnvironment === 'electron') {
        const result = await onBrowser({ typeNavigateur: 'electron' });
        page = result.page;
        currentElectronApp = result.electronApp;
      } else {
        const result = await onBrowser({ typeNavigateur: 'chromium' });
        page = result.page;
        currentBrowser = result.browser;
      }

      // Track console errors
      page.on('console', msg => {
        if (msg.type() === 'error') {
          metrics.errorCount++;
          console.log(`Run ${runNumber} Console Error: ${msg.text()}`);
        }
      });

      // Track performance events
      await page.evaluate(() => {
        // Mark when DOM content is loaded
        if (document.readyState === 'loading') {
          document.addEventListener('DOMContentLoaded', () => {
            (window as unknown as { domContentLoadedTime: number }).domContentLoadedTime = Date.now();
          });
        } else {
          (window as unknown as { domContentLoadedTime: number }).domContentLoadedTime = Date.now();
        }
      });

      // Wait for app to be ready
      await page.waitForSelector('#app', { timeout: 10000 });
      
      // Get DOM content loaded time
      const domLoadedTime = await page.evaluate(() => (window as unknown as { domContentLoadedTime: number }).domContentLoadedTime);
      if (domLoadedTime) {
        metrics.domContentLoaded = domLoadedTime - metrics.startTime;
      }

      // Wait for Vue to be mounted and check for featured content
      await page.waitForFunction(() => {
        const app = document.querySelector('#app');
        return app && app.innerHTML.trim() !== '';
      }, { timeout: 15000 });

      // Check for featured content or "No featured content" message
      let featuredContentFound = false;
      let _noContentMessageFound = false;
      
      const startWait = Date.now();
      
      while (Date.now() - startWait < MAX_WAIT_TIME) {
        try {
          // Check for featured content cards
          const featuredContent = await page.$$('[data-testid="featured-release"], .featured-content, .release-card');
          if (featuredContent.length > 0) {
            featuredContentFound = true;
            metrics.featuredContentAppeared = Date.now() - metrics.startTime;
            metrics.hadFeaturedContent = true;
            break;
          }

          // Check for "No featured content yet" message
          const noContentElement = await page.$('text="No featured content yet"');
          if (noContentElement) {
            _noContentMessageFound = true;
            break;
          }

          // Check for any loading indicators
          const loadingIndicators = await page.$$('.v-progress-circular, .loading, [data-testid="loading"]');
          if (loadingIndicators.length === 0) {
            // No loading indicators visible, assume load is complete
            break;
          }

          await page.waitForTimeout(100); // Short wait before next check
        } catch (_error) {
          // Continue checking
        }
      }

      metrics.totalLoadTime = Date.now() - metrics.startTime;

      console.log(`Run ${runNumber}: ${metrics.totalLoadTime}ms total, ` +
                 `${featuredContentFound ? 'featured content found' : 'no featured content'}, ` +
                 `${metrics.errorCount} errors`);

    } catch (_error) {
      console.error(`Run ${runNumber} failed:`, _error);
      metrics.errorCount++;
      metrics.totalLoadTime = Date.now() - metrics.startTime;
    } finally {
      // Clean up
      try {
        if (currentElectronApp) {
          await currentElectronApp.close();
        } else if (currentBrowser) {
          await currentBrowser.close();
        }
      } catch (_error) {
        console.error('Cleanup error:', error);
      }
    }

    return metrics;
  }

  async function runBenchmark(): Promise<BenchmarkResults> {
    const results: LoadingMetrics[] = [];
    
    for (let i = 1; i <= BENCHMARK_RUNS; i++) {
      const metrics = await measureSingleLoad(i);
      results.push(metrics);
      
      // Small delay between runs to avoid resource conflicts
      await new Promise(resolve => setTimeout(resolve, 500));
    }

    // Calculate statistics
    const loadTimes = results.map(r => r.totalLoadTime);
    const successfulRuns = results.filter(r => r.errorCount === 0);
    const _runsWithFeaturedContent = results.filter(r => r.hadFeaturedContent);
    const featuredContentTimes = results
      .filter(r => r.featuredContentAppeared)
      .map(r => r.featuredContentAppeared!);

    const benchmarkResults: BenchmarkResults = {
      runs: results,
      minLoadTime: Math.min(...loadTimes),
      maxLoadTime: Math.max(...loadTimes),
      avgLoadTime: loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length,
      successRate: (successfulRuns.length / results.length) * 100,
    };

    if (featuredContentTimes.length > 0) {
      benchmarkResults.avgFeaturedContentTime = 
        featuredContentTimes.reduce((a, b) => a + b, 0) / featuredContentTimes.length;
    }

    return benchmarkResults;
  }

  test('Loading Performance Benchmark', async () => {
    const results = await runBenchmark();
    
    console.log('\n📊 BENCHMARK RESULTS:');
    console.log(`Total runs: ${results.runs.length}`);
    console.log(`Success rate: ${results.successRate.toFixed(1)}%`);
    console.log(`Min load time: ${results.minLoadTime}ms`);
    console.log(`Max load time: ${results.maxLoadTime}ms`);
    console.log(`Avg load time: ${results.avgLoadTime.toFixed(1)}ms`);
    
    const runsWithFeaturedContent = results.runs.filter(r => r.hadFeaturedContent);
    console.log(`Runs with featured content: ${runsWithFeaturedContent.length}/${results.runs.length} (${((runsWithFeaturedContent.length / results.runs.length) * 100).toFixed(1)}%)`);
    
    if (results.avgFeaturedContentTime) {
      console.log(`Avg time to featured content: ${results.avgFeaturedContentTime.toFixed(1)}ms`);
    }

    const errorCounts = results.runs.map(r => r.errorCount);
    const totalErrors = errorCounts.reduce((a, b) => a + b, 0);
    console.log(`Total console errors: ${totalErrors}`);

    // Write detailed results to file for analysis
    const detailedResults = {
      timestamp: new Date().toISOString(),
      summary: {
        totalRuns: results.runs.length,
        successRate: results.successRate,
        minLoadTime: results.minLoadTime,
        maxLoadTime: results.maxLoadTime,
        avgLoadTime: results.avgLoadTime,
        runsWithFeaturedContent: runsWithFeaturedContent.length,
        featuredContentRate: (runsWithFeaturedContent.length / results.runs.length) * 100,
        avgFeaturedContentTime: results.avgFeaturedContentTime,
        totalErrors: totalErrors,
      },
      runs: results.runs,
    };

    // This assertion will help track if we're consistently failing to load featured content
    expect(runsWithFeaturedContent.length).toBeGreaterThan(0, 
      'No runs successfully loaded featured content - this indicates a race condition or loading issue');

    // Performance expectations (these can be adjusted based on baseline)
    expect(results.avgLoadTime).toBeLessThan(15000, 
      `Average load time ${results.avgLoadTime.toFixed(1)}ms exceeds 15 second threshold`);
    
    expect(results.successRate).toBeGreaterThan(80, 
      `Success rate ${results.successRate.toFixed(1)}% is below 80% threshold`);

    console.log('\n💾 Detailed results saved for analysis');
    console.log(JSON.stringify(detailedResults, null, 2));
  });
});