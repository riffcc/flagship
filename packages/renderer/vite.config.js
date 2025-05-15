/* eslint-env node */

import vue from '@vitejs/plugin-vue';
import {copyFileSync, mkdirSync} from 'fs';
import {join} from 'node:path';
// import {renderer} from 'unplugin-auto-expose'; // Removed unplugin-auto-expose
import vuetify from 'vite-plugin-vuetify';
// import topLevelAwait from 'vite-plugin-top-level-await'; // Replaced with vite-plugin-wasm
import wasm from 'vite-plugin-wasm'; // Added import
import {chrome} from '../../.electron-vendors.cache.json';
import {injectAppVersion} from '../../version/inject-app-version-plugin.mjs';

import {nodePolyfills} from 'vite-plugin-node-polyfills';

const PACKAGE_ROOT = __dirname;
const PROJECT_ROOT = join(PACKAGE_ROOT, '../..');

const forElectron = !process.env.WEB;

// This is really ugly, but nothing else works with Vite as I can't figure out where to
// change the default index.html location...
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
    // topLevelAwait(), // Replaced with vite-plugin-wasm
    wasm(), // Added plugin
    vue(),
    // https://github.com/vuetifyjs/vuetify-loader/tree/next/packages/vite-plugin
    vuetify({
      autoImport: true,
      styles: {
        configFile: 'src/styles/settings.scss',
      },
    }),
    nodePolyfills(),
  ];
  if (forElectron) {
    // extentions.push( // Usage of unplugin-auto-expose renderer.vite() removed
    //   renderer.vite({
    //     preloadEntry: join(PACKAGE_ROOT, '../preload/src/index.ts'),
    //   }),
    // );
  }
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
    return Object.assign({}, common, {
      '#preload': join(PACKAGE_ROOT, 'src') + '/polyfillPreload',
    });
  }
};

// Same for Electron or web, since this is for the renderer process GUI
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
  base: '', // forElectron ? '' : '/github-repo/', // Only necessary if on non-root url, such as github pages at org.github.io/repo
  server: {
    fs: {
      strict: true,
    },
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
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
