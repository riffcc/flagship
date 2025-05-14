import {join} from 'node:path';
import {node} from '../../.electron-vendors.cache.json';
import {injectAppVersion} from '../../version/inject-app-version-plugin.mjs';

const PACKAGE_ROOT = __dirname;
const PROJECT_ROOT = join(PACKAGE_ROOT, '../..');

/**
 * @type {import('vite').UserConfig}
 * @see https://vitejs.dev/config/
 */
const config = {
  mode: process.env.MODE,
  root: PACKAGE_ROOT,
  envDir: PROJECT_ROOT,
  resolve: {
    alias: {
      '/@/': join(PACKAGE_ROOT, 'src') + '/',
    },
  },
  build: {
    ssr: true,
    sourcemap: 'inline',
    target: `node${node}`,
    outDir: 'dist',
    assetsDir: '.',
    minify: process.env.MODE !== 'development',
    lib: {
      entry: 'src/index.ts',
      formats: ['cjs'],
    },
    rollupOptions: {
      output: {
        entryFileNames: '[name].cjs',
      },
    },
    emptyOutDir: true,
    reportCompressedSize: false,
  },
  ssr: {
    noExternal: [
      '@helia/json',
      'helia',
      'peerbit',
      '@peerbit/document',
      '@peerbit/program',
      '@dao-xyz/borsh',
      /@riffcc\/peerbit-adapter/, // Changed to regex for workspace dependency
      // Add other @helia or @peerbit packages if similar errors occur
    ],
  },
  plugins: [injectAppVersion()],
  test: {
    deps: {
      optimizer: {
        ssr: {
          include: [
            '@helia/json',
            'helia',
            'peerbit',
            '@peerbit/document',
            '@peerbit/program',
            '@dao-xyz/borsh',
            'multiformats', // Often a core part of the ecosystem that might need inlining
            /@riffcc\/peerbit-adapter/, // Changed to regex for workspace dependency
            // You might need to add more entries here if other similar errors appear
            // For example, if specific sub-dependencies of these packages cause issues.
          ],
        },
      },
    },
    coverage: {
      provider: 'istanbul',
    },
  },
};

export default config;
