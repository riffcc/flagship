import { test, expect } from 'vitest';
import { chromium } from 'playwright';
import { createServer } from 'http';
import { createReadStream,  statSync } from 'fs';
import path, { dirname } from 'path';
import { fileURLToPath } from 'url';

interface LoadingMetrics {
  startTime: number;
  totalLoadTime: number;
  hadFeaturedContent: boolean;
  hadNoFeaturedMessage: boolean;
  hadLoadingSpinner: boolean;
  errorCount: number;
}

const BENCHMARK_RUNS = 10;

function createSimpleServer(port: number, webRoot: string): Promise<unknown> {
  return new Promise((resolve) => {
    const server = createServer((req, res) => {
      const filePath = path.join(webRoot, req.url === '/' ? 'index.html' : req.url!);
      
      try {
        const stat = statSync(filePath);
        if (stat.isFile()) {
          const ext = path.extname(filePath);
          const contentType = {
            '.html': 'text/html',
            '.js': 'application/javascript',
            '.css': 'text/css',
            '.png': 'image/png',
            '.svg': 'image/svg+xml',
            '.wasm': 'application/wasm',
          }[ext] || 'application/octet-stream';
          
          res.writeHead(200, { 'Content-Type': contentType });
          createReadStream(filePath).pipe(res);
        } else {
          res.writeHead(404);
          res.end('Not found');
        }
      } catch (_error) {
        res.writeHead(404);
        res.end('Not found');
      }
    });
    
    server.listen(port, () => {
      resolve(server);
    });
  });
}

test('HTTP Loading Benchmark', async () => {
  console.log('🚀 Starting HTTP Loading Benchmark');
  console.log(`Running ${BENCHMARK_RUNS} iterations to measure load performance`);

  // Start HTTP server
  const __dirname_test = dirname(fileURLToPath(import.meta.url));
  const webRoot = path.join(__dirname_test, '..', 'packages', 'renderer', 'dist', 'web');
  const port = 8099;
  const server = await createSimpleServer(port, webRoot);
  const baseUrl = `http://localhost:${port}`;

  const results: LoadingMetrics[] = [];

  try {
    for (let i = 1; i <= BENCHMARK_RUNS; i++) {
      const startTime = Date.now();
      const metrics: LoadingMetrics = {
        startTime,
        totalLoadTime: 0,
        hadFeaturedContent: false,
        hadNoFeaturedMessage: false,
        hadLoadingSpinner: false,
        errorCount: 0,
      };

      const browser = await chromium.launch({ headless: true });
      const page = await browser.newPage();

      // Track console errors and logs
      page.on('console', msg => {
        if (msg.type() === 'error') {
          metrics.errorCount++;
          console.log(`Run ${i} Console Error: ${msg.text()}`);
        }
      });

      try {
        // Navigate to the app
        await page.goto(baseUrl, { waitUntil: 'networkidle' });
        
        // Wait for #app to be visible
        await page.waitForSelector('#app', { state: 'visible', timeout: 10000 });
        
        // Wait a bit more for app initialization
        await page.waitForTimeout(3000);
        
        // Check for loading spinner
        const loadingSpinner = await page.$('[data-testid="loading"]');
        if (loadingSpinner) {
          metrics.hadLoadingSpinner = true;
          console.log(`Run ${i}: Loading spinner detected`);
        }
        
        // Check for featured content
        const featuredContent = await page.$$('[data-testid="featured-content"], .featured-content, .release-card');
        if (featuredContent.length > 0) {
          metrics.hadFeaturedContent = true;
          console.log(`Run ${i}: Featured content detected`);
        }

        // Check for "No featured content" message
        const noContentMessage = await page.$('[data-testid="no-featured-content"]');
        if (noContentMessage) {
          metrics.hadNoFeaturedMessage = true;
          console.log(`Run ${i}: No featured content message detected`);
        }

        // Take screenshots for debugging
        if (i <= 3) {
          await page.screenshot({ path: `/tmp/http-benchmark-run-${i}.png` });
        }

      } catch (_error) {
        console.log(`Run ${i} failed:`, _error);
        metrics.errorCount++;
      } finally {
        metrics.totalLoadTime = Date.now() - startTime;
        await browser.close();
      }

      results.push(metrics);
      console.log(`Run ${i}: ${metrics.totalLoadTime}ms, featured=${metrics.hadFeaturedContent}, noMessage=${metrics.hadNoFeaturedMessage}, loading=${metrics.hadLoadingSpinner}, errors=${metrics.errorCount}`);
    }

  } finally {
    server.close();
  }

  // Calculate statistics
  const loadTimes = results.map(r => r.totalLoadTime);
  const successfulRuns = results.filter(r => r.errorCount === 0);
  const runsWithFeaturedContent = results.filter(r => r.hadFeaturedContent);
  const runsWithNoContentMessage = results.filter(r => r.hadNoFeaturedMessage);
  const runsWithLoadingSpinner = results.filter(r => r.hadLoadingSpinner);

  console.log('\n📊 BENCHMARK RESULTS:');
  console.log(`Total runs: ${results.length}`);
  console.log(`Success rate: ${((successfulRuns.length / results.length) * 100).toFixed(1)}%`);
  console.log(`Min load time: ${Math.min(...loadTimes)}ms`);
  console.log(`Max load time: ${Math.max(...loadTimes)}ms`);
  console.log(`Avg load time: ${(loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length).toFixed(1)}ms`);
  console.log(`Runs with featured content: ${runsWithFeaturedContent.length}/${results.length} (${((runsWithFeaturedContent.length / results.length) * 100).toFixed(1)}%)`);
  console.log(`Runs with "no featured content" message: ${runsWithNoContentMessage.length}/${results.length} (${((runsWithNoContentMessage.length / results.length) * 100).toFixed(1)}%)`);
  console.log(`Runs with loading spinner: ${runsWithLoadingSpinner.length}/${results.length} (${((runsWithLoadingSpinner.length / results.length) * 100).toFixed(1)}%)`);

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
      loadingSpinnerRate: (runsWithLoadingSpinner.length / results.length) * 100,
      totalErrors: totalErrors,
    },
    runs: results,
  };

  console.log('\n💾 BASELINE RESULTS (save this for comparison):');
  console.log(JSON.stringify(baselineResults, null, 2));

  // Create a simple comparison object for easy tracking
  const summary = {
    avgLoadTime: baselineResults.summary.avgLoadTime,
    featuredContentRate: baselineResults.summary.featuredContentRate,
    noContentMessageRate: baselineResults.summary.noContentMessageRate,
    successRate: baselineResults.summary.successRate,
  };

  console.log('\n🎯 KEY METRICS TO TRACK:');
  console.log(`Average Load Time: ${summary.avgLoadTime.toFixed(1)}ms`);
  console.log(`Featured Content Success Rate: ${summary.featuredContentRate.toFixed(1)}%`);
  console.log(`"No Content" Message Rate: ${summary.noContentMessageRate.toFixed(1)}%`);
  console.log(`Overall Success Rate: ${summary.successRate.toFixed(1)}%`);

  // Basic expectations to validate the benchmark works
  expect(results.length).toBe(BENCHMARK_RUNS);
  // We expect at least some runs to complete without major errors
  expect(successfulRuns.length).toBeGreaterThan(0);
}, 300000); // 5 minute timeout