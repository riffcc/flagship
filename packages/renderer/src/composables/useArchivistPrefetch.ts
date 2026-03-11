/**
 * Archivist Manifest Prefetch Cache
 *
 * Starts fetching the CID manifest on album click, BEFORE navigation.
 * albumViewer checks this cache first to avoid duplicate fetches.
 *
 * Now with IndexedDB persistence and stale-while-revalidate:
 * - Cached manifests load instantly on repeat visits
 * - Stale data is served immediately, then refreshed in background
 */

import { isArchivistCid } from '/@/utils';
import { getCachedManifest, cacheManifest, isStale } from './useArchivistCache';

interface PrefetchEntry {
  promise: Promise<any>;
  timestamp: number;
}

// Module-level cache for in-flight requests - survives component unmounts
const prefetchCache = new Map<string, PrefetchEntry>();

// In-memory cache TTL: 30 seconds
const CACHE_TTL = 30000;

/**
 * Build the Archivist data URL for a CID
 */
function buildDataUrl(contentCID: string): string {
  const archivistGateway = import.meta.env.VITE_ARCHIVIST_GATEWAY as string | undefined
    || import.meta.env.VITE_ARCHIVIST_API_URL as string | undefined
    || 'https://uploads.island.riff.cc';
  const baseUrl = archivistGateway.startsWith('http') ? archivistGateway : `https://${archivistGateway}`;
  return `${baseUrl}/api/archivist/v1/data/${contentCID}`;
}

/**
 * Fetch manifest from network and cache it
 */
async function fetchAndCache(contentCID: string): Promise<any> {
  const dataUrl = buildDataUrl(contentCID);

  const res = await fetch(dataUrl, {
    headers: { 'Accept': 'application/json' }
  });
  const data = await res.json();

  // Store in IndexedDB for future visits
  cacheManifest(contentCID, data).catch(() => {});

  return data;
}

/**
 * Start prefetching the Archivist manifest for a contentCID.
 * Call this on album click, before navigation.
 *
 * Uses stale-while-revalidate:
 * - If cached and fresh: skip network
 * - If cached but stale: serve cached, refresh in background
 * - If not cached: fetch from network
 */
export function prefetchArchivistManifest(contentCID: string): void {
  if (!contentCID || !isArchivistCid(contentCID)) {
    return;
  }

  // Already have an in-flight request?
  const existing = prefetchCache.get(contentCID);
  if (existing && Date.now() - existing.timestamp < CACHE_TTL) {
    return;
  }

  console.log('[prefetch] Starting prefetch:', contentCID.substring(0, 12) + '...');

  // Create a promise that checks IndexedDB first, then network
  const promise = (async () => {
    // Check IndexedDB cache
    const cached = await getCachedManifest(contentCID);

    if (cached) {
      if (isStale(cached)) {
        // Stale: return cached data, but refresh in background
        console.log('[prefetch] Cache stale, refreshing in background:', contentCID.substring(0, 12) + '...');
        fetchAndCache(contentCID).catch(() => {});
        return cached.data;
      } else {
        // Fresh: just return cached
        console.log('[prefetch] Cache hit (fresh):', contentCID.substring(0, 12) + '...');
        return cached.data;
      }
    }

    // No cache: fetch from network
    console.log('[prefetch] Cache miss, fetching:', contentCID.substring(0, 12) + '...');
    return fetchAndCache(contentCID);
  })();

  promise.catch(err => {
    console.warn('[prefetch] Prefetch failed:', err);
    prefetchCache.delete(contentCID);
  });

  prefetchCache.set(contentCID, {
    promise,
    timestamp: Date.now(),
  });
}

/**
 * Get a prefetched manifest if available.
 * Checks in-memory cache first, then IndexedDB.
 * Returns a promise if found, null otherwise.
 */
export function getPrefetchedManifest(contentCID: string): Promise<any> | null {
  // Check in-memory cache first (for recent prefetch)
  const entry = prefetchCache.get(contentCID);
  if (entry && Date.now() - entry.timestamp < CACHE_TTL) {
    console.log('[prefetch] Using in-flight prefetch:', contentCID.substring(0, 12) + '...');
    return entry.promise;
  }

  // Clean up stale in-memory entry
  if (entry) {
    prefetchCache.delete(contentCID);
  }

  // Check IndexedDB for persisted cache (async, but we return the promise)
  const idbPromise = (async () => {
    const cached = await getCachedManifest(contentCID);
    if (cached) {
      if (isStale(cached)) {
        // Stale: return data but refresh in background
        console.log('[prefetch] IndexedDB cache stale:', contentCID.substring(0, 12) + '...');
        fetchAndCache(contentCID).catch(() => {});
      } else {
        console.log('[prefetch] IndexedDB cache hit:', contentCID.substring(0, 12) + '...');
      }
      return cached.data;
    }
    return null;
  })();

  // Store the IDB check promise so subsequent calls don't duplicate
  prefetchCache.set(contentCID, {
    promise: idbPromise,
    timestamp: Date.now(),
  });

  return idbPromise;
}

/**
 * Clear a specific entry from cache (e.g., after consumption)
 */
export function clearPrefetchedManifest(contentCID: string): void {
  prefetchCache.delete(contentCID);
}
