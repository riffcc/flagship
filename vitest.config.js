// vite.config.ts
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  // Add this 'test' property
  test: {
    globals: true,
    environment: 'happy-dom',
    exclude: ['**/node_modules/**', '**/dist/**', 'tests/e2e/**'],
  },
});
