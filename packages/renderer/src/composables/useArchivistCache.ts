/**
 * Archivist Manifest IndexedDB Cache
 *
 * Provides persistent caching for Archivist manifests with stale-while-revalidate.
 * - Instant loads for repeat visits (serves from cache)
 * - Background refresh keeps data fresh
 * - Survives browser restarts
 */

const DB_NAME = 'riffcc-archivist-cache';
const DB_VERSION = 1;
const STORE_NAME = 'manifests';

// Cache TTL: 1 hour (manifests rarely change, but we want eventual consistency)
const CACHE_TTL = 60 * 60 * 1000;

// Stale threshold: 5 minutes (after this, refresh in background)
export const STALE_THRESHOLD = 5 * 60 * 1000;

interface CacheEntry {
  cid: string;
  data: any;
  timestamp: number;
}

let dbPromise: Promise<IDBDatabase> | null = null;

/**
 * Open the IndexedDB database (singleton)
 */
function openDB(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise;

  dbPromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => {
      console.warn('[cache] IndexedDB open failed:', request.error);
      reject(request.error);
    };

    request.onsuccess = () => {
      resolve(request.result);
    };

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;

      // Create object store for manifests
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        const store = db.createObjectStore(STORE_NAME, { keyPath: 'cid' });
        store.createIndex('timestamp', 'timestamp', { unique: false });
      }
    };
  });

  return dbPromise;
}

/**
 * Get a cached manifest by CID
 */
export async function getCachedManifest(cid: string): Promise<CacheEntry | null> {
  try {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readonly');
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(cid);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const entry = request.result as CacheEntry | undefined;

        if (!entry) {
          resolve(null);
          return;
        }

        // Check if expired (hard TTL)
        if (Date.now() - entry.timestamp > CACHE_TTL) {
          // Delete expired entry
          deleteFromCache(cid).catch(() => {});
          resolve(null);
          return;
        }

        resolve(entry);
      };
    });
  } catch (err) {
    console.warn('[cache] Get failed:', err);
    return null;
  }
}

/**
 * Store a manifest in the cache
 */
export async function cacheManifest(cid: string, data: any): Promise<void> {
  try {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);

      const entry: CacheEntry = {
        cid,
        data,
        timestamp: Date.now(),
      };

      const request = store.put(entry);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve();
    });
  } catch (err) {
    console.warn('[cache] Put failed:', err);
  }
}

/**
 * Delete a manifest from cache
 */
async function deleteFromCache(cid: string): Promise<void> {
  try {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      const request = store.delete(cid);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve();
    });
  } catch (err) {
    console.warn('[cache] Delete failed:', err);
  }
}

/**
 * Check if a cache entry is stale (should refresh in background)
 */
export function isStale(entry: CacheEntry): boolean {
  return Date.now() - entry.timestamp > STALE_THRESHOLD;
}

/**
 * Clear all cached manifests (for debugging/testing)
 */
export async function clearCache(): Promise<void> {
  try {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      const request = store.clear();
      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve();
    });
  } catch (err) {
    console.warn('[cache] Clear failed:', err);
  }
}

/**
 * Get cache statistics (for debugging)
 */
export async function getCacheStats(): Promise<{ count: number; oldestTimestamp: number | null }> {
  try {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readonly');
      const store = tx.objectStore(STORE_NAME);
      const countRequest = store.count();
      const cursorRequest = store.index('timestamp').openCursor();

      let oldestTimestamp: number | null = null;

      cursorRequest.onsuccess = () => {
        const cursor = cursorRequest.result;
        if (cursor) {
          oldestTimestamp = cursor.value.timestamp;
        }
      };

      countRequest.onerror = () => reject(countRequest.error);
      countRequest.onsuccess = () => {
        resolve({
          count: countRequest.result,
          oldestTimestamp,
        });
      };
    });
  } catch (err) {
    console.warn('[cache] Stats failed:', err);
    return { count: 0, oldestTimestamp: null };
  }
}
