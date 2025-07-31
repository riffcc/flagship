import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  // Look for test files in the 'tests/e2e' directory.
  testDir: './tests/e2e',
  
  // Timeout for each test in milliseconds.
  timeout: 30 * 1000,

  // Global expect timeout.
  expect: {
    timeout: 5000,
  },

  // Run tests in files in parallel.
  fullyParallel: true,

  // Fail the build on CI if you accidentally leave test.only in the source code.
  forbidOnly: !!process.env.CI,

  // Retry on CI only.
  retries: process.env.CI ? 2 : 0,

  // Opt out of parallel tests on CI.
  workers: process.env.CI ? 1 : undefined,

  // Reporter to use. See https://playwright.dev/docs/test-reporters
  reporter: 'html',

  // Shared settings for all the projects below.
  use: {
    // Base URL to use in actions like `await page.goto('/')`.
    // Make sure your dev server runs on this port.
    baseURL: 'http://localhost:5175',

    // Collect trace when retrying the failed test.
    trace: 'on-first-retry',
  },

  // Configure projects for major browsers.
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    // Uncomment to test against other browsers.
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },
  ],
});
