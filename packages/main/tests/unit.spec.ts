
import {vi} from 'vitest';

// Mock import.meta.env to ensure stability for Vite-specific environment variables
vi.mock('import.meta', () => ({
  env: {
    DEV: false, // Simulates a production or test build environment
    VITE_DEV_SERVER_URL: undefined,
    // Add any other VITE_ variables used by mainWindow.ts or its dependencies here
  },
}));
