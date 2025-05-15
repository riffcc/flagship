/* eslint-env node */

import vue from '@vitejs/plugin-vue';
import {copyFileSync, mkdirSync} from 'fs';
import {join} from 'node:path';
import vuetify from 'vite-plugin-vuetify';
import wasm from 'vite-plugin-wasm';
import {chrome} from '../../.electron-vendors.cache.json';
import {injectAppVersion} from '../../version/inject-app-version-plugin.mjs';
import {nodePolyfills} from 'vite-plugin-node-polyfills';

const PACKAGE_ROOT = __dirname;
const PROJECT_ROOT = join(PACKAGE_ROOT, '../..');
const PACKAGES_ROOT = join(PACKAGE_ROOT, '..');

const forElectron = !process.env.WEB;

if (forElectron) {
  copyFileSync(join(PACKAGE_ROOT, 'indexElectron.html'), join(PACKAGE_ROOT, 'index.html'));
} else {
  copyFileSync(join(PACKAGE_ROOT, 'indexBrowser.html'), join(PACKAGE_ROOT, 'index.html'));
}

// Ensure the target directory exists and copy the wasm file
const wasmTargetDir = join(PACKAGE_ROOT, 'public/peerbit');
const wasmSourcePath = join(PROJECT_ROOT, 'node_modules/@peerbit/riblt/dist/rateless_iblt_bg.wasm');
const wasmDestPath = join(wasmTargetDir, 'rateless_iblt_bg.wasm');

try {
  mkdirSync(wasmTargetDir, { recursive: true });
  copyFileSync(wasmSourcePath, wasmDestPath);
  console.log(`Copied rateless_iblt_bg.wasm to ${wasmDestPath}`);
} catch (error) {
  console.error('Error copying WASM file:', error);
  // Decide if you want to throw the error or let Vite continue
  // For now, let's log and continue, but this might hide issues
}

// Copy the any-store-opfs worker
const opfsWorkerSourcePath = join(PROJECT_ROOT, 'node_modules/@peerbit/any-store-opfs/dist/peerbit/anystore-opfs-worker.min.js');
const opfsWorkerDestPath = join(wasmTargetDir, 'anystore-opfs-worker.min.js'); // wasmTargetDir is public/peerbit

try {
  // The directory should already be created by the wasm copy step,
  // but calling mkdirSync again with recursive:true is safe.
  mkdirSync(wasmTargetDir, { recursive: true });
  copyFileSync(opfsWorkerSourcePath, opfsWorkerDestPath);
  console.log(`Copied anystore-opfs-worker.min.js to ${opfsWorkerDestPath}`);
} catch (error) {
  console.error('Error copying OPFS worker file:', error);
}

const générerExtentions = () => {
  const extentions = [
    wasm(),
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
    '/@/lib/': join(PACKAGES_ROOT, 'lib', 'src') + '/',
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
