/* eslint-env node */

import vue from '@vitejs/plugin-vue';
import {copyFileSync} from 'fs';
import {join} from 'node:path';
import {renderer} from 'unplugin-auto-expose';
import vuetify from 'vite-plugin-vuetify';
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

const générerExtentions = () => {
  const extentions = [
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
    extentions.push(
      renderer.vite({
        preloadEntry: join(PACKAGE_ROOT, '../preload/src/index.ts'),
      }),
    );
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
