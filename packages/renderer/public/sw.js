// Service Worker for caching federation index data
const CACHE_NAME = 'lens-cache-v1';
const FEDERATION_CACHE = 'federation-data-v1';

// Cache static assets
const STATIC_ASSETS = [
  '/',
  '/index.html',
  '/manifest.json',
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
  console.log('[SW] Installing service worker...');
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      console.log('[SW] Caching static assets');
      return cache.addAll(STATIC_ASSETS);
    })
  );
  // Skip waiting to activate immediately
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('[SW] Activating service worker...');
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME && cacheName !== FEDERATION_CACHE) {
            console.log('[SW] Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
  // Take control of all clients immediately
  self.clients.claim();
});

// Fetch event - serve from cache when possible
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Cache federation index queries
  if (url.pathname.includes('/federation/') || url.pathname.includes('getFederationIndex')) {
    event.respondWith(
      caches.open(FEDERATION_CACHE).then(async (cache) => {
        // Try cache first
        const cachedResponse = await cache.match(request);
        
        if (cachedResponse) {
          console.log('[SW] Serving federation data from cache:', url.pathname);
          
          // Update cache in background
          fetch(request).then((freshResponse) => {
            if (freshResponse.ok) {
              cache.put(request, freshResponse.clone());
            }
          });
          
          return cachedResponse;
        }
        
        // Not in cache, fetch and cache
        console.log('[SW] Fetching federation data:', url.pathname);
        const response = await fetch(request);
        
        if (response.ok) {
          cache.put(request, response.clone());
        }
        
        return response;
      })
    );
    return;
  }

  // For static assets, use cache-first strategy
  if (STATIC_ASSETS.includes(url.pathname)) {
    event.respondWith(
      caches.match(request).then((response) => {
        return response || fetch(request);
      })
    );
    return;
  }

  // For everything else, use network-first
  event.respondWith(fetch(request));
});

// Listen for messages from the app
self.addEventListener('message', (event) => {
  if (event.data.type === 'CACHE_FEDERATION_DATA') {
    // Pre-cache federation data
    const { url, data } = event.data;
    caches.open(FEDERATION_CACHE).then((cache) => {
      const response = new Response(JSON.stringify(data), {
        headers: { 'Content-Type': 'application/json' }
      });
      cache.put(url, response);
      console.log('[SW] Cached federation data:', url);
    });
  }
  
  if (event.data.type === 'CLEAR_CACHE') {
    caches.delete(FEDERATION_CACHE).then(() => {
      console.log('[SW] Cleared federation cache');
    });
  }
});