#!/usr/bin/env node

/**
 * Race Condition Loading Benchmark
 * 
 * This script runs the real loading benchmark against the dev server
 * to measure actual Peerbit loading performance and race conditions.
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('🏁 Starting Race Condition Loading Benchmark');
console.log('This will test the REAL loading performance with Peerbit');

// Function to check if dev server is running
function checkDevServer() {
  return new Promise((resolve) => {
    const testProcess = spawn('curl', ['-s', '-f', 'http://localhost:5175'], {
      stdio: 'ignore',
    });
    
    testProcess.on('close', (code) => {
      resolve(code === 0);
    });
    
    testProcess.on('error', () => {
      resolve(false);
    });
  });
}

async function runBenchmark() {
  // Check if dev server is running
  const serverRunning = await checkDevServer();
  
  if (!serverRunning) {
    console.log('❌ Dev server not running on http://localhost:5175');
    console.log('Please start the dev server first:');
    console.log('  pnpm watch:web');
    console.log('');
    console.log('Then run this benchmark again:');
    console.log('  node benchmark.js');
    process.exit(1);
  }
  
  console.log('✅ Dev server is running');
  console.log('🚀 Running loading benchmark...');
  
  // Run the benchmark
  const benchmarkProcess = spawn('pnpm', ['vitest', 'run', 'tests/dev-server-benchmark.spec.ts', '--reporter=verbose'], {
    stdio: 'inherit',
    cwd: __dirname,
  });
  
  benchmarkProcess.on('close', (code) => {
    console.log('\n🏁 Benchmark completed');
    if (code === 0) {
      console.log('✅ All tests passed');
    } else {
      console.log('⚠️  Tests completed with issues (this may be expected if race conditions exist)');
    }
  });
  
  benchmarkProcess.on('error', (error) => {
    console.error('❌ Failed to run benchmark:', error);
    process.exit(1);
  });
}

runBenchmark();