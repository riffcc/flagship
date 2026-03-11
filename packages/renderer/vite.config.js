/* eslint-env node */

import Vue from '@vitejs/plugin-vue';
import {copyFileSync} from 'fs';
import {join} from 'node:path';
// Fonts are self-hosted in public/fonts/ and imported via src/styles/fonts.css
import {splitVendorChunkPlugin} from 'vite';
import {VitePWA} from 'vite-plugin-pwa';
import Vuetify, { transformAssetUrls } from 'vite-plugin-vuetify';
import {chrome} from '../../.electron-vendors.cache.json';
import {injectAppVersion} from '../../version/inject-app-version-plugin.mjs';

const PACKAGE_ROOT = __dirname;
const PROJECT_ROOT = join(PACKAGE_ROOT, '../..');

const forElectron = !process.env.WEB;

if (forElectron) {
  copyFileSync(join(PACKAGE_ROOT, 'indexElectron.html'), join(PACKAGE_ROOT, 'index.html'));
} else {
  copyFileSync(join(PACKAGE_ROOT, 'indexBrowser.html'), join(PACKAGE_ROOT, 'index.html'));
}

const générerAliasRésolution = () => {
  const common = {
    '/@/': join(PACKAGE_ROOT, 'src') + '/',
    '@riffcc/citadel-sdk': join(PROJECT_ROOT, 'packages/citadel-sdk/src/index.ts'),
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

/**
 * @type {import('vite').UserConfig}
 * @see https://vitejs.dev/config/
 */
const config = {
  mode: process.env.MODE,
  root: PACKAGE_ROOT,
  envDir: PROJECT_ROOT,
  publicDir: 'public',
  plugins: [
    Vue({
      template: { transformAssetUrls },
    }),
    Vuetify({
      autoImport: true,
      styles: {
        configFile: 'src/styles/settings.scss',
      },
    }),
    splitVendorChunkPlugin(),
    injectAppVersion(),
    // PWA service worker - only for web builds
    !forElectron && VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.ico', 'fonts/**/*', 'images/**/*'],
      manifest: {
        name: 'Riff.CC',
        short_name: 'Riff.CC',
        description: 'Decentralized music, movies, and creative content',
        theme_color: '#1a1a2e',
        background_color: '#1a1a2e',
        display: 'standalone',
        icons: [
          {
            src: '/images/riffcc-192.png',
            sizes: '192x192',
            type: 'image/png',
          },
          {
            src: '/images/riffcc-512.png',
            sizes: '512x512',
            type: 'image/png',
          },
        ],
      },
      workbox: {
        // Cache app shell files (exclude mock data)
        globPatterns: ['**/*.{js,css,html,ico,woff,woff2}'],
        globIgnores: ['mock/**/*', '**/mock/**/*'],
        // Runtime caching for API calls
        runtimeCaching: [
          {
            // Lens API - network first, fall back to cache
            urlPattern: /^https:\/\/.*\/api\/lens\/.*/i,
            handler: 'NetworkFirst',
            options: {
              cacheName: 'lens-api-cache',
              expiration: {
                maxEntries: 100,
                maxAgeSeconds: 60 * 60, // 1 hour
              },
              cacheableResponse: {
                statuses: [0, 200],
              },
            },
          },
          {
            // Archivist manifest API - stale while revalidate
            urlPattern: /^https:\/\/.*\/api\/archivist\/v1\/data\/.*/i,
            handler: 'StaleWhileRevalidate',
            options: {
              cacheName: 'archivist-manifest-cache',
              expiration: {
                maxEntries: 200,
                maxAgeSeconds: 60 * 60 * 24, // 24 hours
              },
              cacheableResponse: {
                statuses: [0, 200],
              },
            },
          },
          {
            // IPFS gateway content - cache first (immutable by CID)
            urlPattern: /^https:\/\/.*\/(ipfs|archivist)\/.*\.(mp3|opus|ogg|flac|wav|m4a)$/i,
            handler: 'CacheFirst',
            options: {
              cacheName: 'audio-cache',
              expiration: {
                maxEntries: 50,
                maxAgeSeconds: 60 * 60 * 24 * 7, // 7 days
              },
              cacheableResponse: {
                statuses: [0, 200, 206],
              },
              rangeRequests: true,
            },
          },
          {
            // Thumbnail images - cache first
            urlPattern: /^https:\/\/.*\/(ipfs|archivist)\/.*\.(jpg|jpeg|png|webp|gif)$/i,
            handler: 'CacheFirst',
            options: {
              cacheName: 'image-cache',
              expiration: {
                maxEntries: 200,
                maxAgeSeconds: 60 * 60 * 24 * 30, // 30 days
              },
              cacheableResponse: {
                statuses: [0, 200],
              },
            },
          },
        ],
      },
    }),
  ].filter(Boolean),
  resolve: {
    alias: générerAliasRésolution(),
    extensions: ['.js', '.json', '.jsx', '.mjs', '.ts', '.tsx', '.vue'],
  },
  base: '/',
  server: {
    port: parseInt(process.env.PORT) || 5175,
    host: process.env.VITE_HOST || 'localhost',
    allowedHosts: ['.riff.cc'],
    fs: {
      strict: true,
    },
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'unsafe-none',
    },
  },
  build: {
    sourcemap: process.env.MODE !== 'production',
    target: forElectron ? `chrome${chrome}` : 'esnext',
    outDir: forElectron ? 'dist' : 'dist/web',
    assetsDir: '.',
    rollupOptions: {
      input: join(PACKAGE_ROOT, 'index.html'),
    },
    emptyOutDir: true,
    reportCompressedSize: false,
    // Increase chunk size warning limit since we're manually chunking
    chunkSizeWarningLimit: 1000,
    // Minification options
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: false,
        drop_debugger: true,
        pure_funcs: [],
      },
    },
  },
  optimizeDeps: {
    exclude: [
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
  css: {
    preprocessorOptions: {
      sass: {
        api: 'modern-compiler',
      },
      scss: {
        api: 'modern-compiler',
      },
    },
  },
};

export default config;
