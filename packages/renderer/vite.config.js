/* eslint-env node */

import vue from '@vitejs/plugin-vue';
import {copyFileSync} from 'fs';
import {join} from 'node:path';
import vuetify from 'vite-plugin-vuetify';
import {chrome} from '../../.electron-vendors.cache.json';
import {injectAppVersion} from '../../version/inject-app-version-plugin.mjs';
import {nodePolyfills} from 'vite-plugin-node-polyfills';
import peerbit from '@peerbit/vite';
import { viteStaticCopy } from 'vite-plugin-static-copy';
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
    peerbit(),
    viteStaticCopy({
      targets: [
        {
          src: join(PACKAGE_ROOT, 'public', 'peerbit', 'sqlite3.wasm'),
          dest: join(PROJECT_ROOT, 'node_modules', '.vite', 'deps'),
        },
      ],
    }),
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
    },
    emptyOutDir: true,
    reportCompressedSize: false,
  },
  optimizeDeps: {
    exclude: dépendsÀExclure,
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
