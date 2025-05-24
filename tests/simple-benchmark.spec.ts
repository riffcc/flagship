import { test, expect } from 'vitest';
import { chromium } from 'playwright';
import path, { dirname } from 'path';
import { fileURLToPath } from 'url';

interface LoadingMetrics {
  startTime: number;
  totalLoadTime: number;
  hadFeaturedContent: boolean;
  hadNoFeaturedMessage: boolean;
  errorCount: number;
}

const BENCHMARK_RUNS = 10;

test('Simple Loading Benchmark', async () => {
  console.log('🚀 Starting Simple Loading Benchmark');
  console.log(`Running ${BENCHMARK_RUNS} iterations to measure load performance`);

  const results: LoadingMetrics[] = [];

  for (let i = 1; i <= BENCHMARK_RUNS; i++) {
    const startTime = Date.now();
    const metrics: LoadingMetrics = {
      startTime,
      totalLoadTime: 0,
      hadFeaturedContent: false,
      hadNoFeaturedMessage: false,
      errorCount: 0,
    };

    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();

    // Track console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        metrics.errorCount++;
        console.log(`Run ${i} Console Error: ${msg.text()}`);
      }
    });

    try {
      // Navigate to the built web app
      const __dirname_test = dirname(fileURLToPath(import.meta.url));
      const htmlFile = path.join(__dirname_test, '..', 'packages', 'renderer', 'dist', 'web', 'index.html');
      
      await page.goto(`file://${htmlFile}`);
      
      // Wait for #app to exist (but not necessarily be visible)
      await page.waitForSelector('#app', { timeout: 15000 });
      
      // Wait a bit for the app to initialize
      await page.waitForTimeout(2000);
      
      // Check for various states
      try {
        // Check for featured content
        const featuredContent = await page.$$('[data-testid="featured-content"], .featured-content, .release-card');
        if (featuredContent.length > 0) {
          metrics.hadFeaturedContent = true;
        }

        // Check for "No featured content" message
        const noContentMessage = await page.$('[data-testid="no-featured-content"]');
        if (noContentMessage) {
          metrics.hadNoFeaturedMessage = true;
        }

        // Take a screenshot for debugging the first few runs
        if (i <= 3) {
          await page.screenshot({ path: `/tmp/benchmark-run-${i}.png` });
        }

      } catch (error) {
        console.log(`Run ${i} content check failed:`, error);
        metrics.errorCount++;
      }

    } catch (error) {
      console.log(`Run ${i} failed:`, error);
      metrics.errorCount++;
    } finally {
      metrics.totalLoadTime = Date.now() - startTime;
      await browser.close();
    }

    results.push(metrics);
    console.log(`Run ${i}: ${metrics.totalLoadTime}ms, featured=${metrics.hadFeaturedContent}, noMessage=${metrics.hadNoFeaturedMessage}, errors=${metrics.errorCount}`);
  }

  // Calculate statistics
  const loadTimes = results.map(r => r.totalLoadTime);
  const successfulRuns = results.filter(r => r.errorCount === 0);
  const runsWithFeaturedContent = results.filter(r => r.hadFeaturedContent);
  const runsWithNoContentMessage = results.filter(r => r.hadNoFeaturedMessage);

  console.log('\n📊 BENCHMARK RESULTS:');
  console.log(`Total runs: ${results.length}`);
  console.log(`Success rate: ${((successfulRuns.length / results.length) * 100).toFixed(1)}%`);
  console.log(`Min load time: ${Math.min(...loadTimes)}ms`);
  console.log(`Max load time: ${Math.max(...loadTimes)}ms`);
  console.log(`Avg load time: ${(loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length).toFixed(1)}ms`);
  console.log(`Runs with featured content: ${runsWithFeaturedContent.length}/${results.length} (${((runsWithFeaturedContent.length / results.length) * 100).toFixed(1)}%)`);
  console.log(`Runs with "no featured content" message: ${runsWithNoContentMessage.length}/${results.length} (${((runsWithNoContentMessage.length / results.length) * 100).toFixed(1)}%)`);

  const totalErrors = results.reduce((sum, r) => sum + r.errorCount, 0);
  console.log(`Total console errors: ${totalErrors}`);

  // Log baseline results
  const baselineResults = {
    timestamp: new Date().toISOString(),
    version: 'baseline-before-optimization',
    summary: {
      totalRuns: results.length,
      successRate: (successfulRuns.length / results.length) * 100,
      minLoadTime: Math.min(...loadTimes),
      maxLoadTime: Math.max(...loadTimes),
      avgLoadTime: loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length,
      featuredContentRate: (runsWithFeaturedContent.length / results.length) * 100,
      noContentMessageRate: (runsWithNoContentMessage.length / results.length) * 100,
      totalErrors: totalErrors,
    },
    runs: results,
  };

  console.log('\n💾 BASELINE RESULTS (save this for comparison):');
  console.log(JSON.stringify(baselineResults, null, 2));

  // Basic expectations to validate the benchmark works
  expect(results.length).toBe(BENCHMARK_RUNS);
  expect(successfulRuns.length).toBeGreaterThan(0);
}, { timeout: 180000 });