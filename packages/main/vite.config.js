import {join} from 'node:path';
import {node} from '../../.electron-vendors.cache.json';
import {injectAppVersion} from '../../version/inject-app-version-plugin.mjs';

const PACKAGE_ROOT = __dirname;
const PROJECT_ROOT = join(PACKAGE_ROOT, '../..');
const PACKAGES_ROOT = join(PACKAGE_ROOT, '..');

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
      '/@/lib/': join(PACKAGES_ROOT, 'lib', 'src') + '/',
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
      formats: ['es'],
    },
    rollupOptions: {
      output: {
        entryFileNames: '[name].js',
      },
    },
    emptyOutDir: true,
    reportCompressedSize: false,
  },
  ssr: {
    external: [
      '@helia/json',
      'helia',
      'peerbit',
      '@peerbit/document',
      '@peerbit/program',
      '@dao-xyz/borsh',
      '@dao-xyz/datastore-level',
      '@peerbit/keychain',
      '@peerbit/crypto',
      '@peerbit/blocks',
      '@peerbit/stream',
      '@peerbit/rpc',
      '@peerbit/time',
      '@peerbit/logger',
      '@peerbit/trusted-network',
      'better-sqlite3',
      'bindings'
    ]
  },
  // server: { // server.deps.external was not the correct fix, removing.
  //   deps: {
  //     external: [
  //       'better-sqlite3',
  //       'bindings',
  //     ],
  //   },
  // },
  plugins: [injectAppVersion()],
  test: {
    deps: {
      optimizer: {
        ssr: {
          exclude: [
            'better-sqlite3',
            'bindings',
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
