/* eslint-env node */

import vue from '@vitejs/plugin-vue';
import {copyFileSync} from 'fs';
import {join} from 'node:path';
import vuetify from 'vite-plugin-vuetify';
import {chrome} from '../../.electron-vendors.cache.json';
import {injectAppVersion} from '../../version/inject-app-version-plugin.mjs';
import {nodePolyfills} from 'vite-plugin-node-polyfills';

const PACKAGE_ROOT = __dirname;
const PROJECT_ROOT = join(PACKAGE_ROOT, '../..');

const forElectron = !process.env.WEB;

if (forElectron) {
  copyFileSync(join(PACKAGE_ROOT, 'indexElectron.html'), join(PACKAGE_ROOT, 'index.html'));
} else {
  copyFileSync(join(PACKAGE_ROOT, 'indexBrowser.html'), join(PACKAGE_ROOT, 'index.html'));
}

const générerExtentions = () => {
  const extentions = [
    vue(),
    vuetify({
      autoImport: true,
      styles: {
        configFile: 'src/styles/settings.scss',
      },
    }),
    nodePolyfills(),
  ];
  // No specific renderer plugin for auto-expose needed here with manual contextBridge
  extentions.push(injectAppVersion());
  return extentions;
};

const générerAliasRésolution = () => {
  const common = {
    '/@/': join(PACKAGE_ROOT, 'src') + '/',
  };
  if (forElectron) {
    return common;
  } else {
    return {
      ...common,
      '#preload': join(PACKAGE_ROOT, 'src') + '/polyfillPreload', // Ensure this polyfillPreload.ts exists if used for web
    };
  }
};

const dépendsÀExclure = ['chokidar', '@libp2p/tcp', '@libp2p/mdns', 'env-paths', 'datastore-fs', 'blockstore-fs'];

/**
 * @type {import('vite').UserConfig}
 * @see https://vitejs.dev/config/
 */
const config = {
  mode: process.env.MODE,
  root: PACKAGE_ROOT,
  envDir: PROJECT_ROOT,
  publicDir: 'public',
  resolve: {
    alias: générerAliasRésolution(),
    extensions: ['.js', '.json', '.jsx', '.mjs', '.ts', '.tsx', '.vue'],
  },
  base: '',
  server: {
    fs: {
      strict: true,
    },
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'unsafe-none',
    },
  },
  build: {
    sourcemap: true,
    target: forElectron ? `chrome${chrome}` : 'esnext',
    outDir: forElectron ? 'dist' : 'dist/web',
    assetsDir: '.',
    rollupOptions: {
      input: join(PACKAGE_ROOT, 'index.html'),
      external: dépendsÀExclure,
      output: {
        manualChunks: (id) => {
          // Vendor chunks
          if (id.includes('node_modules/vue') && !id.includes('vuetify')) {
            return 'vue-vendor';
          }
          if (id.includes('node_modules/vuetify')) {
            return 'vuetify';
          }
          if (id.includes('node_modules/@tanstack')) {
            return 'query-vendor';
          }
          if (id.includes('node_modules/webfontloader')) {
            return 'ui-vendor';
          }
          // PeerBit and database related
          if (id.includes('node_modules/@peerbit')) {
            return 'peerbit-vendor';
          }
          if (id.includes('node_modules/@sqlite')) {
            return 'sqlite-vendor';
          }
          // IPFS and P2P
          if (id.includes('node_modules/ipfs') || id.includes('node_modules/multiformats')) {
            return 'ipfs-vendor';
          }
          if (id.includes('node_modules/libp2p') || id.includes('node_modules/@libp2p')) {
            return 'p2p-vendor';
          }
          // Crypto
          if (id.includes('node_modules/libsodium') || id.includes('node_modules/uuid')) {
            return 'crypto-vendor';
          }
          // SDK
          if (id.includes('node_modules/@riffcc/lens-sdk')) {
            return 'lens-sdk';
          }
        },
        // Optimize chunk size
        chunkFileNames: (chunkInfo) => {
          const facadeModuleId = chunkInfo.facadeModuleId ? chunkInfo.facadeModuleId.split('/').pop() : 'chunk';
          return `${facadeModuleId}-[hash].js`;
        },
      },
    },
    emptyOutDir: true,
    reportCompressedSize: false,
    // Increase chunk size warning limit since we're manually chunking
    chunkSizeWarningLimit: 1000,
    // Minification options
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: process.env.MODE === 'production',
        drop_debugger: true,
        pure_funcs: process.env.MODE === 'production' ? ['console.log', 'console.debug'] : [],
      },
    },
  },
  optimizeDeps: {
    exclude: dépendsÀExclure,
    include: [
      'vue',
      'vue-router',
      'vuetify',
      '@tanstack/vue-query',
    ],
    esbuildOptions: {
      target: 'esnext',
    },
  },
  // Define IS_ELECTRON for renderer code
  define: {
    'import.meta.env.IS_ELECTRON': forElectron,
  },
  test: {
    environment: 'happy-dom',
    server: {
      deps: {
        inline: ['vuetify'],
      },
    },
    coverage: {
      provider: 'istanbul',
    },
  },
  plugins: générerExtentions(),
};

export default config;
